use super::client::{SpeakersInfo, VoicevoxClient};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LINE_CHARS: usize = 500;

#[derive(Clone, Debug)]
pub struct VoiceClip {
    pub text: String,
    pub path: PathBuf,
    pub duration_seconds: f64,
}

#[derive(Debug)]
pub enum VoiceJobEvent {
    Progress {
        current: usize,
        total: usize,
    },
    Done {
        clips: Vec<VoiceClip>,
        error: Option<String>,
    },
    Canceled,
}

#[derive(Clone, Debug)]
pub struct VoiceGenerationRequest {
    pub endpoint: String,
    pub style_id: u32,
    pub speed_scale: f32,
    pub pitch_scale: f32,
    pub intonation_scale: f32,
    pub volume_scale: f32,
    pub lines: Vec<String>,
    pub out_dir: PathBuf,
    pub cancel: Arc<AtomicBool>,
}

pub fn split_utterances(text: &str) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let count = trimmed.chars().count();
        if count > MAX_LINE_CHARS {
            return Err(format!(
                "セリフが長すぎます（{count}文字）。1行は{MAX_LINE_CHARS}文字以内にしてください。"
            ));
        }
        lines.push(trimmed.to_owned());
    }
    if lines.is_empty() {
        return Err("セリフが入力されていません。1行に1セリフを入力してください。".to_owned());
    }
    Ok(lines)
}

pub fn spawn_speakers(endpoint: String) -> Receiver<Result<SpeakersInfo, String>> {
    let (tx, rx) = channel();
    let _ = std::thread::Builder::new()
        .name("voicevox-speakers".to_owned())
        .spawn(move || {
            let result = VoicevoxClient::new(&endpoint).and_then(|client| client.fetch_speakers());
            let _ = tx.send(result);
        });
    rx
}

pub fn spawn_generation(request: VoiceGenerationRequest) -> Receiver<VoiceJobEvent> {
    let (tx, rx) = channel();
    let _ = std::thread::Builder::new()
        .name("voicevox-synthesis".to_owned())
        .spawn(move || {
            run_generation(request, &tx);
        });
    rx
}

fn run_generation(request: VoiceGenerationRequest, tx: &std::sync::mpsc::Sender<VoiceJobEvent>) {
    let total = request.lines.len();
    let client = match VoicevoxClient::new(&request.endpoint) {
        Ok(client) => client,
        Err(err) => {
            let _ = tx.send(VoiceJobEvent::Done {
                clips: Vec::new(),
                error: Some(err),
            });
            return;
        }
    };

    if let Err(err) = std::fs::create_dir_all(&request.out_dir) {
        let _ = tx.send(VoiceJobEvent::Done {
            clips: Vec::new(),
            error: Some(format!(
                "音声の保存フォルダを作成できません ({}): {err}",
                request.out_dir.display()
            )),
        });
        return;
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    let mut created = Vec::new();
    let mut clips = Vec::new();

    for (index, line) in request.lines.iter().enumerate() {
        if request.cancel.load(Ordering::Relaxed) {
            cleanup(&created);
            let _ = tx.send(VoiceJobEvent::Canceled);
            return;
        }

        let result = synthesize_line(&client, &request, line, &request.out_dir, stamp, index);
        let (path, duration_seconds) = match result {
            Ok(value) => value,
            Err(err) => {
                cleanup(&created);
                let _ = tx.send(VoiceJobEvent::Done {
                    clips: Vec::new(),
                    error: Some(format!(
                        "{}行目の音声合成に失敗しました（「{}」）: {err}",
                        index + 1,
                        short_text(line)
                    )),
                });
                return;
            }
        };
        created.push(path.clone());

        if request.cancel.load(Ordering::Relaxed) {
            cleanup(&created);
            let _ = tx.send(VoiceJobEvent::Canceled);
            return;
        }

        clips.push(VoiceClip {
            text: line.clone(),
            path,
            duration_seconds,
        });
        let _ = tx.send(VoiceJobEvent::Progress {
            current: index + 1,
            total,
        });
    }

    let _ = tx.send(VoiceJobEvent::Done { clips, error: None });
}

fn synthesize_line(
    client: &VoicevoxClient,
    request: &VoiceGenerationRequest,
    line: &str,
    out_dir: &Path,
    stamp: u128,
    index: usize,
) -> Result<(PathBuf, f64), String> {
    let mut query = client.audio_query(line, request.style_id)?;
    apply_query_parameters(&mut query, request);
    let wav = client.synthesis(&query, request.style_id)?;
    let duration_seconds = wav_duration_seconds(&wav)?;
    let path = unique_wav_path(out_dir, stamp, index);
    std::fs::write(&path, &wav)
        .map_err(|err| format!("WAV を保存できません ({}): {err}", path.display()))?;
    Ok((path, duration_seconds))
}

fn apply_query_parameters(query: &mut Value, request: &VoiceGenerationRequest) {
    let Some(object) = query.as_object_mut() else {
        return;
    };
    object.insert("speedScale".to_owned(), json!(request.speed_scale));
    object.insert("pitchScale".to_owned(), json!(request.pitch_scale));
    object.insert(
        "intonationScale".to_owned(),
        json!(request.intonation_scale),
    );
    object.insert("volumeScale".to_owned(), json!(request.volume_scale));
}

fn unique_wav_path(out_dir: &Path, stamp: u128, index: usize) -> PathBuf {
    let base = format!("voice_{stamp}_{:02}", index + 1);
    let mut path = out_dir.join(format!("{base}.wav"));
    let mut suffix = 2;
    while path.exists() {
        path = out_dir.join(format!("{base}_{suffix}.wav"));
        suffix += 1;
    }
    path
}

fn cleanup(paths: &[PathBuf]) {
    for path in paths {
        let _ = std::fs::remove_file(path);
    }
}

fn short_text(text: &str) -> String {
    const LIMIT: usize = 24;
    let mut result: String = text.chars().take(LIMIT).collect();
    if text.chars().count() > LIMIT {
        result.push('…');
    }
    result
}

/// VOICEVOX が返す RIFF/WAVE（PCM 16-bit）の実尺を秒で返す。
pub fn wav_duration_seconds(bytes: &[u8]) -> Result<f64, String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("WAV 形式ではありません（RIFF/WAVE ヘッダが不正です）".to_owned());
    }
    let mut offset = 12usize;
    let mut byte_rate = None;
    let mut data_len = None;
    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let size = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .map_err(|_| "WAV チャンクサイズの読み取りに失敗しました".to_owned())?,
        ) as usize;
        let body = offset + 8;
        let end = body
            .checked_add(size)
            .ok_or_else(|| "WAV チャンクサイズが不正です".to_owned())?;
        if end > bytes.len() {
            return Err("WAV データが途中で切れています".to_owned());
        }
        if id == b"fmt " && size >= 16 {
            let channels = u16::from_le_bytes([bytes[body + 2], bytes[body + 3]]);
            let sample_rate = u32::from_le_bytes(
                bytes[body + 4..body + 8]
                    .try_into()
                    .map_err(|_| "WAV サンプルレートの読み取りに失敗しました".to_owned())?,
            );
            let rate = u32::from_le_bytes(
                bytes[body + 8..body + 12]
                    .try_into()
                    .map_err(|_| "WAV バイトレートの読み取りに失敗しました".to_owned())?,
            );
            if channels == 0 || sample_rate == 0 || rate == 0 {
                return Err("WAV の音声形式が不正です".to_owned());
            }
            byte_rate = Some(rate);
        } else if id == b"data" {
            data_len = Some(size);
        }
        offset = end + (size & 1);
    }

    let byte_rate = byte_rate.ok_or_else(|| "WAV に fmt チャンクがありません".to_owned())?;
    let data_len = data_len.ok_or_else(|| "WAV に data チャンクがありません".to_owned())?;
    if data_len == 0 {
        return Err("WAV の音声データが空です".to_owned());
    }
    Ok(data_len as f64 / f64::from(byte_rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav_bytes(byte_rate: u32, data_len: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&24_000u32.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        bytes.resize(bytes.len() + data_len as usize, 0);
        bytes
    }

    #[test]
    fn wav_duration_matches_voicevox_output() {
        let bytes = wav_bytes(48_000, 45_056);
        let duration = wav_duration_seconds(&bytes).unwrap();
        assert!((duration - 0.938_666_666_666_666_7).abs() < 1e-9);
    }

    #[test]
    fn wav_rejects_broken_headers() {
        assert!(wav_duration_seconds(b"not a wav").is_err());
        let mut bytes = wav_bytes(48_000, 100);
        bytes.truncate(30);
        assert!(wav_duration_seconds(&bytes).is_err());
    }

    #[test]
    fn split_skips_blank_lines_and_trims() {
        let lines = split_utterances("  こんにちは  \n\n ありがとう\n").unwrap();
        assert_eq!(lines, vec!["こんにちは", "ありがとう"]);
    }

    #[test]
    fn split_rejects_empty_and_too_long() {
        assert!(split_utterances(" \n \n").is_err());
        let long = "あ".repeat(MAX_LINE_CHARS + 1);
        assert!(split_utterances(&long).is_err());
    }

    #[test]
    fn apply_query_parameters_overwrites_scales() {
        let mut query = json!({"speedScale": 1.0, "pitchScale": 0.0});
        let request = VoiceGenerationRequest {
            endpoint: "http://127.0.0.1:50021".to_owned(),
            style_id: 2,
            speed_scale: 1.4,
            pitch_scale: 0.05,
            intonation_scale: 1.2,
            volume_scale: 0.8,
            lines: vec!["テスト".to_owned()],
            out_dir: PathBuf::from("/tmp/voicevox-test"),
            cancel: Arc::new(AtomicBool::new(false)),
        };
        apply_query_parameters(&mut query, &request);
        // f32 を JSON 数値化すると 2 進誤差が出るため、近似比較する。
        for (key, expected) in [
            ("speedScale", 1.4),
            ("pitchScale", 0.05),
            ("intonationScale", 1.2),
            ("volumeScale", 0.8),
        ] {
            let actual = query[key].as_f64().unwrap_or_default();
            assert!(
                (actual - expected).abs() < 1e-6,
                "{key}: {actual} != {expected}"
            );
        }
    }
}
