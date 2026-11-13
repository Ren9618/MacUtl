//! 一画面ワークスペース（Phase 7）。
//!
//! プレビュー／タイムライン／プロパティを 1 つの [`egui_tiles::Tree`] に載せ、
//! 境界ドラッグによるサイズ変更、ヘッダドラッグによる入れ替え、
//! レイアウトの JSON 保存・復元を提供する。
//!
//! レイアウトはプロジェクト形式（`.npb`）や `SystemSettingsResource` には
//! 保存せず、macOS のアプリデータディレクトリに分離して保存する。

use crate::app::state::SharedAppState;
use crate::ui::dialogs::DialogSet;
use crate::ui::preview::PreviewPanel;
use crate::ui::properties::PropertiesPanel;
use crate::ui::timeline::TimelineWindow;
use egui_tiles::{Behavior, Container, Linear, LinearDir, Tile, TileId, Tiles, Tree, UiResponse};
use egui_wgpu::Renderer as EguiRenderer;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

const LAYOUT_VERSION: u32 = 1;
const WORKSPACE_TREE_ID: &str = "macutl_workspace";
const SAVE_DEBOUNCE: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WorkspacePane {
    Preview,
    Timeline,
    Properties,
}

impl WorkspacePane {
    pub const ALL: [Self; 3] = [Self::Preview, Self::Timeline, Self::Properties];

    pub fn title(self) -> String {
        match self {
            Self::Preview => t!("プレビュー"),
            Self::Timeline => t!("タイムライン"),
            Self::Properties => t!("プロパティ"),
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Preview => 0,
            Self::Timeline => 1,
            Self::Properties => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutPreset {
    /// YMM4 風: 上段にプレビュー＋プロパティ、下段にタイムライン全幅。
    Ymm4,
    /// 従来配置に近い: 上段プレビュー、下段にタイムライン＋プロパティ。
    Classic,
    /// ユーザーがドラッグで変更した状態。
    Custom,
}

impl LayoutPreset {
    pub fn label(self) -> String {
        match self {
            Self::Ymm4 => t!("YMM4 風レイアウト"),
            Self::Classic => t!("従来レイアウト"),
            Self::Custom => t!("カスタム"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceLayout {
    pub version: u32,
    pub preset: LayoutPreset,
    pub tree: Tree<WorkspacePane>,
}

impl WorkspaceLayout {
    fn preset(preset: LayoutPreset) -> Self {
        Self {
            version: LAYOUT_VERSION,
            preset,
            tree: build_preset(preset),
        }
    }

    /// 読み込んだレイアウトが安全に描画できるかを検証する。
    ///
    /// - ルートが存在する
    /// - 到達可能なペインがちょうど 3 つ
    /// - 各ペインが重複なく 1 つずつ
    fn is_valid(&self) -> bool {
        if self.version != LAYOUT_VERSION {
            return false;
        }
        let Some(root) = self.tree.root else {
            return false;
        };
        let mut seen = [false; 3];
        let mut visited = 0usize;
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(tile) = self.tree.tiles.get(id) else {
                return false;
            };
            visited = visited.saturating_add(1);
            if visited > 4096 {
                return false;
            }
            match tile {
                Tile::Pane(pane) => {
                    let index = pane.index();
                    if seen[index] {
                        return false;
                    }
                    seen[index] = true;
                }
                Tile::Container(container) => {
                    stack.extend(container.children().copied());
                }
            }
        }
        seen.into_iter().all(|found| found)
    }
}

/// ヘッダのコンテキストメニューなど、ツリー描画中には適用できない操作。
enum WorkspaceRequest {
    Swap(WorkspacePane, WorkspacePane),
    ApplyPreset(LayoutPreset),
}

pub struct WorkspaceState {
    layout: WorkspaceLayout,
    dirty: bool,
    dirty_since: Option<Instant>,
}

impl WorkspaceState {
    pub fn load() -> Self {
        let layout = load_layout().unwrap_or_else(|| WorkspaceLayout::preset(LayoutPreset::Ymm4));
        Self {
            layout,
            dirty: false,
            dirty_since: None,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        preview: &Rc<RefCell<PreviewPanel>>,
        timeline: &Rc<RefCell<TimelineWindow>>,
        properties: &Rc<RefCell<PropertiesPanel>>,
        state: &SharedAppState,
        dialogs: &Rc<RefCell<DialogSet>>,
        renderer: &mut EguiRenderer,
    ) {
        let mut edited = false;
        let mut request = None;
        {
            let mut behavior = WorkspaceBehavior {
                preview: preview.clone(),
                timeline: timeline.clone(),
                properties: properties.clone(),
                state,
                dialogs,
                renderer,
                edited: &mut edited,
                request: &mut request,
            };
            self.layout.tree.ui(&mut behavior, ui);
        }

        if edited {
            self.layout.preset = LayoutPreset::Custom;
            self.mark_dirty();
        }
        if let Some(request) = request {
            match request {
                WorkspaceRequest::Swap(a, b) => self.swap(a, b),
                WorkspaceRequest::ApplyPreset(preset) => self.apply_preset(preset),
            }
        }

        self.flush(false);
    }

    pub fn focus(&mut self, pane: WorkspacePane) {
        let Some(id) = self.layout.tree.tiles.find_pane(&pane) else {
            return;
        };
        self.layout.tree.set_visible(id, true);
        let _ = self.layout.tree.make_active(|tile_id, _| tile_id == id);
        self.mark_dirty();
    }

    pub fn apply_preset(&mut self, preset: LayoutPreset) {
        if preset == LayoutPreset::Custom {
            return;
        }
        self.layout = WorkspaceLayout::preset(preset);
        self.mark_dirty();
    }

    fn swap(&mut self, a: WorkspacePane, b: WorkspacePane) {
        if a == b {
            return;
        }
        let (Some(id_a), Some(id_b)) = (
            self.layout.tree.tiles.find_pane(&a),
            self.layout.tree.tiles.find_pane(&b),
        ) else {
            return;
        };
        let mut swapped = b;
        if let Some(Tile::Pane(pane)) = self.layout.tree.tiles.get_mut(id_a) {
            std::mem::swap(pane, &mut swapped);
        }
        if let Some(Tile::Pane(pane)) = self.layout.tree.tiles.get_mut(id_b) {
            std::mem::swap(pane, &mut swapped);
        }
        self.layout.preset = LayoutPreset::Custom;
        self.mark_dirty();
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.dirty_since = Some(Instant::now());
    }

    /// デバウンス付き保存。`force` は終了時用。
    pub fn flush(&mut self, force: bool) {
        if !self.dirty {
            return;
        }
        let due = force
            || self
                .dirty_since
                .is_none_or(|since| since.elapsed() >= SAVE_DEBOUNCE);
        if due {
            save_layout(&self.layout);
            self.dirty = false;
            self.dirty_since = None;
        }
    }

    #[cfg(test)]
    fn from_layout(layout: WorkspaceLayout) -> Self {
        Self {
            layout,
            dirty: false,
            dirty_since: None,
        }
    }

    #[cfg(test)]
    fn layout(&self) -> &WorkspaceLayout {
        &self.layout
    }
}

struct WorkspaceBehavior<'a> {
    preview: Rc<RefCell<PreviewPanel>>,
    timeline: Rc<RefCell<TimelineWindow>>,
    properties: Rc<RefCell<PropertiesPanel>>,
    state: &'a SharedAppState,
    dialogs: &'a Rc<RefCell<DialogSet>>,
    renderer: &'a mut EguiRenderer,
    edited: &'a mut bool,
    request: &'a mut Option<WorkspaceRequest>,
}

impl WorkspaceBehavior<'_> {
    fn pane_header(&mut self, ui: &mut egui::Ui, pane: WorkspacePane) -> bool {
        let height = 20.0;
        let width = ui.available_width().max(0.0);
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click_and_drag());

        let visuals = ui.visuals().clone();
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, visuals.faint_bg_color);
        painter.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            egui::Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color),
        );
        painter.text(
            rect.left_center() + egui::vec2(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            format!("⋮⋮ {}", pane.title()),
            egui::FontId::proportional(11.0),
            visuals.weak_text_color(),
        );

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }

        response.context_menu(|ui| {
            ui.set_min_width(200.0);
            for other in WorkspacePane::ALL {
                if other == pane {
                    continue;
                }
                if ui.button(format!("{}と入れ替え", other.title())).clicked() {
                    *self.request = Some(WorkspaceRequest::Swap(pane, other));
                    ui.close();
                }
            }
            ui.separator();
            if ui.button(t!("レイアウトを初期化")).clicked() {
                *self.request = Some(WorkspaceRequest::ApplyPreset(LayoutPreset::Ymm4));
                ui.close();
            }
        });

        response.drag_started()
    }
}

impl Behavior<WorkspacePane> for WorkspaceBehavior<'_> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: TileId,
        pane: &mut WorkspacePane,
    ) -> UiResponse {
        if self.pane_header(ui, *pane) {
            return UiResponse::DragStarted;
        }

        match pane {
            WorkspacePane::Preview => {
                self.preview
                    .borrow_mut()
                    .show_content(ui, self.renderer, self.state);
            }
            WorkspacePane::Timeline => {
                let ctx = ui.ctx().clone();
                self.timeline.borrow_mut().show(
                    &ctx,
                    ui,
                    self.state,
                    &self.preview,
                    &(),
                    self.dialogs,
                );
            }
            WorkspacePane::Properties => {
                let ctx = ui.ctx().clone();
                self.properties.borrow_mut().show(&ctx, ui, self.state);
            }
        }

        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &WorkspacePane) -> egui::WidgetText {
        pane.title().into()
    }

    fn min_size(&self) -> f32 {
        96.0
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        4.0
    }

    fn on_edit(&mut self, _edit_action: egui_tiles::EditAction) {
        *self.edited = true;
    }
}

fn insert_linear(
    tiles: &mut Tiles<WorkspacePane>,
    dir: LinearDir,
    children: [TileId; 2],
    fraction: f32,
) -> TileId {
    tiles.insert_new(Tile::Container(Container::Linear(Linear::new_binary(
        dir, children, fraction,
    ))))
}

fn build_preset(preset: LayoutPreset) -> Tree<WorkspacePane> {
    let mut tiles = Tiles::default();
    let preview = tiles.insert_pane(WorkspacePane::Preview);
    let timeline = tiles.insert_pane(WorkspacePane::Timeline);
    let properties = tiles.insert_pane(WorkspacePane::Properties);

    let root = match preset {
        LayoutPreset::Classic => {
            let bottom = insert_linear(
                &mut tiles,
                LinearDir::Horizontal,
                [timeline, properties],
                0.68,
            );
            insert_linear(&mut tiles, LinearDir::Vertical, [preview, bottom], 0.55)
        }
        LayoutPreset::Ymm4 | LayoutPreset::Custom => {
            let top = insert_linear(
                &mut tiles,
                LinearDir::Horizontal,
                [preview, properties],
                0.58,
            );
            insert_linear(&mut tiles, LinearDir::Vertical, [top, timeline], 0.58)
        }
    };

    Tree::new(WORKSPACE_TREE_ID, root, tiles)
}

pub fn layout_path() -> PathBuf {
    crate::project::runtime_data_dir()
        .join("settings")
        .join("workspace-layout.json")
}

fn load_layout() -> Option<WorkspaceLayout> {
    let bytes = std::fs::read(layout_path()).ok()?;
    let layout: WorkspaceLayout = serde_json::from_slice(&bytes).ok()?;
    layout.is_valid().then_some(layout)
}

fn save_layout(layout: &WorkspaceLayout) {
    let path = layout_path();
    if let Some(parent) = path.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return;
    }
    let Ok(bytes) = serde_json::to_vec_pretty(layout) else {
        return;
    };
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, bytes).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_contain_all_panes_once() {
        for preset in [LayoutPreset::Ymm4, LayoutPreset::Classic] {
            let layout = WorkspaceLayout::preset(preset);
            assert!(layout.is_valid(), "{preset:?} が不正");
            let panes: Vec<_> = layout
                .tree
                .tiles
                .tiles()
                .filter_map(|tile| match tile {
                    Tile::Pane(pane) => Some(*pane),
                    Tile::Container(_) => None,
                })
                .collect();
            assert_eq!(panes.len(), 3);
            for pane in WorkspacePane::ALL {
                assert_eq!(panes.iter().filter(|&&p| p == pane).count(), 1);
            }
        }
    }

    #[test]
    fn layout_json_roundtrip() {
        let layout = WorkspaceLayout::preset(LayoutPreset::Ymm4);
        let json = serde_json::to_string(&layout).expect("serialize");
        let restored: WorkspaceLayout = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.is_valid());
        assert_eq!(layout, restored);
    }

    #[test]
    fn invalid_layout_is_rejected() {
        let mut tiles = Tiles::default();
        let preview = tiles.insert_pane(WorkspacePane::Preview);
        let root = tiles.insert_pane(WorkspacePane::Timeline);
        let _ = preview;
        let layout = WorkspaceLayout {
            version: LAYOUT_VERSION,
            preset: LayoutPreset::Custom,
            tree: Tree::new(WORKSPACE_TREE_ID, root, tiles),
        };
        assert!(!layout.is_valid());
    }

    #[test]
    fn focus_reactivates_tabbed_pane() {
        let mut tiles = Tiles::default();
        let preview = tiles.insert_pane(WorkspacePane::Preview);
        let timeline = tiles.insert_pane(WorkspacePane::Timeline);
        let properties = tiles.insert_pane(WorkspacePane::Properties);
        let root = tiles.insert_tab_tile(vec![preview, timeline, properties]);
        let layout = WorkspaceLayout {
            version: LAYOUT_VERSION,
            preset: LayoutPreset::Custom,
            tree: Tree::new(WORKSPACE_TREE_ID, root, tiles),
        };
        let mut state = WorkspaceState::from_layout(layout);
        state.focus(WorkspacePane::Properties);
        let id = state
            .layout()
            .tree
            .tiles
            .find_pane(&WorkspacePane::Properties)
            .expect("properties pane");
        assert!(state.layout().tree.is_visible(id));
        let active = state
            .layout()
            .tree
            .tiles
            .get_container(root)
            .and_then(|container| match container {
                Container::Tabs(tabs) => tabs.active,
                _ => None,
            });
        assert_eq!(active, Some(id));
    }

    #[test]
    fn swap_exchanges_pane_payloads() {
        let mut state = WorkspaceState::from_layout(WorkspaceLayout::preset(LayoutPreset::Ymm4));
        let preview_id = state
            .layout()
            .tree
            .tiles
            .find_pane(&WorkspacePane::Preview)
            .expect("preview pane");
        let properties_id = state
            .layout()
            .tree
            .tiles
            .find_pane(&WorkspacePane::Properties)
            .expect("properties pane");

        state.swap(WorkspacePane::Preview, WorkspacePane::Properties);
        assert!(state.layout().is_valid());
        assert_eq!(state.layout().preset, LayoutPreset::Custom);

        // 元の位置のタイル ID は維持され、中身だけ入れ替わる。
        assert!(matches!(
            state.layout().tree.tiles.get(preview_id),
            Some(Tile::Pane(WorkspacePane::Properties))
        ));
        assert!(matches!(
            state.layout().tree.tiles.get(properties_id),
            Some(Tile::Pane(WorkspacePane::Preview))
        ));
    }
}
