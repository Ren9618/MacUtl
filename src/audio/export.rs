//! 書き出し用のオフライン音声混合。
//!
//! プレビューの [`crate::audio::mixer::AudioMixer`] と同じ配置規則
//! （再生ヘッド位置・音量・パン・ミュート・キーフレーム、WAV 実尺）で、
//! 指定フレーム区間を PCM に落とす。VST プラグインチェーンは
//! リアルタイムホスト前提のため v1 では適用しない（ネイティブ音声エフェクトは適用する）。

use crate::audio::mixer::{ActiveAudioEntity, lerp_sample, process_audio_effects};
use crate::ecs::EcsWorld;
use crate::ecs::systems::get_active_audio_system;
use neoutl_media_api::AudioBuffer;
use neoutl_media_runtime as media;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default)]
pub struct OfflineMixer {
    decode_cache: HashMap<PathBuf, Arc<AudioBuffer>>,
}

impl OfflineMixer {
    pub fn new() -> Self {
        Self::default()
    }

    /// フレーム区間 `[first..=last]` を混合し、インターリーブ f32
    /// （`out_channels` チャンネル、`out_sample_rate` Hz）で返す。
    ///
    /// 出力サンプル数はフレーム境界の四捨五入で決めるため、区間を分割して
    /// 呼んでも累積誤差は生じない（各フレーム窓がタイル状に連続する）。
    pub fn mix_frames(
        &mut self,
        world: &EcsWorld,
        first: i32,
        last: i32,
        out_sample_rate: u32,
        out_channels: u16,
        fps: u32,
    ) -> Vec<f32> {
        let out_rate = f64::from(out_sample_rate.max(1));
        let channels = usize::from(out_channels.max(1));
        let per_frame = out_rate / f64::from(fps.max(1));
        let frame_count = (last - first + 1).max(0) as usize;
        let windows = sample_windows(frame_count, per_frame);
        let total = windows.last().map_or(0, |&(_, end)| end);
        let mut master = vec![0.0f32; total * channels];

        for (index, &(start, end)) in windows.iter().enumerate() {
            let frame = first + index as i32;
            for entity in get_active_audio_system(world, frame) {
                if entity.audio.mute {
                    continue;
                }
                let Some(source) = &entity.media_source else {
                    continue;
                };
                let Some(buffer) = self.decode(&source.path) else {
                    continue;
                };
                mix_entity_window(
                    &buffer,
                    &entity,
                    frame,
                    start,
                    end,
                    out_rate,
                    channels,
                    &mut master,
                );
            }
        }
        master
    }

    fn decode(&mut self, path: &Path) -> Option<Arc<AudioBuffer>> {
        if let Some(buffer) = self.decode_cache.get(path) {
            return Some(buffer.clone());
        }
        match media::loader::decode_audio(path) {
            Ok(buffer) => {
                let buffer = Arc::new(buffer);
                self.decode_cache.insert(path.to_path_buf(), buffer.clone());
                Some(buffer)
            }
            Err(err) => {
                eprintln!(
                    "[NeoUtl][export] 音声のデコードに失敗しました ({}): {err}",
                    path.display()
                );
                None
            }
        }
    }
}

fn mix_entity_window(
    buffer: &AudioBuffer,
    entity: &ActiveAudioEntity,
    frame: i32,
    start: usize,
    end: usize,
    out_rate: f64,
    channels: usize,
    master: &mut [f32],
) {
    let src_channels = buffer.channels.max(1) as usize;
    let src_rate = f64::from(buffer.sample_rate.max(1));
    // プレビューと同じ式: ソース内位置 = (trim 込みフレーム位置) × ソースレート / タイムライン fps
    let base = entity.source_frame as f64 * src_rate / entity.fps.max(1.0);
    let step = src_rate / out_rate;
    let volume = entity.audio.volume;
    let pan = entity.audio.pan.clamp(-1.0, 1.0);
    let gain_l = volume * (1.0 - pan.max(0.0));
    let gain_r = volume * (1.0 + pan.min(0.0));

    let window = end - start;
    let mut chan_l = vec![0.0f32; window];
    let mut chan_r = vec![0.0f32; window];
    for j in 0..window {
        let position = base + j as f64 * step;
        let index = position as usize;
        if index + 1 >= buffer.frame_count() {
            break;
        }
        let frac = (position - index as f64) as f32;
        chan_l[j] = lerp_sample(&buffer.samples, index, src_channels, 0, frac) * gain_l;
        chan_r[j] = lerp_sample(&buffer.samples, index, src_channels, 1, frac) * gain_r;
    }
    process_audio_effects(
        &entity.effects,
        i64::from(frame),
        entity.fps,
        &mut chan_l,
        &mut chan_r,
    );
    for (j, (value_l, value_r)) in chan_l.into_iter().zip(chan_r).enumerate() {
        let offset = (start + j) * channels;
        master[offset] += value_l;
        if channels > 1 {
            master[offset + 1] += value_r;
        }
    }
}

/// フレーム数と 1 フレームあたりのサンプル数から、四捨五入で連続する
/// 出力サンプル窓 `(start, end)` を作る。
fn sample_windows(frame_count: usize, per_frame: f64) -> Vec<(usize, usize)> {
    (0..frame_count)
        .map(|index| {
            let start = (index as f64 * per_frame).round() as usize;
            let end = (((index + 1) as f64 * per_frame).round() as usize).max(start + 1);
            (start, end)
        })
        .collect()
}

/// 区間に配置された音声オブジェクト（`MediaKind::Audio`）が 1 つでもあるか。
/// デコードを行わない軽量判定。
pub fn has_active_audio(world: &EcsWorld, start_frame: i32, end_frame: i32) -> bool {
    (start_frame..=end_frame).any(|frame| {
        get_active_audio_system(world, frame)
            .iter()
            .any(|entity| entity.media_source.is_some())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_tile_without_gaps_or_overlap() {
        for &(frames, rate, fps) in &[
            (391usize, 48_000u32, 30u32),
            (100, 44_100, 60),
            (7, 48_000, 29),
        ] {
            let per_frame = f64::from(rate) / f64::from(fps);
            let windows = sample_windows(frames, per_frame);
            assert_eq!(windows.first().map_or(0, |&(start, _)| start), 0);
            for pair in windows.windows(2) {
                assert_eq!(pair[0].1, pair[1].0, "窓は隙間なく連続すること");
            }
        }
    }

    #[test]
    fn empty_range_yields_no_windows() {
        assert!(sample_windows(0, 1_600.0).is_empty());
    }
}
