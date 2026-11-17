use crate::app::state::{self as app_state, SharedAppState};
use prost::Message;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportCodec {
    H264,
    H265,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncoderBackend {
    Auto,
    GpuVideo,
    Gstreamer,
}

pub struct ExportJob {
    pub output_path: PathBuf,
    pub codec: ExportCodec,
    pub backend: EncoderBackend,
    pub average_bitrate: u32,
    pub max_bitrate: u32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub progress: Option<Box<dyn FnMut(i32, i32) + Send>>,
    pub cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

#[derive(Clone, Debug)]
pub struct ExportPreset {
    pub name: String,
    pub codec: ExportCodec,
    pub backend: EncoderBackend,
    pub average_bitrate: u32,
    pub max_bitrate: u32,
    pub container_ext: String,
}

pub fn default_export_presets() -> Vec<ExportPreset> {
    vec![
        ExportPreset {
            name: "H.264 標準 (MP4)".into(),
            codec: ExportCodec::H264,
            backend: EncoderBackend::Auto,
            average_bitrate: 8_000_000,
            max_bitrate: 12_000_000,
            container_ext: "mp4".into(),
        },
        ExportPreset {
            name: "H.265 標準 (MP4)".into(),
            codec: ExportCodec::H265,
            backend: EncoderBackend::Auto,
            average_bitrate: 6_000_000,
            max_bitrate: 10_000_000,
            container_ext: "mp4".into(),
        },
    ]
}

fn presets_path() -> PathBuf {
    crate::project::projects_dir()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("settings")
        .join("export_presets.npb")
}

impl From<&ExportPreset> for neoutl_schema::ExportPreset {
    fn from(value: &ExportPreset) -> Self {
        Self {
            name: value.name.clone(),
            codec: match value.codec {
                ExportCodec::H264 => neoutl_schema::ExportCodec::H264 as i32,
                ExportCodec::H265 => neoutl_schema::ExportCodec::H265 as i32,
            },
            backend: match value.backend {
                EncoderBackend::Auto => neoutl_schema::EncoderBackend::Auto as i32,
                EncoderBackend::GpuVideo => neoutl_schema::EncoderBackend::GpuVideo as i32,
                EncoderBackend::Gstreamer => neoutl_schema::EncoderBackend::Gstreamer as i32,
            },
            average_bitrate: value.average_bitrate,
            max_bitrate: value.max_bitrate,
            container_ext: value.container_ext.clone(),
        }
    }
}

impl TryFrom<neoutl_schema::ExportPreset> for ExportPreset {
    type Error = String;

    fn try_from(value: neoutl_schema::ExportPreset) -> Result<Self, Self::Error> {
        let name = value.name.clone();
        let container_ext = value.container_ext.clone();
        Ok(Self {
            name,
            codec: match value.codec() {
                neoutl_schema::ExportCodec::H264 => ExportCodec::H264,
                neoutl_schema::ExportCodec::H265 => ExportCodec::H265,
            },
            backend: match value.backend() {
                neoutl_schema::EncoderBackend::Auto => EncoderBackend::Auto,
                neoutl_schema::EncoderBackend::GpuVideo => EncoderBackend::GpuVideo,
                neoutl_schema::EncoderBackend::Gstreamer => EncoderBackend::Gstreamer,
            },
            average_bitrate: value.average_bitrate,
            max_bitrate: value.max_bitrate,
            container_ext,
        })
    }
}

pub fn load_export_presets() -> Vec<ExportPreset> {
    let path = presets_path();
    let Ok(bytes) = std::fs::read(path) else {
        return default_export_presets();
    };
    let Ok(file) = neoutl_schema::ExportPresetFile::decode(bytes.as_slice()) else {
        return default_export_presets();
    };
    file.presets
        .iter()
        .map(|p| crate::project::schema::SchemaContract::from_schema(p))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|_| default_export_presets())
}

pub fn save_export_presets(presets: &[ExportPreset]) -> Result<(), String> {
    let path = presets_path();
    std::fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))
        .map_err(|e| e.to_string())?;
    let file = neoutl_schema::ExportPresetFile {
        presets: presets
            .iter()
            .map(crate::project::schema::SchemaContract::to_schema)
            .collect(),
    };
    std::fs::write(path, file.encode_to_vec()).map_err(|e| e.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueState {
    Idle,
    Running,
    CancelRequested,
    Completed,
}

pub struct QueuedJob {
    pub job: ExportJob,
    pub project_dir: PathBuf,
    pub id: u64,
}
pub struct JobHandle {
    pub id: u64,
    pub cancel: Arc<std::sync::atomic::AtomicBool>,
}

struct QueueInner {
    jobs: VecDeque<QueuedJob>,
    current: Option<JobHandle>,
    state: QueueState,
    next_id: u64,
}

#[derive(Clone)]
pub struct RenderQueue {
    inner: Arc<Mutex<QueueInner>>,
}

impl RenderQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(QueueInner {
                jobs: VecDeque::new(),
                current: None,
                state: QueueState::Idle,
                next_id: 1,
            })),
        }
    }
    pub fn enqueue(&self, mut job: ExportJob, project_dir: PathBuf) -> u64 {
        let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
        job.cancel = Some(cancel);
        let mut q = self.inner.lock().unwrap();
        let id = q.next_id;
        q.next_id += 1;
        q.jobs.push_back(QueuedJob {
            job,
            project_dir,
            id,
        });
        id
    }
    pub fn pending_count(&self) -> usize {
        self.inner.lock().unwrap().jobs.len()
    }
    pub fn state(&self) -> QueueState {
        self.inner.lock().unwrap().state
    }
    pub fn cancel_current(&self) {
        let mut q = self.inner.lock().unwrap();
        if let Some(h) = &q.current {
            eprintln!("[NeoUtl][export] キャンセル要求 job_id={}", h.id);
            h.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            q.state = QueueState::CancelRequested;
        }
    }
    pub fn start(&self, state: SharedAppState) {
        {
            let q = self.inner.lock().unwrap();
            if q.state == QueueState::Running || q.state == QueueState::CancelRequested {
                return;
            }
        }
        let inner = self.inner.clone();
        std::thread::spawn(move || {
            loop {
                let next = {
                    let mut q = inner.lock().unwrap();
                    let Some(item) = q.jobs.pop_front() else {
                        q.state = QueueState::Completed;
                        q.current = None;
                        break;
                    };
                    let cancel = item.job.cancel.clone().unwrap();
                    q.current = Some(JobHandle {
                        id: item.id,
                        cancel,
                    });
                    q.state = QueueState::Running;
                    item
                };
                if let Err(error) = app_state::activate_session_by_dir(&state, &next.project_dir) {
                    eprintln!("[NeoUtl][export] キュージョブをスキップ: {error}");
                    let mut q = inner.lock().unwrap();
                    q.current = None;
                    continue;
                }
                let _ = run(&state, next.job);
                let mut q = inner.lock().unwrap();
                q.current = None;
            }
        });
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// 書き出し区間に音声オブジェクトがあれば、AAC トラックの設定を返す。
/// プロジェクトの標本レートが AAC 非対応のときは 48000 Hz に置き換える。
fn audio_export_config(
    world_holder: &Arc<Mutex<crate::ecs::EcsWorld>>,
    start_frame: i32,
    end_frame: i32,
) -> Option<neo_media_ffmpeg::AudioExportConfig> {
    let world = world_holder.lock().unwrap();
    if !crate::audio::export::has_active_audio(&world, start_frame, end_frame) {
        return None;
    }
    let project = world.get_project();
    let sample_rate = if project.audio_sample_rate > 0
        && neo_media_ffmpeg::is_aac_sample_rate(project.audio_sample_rate)
    {
        project.audio_sample_rate
    } else {
        eprintln!(
            "[NeoUtl][export] プロジェクトの標本レート {} Hz は AAC 非対応のため 48000 Hz で書き出します",
            project.audio_sample_rate
        );
        48_000
    };
    Some(neo_media_ffmpeg::AudioExportConfig {
        sample_rate,
        channels: 2,
        average_bitrate: 192_000,
    })
}

/// タイムライン区間の音声をチャンクごとに混合してエンコーダへ積む。
fn mix_timeline_audio(
    world_holder: &Arc<Mutex<crate::ecs::EcsWorld>>,
    encoder: &mut neo_media_ffmpeg::VideoEncoder,
    job: &ExportJob,
    config: neo_media_ffmpeg::AudioExportConfig,
    fps: u32,
) -> Result<(), String> {
    let canceled = || {
        job.cancel
            .as_ref()
            .is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed))
    };
    let mut mixer = crate::audio::export::OfflineMixer::new();
    const CHUNK_FRAMES: i32 = 64;
    let mut first = job.start_frame;
    while first <= job.end_frame {
        if canceled() {
            return Err("キャンセルされました".to_owned());
        }
        let last = first
            .saturating_add(CHUNK_FRAMES)
            .saturating_sub(1)
            .min(job.end_frame);
        let chunk = {
            let world = world_holder.lock().unwrap();
            mixer.mix_frames(
                &world,
                first,
                last,
                config.sample_rate,
                config.channels,
                fps,
            )
        };
        encoder.push_audio_interleaved(&chunk)?;
        first = last.saturating_add(1);
    }
    Ok(())
}

pub fn run(state: &SharedAppState, mut job: ExportJob) -> Result<(), String> {
    if job.end_frame < job.start_frame {
        return Err(format!(
            "終了フレーム({})が開始フレーム({})より前です",
            job.end_frame, job.start_frame
        ));
    }
    if let Some(parent) = job.output_path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let is_canceled = |job: &ExportJob| {
        job.cancel
            .as_ref()
            .is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed))
    };
    if is_canceled(&job) {
        return Err("キャンセル済み".to_owned());
    }

    let world_holder = app_state::active_world(state);
    let engine_holder = app_state::active_engine(state);
    let (width, height, fps) = {
        let world = world_holder.lock().unwrap();
        let project = world.get_project();
        (project.width, project.height, project.fps.max(1))
    };

    let audio_config = audio_export_config(&world_holder, job.start_frame, job.end_frame);

    let encoder_codec = match job.codec {
        ExportCodec::H264 => neo_media_ffmpeg::EncoderCodec::H264,
        ExportCodec::H265 => neo_media_ffmpeg::EncoderCodec::H265,
    };
    let encoder_backend = match job.backend {
        EncoderBackend::Auto => neo_media_ffmpeg::EncoderBackend::Auto,
        EncoderBackend::GpuVideo => neo_media_ffmpeg::EncoderBackend::GpuVideo,
        EncoderBackend::Gstreamer => neo_media_ffmpeg::EncoderBackend::Software,
    };
    let mut encoder = neo_media_ffmpeg::VideoEncoder::open(
        &job.output_path,
        neo_media_ffmpeg::EncoderConfig {
            width,
            height,
            fps,
            average_bitrate: job.average_bitrate,
            max_bitrate: job.max_bitrate,
            codec: encoder_codec,
            backend: encoder_backend,
            audio: audio_config,
        },
    )?;
    eprintln!(
        "[NeoUtl][export] エンコーダ選択: {} (HW={})",
        encoder.encoder_name,
        neo_media_ffmpeg::is_hw_encoder_name(&encoder.encoder_name)
    );
    if let Some(audio_config) = &audio_config {
        mix_timeline_audio(&world_holder, &mut encoder, &job, *audio_config, fps)?;
        eprintln!(
            "[NeoUtl][export] 音声トラック混合完了 ({} Hz / {} ch)",
            audio_config.sample_rate, audio_config.channels
        );
    }

    for frame in job.start_frame..=job.end_frame {
        if is_canceled(&job) {
            return Err("キャンセルされました".to_owned());
        }
        let rgba8 = {
            let mut world = world_holder.lock().unwrap();
            world.set_current_frame(frame);
            let (active, captured) = crate::ecs::systems::get_active_objects_system(&world);
            let project = world.get_project();
            let mut engine_lock = engine_holder.lock().unwrap();
            let Some(engine) = engine_lock.as_mut() else {
                return Err(
                    "レンダーエンジン未初期化です。プレビューを一度表示してから書き出してください"
                        .to_owned(),
                );
            };
            if engine.render_width != width || engine.render_height != height {
                engine.resize_render_target(width, height);
            }
            engine.render(&world, &active, &captured, &project);
            engine.read_frame_rgba8()
        };
        encoder
            .encode_rgba8(&rgba8)
            .map_err(|e| format!("エンコード失敗(frame={frame}): {e}"))?;
        if let Some(progress) = &mut job.progress {
            progress(frame, job.end_frame);
        }
    }

    encoder.finish()
}
