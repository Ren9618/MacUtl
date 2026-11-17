use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::client::default_endpoint;

pub const DEFAULT_STYLE_ID: u32 = 2;
pub const DEFAULT_SPEED_SCALE: f32 = 1.0;
pub const DEFAULT_PITCH_SCALE: f32 = 0.0;
pub const DEFAULT_INTONATION_SCALE: f32 = 1.0;
pub const DEFAULT_VOLUME_SCALE: f32 = 1.0;
pub const DEFAULT_SUBTITLE_FONT_SIZE: f32 = 48.0;
pub const DEFAULT_AUDIO_LAYER: i32 = 0;
pub const DEFAULT_SUBTITLE_LAYER: i32 = 1;
pub const MAX_LAYER: i32 = 127;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct VoicevoxSettings {
    pub endpoint: String,
    pub style_id: u32,
    pub speed_scale: f32,
    pub pitch_scale: f32,
    pub intonation_scale: f32,
    pub volume_scale: f32,
    pub subtitle_enabled: bool,
    pub subtitle_font_size: f32,
    pub audio_layer: i32,
    pub subtitle_layer: i32,
}

impl Default for VoicevoxSettings {
    fn default() -> Self {
        Self {
            endpoint: default_endpoint().to_owned(),
            style_id: DEFAULT_STYLE_ID,
            speed_scale: DEFAULT_SPEED_SCALE,
            pitch_scale: DEFAULT_PITCH_SCALE,
            intonation_scale: DEFAULT_INTONATION_SCALE,
            volume_scale: DEFAULT_VOLUME_SCALE,
            subtitle_enabled: true,
            subtitle_font_size: DEFAULT_SUBTITLE_FONT_SIZE,
            audio_layer: DEFAULT_AUDIO_LAYER,
            subtitle_layer: DEFAULT_SUBTITLE_LAYER,
        }
    }
}

impl VoicevoxSettings {
    pub fn clamp(&mut self) {
        self.speed_scale = self.speed_scale.clamp(0.5, 2.0);
        self.pitch_scale = self.pitch_scale.clamp(-0.15, 0.15);
        self.intonation_scale = self.intonation_scale.clamp(0.0, 2.0);
        self.volume_scale = self.volume_scale.clamp(0.0, 2.0);
        self.subtitle_font_size = self.subtitle_font_size.clamp(12.0, 200.0);
        self.audio_layer = self.audio_layer.clamp(0, MAX_LAYER);
        self.subtitle_layer = self.subtitle_layer.clamp(0, MAX_LAYER);
        self.endpoint = self.endpoint.trim().to_owned();
    }
}

fn settings_path() -> PathBuf {
    crate::project::runtime_data_dir()
        .join("settings")
        .join("voicevox.json")
}

pub fn load_from_disk() -> VoicevoxSettings {
    let path = settings_path();
    let Ok(bytes) = std::fs::read(&path) else {
        return VoicevoxSettings::default();
    };
    match serde_json::from_slice::<VoicevoxSettings>(&bytes) {
        Ok(mut settings) => {
            settings.clamp();
            settings
        }
        Err(err) => {
            eprintln!(
                "[NeoUtl][voicevox] 設定の読み込みに失敗しました ({}): {err}",
                path.display()
            );
            VoicevoxSettings::default()
        }
    }
}

pub fn save_to_disk(settings: &VoicevoxSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            format!(
                "VOICEVOX 設定フォルダを作成できません ({}): {err}",
                parent.display()
            )
        })?;
    }
    let mut clamped = settings.clone();
    clamped.clamp();
    let json = serde_json::to_vec_pretty(&clamped)
        .map_err(|err| format!("VOICEVOX 設定のシリアライズに失敗しました: {err}"))?;
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, json)
        .map_err(|err| format!("VOICEVOX 設定を書き込めません ({}): {err}", temp.display()))?;
    std::fs::rename(&temp, &path)
        .map_err(|err| format!("VOICEVOX 設定を保存できません ({}): {err}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let settings = VoicevoxSettings::default();
        assert_eq!(settings.endpoint, "http://127.0.0.1:50021");
        assert_eq!(settings.style_id, 2);
        assert!(settings.subtitle_enabled);
    }

    #[test]
    fn clamp_limits_values() {
        let mut settings = VoicevoxSettings {
            endpoint: " 127.0.0.1:50021 ".to_owned(),
            style_id: 1,
            speed_scale: 99.0,
            pitch_scale: -9.0,
            intonation_scale: 9.0,
            volume_scale: -1.0,
            subtitle_enabled: true,
            subtitle_font_size: 1.0,
            audio_layer: 999,
            subtitle_layer: -3,
        };
        settings.clamp();
        assert_eq!(settings.endpoint, "127.0.0.1:50021");
        assert_eq!(settings.speed_scale, 2.0);
        assert_eq!(settings.pitch_scale, -0.15);
        assert_eq!(settings.intonation_scale, 2.0);
        assert_eq!(settings.volume_scale, 0.0);
        assert_eq!(settings.subtitle_font_size, 12.0);
        assert_eq!(settings.audio_layer, MAX_LAYER);
        assert_eq!(settings.subtitle_layer, 0);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let settings: VoicevoxSettings =
            serde_json::from_str(r#"{"endpoint":"http://localhost:50021"}"#).unwrap();
        assert_eq!(settings.style_id, DEFAULT_STYLE_ID);
        assert!(settings.subtitle_enabled);
    }
}
