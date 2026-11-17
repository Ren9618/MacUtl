use std::ffi::CString;
use std::path::Path;
use std::ptr;

use ffmpeg_sys_next as sys;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncoderCodec {
    H264,
    H265,
}

/// MP4 に多重化する音声トラックの設定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioExportConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub average_bitrate: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncoderBackend {
    Auto,
    GpuVideo,
    Software,
}

#[derive(Clone, Copy, Debug)]
pub struct EncoderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub average_bitrate: u32,
    pub max_bitrate: u32,
    pub codec: EncoderCodec,
    pub backend: EncoderBackend,
    pub audio: Option<AudioExportConfig>,
}

fn hw_candidates(codec: EncoderCodec) -> &'static [&'static str] {
    match codec {
        EncoderCodec::H264 => &["h264_nvenc", "h264_qsv", "h264_amf", "h264_vaapi"],
        EncoderCodec::H265 => &["hevc_nvenc", "hevc_qsv", "hevc_amf", "hevc_vaapi"],
    }
}

fn software_encoder(codec: EncoderCodec) -> &'static str {
    match codec {
        EncoderCodec::H264 => "libx264",
        EncoderCodec::H265 => "libx265",
    }
}

pub fn is_hw_encoder_name(name: &str) -> bool {
    name.ends_with("_nvenc")
        || name.ends_with("_qsv")
        || name.ends_with("_amf")
        || name.ends_with("_vaapi")
}

/// ネイティブ AAC エンコーダが受け付ける MPEG-4 標準の標本レートか。
pub fn is_aac_sample_rate(rate: u32) -> bool {
    matches!(
        rate,
        96_000
            | 88_200
            | 64_000
            | 48_000
            | 44_100
            | 32_000
            | 24_000
            | 22_050
            | 16_000
            | 12_000
            | 11_025
            | 8_000
            | 7_350
    )
}

fn averror_eagain() -> i32 {
    -libc::EAGAIN
}

pub struct VideoEncoder {
    fmt_ctx: *mut sys::AVFormatContext,
    enc_ctx: *mut sys::AVCodecContext,
    sws_ctx: *mut sys::SwsContext,
    frame: *mut sys::AVFrame,
    packet: *mut sys::AVPacket,
    stream_index: i32,
    time_base: sys::AVRational,
    next_pts: i64,
    width: u32,
    height: u32,
    header_written: bool,
    output_opened: bool,
    pub encoder_name: String,
    audio_enc_ctx: *mut sys::AVCodecContext,
    audio_frame: *mut sys::AVFrame,
    audio_packet: *mut sys::AVPacket,
    audio_stream_index: i32,
    audio_time_base: sys::AVRational,
    audio_next_pts: i64,
    audio_sample_fmt: sys::AVSampleFormat,
    audio_samples_per_frame: usize,
    audio_channels: u16,
    pending_audio: Vec<f32>,
}

unsafe impl Send for VideoEncoder {}

impl VideoEncoder {
    pub fn open(output_path: &Path, config: EncoderConfig) -> Result<Self, String> {
        let mut candidates: Vec<&str> = match config.backend {
            EncoderBackend::Auto | EncoderBackend::GpuVideo => hw_candidates(config.codec).to_vec(),
            EncoderBackend::Software => Vec::new(),
        };
        if matches!(
            config.backend,
            EncoderBackend::Auto | EncoderBackend::Software
        ) {
            candidates.push(software_encoder(config.codec));
        }

        let mut last_error = String::from("エンコーダ候補が空です");
        for name in candidates {
            match Self::open_with_encoder(output_path, config, name) {
                Ok(encoder) => return Ok(encoder),
                Err(err) => {
                    eprintln!("[neo-media-ffmpeg][encoder] {name} 初期化失敗: {err}、次候補へ縮退");
                    last_error = err;
                }
            }
        }
        Err(format!("全エンコーダ候補が失敗: {last_error}"))
    }

    fn open_with_encoder(
        output_path: &Path,
        config: EncoderConfig,
        encoder_name: &str,
    ) -> Result<Self, String> {
        unsafe {
            let codec = {
                let cname = CString::new(encoder_name).map_err(|e| e.to_string())?;
                sys::avcodec_find_encoder_by_name(cname.as_ptr())
            };
            if codec.is_null() {
                return Err(format!("エンコーダ{encoder_name}未検出"));
            }

            let path_str = output_path.to_string_lossy().into_owned();
            let out_cpath = CString::new(path_str.clone()).map_err(|e| e.to_string())?;

            let mut fmt_ctx: *mut sys::AVFormatContext = ptr::null_mut();
            if sys::avformat_alloc_output_context2(
                &mut fmt_ctx,
                ptr::null(),
                ptr::null(),
                out_cpath.as_ptr(),
            ) < 0
                || fmt_ctx.is_null()
            {
                return Err("avformat_alloc_output_context2失敗".to_owned());
            }

            let stream = sys::avformat_new_stream(fmt_ctx, ptr::null());
            if stream.is_null() {
                sys::avformat_free_context(fmt_ctx);
                return Err("avformat_new_stream失敗".to_owned());
            }
            let stream_index = (*stream).index;

            let enc_ctx = sys::avcodec_alloc_context3(codec);
            if enc_ctx.is_null() {
                sys::avformat_free_context(fmt_ctx);
                return Err("avcodec_alloc_context3失敗".to_owned());
            }

            let time_base = sys::AVRational {
                num: 1,
                den: config.fps.max(1) as i32,
            };
            (*enc_ctx).width = config.width as i32;
            (*enc_ctx).height = config.height as i32;
            (*enc_ctx).time_base = time_base;
            (*enc_ctx).framerate = sys::AVRational {
                num: config.fps.max(1) as i32,
                den: 1,
            };
            (*enc_ctx).bit_rate = config.average_bitrate as i64;
            (*enc_ctx).rc_max_rate = config.max_bitrate as i64;
            (*enc_ctx).rc_buffer_size = (config.max_bitrate * 2) as i32;
            (*enc_ctx).gop_size = config.fps.max(1) as i32 * 2;
            (*enc_ctx).pix_fmt = pick_pix_fmt(enc_ctx, codec);
            if (*(*fmt_ctx).oformat).flags & sys::AVFMT_GLOBALHEADER != 0 {
                (*enc_ctx).flags |= sys::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
            }

            let mut opts: *mut sys::AVDictionary = ptr::null_mut();
            if is_hw_encoder_name(encoder_name) {
                set_dict(&mut opts, "rc", "vbr");
            } else {
                set_dict(&mut opts, "preset", "medium");
            }

            let open_result = sys::avcodec_open2(enc_ctx, codec, &mut opts);
            sys::av_dict_free(&mut opts);
            if open_result < 0 {
                sys::avcodec_free_context(&mut { enc_ctx });
                sys::avformat_free_context(fmt_ctx);
                return Err(format!("avcodec_open2失敗(code={open_result})"));
            }

            if sys::avcodec_parameters_from_context((*stream).codecpar, enc_ctx) < 0 {
                sys::avcodec_free_context(&mut { enc_ctx });
                sys::avformat_free_context(fmt_ctx);
                return Err("avcodec_parameters_from_context失敗".to_owned());
            }
            (*stream).time_base = time_base;

            let mut audio = match config.audio {
                Some(audio_cfg) => match open_audio_stream(fmt_ctx, audio_cfg) {
                    Ok(setup) => Some(setup),
                    Err(err) => {
                        sys::avcodec_free_context(&mut { enc_ctx });
                        sys::avformat_free_context(fmt_ctx);
                        return Err(err);
                    }
                },
                None => None,
            };

            let audio_packet = if audio.is_some() {
                let packet = sys::av_packet_alloc();
                if packet.is_null() {
                    if let Some(mut audio) = audio.take() {
                        sys::avcodec_free_context(&mut audio.enc_ctx);
                    }
                    sys::avcodec_free_context(&mut { enc_ctx });
                    sys::avformat_free_context(fmt_ctx);
                    return Err("音声パケットの確保に失敗しました".to_owned());
                }
                packet
            } else {
                ptr::null_mut()
            };
            let audio_frame = if audio.is_some() {
                let frame = sys::av_frame_alloc();
                if frame.is_null() {
                    sys::av_packet_free(&mut { audio_packet });
                    if let Some(mut audio) = audio.take() {
                        sys::avcodec_free_context(&mut audio.enc_ctx);
                    }
                    sys::avcodec_free_context(&mut { enc_ctx });
                    sys::avformat_free_context(fmt_ctx);
                    return Err("音声フレームの確保に失敗しました".to_owned());
                }
                frame
            } else {
                ptr::null_mut()
            };
            if let Some(setup) = &audio {
                (*audio_frame).format = setup.sample_fmt as i32;
                (*audio_frame).nb_samples = setup.samples_per_frame as i32;
                (*audio_frame).sample_rate = (*setup.enc_ctx).sample_rate;
                if sys::av_channel_layout_copy(
                    &mut (*audio_frame).ch_layout,
                    &(*setup.enc_ctx).ch_layout,
                ) < 0
                    || sys::av_frame_get_buffer(audio_frame, 0) < 0
                {
                    sys::av_frame_free(&mut { audio_frame });
                    sys::av_packet_free(&mut { audio_packet });
                    sys::avcodec_free_context(&mut { setup.enc_ctx });
                    sys::avcodec_free_context(&mut { enc_ctx });
                    sys::avformat_free_context(fmt_ctx);
                    return Err("音声フレームバッファの確保に失敗しました".to_owned());
                }
            }

            let mut output_opened = false;
            if (*(*fmt_ctx).oformat).flags & sys::AVFMT_NOFILE == 0 {
                let avio_res =
                    sys::avio_open(&mut (*fmt_ctx).pb, out_cpath.as_ptr(), sys::AVIO_FLAG_WRITE);
                if avio_res < 0 {
                    if !audio_frame.is_null() {
                        sys::av_frame_free(&mut { audio_frame });
                    }
                    if !audio_packet.is_null() {
                        sys::av_packet_free(&mut { audio_packet });
                    }
                    if let Some(mut audio) = audio.take() {
                        sys::avcodec_free_context(&mut audio.enc_ctx);
                    }
                    sys::avcodec_free_context(&mut { enc_ctx });
                    sys::avformat_free_context(fmt_ctx);
                    return Err(format!("avio_open失敗(code={avio_res}, path={path_str:?})"));
                }
                output_opened = true;
            }

            if sys::avformat_write_header(fmt_ctx, ptr::null_mut()) < 0 {
                if output_opened {
                    sys::avio_closep(&mut (*fmt_ctx).pb);
                }
                sys::avcodec_free_context(&mut { enc_ctx });
                sys::avformat_free_context(fmt_ctx);
                return Err("avformat_write_header失敗".to_owned());
            }

            let frame = sys::av_frame_alloc();
            if frame.is_null() {
                return Err("av_frame_alloc失敗".to_owned());
            }
            (*frame).format = (*enc_ctx).pix_fmt as i32;
            (*frame).width = config.width as i32;
            (*frame).height = config.height as i32;
            if sys::av_frame_get_buffer(frame, 32) < 0 {
                sys::av_frame_free(&mut { frame });
                return Err("av_frame_get_buffer失敗".to_owned());
            }

            let sws_ctx = sys::sws_getContext(
                config.width as i32,
                config.height as i32,
                sys::AVPixelFormat::AV_PIX_FMT_RGBA,
                config.width as i32,
                config.height as i32,
                (*enc_ctx).pix_fmt,
                sys::SwsFlags::SWS_BILINEAR as i32,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            );
            if sws_ctx.is_null() {
                sys::av_frame_free(&mut { frame });
                return Err("sws_getContext失敗".to_owned());
            }

            let packet = sys::av_packet_alloc();
            if packet.is_null() {
                sys::sws_freeContext(sws_ctx);
                sys::av_frame_free(&mut { frame });
                return Err("av_packet_alloc失敗".to_owned());
            }

            Ok(Self {
                fmt_ctx,
                enc_ctx,
                sws_ctx,
                frame,
                packet,
                stream_index,
                time_base,
                next_pts: 0,
                width: config.width,
                height: config.height,
                header_written: true,
                output_opened,
                encoder_name: encoder_name.to_owned(),
                audio_enc_ctx: audio
                    .as_ref()
                    .map_or(ptr::null_mut(), |setup| setup.enc_ctx),
                audio_frame,
                audio_packet,
                audio_stream_index: audio.as_ref().map_or(-1, |setup| setup.stream_index),
                audio_time_base: audio
                    .as_ref()
                    .map_or(sys::AVRational { num: 0, den: 1 }, |setup| setup.time_base),
                audio_next_pts: 0,
                audio_sample_fmt: audio
                    .as_ref()
                    .map_or(sys::AVSampleFormat::AV_SAMPLE_FMT_NONE, |setup| {
                        setup.sample_fmt
                    }),
                audio_samples_per_frame: audio.as_ref().map_or(0, |setup| setup.samples_per_frame),
                audio_channels: audio.as_ref().map_or(0, |setup| {
                    (*setup.enc_ctx).ch_layout.nb_channels.max(0) as u16
                }),
                pending_audio: Vec::new(),
            })
        }
    }

    pub fn encode_rgba8(&mut self, rgba: &[u8]) -> Result<(), String> {
        let expected = self.width as usize * self.height as usize * 4;
        if rgba.len() != expected {
            return Err(format!(
                "フレームサイズ不一致: 期待{expected}バイト、実際{}バイト",
                rgba.len()
            ));
        }
        unsafe {
            if sys::av_frame_make_writable(self.frame) < 0 {
                return Err("av_frame_make_writable失敗".to_owned());
            }
            let src_stride = [self.width as i32 * 4, 0, 0, 0];
            let src_slices = [rgba.as_ptr(), ptr::null(), ptr::null(), ptr::null()];
            sys::sws_scale(
                self.sws_ctx,
                src_slices.as_ptr(),
                src_stride.as_ptr(),
                0,
                self.height as i32,
                (*self.frame).data.as_ptr(),
                (*self.frame).linesize.as_ptr(),
            );
            (*self.frame).pts = self.next_pts;
            self.next_pts += 1;

            self.send_frame_and_drain(self.frame)
        }
    }

    unsafe fn send_frame_and_drain(&mut self, frame: *mut sys::AVFrame) -> Result<(), String> {
        unsafe {
            let ret = sys::avcodec_send_frame(self.enc_ctx, frame);
            if ret < 0 {
                return Err(format!("avcodec_send_frame失敗(code={ret})"));
            }
            self.drain_packets()
        }
    }

    unsafe fn drain_packets(&mut self) -> Result<(), String> {
        unsafe {
            loop {
                let ret = sys::avcodec_receive_packet(self.enc_ctx, self.packet);
                if ret == averror_eagain() || ret == sys::AVERROR_EOF {
                    break;
                }
                if ret < 0 {
                    return Err(format!("avcodec_receive_packet失敗(code={ret})"));
                }
                (*self.packet).stream_index = self.stream_index;
                sys::av_packet_rescale_ts(
                    self.packet,
                    self.time_base,
                    (*(*(*self.fmt_ctx).streams.add(self.stream_index as usize))).time_base,
                );
                let write_ret = sys::av_interleaved_write_frame(self.fmt_ctx, self.packet);
                sys::av_packet_unref(self.packet);
                if write_ret < 0 {
                    return Err(format!("av_interleaved_write_frame失敗(code={write_ret})"));
                }
            }
            Ok(())
        }
    }

    /// インターリーブ f32 PCM（`audio` 設定と同じチャンネル数）を音声トラックへ追加する。
    ///
    /// 内部で AAC フレーム単位（既定 1024 サンプル）に分割してエンコードする。
    /// 端数は `finish` 時に最終フレームとして送出される。
    pub fn push_audio_interleaved(&mut self, samples: &[f32]) -> Result<(), String> {
        if self.audio_enc_ctx.is_null() {
            return Err("このエンコーダには音声ストリームがありません".to_owned());
        }
        let channels = self.audio_channels.max(1) as usize;
        let frame_bytes = self.audio_samples_per_frame.max(1) * channels;
        self.pending_audio.extend_from_slice(samples);
        while self.pending_audio.len() >= frame_bytes {
            let chunk: Vec<f32> = self.pending_audio.drain(..frame_bytes).collect();
            unsafe { self.write_audio_frame(&chunk)? };
        }
        Ok(())
    }

    unsafe fn write_audio_frame(&mut self, chunk: &[f32]) -> Result<(), String> {
        unsafe {
            let channels = self.audio_channels.max(1) as usize;
            let nb_samples = chunk.len() / channels;
            if nb_samples == 0 {
                return Ok(());
            }
            if sys::av_frame_make_writable(self.audio_frame) < 0 {
                return Err("音声フレームの書き込み準備に失敗しました".to_owned());
            }
            (*self.audio_frame).nb_samples = nb_samples as i32;
            (*self.audio_frame).pts = self.audio_next_pts;

            if sys::av_sample_fmt_is_planar(self.audio_sample_fmt) != 0 {
                for ch in 0..channels {
                    let dst = (*self.audio_frame).data[ch] as *mut f32;
                    for (i, sample) in chunk.iter().skip(ch).step_by(channels).enumerate() {
                        *dst.add(i) = *sample;
                    }
                }
            } else {
                let dst = (*self.audio_frame).data[0] as *mut f32;
                ptr::copy_nonoverlapping(chunk.as_ptr(), dst, chunk.len());
            }

            let ret = sys::avcodec_send_frame(self.audio_enc_ctx, self.audio_frame);
            if ret < 0 {
                return Err(format!("音声フレームの送信に失敗しました(code={ret})"));
            }
            self.audio_next_pts += nb_samples as i64;
            self.drain_audio_packets()
        }
    }

    unsafe fn drain_audio_packets(&mut self) -> Result<(), String> {
        unsafe {
            loop {
                let ret = sys::avcodec_receive_packet(self.audio_enc_ctx, self.audio_packet);
                if ret == averror_eagain() || ret == sys::AVERROR_EOF {
                    break;
                }
                if ret < 0 {
                    return Err(format!("音声パケットの受信に失敗しました(code={ret})"));
                }
                (*self.audio_packet).stream_index = self.audio_stream_index;
                sys::av_packet_rescale_ts(
                    self.audio_packet,
                    self.audio_time_base,
                    (*(*(*self.fmt_ctx)
                        .streams
                        .add(self.audio_stream_index as usize)))
                    .time_base,
                );
                let write_ret = sys::av_interleaved_write_frame(self.fmt_ctx, self.audio_packet);
                sys::av_packet_unref(self.audio_packet);
                if write_ret < 0 {
                    return Err(format!(
                        "音声パケットの書き込みに失敗しました(code={write_ret})"
                    ));
                }
            }
            Ok(())
        }
    }

    unsafe fn flush_audio(&mut self) -> Result<(), String> {
        unsafe {
            if self.audio_enc_ctx.is_null() {
                return Ok(());
            }
            let remainder = std::mem::take(&mut self.pending_audio);
            if !remainder.is_empty() {
                self.write_audio_frame(&remainder)?;
            }
            let ret = sys::avcodec_send_frame(self.audio_enc_ctx, ptr::null_mut());
            if ret < 0 && ret != sys::AVERROR_EOF {
                return Err(format!(
                    "音声エンコーダのフラッシュに失敗しました(code={ret})"
                ));
            }
            self.drain_audio_packets()
        }
    }

    pub fn finish(mut self) -> Result<(), String> {
        unsafe {
            self.flush_audio()?;
            self.send_frame_and_drain(ptr::null_mut())?;
            if sys::av_write_trailer(self.fmt_ctx) < 0 {
                return Err("av_write_trailer失敗".to_owned());
            }
        }
        Ok(())
    }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        unsafe {
            if !self.audio_packet.is_null() {
                sys::av_packet_free(&mut self.audio_packet);
            }
            if !self.audio_frame.is_null() {
                sys::av_frame_free(&mut self.audio_frame);
            }
            if !self.audio_enc_ctx.is_null() {
                sys::avcodec_free_context(&mut self.audio_enc_ctx);
            }
            if !self.packet.is_null() {
                sys::av_packet_free(&mut self.packet);
            }
            if !self.sws_ctx.is_null() {
                sys::sws_freeContext(self.sws_ctx);
            }
            if !self.frame.is_null() {
                sys::av_frame_free(&mut self.frame);
            }
            if !self.enc_ctx.is_null() {
                sys::avcodec_free_context(&mut self.enc_ctx);
            }
            if self.header_written && !self.fmt_ctx.is_null() {
                if self.output_opened {
                    sys::avio_closep(&mut (*self.fmt_ctx).pb);
                }
                sys::avformat_free_context(self.fmt_ctx);
            }
        }
    }
}

unsafe fn set_dict(dict: &mut *mut sys::AVDictionary, key: &str, value: &str) {
    unsafe {
        let Ok(k) = CString::new(key) else { return };
        let Ok(v) = CString::new(value) else { return };
        sys::av_dict_set(dict, k.as_ptr(), v.as_ptr(), 0);
    }
}

struct AudioStreamSetup {
    enc_ctx: *mut sys::AVCodecContext,
    stream_index: i32,
    time_base: sys::AVRational,
    sample_fmt: sys::AVSampleFormat,
    samples_per_frame: usize,
}

/// MP4 出力へ AAC 音声ストリームを追加する。失敗時は自分のリソースを解放する。
unsafe fn open_audio_stream(
    fmt_ctx: *mut sys::AVFormatContext,
    config: AudioExportConfig,
) -> Result<AudioStreamSetup, String> {
    unsafe {
        let codec = sys::avcodec_find_encoder(sys::AVCodecID::AV_CODEC_ID_AAC);
        if codec.is_null() {
            return Err("AAC エンコーダが見つかりません".to_owned());
        }

        let enc_ctx = sys::avcodec_alloc_context3(codec);
        if enc_ctx.is_null() {
            return Err("音声エンコーダコンテキストの確保に失敗しました".to_owned());
        }

        let channels = config.channels.max(1);
        let time_base = sys::AVRational {
            num: 1,
            den: config.sample_rate.max(1) as i32,
        };
        sys::av_channel_layout_default(&mut (*enc_ctx).ch_layout, channels as i32);
        (*enc_ctx).sample_rate = config.sample_rate.max(1) as i32;
        (*enc_ctx).sample_fmt = pick_sample_fmt(enc_ctx, codec);
        (*enc_ctx).time_base = time_base;
        (*enc_ctx).bit_rate = if config.average_bitrate > 0 {
            config.average_bitrate as i64
        } else {
            192_000
        };
        if (*(*fmt_ctx).oformat).flags & sys::AVFMT_GLOBALHEADER != 0 {
            (*enc_ctx).flags |= sys::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }

        let open_result = sys::avcodec_open2(enc_ctx, codec, ptr::null_mut());
        if open_result < 0 {
            sys::avcodec_free_context(&mut { enc_ctx });
            return Err(format!(
                "音声エンコーダの初期化に失敗しました(code={open_result})"
            ));
        }

        let stream = sys::avformat_new_stream(fmt_ctx, ptr::null());
        if stream.is_null() {
            sys::avcodec_free_context(&mut { enc_ctx });
            return Err("音声ストリームの作成に失敗しました".to_owned());
        }
        let stream_index = (*stream).index;
        if sys::avcodec_parameters_from_context((*stream).codecpar, enc_ctx) < 0 {
            sys::avcodec_free_context(&mut { enc_ctx });
            return Err("音声ストリームパラメータの設定に失敗しました".to_owned());
        }
        (*stream).time_base = time_base;

        let samples_per_frame = (*enc_ctx).frame_size;
        Ok(AudioStreamSetup {
            enc_ctx,
            stream_index,
            time_base,
            sample_fmt: (*enc_ctx).sample_fmt,
            samples_per_frame: if samples_per_frame > 0 {
                samples_per_frame as usize
            } else {
                1024
            },
        })
    }
}

fn pick_sample_fmt(
    enc_ctx: *const sys::AVCodecContext,
    codec: *const sys::AVCodec,
) -> sys::AVSampleFormat {
    unsafe {
        let mut out_ptr: *const std::ffi::c_void = ptr::null();
        let mut out_num: i32 = 0;
        let ret = sys::avcodec_get_supported_config(
            enc_ctx,
            codec,
            sys::AVCodecConfig::AV_CODEC_CONFIG_SAMPLE_FORMAT,
            0,
            &mut out_ptr,
            &mut out_num,
        );
        if ret < 0 || out_ptr.is_null() || out_num <= 0 {
            return sys::AVSampleFormat::AV_SAMPLE_FMT_FLTP;
        }
        let list = out_ptr as *const sys::AVSampleFormat;
        for i in 0..out_num as isize {
            let fmt = *list.offset(i);
            if fmt == sys::AVSampleFormat::AV_SAMPLE_FMT_FLTP
                || fmt == sys::AVSampleFormat::AV_SAMPLE_FMT_FLT
            {
                return fmt;
            }
        }
        *list
    }
}

fn pick_pix_fmt(
    enc_ctx: *const sys::AVCodecContext,
    codec: *const sys::AVCodec,
) -> sys::AVPixelFormat {
    unsafe {
        let mut out_ptr: *const std::ffi::c_void = ptr::null();
        let mut out_num: i32 = 0;
        let ret = sys::avcodec_get_supported_config(
            enc_ctx,
            codec,
            sys::AVCodecConfig::AV_CODEC_CONFIG_PIX_FORMAT,
            0,
            &mut out_ptr,
            &mut out_num,
        );
        if ret < 0 || out_ptr.is_null() || out_num <= 0 {
            return sys::AVPixelFormat::AV_PIX_FMT_YUV420P;
        }
        let list = out_ptr as *const sys::AVPixelFormat;
        for i in 0..out_num as isize {
            let fmt = *list.offset(i);
            if fmt == sys::AVPixelFormat::AV_PIX_FMT_YUV420P
                || fmt == sys::AVPixelFormat::AV_PIX_FMT_NV12
            {
                return fmt;
            }
        }
        *list
    }
}
