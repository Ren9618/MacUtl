use crate::app::state::SharedAppState;
use crate::ecs::components::{MediaSource, TextAlign, TextContent};
use crate::ui::ui_ext::{row_end_style, row_style};
use crate::voicevox::client::SpeakersInfo;
use crate::voicevox::runner::VoiceJobEvent;
use crate::voicevox::settings::{
    MAX_LAYER, VoicevoxSettings, load_from_disk as load_voicevox_settings,
    save_to_disk as save_voicevox_settings,
};
use crate::voicevox::{VoiceClip, VoiceGenerationRequest, split_utterances};
use egui::{Context, Ui};
use egui_taffy::{TuiBuilderLogic, tui};
use elegance::{Button, Checkbox, ProgressBar, Slider, TextArea, TextInput};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

enum SpeakersState {
    Idle,
    Loading,
    Ready(SpeakersInfo),
    Failed(String),
}

pub struct VoiceDialog {
    pub open: bool,
    settings: VoicevoxSettings,
    text: String,
    speakers: SpeakersState,
    speakers_rx: Option<Receiver<Result<SpeakersInfo, String>>>,
    job_rx: Option<Receiver<VoiceJobEvent>>,
    cancel: Option<Arc<AtomicBool>>,
    progress_current: usize,
    progress_total: usize,
    status: String,
    status_is_error: bool,
    /// 直近の挿入開始フレーム（ステータス表示用）。
    inserted_start: i32,
}

impl VoiceDialog {
    pub fn new() -> Self {
        Self {
            open: false,
            settings: load_voicevox_settings(),
            text: String::new(),
            speakers: SpeakersState::Idle,
            speakers_rx: None,
            job_rx: None,
            cancel: None,
            progress_current: 0,
            progress_total: 0,
            status: String::new(),
            status_is_error: false,
            inserted_start: 0,
        }
    }

    pub fn request_open(&mut self) {
        self.open = true;
        if matches!(self.speakers, SpeakersState::Failed(_)) {
            self.speakers = SpeakersState::Idle;
        }
    }

    pub fn save_settings(&self) {
        if let Err(err) = save_voicevox_settings(&self.settings) {
            eprintln!("[NeoUtl][voicevox] {err}");
        }
    }

    fn speakers_loading(&self) -> bool {
        matches!(self.speakers, SpeakersState::Loading)
    }

    fn job_running(&self) -> bool {
        self.job_rx.is_some()
    }

    fn start_speaker_load(&mut self) {
        if self.speakers_loading() {
            return;
        }
        let endpoint = self.settings.endpoint.clone();
        self.speakers = SpeakersState::Loading;
        self.speakers_rx = Some(crate::voicevox::spawn_speakers(endpoint));
    }

    fn poll_speakers(&mut self) {
        let Some(rx) = &self.speakers_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(info)) => {
                if !info
                    .styles
                    .iter()
                    .any(|s| s.style_id == self.settings.style_id)
                    && let Some(first) = info.styles.first()
                {
                    self.settings.style_id = first.style_id;
                }
                self.speakers = SpeakersState::Ready(info);
                self.speakers_rx = None;
            }
            Ok(Err(err)) => {
                self.speakers = SpeakersState::Failed(err);
                self.speakers_rx = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.speakers = SpeakersState::Failed(
                    "話者一覧の取得が中断されました。再試行してください。".to_owned(),
                );
                self.speakers_rx = None;
            }
        }
    }

    fn poll_job(&mut self, state: &SharedAppState) {
        let Some(rx) = &self.job_rx else {
            return;
        };
        loop {
            match rx.try_recv() {
                Ok(VoiceJobEvent::Progress { current, total }) => {
                    self.progress_current = current;
                    self.progress_total = total;
                }
                Ok(VoiceJobEvent::Done { clips, error }) => {
                    self.job_rx = None;
                    self.cancel = None;
                    if let Some(error) = error {
                        self.status = error;
                        self.status_is_error = true;
                    } else if clips.is_empty() {
                        self.status = "音声が生成されませんでした。".to_owned();
                        self.status_is_error = true;
                    } else {
                        match self.insert_clips(state, &clips) {
                            Ok(count) => {
                                self.status = format!(
                                    "再生ヘッド {} から音声 {} 件と字幕を配置しました。",
                                    self.inserted_start, count
                                );
                                self.status_is_error = false;
                                self.text.clear();
                            }
                            Err(err) => {
                                self.status = err;
                                self.status_is_error = true;
                            }
                        }
                    }
                    self.progress_current = 0;
                    self.progress_total = 0;
                    break;
                }
                Ok(VoiceJobEvent::Canceled) => {
                    self.job_rx = None;
                    self.cancel = None;
                    self.progress_current = 0;
                    self.progress_total = 0;
                    self.status = "音声生成をキャンセルしました。".to_owned();
                    self.status_is_error = false;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.job_rx = None;
                    self.cancel = None;
                    self.status = "音声生成スレッドが終了しました。".to_owned();
                    self.status_is_error = true;
                    break;
                }
            }
        }
    }

    fn insert_clips(
        &mut self,
        state: &SharedAppState,
        clips: &[VoiceClip],
    ) -> Result<usize, String> {
        let world_holder = crate::app::state::active_world(state);
        crate::app::state::snapshot_before_edit(state);
        let mut world = world_holder.lock().unwrap();
        let project = world.get_project();
        let fps = project.fps.max(1);
        let start = world.current_frame();
        self.inserted_start = start;
        let audio_kind = crate::objects::loader::ensure_kind_id(neoutl_object_api::AUDIO_STABLE_ID);
        let text_kind = crate::objects::loader::ensure_kind_id(neoutl_object_api::TEXT_STABLE_ID);
        let subtitle_y = -(project.height as f32 * 0.32);
        let mut cursor = start;

        for clip in clips {
            let duration = ((clip.duration_seconds * f64::from(fps)).ceil() as i32).max(1);
            let media = MediaSource {
                path: clip.path.clone(),
                kind: neoutl_media_runtime::MediaKind::Audio,
                trim_in_frame: 0,
            };
            world.add_media_object(
                cursor,
                duration,
                audio_kind,
                self.settings.audio_layer,
                media,
            );

            if self.settings.subtitle_enabled {
                let text = TextContent {
                    text: clip.text.clone(),
                    font_size: self.settings.subtitle_font_size,
                    color: [1.0, 1.0, 1.0, 1.0],
                    font_family_stack: vec![String::new()],
                    bold: false,
                    italic: false,
                    align: TextAlign::Center,
                    line_height: 1.2,
                    outline_width: 4.0,
                    outline_color: [0.0, 0.0, 0.0, 1.0],
                };
                let text_id = world.add_object(
                    cursor,
                    duration,
                    text_kind,
                    self.settings.subtitle_layer,
                    Some(text),
                );
                if let Some(mut transform) = world.get_transform(text_id) {
                    transform.y = subtitle_y;
                    world.set_transform(text_id, transform);
                }
            }
            cursor += duration;
        }

        Ok(clips.len())
    }

    fn start_generation(&mut self, state: &SharedAppState) {
        if self.job_running() {
            return;
        }
        let lines = match split_utterances(&self.text) {
            Ok(lines) => lines,
            Err(err) => {
                self.status = err;
                self.status_is_error = true;
                return;
            }
        };
        let project_dir = {
            let session = state.lock().unwrap();
            session.sessions[session.active].meta.dir.clone()
        };
        let out_dir: PathBuf = project_dir.join("voicevox");
        let cancel = Arc::new(AtomicBool::new(false));
        let request = VoiceGenerationRequest {
            endpoint: self.settings.endpoint.clone(),
            style_id: self.settings.style_id,
            speed_scale: self.settings.speed_scale,
            pitch_scale: self.settings.pitch_scale,
            intonation_scale: self.settings.intonation_scale,
            volume_scale: self.settings.volume_scale,
            lines,
            out_dir,
            cancel: cancel.clone(),
        };
        self.progress_current = 0;
        self.progress_total = request.lines.len();
        self.status.clear();
        self.status_is_error = false;
        self.cancel = Some(cancel);
        self.job_rx = Some(crate::voicevox::spawn_generation(request));
        self.save_settings();
    }

    fn cancel_generation(&mut self) {
        if let Some(cancel) = &self.cancel {
            cancel.store(true, Ordering::Relaxed);
            self.status = "キャンセルしています…".to_owned();
            self.status_is_error = false;
        }
    }

    fn speaker_label(&self) -> String {
        match &self.speakers {
            SpeakersState::Ready(info) => info
                .styles
                .iter()
                .find(|style| style.style_id == self.settings.style_id)
                .map_or_else(|| "話者を選択".to_owned(), |style| style.label()),
            SpeakersState::Loading => "話者一覧を取得中…".to_owned(),
            SpeakersState::Failed(_) => "話者一覧を取得できません".to_owned(),
            SpeakersState::Idle => "未接続".to_owned(),
        }
    }

    fn speaker_combo(&mut self, ui: &mut Ui) {
        let speakers = match &self.speakers {
            SpeakersState::Ready(info) => info.styles.clone(),
            _ => Vec::new(),
        };
        let enabled = !speakers.is_empty() && !self.job_running();
        ui.add_enabled_ui(enabled, |ui| {
            egui::ComboBox::from_id_salt("voicevox_speaker_combo")
                .width(ui.available_width())
                .selected_text(self.speaker_label())
                .show_ui(ui, |ui| {
                    for style in &speakers {
                        if ui
                            .selectable_label(
                                style.style_id == self.settings.style_id,
                                style.label(),
                            )
                            .clicked()
                        {
                            self.settings.style_id = style.style_id;
                        }
                    }
                });
        });
    }

    fn text_area(&mut self, ui: &mut Ui) {
        #[cfg(target_os = "macos")]
        {
            let ime_state_id = ui.id().with("voicevox_text_ime_composition_state");
            let mut preedit_text: String =
                ui.data_mut(|data| data.get_temp(ime_state_id).unwrap_or_default());
            let mut commit_text: Option<String> = None;
            ui.input(|input| {
                for event in &input.events {
                    match event {
                        egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => {
                            preedit_text = text.clone();
                        }
                        egui::Event::Ime(egui::ImeEvent::Commit(text)) => {
                            commit_text = Some(text.clone());
                            preedit_text.clear();
                        }
                        _ => {}
                    }
                }
            });
            if let Some(commit) = commit_text
                && !commit.is_empty()
            {
                self.text.push_str(&commit);
                ui.ctx().request_repaint();
            }
            ui.data_mut(|data| data.insert_temp(ime_state_id, preedit_text.clone()));

            let response = ui.add(
                TextArea::new(&mut self.text)
                    .id_salt("voicevox_text_area_input")
                    .rows(6)
                    .hint("1行に1セリフを入力します（例: こんにちは、ずんだもんです）"),
            );

            if !preedit_text.is_empty() {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(0xff, 0xa5, 0x00), "【変換中】");
                    ui.colored_label(egui::Color32::from_rgb(0x8a, 0xab, 0xff), &preedit_text);
                });
            }

            if response.has_focus() {
                ui.output_mut(|output| {
                    output.ime = Some(egui::output::IMEOutput {
                        purpose: egui::IMEPurpose::Normal,
                        rect: response.rect,
                        cursor_rect: response.rect,
                        should_interrupt_composition: false,
                    });
                });
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            ui.add(
                TextArea::new(&mut self.text)
                    .id_salt("voicevox_text_area_input")
                    .rows(6)
                    .hint("1行に1セリフを入力します（例: こんにちは、ずんだもんです）"),
            );
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut Ui, state: &SharedAppState) {
        if !self.open {
            return;
        }
        self.poll_speakers();
        self.poll_job(state);
        if matches!(self.speakers, SpeakersState::Idle) {
            self.start_speaker_load();
        }
        if self.speakers_loading() || self.job_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        let running = self.job_running();
        let mut close_requested = false;
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("VOICEVOX セリフ入力");
            ui.label(
                "VOICEVOX を起動した状態で、1行に1セリフを入力してください。音声と字幕を再生ヘッド位置から順に配置します。",
            );

            ui.separator();
            let action_height = 92.0;
            let scroll_height = (ui.available_height() - action_height).max(120.0);
            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.label("接続先");
                    let mut reload_speakers = false;
                    tui(ui, ui.id().with("voicevox_endpoint_row"))
                        .style(row_style(8.0))
                        .show(|tui| {
                            tui.ui(|ui| {
                                ui.add_enabled(
                                    !running,
                                    TextInput::new(&mut self.settings.endpoint)
                                        .hint("http://127.0.0.1:50021")
                                        .desired_width(ui.available_width() - 120.0),
                                );
                            });
                            tui.ui(|ui| {
                                if ui
                                    .add_enabled(!running, Button::new("接続 / 話者更新"))
                                    .clicked()
                                {
                                    reload_speakers = true;
                                }
                            });
                        });
                    if reload_speakers {
                        self.speakers = SpeakersState::Idle;
                        self.start_speaker_load();
                    }

                    match &self.speakers {
                        SpeakersState::Loading => {
                            ui.horizontal(|ui| {
                                ui.add(elegance::Spinner::new());
                                ui.label("VOICEVOX に接続して話者一覧を取得しています…");
                            });
                        }
                        SpeakersState::Failed(err) => {
                            ui.colored_label(
                                egui::Color32::from_rgb(0xd9, 0x4f, 0x4f),
                                err,
                            );
                            if ui
                                .add_enabled(!running, Button::new("再試行").outline())
                                .clicked()
                            {
                                self.speakers = SpeakersState::Idle;
                                self.start_speaker_load();
                            }
                        }
                        SpeakersState::Ready(info) => {
                            ui.label(format!(
                                "エンジン {} / 話者 {} 件",
                                info.engine_version,
                                info.styles.len()
                            ));
                        }
                        SpeakersState::Idle => {}
                    }

                    ui.label("話者");
                    self.speaker_combo(ui);

                    ui.separator();
                    ui.label("セリフ（1行 = 1音声）");
                    self.text_area(ui);

                    ui.separator();
                    ui.label("音声パラメータ");
                    tui(ui, ui.id().with("voicevox_param_rows"))
                        .style(row_style(12.0))
                        .show(|tui| {
                            tui.style(crate::ui::ui_ext::col_style(3.0)).add(|tui| {
                                tui.ui(|ui| {
                                    ui.add(
                                        Slider::new(&mut self.settings.speed_scale, 0.5..=2.0)
                                            .label("話速")
                                            .decimals(2),
                                    );
                                });
                                tui.ui(|ui| {
                                    ui.add(
                                        Slider::new(&mut self.settings.pitch_scale, -0.15..=0.15)
                                            .label("音高")
                                            .decimals(2),
                                    );
                                });
                            });
                            tui.style(crate::ui::ui_ext::col_style(3.0)).add(|tui| {
                                tui.ui(|ui| {
                                    ui.add(
                                        Slider::new(
                                            &mut self.settings.intonation_scale,
                                            0.0..=2.0,
                                        )
                                        .label("抑揚")
                                        .decimals(2),
                                    );
                                });
                                tui.ui(|ui| {
                                    ui.add(
                                        Slider::new(&mut self.settings.volume_scale, 0.0..=2.0)
                                            .label("音量")
                                            .decimals(2),
                                    );
                                });
                            });
                        });

                    ui.separator();
                    ui.label("字幕");
                    ui.horizontal(|ui| {
                        ui.add(Checkbox::new(
                            &mut self.settings.subtitle_enabled,
                            "同じ文面の字幕を配置する",
                        ));
                    });
                    ui.add_enabled_ui(self.settings.subtitle_enabled, |ui| {
                        ui.add(
                            Slider::new(&mut self.settings.subtitle_font_size, 12.0..=120.0)
                                .label("字幕フォントサイズ")
                                .decimals(0),
                        );
                    });

                    ui.label("配置レイヤー");
                    ui.horizontal(|ui| {
                        ui.label("音声");
                        ui.add(
                            egui::DragValue::new(&mut self.settings.audio_layer)
                                .range(0..=MAX_LAYER),
                        );
                        ui.label("字幕");
                        ui.add(
                            egui::DragValue::new(&mut self.settings.subtitle_layer)
                                .range(0..=MAX_LAYER),
                        );
                    });
                    ui.label("配置位置: 再生ヘッドから順に追加します。");
                });

            ui.separator();
            if running {
                ui.label(format!(
                    "生成中: {} / {}",
                    self.progress_current, self.progress_total
                ));
                let fraction = if self.progress_total > 0 {
                    self.progress_current as f32 / self.progress_total as f32
                } else {
                    0.0
                };
                ui.add(ProgressBar::new(fraction));
            }

            if !self.status.is_empty() {
                let color = if self.status_is_error {
                    egui::Color32::from_rgb(0xd9, 0x4f, 0x4f)
                } else {
                    egui::Color32::from_rgb(0x4f, 0xd9, 0x7a)
                };
                ui.colored_label(color, &self.status);
            }

            let width = ui.available_width();
            tui(ui, ui.id().with("voicevox_actions_row"))
                .style(row_end_style(8.0, width))
                .show(|tui| {
                    tui.ui(|ui| {
                        let label = if running { "キャンセル" } else { "閉じる" };
                        if ui.add(Button::new(label).outline()).clicked() {
                            if running {
                                self.cancel_generation();
                            } else {
                                close_requested = true;
                            }
                        }
                    });
                    tui.ui(|ui| {
                        if ui
                            .add_enabled(!running, Button::new("生成して配置"))
                            .clicked()
                        {
                            self.start_generation(state);
                        }
                    });
                });
        });

        if close_requested {
            self.save_settings();
            self.open = false;
        }
    }
}
