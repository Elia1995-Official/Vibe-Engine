use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::{egui, App};
use egui::{Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Vec2};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

// ─── PALETTE ─────────────────────────────────────────────────────────────────
// Background layers
const C_BG0: Color32 = Color32::from_rgb(13, 15, 21);   // deepest
const C_BG1: Color32 = Color32::from_rgb(19, 21, 30);   // panels
const C_BG2: Color32 = Color32::from_rgb(25, 28, 42);   // cards / headers
const C_BG3: Color32 = Color32::from_rgb(33, 37, 56);   // hover
const C_BORDER: Color32 = Color32::from_rgb(42, 47, 70); // borders
const C_SEP: Color32 = Color32::from_rgb(34, 38, 58);   // separators
// Text
const C_TEXT1: Color32 = Color32::from_rgb(222, 228, 250);
const C_TEXT2: Color32 = Color32::from_rgb(140, 152, 188);
const C_TEXT3: Color32 = Color32::from_rgb(82, 92, 128);
// Accents
const C_BLUE: Color32 = Color32::from_rgb(68, 126, 242);
const C_GREEN: Color32 = Color32::from_rgb(70, 196, 118);
const C_AMBER: Color32 = Color32::from_rgb(228, 178, 62);
const C_RED: Color32 = Color32::from_rgb(214, 68, 68);
const C_PURPLE: Color32 = Color32::from_rgb(148, 104, 238);
const C_TEAL: Color32 = Color32::from_rgb(60, 188, 188);
// Selection
const C_SEL: Color32 = Color32::from_rgb(48, 78, 165);

// ─── MAIN ────────────────────────────────────────────────────────────────────
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vibe Editor")
            .with_inner_size([1600.0, 960.0])
            .with_min_inner_size([1100.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vibe Editor",
        options,
        Box::new(|cc| {
            let recent: Vec<String> = cc
                .storage
                .and_then(|s| {
                    serde_json::from_str(
                        &s.get_string("recent_files").unwrap_or_default(),
                    )
                    .ok()
                })
                .unwrap_or_default();
            Ok(Box::new(EditorApp::new(recent)))
        }),
    )
}

// ─── ENUMS ───────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolMode { Select, Move, Rotate, Scale }


#[derive(Clone, Copy, PartialEq, Eq)]
enum BottomTab { Assets, Console }

// ─── DATA STRUCTURES ─────────────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Clone)]
struct ProjectSettings {
    startup_scene: String,
    gravity: f32,
    fixed_timestep: f32,
    lighting_quality: String,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            startup_scene: "Main Scene".to_string(),
            gravity: 9.81,
            fixed_timestep: 1.0 / 60.0,
            lighting_quality: "High".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ProjectDocument {
    name: String,
    author: String,
    engine_version: String,
    created_unix: u64,
    modified_unix: u64,
    notes: String,
    settings: ProjectSettings,
    scenes: Vec<SceneDocument>,
    assets: Vec<AssetDocument>,
}

impl Default for ProjectDocument {
    fn default() -> Self {
        let now = unix_now();
        Self {
            name: "New Vibe Project".to_string(),
            author: "Unknown".to_string(),
            engine_version: "vibe-engine-0.1.0".to_string(),
            created_unix: now,
            modified_unix: now,
            notes: String::new(),
            settings: ProjectSettings::default(),
            scenes: vec![SceneDocument::default()],
            assets: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct SceneDocument {
    id: u64,
    name: String,
    ambient_light: [f32; 3],
    clear_color: [f32; 3],
    grid_size: f32,
    entities: Vec<EntityDocument>,
}

impl Default for SceneDocument {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Main Scene".to_string(),
            ambient_light: [0.22, 0.24, 0.27],
            clear_color: [0.06, 0.07, 0.09],
            grid_size: 1.0,
            entities: vec![EntityDocument::default_camera()],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct EntitySnapshot {
    name: String,
    enabled: bool,
    tags: String,
    transform: TransformComponent,
    render: Option<RenderComponent>,
    collider: Option<ColliderComponent>,
    script: Option<ScriptComponent>,
}

impl From<&EntityDocument> for EntitySnapshot {
    fn from(e: &EntityDocument) -> Self {
        Self {
            name: e.name.clone(),
            enabled: e.enabled,
            tags: e.tags.clone(),
            transform: e.transform.clone(),
            render: e.render.clone(),
            collider: e.collider.clone(),
            script: e.script.clone(),
        }
    }
}

impl EntitySnapshot {
    fn apply(&self, e: &mut EntityDocument) {
        e.name.clone_from(&self.name);
        e.enabled = self.enabled;
        e.tags.clone_from(&self.tags);
        e.transform = self.transform.clone();
        e.render.clone_from(&self.render);
        e.collider.clone_from(&self.collider);
        e.script.clone_from(&self.script);
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct EntityDocument {
    id: u64,
    name: String,
    enabled: bool,
    tags: String,
    transform: TransformComponent,
    render: Option<RenderComponent>,
    collider: Option<ColliderComponent>,
    script: Option<ScriptComponent>,
}

impl EntityDocument {
    fn default_camera() -> Self {
        Self {
            id: 2,
            name: "Camera".to_string(),
            enabled: true,
            tags: "camera".to_string(),
            transform: TransformComponent {
                position: [0.0, 2.0, 7.0],
                rotation: [-10.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
            render: None,
            collider: None,
            script: Some(ScriptComponent {
                script_path: "scripts/camera_controller.rs".to_string(),
                entry: "update".to_string(),
            }),
        }
    }

    fn default_cube(id: u64) -> Self {
        Self {
            id,
            name: format!("Entity {}", id),
            enabled: true,
            tags: "gameplay".to_string(),
            transform: TransformComponent::default(),
            render: Some(RenderComponent::default()),
            collider: Some(ColliderComponent::default()),
            script: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct TransformComponent {
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum RenderKind { Mesh, Sprite, Light }

impl RenderKind {
    fn label(self) -> &'static str {
        match self { RenderKind::Mesh => "Mesh", RenderKind::Sprite => "Sprite", RenderKind::Light => "Light" }
    }
    fn all() -> [RenderKind; 3] { [RenderKind::Mesh, RenderKind::Sprite, RenderKind::Light] }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct RenderComponent {
    kind: RenderKind,
    mesh: String,
    material: String,
    color: [f32; 4],
    layer: i32,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            kind: RenderKind::Mesh,
            mesh: "meshes/cube.glb".to_string(),
            material: "materials/default.mat".to_string(),
            color: [0.35, 0.68, 0.95, 1.0],
            layer: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum ColliderShape { Box, Sphere, Capsule }

impl ColliderShape {
    fn label(self) -> &'static str {
        match self { ColliderShape::Box => "Box", ColliderShape::Sphere => "Sphere", ColliderShape::Capsule => "Capsule" }
    }
    fn all() -> [ColliderShape; 3] { [ColliderShape::Box, ColliderShape::Sphere, ColliderShape::Capsule] }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct ColliderComponent {
    shape: ColliderShape,
    size: [f32; 3],
    is_trigger: bool,
}

impl Default for ColliderComponent {
    fn default() -> Self {
        Self { shape: ColliderShape::Box, size: [1.0, 1.0, 1.0], is_trigger: false }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct ScriptComponent {
    script_path: String,
    entry: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum AssetKind { Texture, Mesh, Material, Audio, Script, Other }

impl AssetKind {
    fn label(self) -> &'static str {
        match self {
            AssetKind::Texture => "Texture", AssetKind::Mesh => "Mesh",
            AssetKind::Material => "Material", AssetKind::Audio => "Audio",
            AssetKind::Script => "Script", AssetKind::Other => "Other",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            AssetKind::Texture => "🖼", AssetKind::Mesh => "⬡",
            AssetKind::Material => "◈", AssetKind::Audio => "♪",
            AssetKind::Script => "⌥", AssetKind::Other => "○",
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct AssetDocument {
    id: u64,
    name: String,
    path: String,
    kind: AssetKind,
}

#[derive(Serialize)]
struct RuntimeExport {
    project_name: String,
    exported_unix: u64,
    scenes: Vec<RuntimeScene>,
}

#[derive(Serialize)]
struct RuntimeScene {
    name: String,
    ambient_light: [f32; 3],
    clear_color: [f32; 3],
    entities: Vec<RuntimeEntity>,
}

#[derive(Serialize)]
struct RuntimeEntity {
    name: String,
    enabled: bool,
    tags: Vec<String>,
    transform: TransformComponent,
    render: Option<RenderComponent>,
    collider: Option<ColliderComponent>,
    script: Option<ScriptComponent>,
}

#[derive(Clone)]
enum UndoCommand {
    AddEntity { scene_index: usize, entity: EntityDocument },
    RemoveEntity { scene_index: usize, entity: EntityDocument, index: usize },
    ModifyEntity { scene_index: usize, entity_id: u64, before: EntitySnapshot, after: EntitySnapshot },
    AddScene { scene: SceneDocument, index: usize },
    RemoveScene { scene: SceneDocument, index: usize },
    DuplicateEntity { scene_index: usize, entity: EntityDocument },
}

#[derive(Clone)]
enum SaveBeforeAction { NewProject, OpenProject, Welcome }

// ─── APP STATE ───────────────────────────────────────────────────────────────
struct EditorApp {
    project: ProjectDocument,
    project_path: Option<PathBuf>,
    selected_scene: usize,
    selected_entity: Option<u64>,
    selected_asset: Option<u64>,
    hierarchy_filter: String,
    asset_filter: String,
    tool_mode: ToolMode,
    viewport_zoom: f32,
    viewport_pan: Vec2,
    snap_enabled: bool,
    snap_size: f32,
    show_grid: bool,
    play_mode: bool,
    dirty: bool,
    status_line: String,
    console: Vec<String>,
    next_id: u64,
    undo_stack: Vec<UndoCommand>,
    redo_stack: Vec<UndoCommand>,

    show_welcome: bool,
    recent_files: Vec<PathBuf>,
    confirm_delete_entity: Option<u64>,
    confirm_delete_scene: Option<usize>,
    confirm_new_project: bool,
    confirm_quit: bool,
    save_before_action: Option<SaveBeforeAction>,
    rename_target: Option<u64>,
    rename_buffer: String,
    entity_before_edit: Option<EntitySnapshot>,

    bottom_tab: BottomTab,
}

impl EditorApp {
    fn new(recent_paths: Vec<String>) -> Self {
        Self {
            project: ProjectDocument::default(),
            project_path: None,
            selected_scene: 0,
            selected_entity: None,
            selected_asset: None,
            hierarchy_filter: String::new(),
            asset_filter: String::new(),

            tool_mode: ToolMode::Select,
            viewport_zoom: 48.0,
            viewport_pan: Vec2::ZERO,
            snap_enabled: true,
            snap_size: 0.5,
            show_grid: true,
            play_mode: false,
            dirty: false,
            status_line: "Ready".to_string(),
            console: vec!["Editor started".to_string()],
            next_id: 10,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            show_welcome: true,
            recent_files: recent_paths.into_iter().map(PathBuf::from).collect(),
            confirm_delete_entity: None,
            confirm_delete_scene: None,
            confirm_new_project: false,
            confirm_quit: false,
            save_before_action: None,
            rename_target: None,
            rename_buffer: String::new(),
            entity_before_edit: None,
            bottom_tab: BottomTab::Console,
        }
    }

    fn save_recent(&self, storage: &mut dyn eframe::Storage) {
        let paths: Vec<String> = self
            .recent_files
            .iter()
            .filter_map(|p| p.to_str().map(String::from))
            .collect();
        storage.set_string("recent_files", serde_json::to_string(&paths).unwrap_or_default());
    }
}

// ─── eframe::App ─────────────────────────────────────────────────────────────
impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        configure_visuals(ctx);
        self.handle_keyboard(ctx);
        self.commit_pending_entity_edit();
        self.ensure_valid_selection();

        if self.show_welcome {
            self.show_welcome_screen(ctx);
        } else {
            self.show_toolbar(ctx);
            self.show_hierarchy_panel(ctx);
            self.show_inspector_panel(ctx);
            self.show_bottom_panel(ctx);
            self.show_viewport(ctx);
        }

        self.show_dialogs(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_recent(storage);
    }
}

// ─── KEYBOARD HANDLING ───────────────────────────────────────────────────────
impl EditorApp {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if self.show_welcome { return; }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Z)) {
            self.undo();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Y)) {
            self.redo();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S) && !i.modifiers.shift) {
            self.save_project(false);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::S)) {
            self.save_project(true);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::O)) {
            if self.dirty { self.save_before_action = Some(SaveBeforeAction::OpenProject); }
            else { self.open_project_dialog(); }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::N)) {
            if self.dirty { self.save_before_action = Some(SaveBeforeAction::NewProject); }
            else { self.new_project(); }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Q)) {
            if self.dirty { self.confirm_quit = true; }
            else { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
        }

        if self.rename_target.is_some() {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                let target = self.rename_target.take();
                let new_name = self.rename_buffer.clone();
                if let Some(rename_id) = target {
                    let name_before = self.get_entity(Some(rename_id))
                        .map(|e| e.name.clone())
                        .unwrap_or_default();
                    if self.selected_entity == Some(rename_id) {
                        if let Some(entity) = self.selected_entity_mut() {
                            entity.name = new_name;
                            if let Some(entity) = self.selected_entity() {
                                self.push_undo(UndoCommand::ModifyEntity {
                                    scene_index: self.selected_scene,
                                    entity_id: rename_id,
                                    before: EntitySnapshot { name: name_before, ..EntitySnapshot::from(entity) },
                                    after: EntitySnapshot::from(entity),
                                });
                            }
                        }
                    }
                }
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                self.rename_target = None;
            }
            return;
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete)) {
            if self.selected_entity.is_some() {
                self.confirm_delete_entity = self.selected_entity;
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F2)) {
            if let Some(name) = self.selected_entity
                .and_then(|id| self.active_scene().entities.iter().find(|e| e.id == id).map(|e| e.name.clone()))
            {
                self.rename_target = self.selected_entity;
                self.rename_buffer = name;
            }
        }

        for (key, mode) in [
            (egui::Key::Num1, ToolMode::Select), (egui::Key::Num2, ToolMode::Move),
            (egui::Key::Num3, ToolMode::Rotate),  (egui::Key::Num4, ToolMode::Scale),
        ] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                self.tool_mode = mode;
            }
        }
    }
}

// ─── UNDO / REDO ─────────────────────────────────────────────────────────────
impl EditorApp {
    fn commit_pending_entity_edit(&mut self) {
        let before = self.entity_before_edit.take();
        if let Some(before) = before {
            if let Some(entity) = self.selected_entity() {
                let after = EntitySnapshot::from(entity);
                if before != after {
                    self.push_undo(UndoCommand::ModifyEntity {
                        scene_index: self.selected_scene,
                        entity_id: entity.id,
                        before,
                        after,
                    });
                }
            }
        }
        if self.selected_entity.is_some() {
            if let Some(entity) = self.selected_entity() {
                self.entity_before_edit = Some(EntitySnapshot::from(entity));
            }
        }
    }

    fn push_undo(&mut self, cmd: UndoCommand) {
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    fn push_or_replace_entity_undo(&mut self, cmd: UndoCommand) {
        if let UndoCommand::ModifyEntity { scene_index, entity_id, .. } = &cmd {
            if let Some(UndoCommand::ModifyEntity { scene_index: ls, entity_id: lid, .. }) =
                self.undo_stack.last_mut()
            {
                if *ls == *scene_index && *lid == *entity_id {
                    *self.undo_stack.last_mut().unwrap() = cmd;
                    self.redo_stack.clear();
                    return;
                }
            }
        }
        self.push_undo(cmd);
    }

    fn undo(&mut self) {
        if let Some(cmd) = self.undo_stack.pop() {
            match cmd.clone() {
                UndoCommand::AddEntity { scene_index, .. } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) { scene.entities.pop(); }
                }
                UndoCommand::RemoveEntity { scene_index, entity, index } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) { scene.entities.insert(index, entity); }
                }
                UndoCommand::ModifyEntity { scene_index, entity_id, before, .. } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        if let Some(e) = scene.entities.iter_mut().find(|e| e.id == entity_id) { before.apply(e); }
                    }
                }
                UndoCommand::AddScene { index, .. } => {
                    self.project.scenes.remove(index);
                    if self.selected_scene >= self.project.scenes.len() {
                        self.selected_scene = self.project.scenes.len().saturating_sub(1);
                    }
                }
                UndoCommand::RemoveScene { scene, index } => { self.project.scenes.insert(index, scene); }
                UndoCommand::DuplicateEntity { scene_index, entity } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        if let Some(pos) = scene.entities.iter().position(|e| e.id == entity.id) {
                            scene.entities.remove(pos);
                        }
                    }
                }
            }
            self.redo_stack.push(cmd);
            self.dirty = true;
        }
    }

    fn redo(&mut self) {
        if let Some(cmd) = self.redo_stack.pop() {
            match cmd.clone() {
                UndoCommand::AddEntity { scene_index, entity, .. } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) { scene.entities.push(entity); }
                }
                UndoCommand::RemoveEntity { scene_index, entity, .. } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        if let Some(pos) = scene.entities.iter().position(|e| e.id == entity.id) { scene.entities.remove(pos); }
                    }
                }
                UndoCommand::ModifyEntity { scene_index, entity_id, after, .. } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        if let Some(e) = scene.entities.iter_mut().find(|e| e.id == entity_id) { after.apply(e); }
                    }
                }
                UndoCommand::AddScene { scene, index } => { self.project.scenes.insert(index, scene); }
                UndoCommand::RemoveScene { index, .. } => {
                    if index < self.project.scenes.len() {
                        self.project.scenes.remove(index);
                        if self.selected_scene >= self.project.scenes.len() {
                            self.selected_scene = self.project.scenes.len().saturating_sub(1);
                        }
                    }
                }
                UndoCommand::DuplicateEntity { scene_index, entity, .. } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) { scene.entities.push(entity); }
                }
            }
            self.undo_stack.push(cmd);
            self.dirty = true;
        }
    }
}

// ─── WELCOME SCREEN ──────────────────────────────────────────────────────────
impl EditorApp {
    fn show_welcome_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(C_BG0))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let available = ui.available_size();
                let cx = rect.center().x;
                let header_y = available.y * 0.22;

                let p = ui.painter();
                // Subtle gradient-like top stripe
                p.rect_filled(
                    Rect::from_min_size(rect.min, Vec2::new(available.x, 3.0)),
                    0.0, C_BLUE,
                );
                p.text(Pos2::new(cx, header_y), egui::Align2::CENTER_CENTER,
                    "Vibe Editor", FontId::proportional(40.0), C_TEXT1);
                p.text(Pos2::new(cx, header_y + 44.0), egui::Align2::CENTER_CENTER,
                    "Scene & Entity Editor for Vibe Engine", FontId::proportional(15.0), C_TEXT2);

                let card_w = 270.0;
                let card_h = 130.0;
                let gap = 20.0;
                let start_x = cx - (card_w * 2.0 + gap) / 2.0;
                let cards_y = header_y + 90.0;

                let cards: [(&str, &str, &str, Color32); 2] = [
                    ("New Project", "Start a blank project", "◻", C_BLUE),
                    ("Open Project", "Browse for a .vibe.json file", "◈", C_TEAL),
                ];

                for (i, (title, desc, icon, accent)) in cards.iter().enumerate() {
                    let x = start_x + i as f32 * (card_w + gap);
                    let card_rect = Rect::from_min_size(Pos2::new(x, cards_y), Vec2::new(card_w, card_h));
                    let id = ui.id().with(("welcome_card", i));
                    let resp = ui.interact(card_rect, id, Sense::click());

                    let bg = if resp.hovered() { C_BG3 } else { C_BG2 };
                    let border = if resp.hovered() { *accent } else { C_BORDER };
                    let p = ui.painter();
                    p.rect_filled(card_rect, Rounding::same(8.0), bg);
                    p.rect_stroke(card_rect, Rounding::same(8.0), Stroke::new(1.5, border));
                    // Top accent stripe
                    p.rect_filled(Rect::from_min_size(card_rect.min, Vec2::new(card_w, 3.0)),
                        Rounding { nw: 8.0, ne: 8.0, sw: 0.0, se: 0.0 }, *accent);
                    // Icon
                    p.text(card_rect.min + Vec2::new(18.0, 24.0), egui::Align2::LEFT_TOP,
                        icon, FontId::proportional(24.0), *accent);
                    // Title
                    p.text(card_rect.min + Vec2::new(18.0, 60.0), egui::Align2::LEFT_TOP,
                        title, FontId::proportional(17.0), C_TEXT1);
                    // Desc
                    p.text(card_rect.min + Vec2::new(18.0, 84.0), egui::Align2::LEFT_TOP,
                        desc, FontId::proportional(12.0), C_TEXT2);

                    if resp.clicked() {
                        self.show_welcome = false;
                        if i == 0 { self.new_project(); } else { self.open_project_dialog(); }
                    }
                    if resp.hovered() { ctx.set_cursor_icon(egui::CursorIcon::PointingHand); }
                }

                if !self.recent_files.is_empty() {
                    let recent_y = cards_y + card_h + 44.0;
                    ui.painter().text(Pos2::new(cx, recent_y), egui::Align2::CENTER_CENTER,
                        "RECENT PROJECTS", FontId::proportional(11.0), C_TEXT3);

                    let recent = self.recent_files.clone();
                    for (i, path) in recent.iter().enumerate() {
                        let y = recent_y + 24.0 + i as f32 * 28.0;
                        let item_rect = Rect::from_min_size(
                            Pos2::new(cx - 160.0, y - 12.0),
                            Vec2::new(320.0, 24.0),
                        );
                        let id = ui.id().with(("recent", i));
                        let resp = ui.interact(item_rect, id, Sense::click());
                        if resp.hovered() {
                            ui.painter().rect_filled(item_rect, Rounding::same(4.0),
                                Color32::from_rgba_premultiplied(68, 126, 242, 30));
                            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        ui.painter().text(
                            item_rect.left_center() + Vec2::new(10.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            display_name(path),
                            FontId::proportional(13.0),
                            if resp.hovered() { C_BLUE } else { C_TEXT2 },
                        );
                        if resp.clicked() {
                            self.show_welcome = false;
                            self.open_project(path.clone());
                        }
                    }
                }
            });
    }
}

// ─── TOOLBAR ─────────────────────────────────────────────────────────────────
impl EditorApp {
    fn show_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar")
            .exact_height(48.0)
            .frame(egui::Frame::none()
                .fill(Color32::from_rgb(15, 17, 23))
                .stroke(Stroke::new(1.0, C_SEP))
                .inner_margin(egui::Margin { left: 10.0, right: 10.0, top: 6.0, bottom: 6.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Brand
                    ui.label(egui::RichText::new("VIBE").color(C_BLUE).strong().size(14.0));
                    ui.label(egui::RichText::new("ENGINE").color(C_TEXT2).size(13.0));

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // File menu
                    ui.menu_button(egui::RichText::new("File").size(13.0), |ui| {
                        if ui.button("New Project      Ctrl+N").clicked() {
                            if self.dirty { self.save_before_action = Some(SaveBeforeAction::NewProject); }
                            else { self.new_project(); }
                            ui.close_menu();
                        }
                        if ui.button("Open Project...  Ctrl+O").clicked() {
                            if self.dirty { self.save_before_action = Some(SaveBeforeAction::OpenProject); }
                            else { self.open_project_dialog(); }
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Save             Ctrl+S").clicked() { self.save_project(false); ui.close_menu(); }
                        if ui.button("Save As...  Ctrl+Shift+S").clicked() { self.save_project(true); ui.close_menu(); }
                        ui.separator();
                        if ui.button("Export Runtime Pack...").clicked() { self.export_runtime_dialog(); ui.close_menu(); }
                        if !self.recent_files.is_empty() {
                            ui.separator();
                            ui.menu_button("Recent Projects", |ui| {
                                let recent = self.recent_files.clone();
                                for path in &recent {
                                    if ui.button(display_name(path)).clicked() {
                                        self.open_project(path.clone()); ui.close_menu();
                                    }
                                }
                                ui.separator();
                                if ui.button("Clear Recent").clicked() { self.recent_files.clear(); ui.close_menu(); }
                            });
                        }
                        ui.separator();
                        if ui.button("Welcome Screen").clicked() {
                            if self.dirty { self.save_before_action = Some(SaveBeforeAction::Welcome); }
                            else { self.show_welcome = true; }
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Quit             Ctrl+Q").clicked() {
                            if self.dirty { self.confirm_quit = true; }
                            else { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                            ui.close_menu();
                        }
                    });

                    ui.menu_button(egui::RichText::new("Edit").size(13.0), |ui| {
                        if ui.add_enabled(!self.undo_stack.is_empty(), egui::Button::new("Undo  Ctrl+Z")).clicked() {
                            self.undo(); ui.close_menu();
                        }
                        if ui.add_enabled(!self.redo_stack.is_empty(), egui::Button::new("Redo  Ctrl+Y")).clicked() {
                            self.redo(); ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Add Entity").clicked() { self.add_entity(); ui.close_menu(); }
                        if ui.button("Duplicate Entity").clicked() { self.duplicate_selected_entity(); ui.close_menu(); }
                    });

                    ui.menu_button(egui::RichText::new("Scene").size(13.0), |ui| {
                        if ui.button("Add Scene").clicked() { self.add_scene(); ui.close_menu(); }
                        ui.separator();
                        if ui.button("Launch Game").clicked() { self.launch_binary("vibe-engine"); ui.close_menu(); }
                        if ui.button("Launch Launcher").clicked() { self.launch_binary("vibe-launcher"); ui.close_menu(); }
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Tool mode buttons
                    let tools: &[(&str, &str, ToolMode)] = &[
                        ("◉", "Select  [1]", ToolMode::Select),
                        ("✛", "Move    [2]", ToolMode::Move),
                        ("↺", "Rotate  [3]", ToolMode::Rotate),
                        ("⤡", "Scale   [4]", ToolMode::Scale),
                    ];
                    for (icon, tip, mode) in tools {
                        let active = self.tool_mode == *mode;
                        let btn = egui::Button::new(
                            egui::RichText::new(*icon).size(16.0)
                                .color(if active { Color32::WHITE } else { C_TEXT2 })
                        )
                        .fill(if active { C_SEL } else { Color32::TRANSPARENT })
                        .stroke(if active { Stroke::new(1.0, C_BLUE) } else { Stroke::NONE })
                        .min_size(Vec2::new(34.0, 34.0))
                        .rounding(Rounding::same(5.0));
                        if ui.add(btn).on_hover_text(*tip).clicked() {
                            self.tool_mode = *mode;
                        }
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Play / Stop
                    let (icon, label, fg, bg) = if self.play_mode {
                        ("■", "Stop", Color32::from_rgb(230, 100, 100), Color32::from_rgba_premultiplied(90, 25, 25, 100))
                    } else {
                        ("▶", "Play", C_GREEN, Color32::from_rgba_premultiplied(22, 80, 50, 100))
                    };
                    let play_btn = egui::Button::new(
                        egui::RichText::new(format!("{}  {}", icon, label)).color(fg).size(13.0)
                    )
                    .fill(bg)
                    .min_size(Vec2::new(76.0, 34.0))
                    .rounding(Rounding::same(5.0));
                    if ui.add(play_btn).clicked() {
                        self.play_mode = !self.play_mode;
                        if self.play_mode { self.launch_binary("vibe-engine"); }
                        self.push_log(if self.play_mode { "Preview started" } else { "Preview stopped" });
                    }

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // Snap controls
                    ui.checkbox(&mut self.show_grid, egui::RichText::new("Grid").size(12.0));
                    ui.checkbox(&mut self.snap_enabled, egui::RichText::new("Snap").size(12.0));
                    if self.snap_enabled {
                        ui.add(egui::DragValue::new(&mut self.snap_size)
                            .range(0.1..=5.0).speed(0.05).prefix("Step "));
                    }

                    // ── Right side ──
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (dirty_text, dirty_color) = if self.dirty {
                            ("● Unsaved", C_AMBER)
                        } else {
                            ("✓ Saved", C_GREEN)
                        };
                        ui.label(egui::RichText::new(dirty_text).color(dirty_color).size(11.0));
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(2.0);

                        if ui.add(egui::Button::new(
                            egui::RichText::new("Save")
                                .color(if self.dirty { C_AMBER } else { C_TEXT2 })
                                .size(13.0)
                        ).fill(Color32::TRANSPARENT)).on_hover_text("Ctrl+S").clicked() {
                            self.save_project(false);
                        }
                        ui.add_space(2.0);
                        ui.separator();
                        ui.add_space(2.0);

                        if ui.add_enabled(!self.redo_stack.is_empty(),
                            egui::Button::new(egui::RichText::new("↪").size(16.0)).fill(Color32::TRANSPARENT)
                        ).on_hover_text("Redo (Ctrl+Y)").clicked() { self.redo(); }

                        if ui.add_enabled(!self.undo_stack.is_empty(),
                            egui::Button::new(egui::RichText::new("↩").size(16.0)).fill(Color32::TRANSPARENT)
                        ).on_hover_text("Undo (Ctrl+Z)").clicked() { self.undo(); }

                        ui.add_space(2.0);
                        ui.separator();
                        ui.add_space(4.0);

                        let scene_name = self.active_scene().name.clone();
                        let proj_name = self.project.name.clone();
                        ui.label(egui::RichText::new(scene_name).color(C_TEXT3).size(11.5));
                        ui.label(egui::RichText::new("  |  ").color(C_SEP).size(11.5));
                        ui.label(egui::RichText::new(proj_name).color(C_TEXT2).size(11.5));
                    });
                });
            });
    }
}

// ─── HIERARCHY PANEL ─────────────────────────────────────────────────────────
impl EditorApp {
    fn show_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("hierarchy")
            .resizable(true)
            .default_width(272.0)
            .min_width(180.0)
            .frame(egui::Frame::none()
                .fill(C_BG1)
                .stroke(Stroke::new(1.0, C_BORDER)))
            .show(ctx, |ui| {
                // Panel header
                panel_header(ui, "HIERARCHY", C_BLUE);

                // Action buttons
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if ui.add(small_action_btn("+ Entity", C_BLUE)).clicked() {
                        self.add_entity();
                    }
                    if ui.add(small_action_btn("+ Scene", C_PURPLE)).clicked() {
                        self.add_scene();
                    }
                    if self.selected_entity.is_some() {
                        if ui.add(small_action_btn("⊕", C_TEAL)).on_hover_text("Duplicate").clicked() {
                            self.duplicate_selected_entity();
                        }
                        if ui.add(small_action_btn("✕", C_RED)).on_hover_text("Delete").clicked() {
                            self.confirm_delete_entity = self.selected_entity;
                        }
                    }
                });
                ui.add_space(4.0);

                // Search
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.add(egui::TextEdit::singleline(&mut self.hierarchy_filter)
                        .hint_text("🔍 Search entities...")
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Small));
                    ui.add_space(6.0);
                });
                ui.add_space(4.0);

                let (_, sep_rect) = ui.allocate_space(Vec2::new(ui.available_width(), 1.0));
                ui.painter().rect_filled(sep_rect, 0.0, C_SEP);

                // Scene tree
                let filter = self.hierarchy_filter.to_lowercase();
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        self.draw_scene_tree(ui, &filter);
                    });
            });
    }

    fn draw_scene_tree(&mut self, ui: &mut egui::Ui, filter: &str) {
        let mut pending_select: Option<(usize, Option<u64>)> = None;
        let mut pending_delete_entity: Option<u64> = None;
        let mut pending_rename: Option<u64> = None;
        let mut pending_duplicate: Option<u64> = None;
        let mut pending_delete_scene: Option<usize> = None;
        let mut pending_toggle_scene: Option<(egui::Id, bool)> = None;

        // Snapshot scene info to avoid borrowing issues
        let scenes_info: Vec<(usize, String, usize, u64)> = self.project.scenes.iter().enumerate()
            .map(|(i, s)| (i, s.name.clone(), s.entities.len(), s.id))
            .collect();

        for (scene_idx, scene_name, entity_count, scene_id) in &scenes_info {
            let scene_idx = *scene_idx;
            let is_active = scene_idx == self.selected_scene;
            let expand_key = egui::Id::new("scene_open").with(*scene_id);

            // Load or default open state (active scene defaults open)
            let is_open: bool = ui.ctx().data_mut(|d| {
                *d.get_temp_mut_or_insert_with(expand_key, || is_active)
            });

            // ── Scene row ──────────────────────────────────────────────────
            let avail_w = ui.available_width();
            let (_, scene_row) = ui.allocate_space(Vec2::new(avail_w, 30.0));
            let scene_id_interact = ui.id().with(("scene_row", scene_idx));
            let scene_resp = ui.interact(scene_row, scene_id_interact, Sense::click());

            let row_bg = if is_active && self.selected_entity.is_none() {
                Color32::from_rgb(34, 50, 100)
            } else if scene_resp.hovered() {
                C_BG3
            } else {
                C_BG2
            };
            ui.painter().rect_filled(scene_row, 0.0, row_bg);
            // Left accent stripe
            ui.painter().rect_filled(
                Rect::from_min_size(scene_row.min, Vec2::new(3.0, 30.0)),
                0.0, C_PURPLE,
            );
            // Expand arrow
            let arrow = if is_open { "▾" } else { "▸" };
            ui.painter().text(scene_row.min + Vec2::new(10.0, 8.0), egui::Align2::LEFT_TOP,
                arrow, FontId::proportional(12.0), C_TEXT3);
            // Scene icon + name
            ui.painter().text(scene_row.min + Vec2::new(26.0, 7.0), egui::Align2::LEFT_TOP,
                "◈", FontId::proportional(13.0), C_PURPLE);
            ui.painter().text(scene_row.min + Vec2::new(44.0, 7.0), egui::Align2::LEFT_TOP,
                &format!("{} ({})", scene_name, entity_count),
                FontId::proportional(13.0),
                if is_active { C_TEXT1 } else { C_TEXT2 });

            if scene_resp.clicked() {
                if is_active {
                    // Toggle expansion for the active scene
                    pending_toggle_scene = Some((expand_key, !is_open));
                } else {
                    pending_select = Some((scene_idx, None));
                    if !is_open {
                        pending_toggle_scene = Some((expand_key, true));
                    }
                }
            }
            scene_resp.context_menu(|ui| {
                if ui.button("Delete Scene").clicked() {
                    pending_delete_scene = Some(scene_idx);
                    ui.close_menu();
                }
            });

            // ── Entity rows (when expanded) ──────────────────────────────
            if is_open {
                let entities: Vec<(u64, String, bool, bool, bool)> = self.project.scenes
                    .get(scene_idx)
                    .map(|s| s.entities.iter()
                        .filter(|e| filter.is_empty() || e.name.to_lowercase().contains(filter))
                        .map(|e| (e.id, e.name.clone(), e.enabled, e.render.is_some(),
                                  e.name.to_lowercase().contains("camera")))
                        .collect())
                    .unwrap_or_default();

                for (eid, ename, enabled, has_render, is_camera) in &entities {
                    let eid = *eid;
                    let selected = self.selected_scene == scene_idx && self.selected_entity == Some(eid);

                    let (_, ent_row) = ui.allocate_space(Vec2::new(avail_w, 26.0));
                    let ent_id = ui.id().with(("entity_row", eid));
                    let ent_resp = ui.interact(ent_row, ent_id, Sense::click());

                    // Background
                    let ent_bg = if selected {
                        C_SEL
                    } else if ent_resp.hovered() {
                        C_BG3
                    } else {
                        Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(ent_row, 0.0, ent_bg);

                    // Icon
                    let icon = if *is_camera { "◎" } else if *has_render { "■" } else { "○" };
                    let icon_color = if !enabled {
                        C_TEXT3
                    } else if selected {
                        Color32::from_rgb(190, 218, 255)
                    } else {
                        Color32::from_rgb(120, 155, 220)
                    };
                    let text_color = if !enabled { C_TEXT3 } else if selected { Color32::WHITE } else { C_TEXT1 };

                    ui.painter().text(ent_row.min + Vec2::new(32.0, 6.0), egui::Align2::LEFT_TOP,
                        icon, FontId::proportional(12.0), icon_color);
                    ui.painter().text(ent_row.min + Vec2::new(50.0, 6.0), egui::Align2::LEFT_TOP,
                        ename.as_str(), FontId::proportional(13.0), text_color);

                    if ent_resp.clicked() {
                        pending_select = Some((scene_idx, Some(eid)));
                    }
                    ent_resp.context_menu(|ui| {
                        if ui.button("Rename  F2").clicked() { pending_rename = Some(eid); ui.close_menu(); }
                        if ui.button("Duplicate").clicked() { pending_duplicate = Some(eid); ui.close_menu(); }
                        if ui.button("Delete  Del").clicked() { pending_delete_entity = Some(eid); ui.close_menu(); }
                    });
                }
            }
        }

        // Inline rename widget
        if self.rename_target.is_some() {
            ui.add_space(6.0);
            egui::Frame::none()
                .fill(C_BG2)
                .stroke(Stroke::new(1.0, C_BLUE))
                .rounding(Rounding::same(5.0))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Rename Entity").color(C_TEXT2).size(11.0));
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.rename_buffer)
                            .desired_width(f32::INFINITY));
                        if ui.button("✓").clicked() {
                            let target = self.rename_target.take();
                            let new_name = self.rename_buffer.clone();
                            if let Some(id) = target {
                                let si = self.selected_scene;
                                if let Some(e) = self.project.scenes.get_mut(si)
                                    .and_then(|s| s.entities.iter_mut().find(|e| e.id == id))
                                {
                                    e.name = new_name;
                                    self.dirty = true;
                                }
                            }
                        }
                        if ui.button("✕").clicked() { self.rename_target = None; }
                    });
                });
            ui.add_space(4.0);
        }

        // Apply deferred mutations
        if let Some((key, open)) = pending_toggle_scene {
            ui.ctx().data_mut(|d| { *d.get_temp_mut_or_insert_with(key, || open) = open; });
        }
        if let Some((si, eid)) = pending_select {
            self.selected_scene = si;
            self.selected_entity = eid;
        }
        if let Some(id) = pending_delete_entity { self.confirm_delete_entity = Some(id); }
        if let Some(id) = pending_rename {
            self.selected_entity = Some(id);
            self.rename_target = Some(id);
            if let Some(e) = self.get_entity(Some(id)) { self.rename_buffer = e.name.clone(); }
        }
        if let Some(id) = pending_duplicate {
            self.selected_entity = Some(id);
            self.duplicate_selected_entity();
        }
        if let Some(idx) = pending_delete_scene {
            if self.project.scenes.len() > 1 { self.confirm_delete_scene = Some(idx); }
            else { self.push_log("Need at least one scene"); }
        }
    }
}

// ─── INSPECTOR PANEL ─────────────────────────────────────────────────────────
impl EditorApp {
    fn show_inspector_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(320.0)
            .min_width(220.0)
            .frame(egui::Frame::none()
                .fill(C_BG1)
                .stroke(Stroke::new(1.0, C_BORDER)))
            .show(ctx, |ui| {
                panel_header(ui, "INSPECTOR", C_TEAL);

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(6.0);

                        if self.selected_entity.is_some() {
                            let before = self.selected_entity()
                                .map(EntitySnapshot::from);
                            self.draw_entity_inspector(ui);
                            if let Some(before) = before {
                                if let Some(after) = self.selected_entity().map(EntitySnapshot::from) {
                                    if before != after {
                                        let id = self.selected_entity.unwrap();
                                        self.push_or_replace_entity_undo(UndoCommand::ModifyEntity {
                                            scene_index: self.selected_scene,
                                            entity_id: id,
                                            before,
                                            after,
                                        });
                                        self.dirty = true;
                                    }
                                }
                            }
                        } else {
                            self.draw_scene_inspector(ui);
                        }

                        ui.add_space(8.0);
                    });
            });
    }

    fn draw_entity_inspector(&mut self, ui: &mut egui::Ui) {
        let entity = match self.selected_entity_mut() { Some(e) => e, None => return };

        // ── Entity header card ──────────────────────────────────────────────
        insp_card(ui, "ENTITY", C_TEXT3, false, |ui| {
            compact_field(ui, "Name");
            ui.text_edit_singleline(&mut entity.name);
            ui.add_space(2.0);
            compact_field(ui, "Tags");
            ui.text_edit_singleline(&mut entity.tags);
            ui.add_space(2.0);
            ui.checkbox(&mut entity.enabled, egui::RichText::new("Enabled").size(12.0));
        });

        ui.add_space(4.0);

        // ── Transform ──────────────────────────────────────────────────────
        insp_card(ui, "TRANSFORM", C_BLUE, false, |ui| {
            vec3_row(ui, "Position", &mut entity.transform.position);
            vec3_row(ui, "Rotation", &mut entity.transform.rotation);
            vec3_row(ui, "Scale",    &mut entity.transform.scale);
        });

        ui.add_space(4.0);

        // ── Render ─────────────────────────────────────────────────────────
        let mut remove_render = false;
        if entity.render.is_some() {
            remove_render = insp_card(ui, "RENDER", C_AMBER, true, |ui| {
                if let Some(render) = entity.render.as_mut() {
                    egui::ComboBox::from_id_salt("render_kind")
                        .selected_text(render.kind.label())
                        .show_ui(ui, |ui| {
                            for k in RenderKind::all() { ui.selectable_value(&mut render.kind, k, k.label()); }
                        });
                    compact_field(ui, "Mesh");
                    ui.text_edit_singleline(&mut render.mesh);
                    compact_field(ui, "Material");
                    ui.text_edit_singleline(&mut render.material);
                    ui.horizontal(|ui| {
                        compact_field(ui, "Layer");
                        ui.add(egui::DragValue::new(&mut render.layer));
                    });
                    ui.horizontal(|ui| {
                        compact_field(ui, "Color");
                        ui.color_edit_button_rgba_unmultiplied(&mut render.color);
                    });
                }
            });
            ui.add_space(4.0);
        }
        if remove_render { entity.render = None; }

        // ── Collider ───────────────────────────────────────────────────────
        let mut remove_collider = false;
        if entity.collider.is_some() {
            remove_collider = insp_card(ui, "COLLIDER", C_GREEN, true, |ui| {
                if let Some(col) = entity.collider.as_mut() {
                    egui::ComboBox::from_id_salt("col_shape")
                        .selected_text(col.shape.label())
                        .show_ui(ui, |ui| {
                            for s in ColliderShape::all() { ui.selectable_value(&mut col.shape, s, s.label()); }
                        });
                    vec3_row(ui, "Size", &mut col.size);
                    ui.checkbox(&mut col.is_trigger, egui::RichText::new("Is Trigger").size(12.0));
                }
            });
            ui.add_space(4.0);
        }
        if remove_collider { entity.collider = None; }

        // ── Script ─────────────────────────────────────────────────────────
        let mut remove_script = false;
        if entity.script.is_some() {
            remove_script = insp_card(ui, "SCRIPT", C_PURPLE, true, |ui| {
                if let Some(sc) = entity.script.as_mut() {
                    compact_field(ui, "Path");
                    ui.text_edit_singleline(&mut sc.script_path);
                    compact_field(ui, "Entry");
                    ui.text_edit_singleline(&mut sc.entry);
                }
            });
            ui.add_space(4.0);
        }
        if remove_script { entity.script = None; }

        // ── Add component ──────────────────────────────────────────────────
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            let has_render = entity.render.is_some();
            let has_col = entity.collider.is_some();
            let has_sc = entity.script.is_some();
            if !has_render && ui.add(small_action_btn("+ Render", C_AMBER)).clicked() {
                entity.render = Some(RenderComponent::default());
            }
            if !has_col && ui.add(small_action_btn("+ Collider", C_GREEN)).clicked() {
                entity.collider = Some(ColliderComponent::default());
            }
            if !has_sc && ui.add(small_action_btn("+ Script", C_PURPLE)).clicked() {
                entity.script = Some(ScriptComponent {
                    script_path: "scripts/new_script.rs".to_string(),
                    entry: "update".to_string(),
                });
            }
        });
        ui.add_space(8.0);
    }

    fn draw_scene_inspector(&mut self, ui: &mut egui::Ui) {
        insp_card(ui, "SCENE", C_PURPLE, false, |ui| {
            let scene = &mut self.project.scenes[self.selected_scene];
            compact_field(ui, "Name");
            ui.text_edit_singleline(&mut scene.name);
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                compact_field(ui, "Grid Size");
                ui.add(egui::DragValue::new(&mut scene.grid_size).range(0.1..=8.0).speed(0.05));
            });
            color_row_rgb(ui, "Ambient Light", &mut scene.ambient_light);
            color_row_rgb(ui, "Clear Color",   &mut scene.clear_color);
        });

        ui.add_space(4.0);

        insp_card(ui, "PROJECT", C_TEXT2, false, |ui| {
            compact_field(ui, "Name");
            ui.text_edit_singleline(&mut self.project.name);
            compact_field(ui, "Author");
            ui.text_edit_singleline(&mut self.project.author);
            compact_field(ui, "Startup Scene");
            ui.text_edit_singleline(&mut self.project.settings.startup_scene);
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                compact_field(ui, "Gravity");
                ui.add(egui::DragValue::new(&mut self.project.settings.gravity).speed(0.05));
            });
            ui.horizontal(|ui| {
                compact_field(ui, "Fixed dt");
                ui.add(egui::DragValue::new(&mut self.project.settings.fixed_timestep)
                    .speed(0.0005).range(0.001..=0.1));
            });
            compact_field(ui, "Notes");
            ui.add(egui::TextEdit::multiline(&mut self.project.notes)
                .desired_rows(3).hint_text("Project notes..."));
        });
    }
}

// ─── BOTTOM PANEL ────────────────────────────────────────────────────────────
impl EditorApp {
    fn show_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom")
            .resizable(true)
            .default_height(200.0)
            .min_height(80.0)
            .frame(egui::Frame::none()
                .fill(C_BG1)
                .stroke(Stroke::new(1.0, C_BORDER)))
            .show(ctx, |ui| {
                // Tab bar
                let tab_h = 32.0;
                let (_, tab_bar) = ui.allocate_space(Vec2::new(ui.available_width(), tab_h));
                ui.painter().rect_filled(tab_bar, 0.0, C_BG2);
                ui.painter().rect_filled(
                    Rect::from_min_size(tab_bar.min, Vec2::new(tab_bar.width(), 1.0)),
                    0.0, C_SEP,
                );

                let tabs: &[(&str, BottomTab, Color32)] = &[
                    ("ASSETS", BottomTab::Assets, C_AMBER),
                    ("CONSOLE", BottomTab::Console, C_GREEN),
                ];
                let tab_w = 100.0;
                for (i, (label, tab, accent)) in tabs.iter().enumerate() {
                    let tab_rect = Rect::from_min_size(
                        tab_bar.min + Vec2::new(i as f32 * tab_w, 0.0),
                        Vec2::new(tab_w, tab_h),
                    );
                    let id = ui.id().with(("tab", i));
                    let resp = ui.interact(tab_rect, id, Sense::click());
                    let active = self.bottom_tab == *tab;

                    if active {
                        ui.painter().rect_filled(tab_rect, 0.0, C_BG3);
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                Pos2::new(tab_rect.min.x, tab_rect.max.y - 2.0),
                                Vec2::new(tab_w, 2.0)),
                            0.0, *accent,
                        );
                    } else if resp.hovered() {
                        ui.painter().rect_filled(tab_rect, 0.0, C_BG3);
                    }

                    ui.painter().text(
                        tab_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *label,
                        FontId::proportional(11.0),
                        if active { *accent } else { C_TEXT2 },
                    );

                    if resp.clicked() { self.bottom_tab = *tab; }
                }

                // Tab content
                ui.add_space(4.0);
                match self.bottom_tab {
                    BottomTab::Assets  => self.draw_assets_tab(ui),
                    BottomTab::Console => self.draw_console_tab(ui),
                }
            });
    }

    fn draw_assets_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if ui.add(small_action_btn("Import File...", C_BLUE)).clicked() {
                self.import_asset_dialog();
            }
            if self.selected_asset.is_some() {
                if ui.add(small_action_btn("Remove", C_RED)).clicked() {
                    self.remove_selected_asset();
                }
            }
            ui.add_space(8.0);
            ui.add(egui::TextEdit::singleline(&mut self.asset_filter)
                .hint_text("Filter...")
                .desired_width(140.0)
                .font(egui::TextStyle::Small));
        });
        ui.add_space(4.0);

        let filter = self.asset_filter.to_lowercase();
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.add_space(2.0);
            let assets: Vec<(u64, String, AssetKind)> = self.project.assets.iter()
                .filter(|a| filter.is_empty() || a.name.to_lowercase().contains(&filter))
                .map(|a| (a.id, a.name.clone(), a.kind))
                .collect();

            for (id, name, kind) in &assets {
                let selected = self.selected_asset == Some(*id);
                let avail = ui.available_width();
                let (_, row) = ui.allocate_space(Vec2::new(avail, 24.0));
                let resp = ui.interact(row, ui.id().with(("asset", id)), Sense::click());

                let bg = if selected { C_SEL } else if resp.hovered() { C_BG3 } else { Color32::TRANSPARENT };
                ui.painter().rect_filled(row, 0.0, bg);

                ui.painter().text(row.min + Vec2::new(12.0, 5.0), egui::Align2::LEFT_TOP,
                    kind.icon(), FontId::proportional(12.0),
                    match kind {
                        AssetKind::Texture => C_AMBER, AssetKind::Mesh => C_BLUE,
                        AssetKind::Audio => C_GREEN, AssetKind::Script => C_PURPLE,
                        _ => C_TEXT2
                    });
                ui.painter().text(row.min + Vec2::new(28.0, 5.0), egui::Align2::LEFT_TOP,
                    name.as_str(), FontId::proportional(12.0),
                    if selected { Color32::WHITE } else { C_TEXT1 });
                ui.painter().text(row.min + Vec2::new(28.0 + 160.0, 5.0), egui::Align2::LEFT_TOP,
                    kind.label(), FontId::proportional(11.0), C_TEXT3);

                if resp.clicked() { self.selected_asset = Some(*id); }
            }
            if self.project.assets.is_empty() {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("No assets imported.").color(C_TEXT3).size(12.0));
                });
            }
        });
    }

    fn draw_console_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(&self.status_line).color(C_TEXT2).size(11.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                if ui.add(small_action_btn("Clear", C_RED)).clicked() {
                    self.console.clear();
                }
            });
        });
        ui.add_space(2.0);

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);
                for line in &self.console {
                    let color = if line.contains("error") || line.contains("Error") {
                        C_RED
                    } else if line.contains("warn") || line.contains("Warn") {
                        C_AMBER
                    } else {
                        C_TEXT2
                    };
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(line).color(color).size(11.0).monospace());
                    });
                }
                ui.add_space(4.0);
            });
    }
}

// ─── VIEWPORT ────────────────────────────────────────────────────────────────
impl EditorApp {
    fn show_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(C_BG0))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());
                let painter = ui.painter_at(rect);

                let scene = self.active_scene().clone();
                painter.rect_filled(rect, 0.0, color32_from_rgb(scene.clear_color));

                if self.show_grid {
                    self.draw_grid(&painter, rect, scene.grid_size);
                }
                self.draw_entities(&painter, rect);

                // Zoom on scroll
                if response.hovered() {
                    let scroll = ui.input(|i| i.raw_scroll_delta.y);
                    if scroll.abs() > f32::EPSILON {
                        self.viewport_zoom = (self.viewport_zoom * (1.0 + scroll * 0.0015)).clamp(10.0, 300.0);
                    }
                }

                // Click to select entity
                if response.clicked_by(egui::PointerButton::Primary) {
                    self.commit_pending_entity_edit();
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.select_entity_at_screen(rect, pos);
                    }
                }

                // Drag to move entity (Move tool)
                if self.tool_mode == ToolMode::Move
                    && response.dragged_by(egui::PointerButton::Primary)
                {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let (wx, wz) = self.screen_to_world(rect, pos);
                        let x = self.snap_value(wx);
                        let z = self.snap_value(wz);
                        if let Some(entity) = self.selected_entity_mut() {
                            entity.transform.position[0] = x;
                            entity.transform.position[2] = z;
                            self.dirty = true;
                        }
                    }
                }

                // Double-click to spawn entity
                if response.double_clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let (wx, wz) = self.screen_to_world(rect, pos);
                        self.commit_pending_entity_edit();
                        self.add_entity_at(wx, wz);
                    }
                }

                // Middle-drag to pan
                if response.hovered()
                    && ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle))
                {
                    self.viewport_pan += ui.input(|i| i.pointer.delta());
                }

                // ── Viewport overlay ────────────────────────────────────────
                // Top-left: scene info
                painter.text(rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP,
                    &format!("{} • {} entities", scene.name, scene.entities.len()),
                    FontId::monospace(12.0), Color32::from_rgba_premultiplied(210, 224, 255, 200));

                // Tool indicator
                let tool_str = match self.tool_mode {
                    ToolMode::Select => "◉ Select",
                    ToolMode::Move   => "✛ Move",
                    ToolMode::Rotate => "↺ Rotate",
                    ToolMode::Scale  => "⤡ Scale",
                };
                painter.text(rect.min + Vec2::new(10.0, 28.0), egui::Align2::LEFT_TOP,
                    tool_str, FontId::monospace(11.0), C_BLUE);

                // Top-right mini toolbar
                let zoom_text = format!("{:.0}%", self.viewport_zoom / 48.0 * 100.0);
                painter.text(rect.max - Vec2::new(10.0, 10.0), egui::Align2::RIGHT_BOTTOM,
                    &zoom_text, FontId::monospace(11.0),
                    Color32::from_rgba_premultiplied(140, 160, 200, 180));

                // Play mode overlay
                if self.play_mode {
                    painter.rect_filled(
                        Rect::from_min_size(rect.min, Vec2::new(rect.width(), 3.0)),
                        0.0, C_GREEN,
                    );
                    painter.text(rect.center() - Vec2::new(0.0, 20.0),
                        egui::Align2::CENTER_CENTER, "PREVIEW MODE",
                        FontId::monospace(14.0),
                        Color32::from_rgba_premultiplied(100, 220, 140, 180));
                }
            });
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, grid_size: f32) {
        let half = 24;
        let minor = Color32::from_rgba_premultiplied(55, 65, 90, 55);
        let major = Color32::from_rgba_premultiplied(80, 100, 140, 90);
        let axis  = Color32::from_rgba_premultiplied(120, 160, 210, 130);

        for i in -half..=half {
            let x = i as f32 * grid_size;
            let from = self.world_to_screen(rect, [x, 0.0, -(half as f32) * grid_size]);
            let to   = self.world_to_screen(rect, [x, 0.0,  (half as f32) * grid_size]);
            let stroke = if i == 0 { Stroke::new(1.5, axis) } else if i % 5 == 0 { Stroke::new(1.0, major) } else { Stroke::new(0.8, minor) };
            painter.line_segment([from, to], stroke);
        }
        for i in -half..=half {
            let z = i as f32 * grid_size;
            let from = self.world_to_screen(rect, [-(half as f32) * grid_size, 0.0, z]);
            let to   = self.world_to_screen(rect, [ (half as f32) * grid_size, 0.0, z]);
            let stroke = if i == 0 { Stroke::new(1.5, axis) } else if i % 5 == 0 { Stroke::new(1.0, major) } else { Stroke::new(0.8, minor) };
            painter.line_segment([from, to], stroke);
        }
    }

    fn draw_entities(&self, painter: &egui::Painter, rect: Rect) {
        let scene = self.active_scene();
        for entity in &scene.entities {
            let pos = self.world_to_screen(rect, entity.transform.position);
            let base = entity.render.as_ref()
                .map(|r| color32_from_rgba(r.color))
                .unwrap_or(Color32::from_rgb(160, 180, 225));
            let color = if entity.enabled { base } else { Color32::from_gray(75) };
            let selected = self.selected_entity == Some(entity.id);
            let radius = if selected { 9.0 } else { 6.5 };

            if selected {
                // Glow ring
                painter.circle_filled(pos, radius + 4.5,
                    Color32::from_rgba_premultiplied(255, 215, 100, 35));
                painter.circle_stroke(pos, radius + 2.5,
                    Stroke::new(1.8, Color32::from_rgb(255, 215, 100)));
            }
            painter.circle_filled(pos, radius, color);
            painter.circle_stroke(pos, radius, Stroke::new(1.0,
                Color32::from_rgba_premultiplied(255, 255, 255, 60)));
            painter.text(pos + Vec2::new(12.0, -14.0), egui::Align2::LEFT_TOP,
                &entity.name, FontId::monospace(11.0),
                Color32::from_rgba_premultiplied(215, 228, 255, 210));
        }
    }
}

// ─── DIALOGS ─────────────────────────────────────────────────────────────────
impl EditorApp {
    fn show_dialogs(&mut self, ctx: &egui::Context) {
        if let Some(id) = self.confirm_delete_entity.take() {
            let name = self.active_scene().entities.iter()
                .find(|e| e.id == id).map(|e| e.name.clone()).unwrap_or_default();
            let mut ok = false;
            let mut cancel = false;
            egui::Window::new("Delete Entity").anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
                .collapsible(false).resizable(false).show(ctx, |ui| {
                    ui.label(format!("Delete \"{}\"?", name));
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() { ok = true; }
                        if ui.button("Cancel").clicked() { cancel = true; }
                    });
                });
            if ok { self.remove_entity_by_id(id); }
            else if !cancel { self.confirm_delete_entity = Some(id); }
        }

        if let Some(index) = self.confirm_delete_scene.take() {
            let name = self.project.scenes.get(index).map(|s| s.name.clone()).unwrap_or_default();
            let mut ok = false;
            let mut cancel = false;
            egui::Window::new("Delete Scene").anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
                .collapsible(false).resizable(false).show(ctx, |ui| {
                    ui.label(format!("Delete scene \"{}\"?", name));
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() { ok = true; }
                        if ui.button("Cancel").clicked() { cancel = true; }
                    });
                });
            if ok {
                let scene_data = self.project.scenes[index].clone();
                self.project.scenes.remove(index);
                self.push_undo(UndoCommand::RemoveScene { scene: scene_data, index });
                self.selected_scene = self.selected_scene.min(self.project.scenes.len().saturating_sub(1));
                self.selected_entity = None;
                self.dirty = true;
                self.push_log(format!("Deleted scene \"{}\"", name));
            } else if !cancel {
                self.confirm_delete_scene = Some(index);
            }
        }

        if self.confirm_new_project {
            let mut action: Option<bool> = None;
            egui::Window::new("New Project").anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
                .collapsible(false).resizable(false).show(ctx, |ui| {
                    ui.label("Save current project first?");
                    ui.horizontal(|ui| {
                        if ui.button("Save & New").clicked() { action = Some(true); }
                        if ui.button("Discard & New").clicked() { action = Some(false); }
                        if ui.button("Cancel").clicked() { action = Some(false); self.confirm_new_project = false; }
                    });
                });
            match action {
                Some(true)  => { self.save_project(false); self.new_project(); self.confirm_new_project = false; }
                Some(false) => { self.confirm_new_project = false; }
                None => {}
            }
        }

        if self.confirm_quit {
            let mut action: Option<bool> = None;
            egui::Window::new("Unsaved Changes").anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
                .collapsible(false).resizable(false).show(ctx, |ui| {
                    ui.label("Save changes before quitting?");
                    ui.horizontal(|ui| {
                        if ui.button("Save & Quit").clicked() { action = Some(true); }
                        if ui.button("Discard & Quit").clicked() { action = Some(false); }
                        if ui.button("Cancel").clicked() { action = Some(false); self.confirm_quit = false; }
                    });
                });
            match action {
                Some(true) => { self.save_project(false); self.confirm_quit = false; ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                Some(false) => { self.confirm_quit = false; ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                None => {}
            }
        }

        if let Some(pending_action) = self.save_before_action.take() {
            let mut chosen: Option<bool> = None;
            egui::Window::new("Unsaved Changes").anchor(egui::Align2::CENTER_CENTER, [0.0; 2])
                .collapsible(false).resizable(false).show(ctx, |ui| {
                    ui.label("Save current project first?");
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() { chosen = Some(true); }
                        if ui.button("Discard").clicked() { chosen = Some(false); }
                        if ui.button("Cancel").clicked() { chosen = Some(false); }
                    });
                });
            match (chosen, pending_action.clone()) {
                (Some(true),  SaveBeforeAction::NewProject)  => { self.save_project(false); self.new_project(); }
                (Some(false), SaveBeforeAction::NewProject)  => { self.new_project(); }
                (Some(true),  SaveBeforeAction::OpenProject) => { self.save_project(false); self.open_project_dialog(); }
                (Some(false), SaveBeforeAction::OpenProject) => { self.open_project_dialog(); }
                (Some(true),  SaveBeforeAction::Welcome)     => { self.save_project(false); self.show_welcome = true; }
                (Some(false), SaveBeforeAction::Welcome)     => { self.show_welcome = true; }
                (None, _) => { self.save_before_action = Some(pending_action); }
            }
        }
    }
}

// ─── BUSINESS LOGIC ──────────────────────────────────────────────────────────
impl EditorApp {
    fn world_to_screen(&self, rect: Rect, world: [f32; 3]) -> Pos2 {
        let center = rect.center() + self.viewport_pan;
        Pos2::new(center.x + world[0] * self.viewport_zoom, center.y - world[2] * self.viewport_zoom)
    }

    fn screen_to_world(&self, rect: Rect, screen: Pos2) -> (f32, f32) {
        let center = rect.center() + self.viewport_pan;
        ((screen.x - center.x) / self.viewport_zoom, -(screen.y - center.y) / self.viewport_zoom)
    }

    fn snap_value(&self, v: f32) -> f32 {
        if self.snap_enabled { (v / self.snap_size).round() * self.snap_size } else { v }
    }

    fn select_entity_at_screen(&mut self, rect: Rect, pointer: Pos2) {
        let mut best = None;
        let mut best_d = f32::MAX;
        for entity in &self.active_scene().entities {
            let d = self.world_to_screen(rect, entity.transform.position).distance(pointer);
            if d < 14.0 && d < best_d { best = Some(entity.id); best_d = d; }
        }
        self.selected_entity = best;
        if best.is_none() { self.status_line = "Scene selected".to_string(); }
    }

    fn add_scene(&mut self) {
        let id = self.allocate_id();
        let scene = SceneDocument {
            id, name: format!("Scene {}", self.project.scenes.len() + 1),
            ambient_light: [0.22, 0.24, 0.27], clear_color: [0.06, 0.07, 0.09],
            grid_size: 1.0, entities: vec![EntityDocument::default_camera()],
        };
        let index = self.project.scenes.len();
        self.project.scenes.push(scene.clone());
        self.push_undo(UndoCommand::AddScene { scene, index });
        self.selected_scene = index;
        self.selected_entity = None;
        self.dirty = true;
        self.push_log("Added scene");
    }

    fn add_entity(&mut self) {
        let id = self.allocate_id();
        let entity = EntityDocument::default_cube(id);
        let scene_index = self.selected_scene;
        self.active_scene_mut().entities.push(entity.clone());
        self.push_undo(UndoCommand::AddEntity { scene_index, entity });
        self.selected_entity = Some(id);
        self.dirty = true;
        self.push_log("Added entity");
    }

    fn add_entity_at(&mut self, world_x: f32, world_z: f32) {
        let id = self.allocate_id();
        let mut entity = EntityDocument::default_cube(id);
        entity.transform.position[0] = self.snap_value(world_x);
        entity.transform.position[2] = self.snap_value(world_z);
        let scene_index = self.selected_scene;
        self.active_scene_mut().entities.push(entity.clone());
        self.push_undo(UndoCommand::AddEntity { scene_index, entity });
        self.selected_entity = Some(id);
        self.dirty = true;
        self.push_log("Added entity at viewport position");
    }

    fn remove_entity_by_id(&mut self, id: u64) {
        let scene_index = self.selected_scene;
        if let Some(index) = self.active_scene_mut().entities.iter().position(|e| e.id == id) {
            let entity = self.active_scene_mut().entities.remove(index);
            self.push_undo(UndoCommand::RemoveEntity { scene_index, entity, index });
            self.selected_entity = None;
            self.dirty = true;
            self.push_log("Removed entity");
        }
    }

    fn duplicate_selected_entity(&mut self) {
        let id = match self.selected_entity { Some(id) => id, None => return };
        let tmpl = self.active_scene().entities.iter().find(|e| e.id == id).cloned();
        if let Some(mut entity) = tmpl {
            entity.id = self.allocate_id();
            entity.name = format!("{} Copy", entity.name);
            entity.transform.position[0] += self.snap_size.max(0.2);
            entity.transform.position[2] += self.snap_size.max(0.2);
            let new_id = entity.id;
            let scene_index = self.selected_scene;
            self.active_scene_mut().entities.push(entity.clone());
            self.push_undo(UndoCommand::DuplicateEntity { scene_index, entity });
            self.selected_entity = Some(new_id);
            self.dirty = true;
            self.push_log("Duplicated entity");
        }
    }

    fn get_entity(&self, id: Option<u64>) -> Option<&EntityDocument> {
        let id = id?;
        self.active_scene().entities.iter().find(|e| e.id == id)
    }
    fn selected_entity(&self) -> Option<&EntityDocument> { self.get_entity(self.selected_entity) }
    fn selected_entity_mut(&mut self) -> Option<&mut EntityDocument> {
        let id = self.selected_entity?;
        self.active_scene_mut().entities.iter_mut().find(|e| e.id == id)
    }
    fn active_scene(&self) -> &SceneDocument { &self.project.scenes[self.selected_scene] }
    fn active_scene_mut(&mut self) -> &mut SceneDocument { &mut self.project.scenes[self.selected_scene] }

    fn ensure_valid_selection(&mut self) {
        if self.project.scenes.is_empty() { self.project.scenes.push(SceneDocument::default()); }
        if self.selected_scene >= self.project.scenes.len() {
            self.selected_scene = self.project.scenes.len() - 1;
        }
        if let Some(eid) = self.selected_entity {
            if !self.active_scene().entities.iter().any(|e| e.id == eid) {
                self.selected_entity = None;
            }
        }
    }

    fn new_project(&mut self) {
        self.project = ProjectDocument::default();
        self.project_path = None;
        self.selected_scene = 0;
        self.selected_entity = None;
        self.selected_asset = None;
        self.viewport_zoom = 48.0;
        self.viewport_pan = Vec2::ZERO;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.dirty = false;
        self.recompute_next_id();
        self.push_log("Created new project");
    }

    fn open_project_dialog(&mut self) {
        if let Some(path) = FileDialog::new().add_filter("Vibe Project", &["json"]).pick_file() {
            self.open_project(path);
        }
    }

    fn open_project(&mut self, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str::<ProjectDocument>(&data) {
                Ok(project) => {
                    self.project = project;
                    self.project_path = Some(path.clone());
                    self.selected_scene = 0;
                    self.selected_entity = None;
                    self.selected_asset = None;
                    self.undo_stack.clear();
                    self.redo_stack.clear();
                    self.dirty = false;
                    self.recompute_next_id();
                    self.show_welcome = false;
                    self.add_recent_file(path.clone());
                    self.push_log(format!("Opened project {}", display_name(&path)));
                }
                Err(e) => { self.push_log(format!("Parse error: {}", e)); }
            },
            Err(e) => { self.push_log(format!("Open error: {}", e)); }
        }
    }

    fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 { self.recent_files.truncate(10); }
    }

    fn save_project(&mut self, save_as: bool) {
        let path = if save_as || self.project_path.is_none() {
            FileDialog::new().add_filter("Vibe Project", &["json"])
                .set_file_name("project.vibe.json").save_file()
        } else {
            self.project_path.clone()
        };
        if let Some(path) = path { self.write_project(path); }
    }

    fn write_project(&mut self, path: PathBuf) {
        self.project.modified_unix = unix_now();
        match serde_json::to_string_pretty(&self.project) {
            Ok(data) => match fs::write(&path, data) {
                Ok(()) => {
                    let name = display_name(&path);
                    self.project_path = Some(path);
                    self.dirty = false;
                    self.push_log(format!("Saved {}", name));
                }
                Err(e) => { self.push_log(format!("Save error: {}", e)); }
            },
            Err(e) => { self.push_log(format!("Serialize error: {}", e)); }
        }
    }

    fn export_runtime_dialog(&mut self) {
        let Some(path) = FileDialog::new()
            .add_filter("Runtime Pack", &["json"])
            .set_file_name("runtime_pack.json").save_file()
        else { return; };
        self.export_runtime(path);
    }

    fn export_runtime(&mut self, path: PathBuf) {
        let runtime = RuntimeExport {
            project_name: self.project.name.clone(),
            exported_unix: unix_now(),
            scenes: self.project.scenes.iter().map(|s| RuntimeScene {
                name: s.name.clone(),
                ambient_light: s.ambient_light,
                clear_color: s.clear_color,
                entities: s.entities.iter().map(|e| RuntimeEntity {
                    name: e.name.clone(), enabled: e.enabled,
                    tags: e.tags.split(',').map(str::trim).filter(|t| !t.is_empty()).map(ToOwned::to_owned).collect(),
                    transform: e.transform.clone(),
                    render: e.render.clone(), collider: e.collider.clone(), script: e.script.clone(),
                }).collect(),
            }).collect(),
        };
        match serde_json::to_string_pretty(&runtime) {
            Ok(data) => match fs::write(&path, data) {
                Ok(()) => { self.push_log(format!("Exported to {}", display_name(&path))); }
                Err(e) => { self.push_log(format!("Export error: {}", e)); }
            },
            Err(e) => { self.push_log(format!("Export serialize error: {}", e)); }
        }
    }

    fn import_asset_dialog(&mut self) {
        if let Some(path) = FileDialog::new().pick_file() { self.import_asset_from_path(path); }
    }

    fn import_asset_from_path(&mut self, path: PathBuf) {
        let id = self.allocate_id();
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("asset").to_string();
        let kind = infer_asset_kind(&path);
        self.project.assets.push(AssetDocument { id, name: name.clone(), path: path.to_string_lossy().to_string(), kind });
        self.selected_asset = Some(id);
        self.dirty = true;
        self.push_log(format!("Imported asset {}", name));
    }

    fn remove_selected_asset(&mut self) {
        if let Some(id) = self.selected_asset {
            if let Some(i) = self.project.assets.iter().position(|a| a.id == id) {
                self.project.assets.remove(i);
                self.selected_asset = None;
                self.dirty = true;
                self.push_log("Removed asset");
            }
        }
    }

    fn launch_binary(&mut self, name: &str) {
        match spawn_binary(name) {
            Ok(()) => { self.push_log(format!("Launched {}", name)); }
            Err(e) => { self.push_log(e); }
        }
    }

    fn push_log(&mut self, message: impl Into<String>) {
        let entry = format!("[{}] {}", unix_now(), message.into());
        self.status_line = entry.clone();
        self.console.push(entry);
    }

    fn allocate_id(&mut self) -> u64 { let id = self.next_id; self.next_id += 1; id }

    fn recompute_next_id(&mut self) {
        let mut max = 1u64;
        for s in &self.project.scenes { max = max.max(s.id); for e in &s.entities { max = max.max(e.id); } }
        for a in &self.project.assets { max = max.max(a.id); }
        self.next_id = max + 1;
    }
}

// ─── UI HELPERS ──────────────────────────────────────────────────────────────

/// Apply the dark modern theme globally (idempotent, safe to call every frame).
fn configure_visuals(ctx: &egui::Context) {
    let mut vis = egui::Visuals::dark();

    vis.panel_fill           = C_BG1;
    vis.window_fill          = C_BG2;
    vis.faint_bg_color       = Color32::from_rgb(22, 24, 35);
    vis.extreme_bg_color     = C_BG0;
    vis.window_rounding      = Rounding::same(8.0);
    vis.window_stroke        = Stroke::new(1.0, C_BORDER);
    vis.window_shadow        = egui::Shadow::NONE;
    vis.popup_shadow         = egui::Shadow::NONE;

    macro_rules! set_widget {
        ($w:expr, $bg:expr, $fg:expr, $stroke:expr) => {
            $w.bg_fill   = $bg;
            $w.fg_stroke = Stroke::new(1.0, $fg);
            $w.bg_stroke = Stroke::new(1.0, $stroke);
            $w.rounding  = Rounding::same(4.0);
        };
    }
    set_widget!(vis.widgets.noninteractive, C_BG1,  C_TEXT2, C_SEP);
    set_widget!(vis.widgets.inactive,       C_BG3,  C_TEXT1, C_BORDER);
    set_widget!(vis.widgets.hovered,        Color32::from_rgb(44, 52, 78), C_TEXT1, C_BLUE);
    set_widget!(vis.widgets.active,         C_SEL,  Color32::WHITE,  C_BLUE);
    set_widget!(vis.widgets.open,           C_BG3,  C_TEXT1, C_BLUE);

    vis.selection.bg_fill    = C_SEL;
    vis.selection.stroke     = Stroke::new(1.0, Color32::from_rgb(110, 160, 255));
    vis.hyperlink_color      = C_BLUE;

    ctx.set_visuals(vis);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing   = Vec2::new(6.0, 4.0);
    style.spacing.button_padding = Vec2::new(8.0, 4.0);
    style.spacing.indent         = 16.0;
    ctx.set_style(style);
}

/// Draws a panel header with colored top accent bar and small caps label.
fn panel_header(ui: &mut egui::Ui, title: &str, accent: Color32) {
    let avail_w = ui.available_width();
    let h = 34.0;
    let (_, rect) = ui.allocate_space(Vec2::new(avail_w, h));
    ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(22, 24, 34));
    ui.painter().rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(avail_w, 2.0)), 0.0, accent);
    ui.painter().text(
        rect.min + Vec2::new(12.0, h / 2.0 - 6.0), egui::Align2::LEFT_TOP,
        title, FontId::proportional(11.0), C_TEXT2);
}

/// A small labeled field row in the inspector.
fn compact_field(ui: &mut egui::Ui, label: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(label).color(C_TEXT2).size(11.0));
}

/// X/Y/Z drag-value row for the inspector.
fn vec3_row(ui: &mut egui::Ui, label: &str, v: &mut [f32; 3]) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(C_TEXT2).size(11.0).strong());
        ui.add_space(4.0);
        for (prefix, idx, color) in [("X", 0, Color32::from_rgb(210, 80, 80)),
                                      ("Y", 1, Color32::from_rgb(80, 200, 100)),
                                      ("Z", 2, Color32::from_rgb(80, 130, 230))] {
            ui.colored_label(color, egui::RichText::new(prefix).size(10.0).strong());
            ui.add(egui::DragValue::new(&mut v[idx]).speed(0.05)
                .min_decimals(2).max_decimals(3));
        }
    });
}

/// Inspector component card (returns true if the remove button was clicked).
fn insp_card(ui: &mut egui::Ui, title: &str, accent: Color32, removable: bool,
             content: impl FnOnce(&mut egui::Ui)) -> bool {
    let mut removed = false;
    egui::Frame::none()
        .fill(C_BG2)
        .rounding(Rounding::same(6.0))
        .stroke(Stroke::new(1.0, C_BORDER))
        .inner_margin(egui::Margin::same(0.0))
        .outer_margin(egui::Margin { left: 8.0, right: 8.0, top: 0.0, bottom: 0.0 })
        .show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                // Accent bar
                let (_, bar_rect) = ui.allocate_space(Vec2::new(4.0, 18.0));
                ui.painter().rect_filled(bar_rect, Rounding::same(2.0), accent);
                ui.add_space(6.0);
                ui.label(egui::RichText::new(title).size(11.0).color(C_TEXT2).strong());
                if removable {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new(
                            egui::RichText::new("✕").size(10.0).color(C_TEXT3)
                        ).fill(Color32::TRANSPARENT).frame(false)).clicked() {
                            removed = true;
                        }
                    });
                }
            });

            ui.add(egui::Separator::default().spacing(4.0));

            egui::Frame::none()
                .inner_margin(egui::Margin { left: 10.0, right: 10.0, top: 2.0, bottom: 8.0 })
                .show(ui, |ui| { content(ui); });
        });
    removed
}

/// Small tinted action button for toolbars.
fn small_action_btn(label: &str, color: Color32) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(label.to_owned()).color(color).size(11.5)
    )
    .fill(Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 28))
    .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 70)))
    .rounding(Rounding::same(4.0))
}

fn color_row_rgb(ui: &mut egui::Ui, label: &str, v: &mut [f32; 3]) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(C_TEXT2).size(11.0));
        ui.add(egui::DragValue::new(&mut v[0]).range(0.0..=1.0).speed(0.01).prefix("R "));
        ui.add(egui::DragValue::new(&mut v[1]).range(0.0..=1.0).speed(0.01).prefix("G "));
        ui.add(egui::DragValue::new(&mut v[2]).range(0.0..=1.0).speed(0.01).prefix("B "));
    });
}

// ─── FREE FUNCTIONS ──────────────────────────────────────────────────────────
fn infer_asset_kind(path: &Path) -> AssetKind {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png"|"jpg"|"jpeg"|"webp"|"bmp"|"tga" => AssetKind::Texture,
        "obj"|"fbx"|"gltf"|"glb"              => AssetKind::Mesh,
        "wav"|"ogg"|"mp3"|"flac"              => AssetKind::Audio,
        "mat"|"material"                      => AssetKind::Material,
        "rs"|"lua"|"js"|"ts"                  => AssetKind::Script,
        _                                     => AssetKind::Other,
    }
}

fn display_name(path: &Path) -> String {
    path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string()
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn color32_from_rgb(rgb: [f32; 3]) -> Color32 {
    Color32::from_rgb(
        (rgb[0].clamp(0.0, 1.0) * 255.0) as u8,
        (rgb[1].clamp(0.0, 1.0) * 255.0) as u8,
        (rgb[2].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn color32_from_rgba(rgba: [f32; 4]) -> Color32 {
    Color32::from_rgba_premultiplied(
        (rgba[0].clamp(0.0, 1.0) * 255.0) as u8,
        (rgba[1].clamp(0.0, 1.0) * 255.0) as u8,
        (rgba[2].clamp(0.0, 1.0) * 255.0) as u8,
        (rgba[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn spawn_binary(binary_name: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut candidates = Vec::new();
    if let Some(parent) = exe.parent() {
        candidates.push(parent.join(binary_name));
        candidates.push(parent.join(format!("{}.exe", binary_name)));
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for profile in ["debug", "release"] {
        candidates.push(manifest.join("target").join(profile).join(binary_name));
        candidates.push(manifest.join("target").join(profile).join(format!("{}.exe", binary_name)));
    }
    for c in &candidates {
        if c.exists() {
            Command::new(c).spawn().map_err(|e| format!("Failed to launch {}: {}", binary_name, e))?;
            return Ok(());
        }
    }
    Command::new("cargo")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["run", "--bin", binary_name])
        .spawn()
        .map_err(|e| format!("Failed to fallback-launch {}: {}", binary_name, e))?;
    Ok(())
}
