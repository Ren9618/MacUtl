use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:50021";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeakerStyle {
    pub speaker_name: String,
    pub style_name: String,
    pub style_id: u32,
}

impl SpeakerStyle {
    pub fn label(&self) -> String {
        format!("{}（{}）", self.speaker_name, self.style_name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpeakersInfo {
    pub engine_version: String,
    pub styles: Vec<SpeakerStyle>,
}

#[derive(Clone, Debug)]
pub struct VoicevoxClient {
    base_url: String,
    agent: ureq::Agent,
}

#[derive(Deserialize)]
struct SpeakerResponse {
    name: String,
    #[serde(default)]
    styles: Vec<StyleResponse>,
}

#[derive(Deserialize)]
struct StyleResponse {
    name: String,
    id: u32,
    #[serde(default, rename = "type")]
    kind: String,
}

impl VoicevoxClient {
    pub fn new(endpoint: &str) -> Result<Self, String> {
        let base_url = normalize_endpoint(endpoint)?;
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .build();
        Ok(Self {
            base_url,
            agent: ureq::Agent::new_with_config(config),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn fetch_speakers(&self) -> Result<SpeakersInfo, String> {
        let engine_version = self.version()?;
        let url = format!("{}/speakers", self.base_url);
        let speakers: Vec<SpeakerResponse> = self
            .agent
            .get(&url)
            .call()
            .map_err(|err| request_error(&self.base_url, "話者一覧の取得", err))?
            .body_mut()
            .read_json()
            .map_err(|err| format!("話者一覧の解析に失敗しました: {err}"))?;

        let mut styles = Vec::new();
        for speaker in speakers {
            for style in speaker.styles {
                if !style.kind.is_empty() && style.kind != "talk" {
                    continue;
                }
                styles.push(SpeakerStyle {
                    speaker_name: speaker.name.clone(),
                    style_name: style.name,
                    style_id: style.id,
                });
            }
        }
        styles.sort_by(|a, b| {
            a.speaker_name
                .cmp(&b.speaker_name)
                .then(a.style_name.cmp(&b.style_name))
        });

        if styles.is_empty() {
            return Err(
                "VOICEVOX の話者一覧が空でした。エンジンの状態を確認してください。".to_owned(),
            );
        }

        Ok(SpeakersInfo {
            engine_version,
            styles,
        })
    }

    pub fn version(&self) -> Result<String, String> {
        let url = format!("{}/version", self.base_url);
        self.agent
            .get(&url)
            .call()
            .map_err(|err| request_error(&self.base_url, "バージョン取得", err))?
            .body_mut()
            .read_json()
            .map_err(|err| format!("VOICEVOX のバージョン解析に失敗しました: {err}"))
    }

    pub fn audio_query(&self, text: &str, style_id: u32) -> Result<Value, String> {
        let url = format!("{}/audio_query", self.base_url);
        let mut response = self
            .agent
            .post(&url)
            .query("text", text)
            .query("speaker", style_id.to_string())
            .send_empty()
            .map_err(|err| request_error(&self.base_url, "音声クエリ生成", err))?;
        response
            .body_mut()
            .read_json()
            .map_err(|err| format!("音声クエリの解析に失敗しました: {err}"))
    }

    pub fn synthesis(&self, query: &Value, style_id: u32) -> Result<Vec<u8>, String> {
        let url = format!("{}/synthesis", self.base_url);
        let mut response = self
            .agent
            .post(&url)
            .query("speaker", style_id.to_string())
            .send_json(query)
            .map_err(|err| request_error(&self.base_url, "音声合成", err))?;
        response
            .body_mut()
            .read_to_vec()
            .map_err(|err| format!("合成音声の受信に失敗しました: {err}"))
    }
}

pub fn normalize_endpoint(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("接続先 URL が空です。VOICEVOX のエンジン URL を入力してください。".to_owned());
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err("接続先 URL に空白が含まれています。".to_owned());
    }
    let lowered = trimmed.to_ascii_lowercase();
    let with_scheme = if lowered.starts_with("http://") || lowered.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("http://{trimmed}")
    };
    Ok(with_scheme.trim_end_matches('/').to_owned())
}

pub fn default_endpoint() -> &'static str {
    DEFAULT_ENDPOINT
}

fn request_error(base_url: &str, action: &str, err: ureq::Error) -> String {
    match &err {
        ureq::Error::StatusCode(code) => format!(
            "VOICEVOX API がエラーを返しました（{action}: HTTP {code}）。接続先 {base_url} と話者を確認してください。"
        ),
        ureq::Error::Timeout(_) => format!(
            "VOICEVOX の応答がタイムアウトしました（{action}）。VOICEVOX の状態を確認して再試行してください。"
        ),
        _ => format!(
            "VOICEVOX に接続できません（{action}: {err}）。\n{base_url} で VOICEVOX が起動しているか確認してください。"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_gets_scheme_and_no_trailing_slash() {
        assert_eq!(
            normalize_endpoint(" 127.0.0.1:50021/ ").unwrap(),
            "http://127.0.0.1:50021"
        );
        assert_eq!(
            normalize_endpoint("https://example.com/voicevox/").unwrap(),
            "https://example.com/voicevox"
        );
    }

    #[test]
    fn endpoint_rejects_empty_and_spaces() {
        assert!(normalize_endpoint("").is_err());
        assert!(normalize_endpoint("http://127.0.0.1:50021 bad").is_err());
    }

    #[test]
    fn speaker_label_contains_speaker_and_style() {
        let style = SpeakerStyle {
            speaker_name: "四国めたん".to_owned(),
            style_name: "ノーマル".to_owned(),
            style_id: 2,
        };
        assert_eq!(style.label(), "四国めたん（ノーマル）");
    }
}
