use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::{egui, App};
use egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolMode {
    Select,
    Move,
    Rotate,
    Scale,
}

impl ToolMode {
    fn all() -> [ToolMode; 4] {
        [
            ToolMode::Select,
            ToolMode::Move,
            ToolMode::Rotate,
            ToolMode::Scale,
        ]
    }
}

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
enum RenderKind {
    Mesh,
    Sprite,
    Light,
}

impl RenderKind {
    fn label(self) -> &'static str {
        match self {
            RenderKind::Mesh => "Mesh",
            RenderKind::Sprite => "Sprite",
            RenderKind::Light => "Light",
        }
    }

    fn all() -> [RenderKind; 3] {
        [RenderKind::Mesh, RenderKind::Sprite, RenderKind::Light]
    }
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
enum ColliderShape {
    Box,
    Sphere,
    Capsule,
}

impl ColliderShape {
    fn label(self) -> &'static str {
        match self {
            ColliderShape::Box => "Box",
            ColliderShape::Sphere => "Sphere",
            ColliderShape::Capsule => "Capsule",
        }
    }

    fn all() -> [ColliderShape; 3] {
        [ColliderShape::Box, ColliderShape::Sphere, ColliderShape::Capsule]
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct ColliderComponent {
    shape: ColliderShape,
    size: [f32; 3],
    is_trigger: bool,
}

impl Default for ColliderComponent {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Box,
            size: [1.0, 1.0, 1.0],
            is_trigger: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct ScriptComponent {
    script_path: String,
    entry: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
enum AssetKind {
    Texture,
    Mesh,
    Material,
    Audio,
    Script,
    Other,
}

impl AssetKind {
    fn label(self) -> &'static str {
        match self {
            AssetKind::Texture => "Texture",
            AssetKind::Mesh => "Mesh",
            AssetKind::Material => "Material",
            AssetKind::Audio => "Audio",
            AssetKind::Script => "Script",
            AssetKind::Other => "Other",
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
    AddEntity {
        scene_index: usize,
        entity: EntityDocument,
    },
    RemoveEntity {
        scene_index: usize,
        entity: EntityDocument,
        index: usize,
    },
    ModifyEntity {
        scene_index: usize,
        entity_id: u64,
        before: EntitySnapshot,
        after: EntitySnapshot,
    },
    AddScene {
        scene: SceneDocument,
        index: usize,
    },
    RemoveScene {
        scene: SceneDocument,
        index: usize,
    },
    DuplicateEntity {
        scene_index: usize,
        entity: EntityDocument,
    },
}

struct EditorApp {
    project: ProjectDocument,
    project_path: Option<PathBuf>,
    selected_scene: usize,
    selected_entity: Option<u64>,
    selected_asset: Option<u64>,
    hierarchy_filter: String,
    asset_filter: String,
    import_path_buffer: String,
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
}

#[derive(Clone)]
enum SaveBeforeAction {
    NewProject,
    OpenProject,
    Welcome,
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
            import_path_buffer: String::new(),
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
        }
    }

    fn save_recent(&self, storage: &mut dyn eframe::Storage) {
        let paths: Vec<String> = self
            .recent_files
            .iter()
            .filter_map(|p| p.to_str().map(String::from))
            .collect();
        storage.set_string(
            "recent_files",
            serde_json::to_string(&paths).unwrap_or_default(),
        );
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);
        self.commit_pending_entity_edit();
        self.ensure_valid_selection();

        if self.show_welcome {
            self.show_welcome_screen(ctx);
        } else {
            self.show_top_bar(ctx);
            self.show_hierarchy_panel(ctx);
            self.show_inspector_panel(ctx);
            self.show_viewport(ctx);
            self.show_bottom_panel(ctx);
        }

        self.show_dialogs(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_recent(storage);
    }
}

impl EditorApp {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if self.show_welcome {
            return;
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Z)) {
            self.undo();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Y)) {
            self.redo();
        }
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL, egui::Key::S) && !i.modifiers.shift
        }) {
            self.save_project(false);
        }
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::S)
        }) {
            self.save_project(true);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::O)) {
            if self.dirty {
                self.save_before_action = Some(SaveBeforeAction::OpenProject);
            } else {
                self.open_project_dialog();
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::N)) {
            if self.dirty {
                self.save_before_action = Some(SaveBeforeAction::NewProject);
            } else {
                self.new_project();
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Q)) {
            if self.dirty {
                self.confirm_quit = true;
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        if self.rename_target.is_some() {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                let target = self.rename_target.take();
                let new_name = self.rename_buffer.clone();
                if let Some(rename_id) = target {
                    let name_before = self
                        .get_entity(Some(rename_id))
                        .map(|e| e.name.clone())
                        .unwrap_or_default();
                    if self.selected_entity == Some(rename_id) {
                        if let Some(entity) = self.selected_entity_mut() {
                            entity.name = new_name;
                            if let Some(entity) = self.selected_entity() {
                                self.push_undo(UndoCommand::ModifyEntity {
                                    scene_index: self.selected_scene,
                                    entity_id: rename_id,
                                    before: EntitySnapshot {
                                        name: name_before,
                                        ..EntitySnapshot::from(entity)
                                    },
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
            let ent_name = self.selected_entity.and_then(|id| {
                self.active_scene()
                    .entities
                    .iter()
                    .find(|e| e.id == id)
                    .map(|e| e.name.clone())
            });
            if let Some(name) = ent_name {
                self.rename_target = self.selected_entity;
                self.rename_buffer = name;
            }
        }

        let tool_keys = [
            (egui::Key::Num1, ToolMode::Select),
            (egui::Key::Num2, ToolMode::Move),
            (egui::Key::Num3, ToolMode::Rotate),
            (egui::Key::Num4, ToolMode::Scale),
        ];
        for (key, mode) in tool_keys {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                self.tool_mode = mode;
            }
        }
    }

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
        if let UndoCommand::ModifyEntity {
            scene_index,
            entity_id,
            ..
        } = &cmd
        {
            if let Some(UndoCommand::ModifyEntity {
                scene_index: ls,
                entity_id: lid,
                ..
            }) = self.undo_stack.last_mut()
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
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        scene.entities.pop();
                    }
                }
                UndoCommand::RemoveEntity {
                    scene_index,
                    entity,
                    index,
                } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        scene.entities.insert(index, entity);
                    }
                }
                UndoCommand::ModifyEntity {
                    scene_index,
                    entity_id,
                    before,
                    ..
                } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        if let Some(entity) = scene.entities.iter_mut().find(|e| e.id == entity_id)
                        {
                            before.apply(entity);
                        }
                    }
                }
                UndoCommand::AddScene { index, .. } => {
                    self.project.scenes.remove(index);
                    if self.selected_scene >= self.project.scenes.len() {
                        self.selected_scene = self.project.scenes.len().saturating_sub(1);
                    }
                }
                UndoCommand::RemoveScene { scene, index } => {
                    self.project.scenes.insert(index, scene);
                }
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
                UndoCommand::AddEntity {
                    scene_index, entity, ..
                } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        scene.entities.push(entity);
                    }
                }
                UndoCommand::RemoveEntity {
                    scene_index,
                    entity,
                    ..
                } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        if let Some(pos) = scene.entities.iter().position(|e| e.id == entity.id) {
                            scene.entities.remove(pos);
                        }
                    }
                }
                UndoCommand::ModifyEntity {
                    scene_index,
                    entity_id,
                    after,
                    ..
                } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        if let Some(entity) = scene.entities.iter_mut().find(|e| e.id == entity_id)
                        {
                            after.apply(entity);
                        }
                    }
                }
                UndoCommand::AddScene { scene, index } => {
                    self.project.scenes.insert(index, scene);
                }
                UndoCommand::RemoveScene { index, .. } => {
                    if index < self.project.scenes.len() {
                        self.project.scenes.remove(index);
                        if self.selected_scene >= self.project.scenes.len() {
                            self.selected_scene = self.project.scenes.len().saturating_sub(1);
                        }
                    }
                }
                UndoCommand::DuplicateEntity {
                    scene_index, entity, ..
                } => {
                    if let Some(scene) = self.project.scenes.get_mut(scene_index) {
                        scene.entities.push(entity);
                    }
                }
            }
            self.undo_stack.push(cmd);
            self.dirty = true;
        }
    }

    fn show_welcome_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let available = ui.available_size();
            let center_x = rect.center().x;

            let header_y = available.y * 0.25;

            {
                let painter = ui.painter();
                painter.rect_filled(rect, 0.0, Color32::from_rgb(22, 24, 30));
                painter.text(
                    Pos2::new(center_x, header_y),
                    egui::Align2::CENTER_CENTER,
                    "Vibe Editor",
                    FontId::proportional(36.0),
                    Color32::from_rgb(220, 224, 245),
                );
                painter.text(
                    Pos2::new(center_x, header_y + 36.0),
                    egui::Align2::CENTER_CENTER,
                    "Scene & entity editor for Vibe Engine",
                    FontId::proportional(16.0),
                    Color32::from_rgb(140, 148, 180),
                );
            }

            let card_w = 260.0;
            let card_h = 140.0;
            let gap = 20.0;
            let total_w = card_w * 2.0 + gap;
            let start_x = center_x - total_w / 2.0;
            let cards_y = header_y + 80.0;

            let cards: [(&str, &str, Color32); 2] = [
                ("New Project", "Start a blank project", Color32::from_rgb(65, 72, 110)),
                ("Open Project", "Browse for a .vibe.json file", Color32::from_rgb(72, 90, 110)),
            ];

            for (i, (title, desc, bg_color)) in cards.iter().enumerate() {
                let x = start_x + i as f32 * (card_w + gap);
                let card_rect =
                    Rect::from_min_size(Pos2::new(x, cards_y), Vec2::new(card_w, card_h));
                let id = ui.next_auto_id();
                let resp = ui.interact(card_rect, id, egui::Sense::click());
                let p = ui.painter();
                p.rect_filled(card_rect, 8.0, *bg_color);
                p.rect_stroke(
                    card_rect,
                    8.0,
                    Stroke::new(1.0, Color32::from_rgb(90, 100, 140)),
                );
                p.text(
                    card_rect.left_center() + Vec2::new(16.0, -10.0),
                    egui::Align2::LEFT_CENTER,
                    *title,
                    FontId::proportional(18.0),
                    Color32::from_rgb(235, 238, 255),
                );
                p.text(
                    card_rect.left_center() + Vec2::new(16.0, 14.0),
                    egui::Align2::LEFT_CENTER,
                    *desc,
                    FontId::proportional(13.0),
                    Color32::from_rgb(170, 178, 210),
                );

                if resp.clicked() {
                    self.show_welcome = false;
                    if i == 0 {
                        self.new_project();
                    } else {
                        self.open_project_dialog();
                    }
                }
                if resp.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            }

            if !self.recent_files.is_empty() {
                let recent_y = cards_y + card_h + 40.0;
                {
                    let painter = ui.painter();
                    painter.text(
                        Pos2::new(center_x, recent_y),
                        egui::Align2::CENTER_CENTER,
                        "Recent Projects",
                        FontId::proportional(14.0),
                        Color32::from_rgb(140, 148, 180),
                    );
                }

                let recent = self.recent_files.clone();
                for (i, path) in recent.iter().enumerate() {
                    let y = recent_y + 30.0 + i as f32 * 26.0;
                    let item_rect = Rect::from_min_size(
                        Pos2::new(center_x - 120.0, y - 12.0),
                        Vec2::new(240.0, 24.0),
                    );
                    let id = ui.next_auto_id();
                    let resp = ui.interact(item_rect, id, egui::Sense::click());
                    let p = ui.painter();
                    if resp.hovered() {
                        p.rect_filled(item_rect, 4.0, Color32::from_rgba_premultiplied(100, 110, 150, 40));
                    }
                    p.text(
                        item_rect.left_center() + Vec2::new(8.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        display_name(path),
                        FontId::proportional(14.0),
                        Color32::from_rgb(180, 188, 220),
                    );
                    if resp.clicked() {
                        self.show_welcome = false;
                        self.open_project(path.clone());
                    }
                    if resp.hovered() {
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            }
        });
    }

    fn show_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("editor_top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        if self.dirty {
                            self.save_before_action = Some(SaveBeforeAction::NewProject);
                        } else {
                            self.new_project();
                        }
                        ui.close_menu();
                    }
                    if ui.button("Open Project...").clicked() {
                        if self.dirty {
                            self.save_before_action = Some(SaveBeforeAction::OpenProject);
                        } else {
                            self.open_project_dialog();
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save  Ctrl+S").clicked() {
                        self.save_project(false);
                        ui.close_menu();
                    }
                    if ui.button("Save As...  Ctrl+Shift+S").clicked() {
                        self.save_project(true);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export Runtime Pack...").clicked() {
                        self.export_runtime_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if !self.recent_files.is_empty() {
                        ui.menu_button("Recent", |ui| {
                            let recent = self.recent_files.clone();
                            for path in &recent {
                                if ui.button(display_name(path)).clicked() {
                                    self.open_project(path.clone());
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                            if ui.button("Clear Recent").clicked() {
                                self.recent_files.clear();
                                ui.close_menu();
                            }
                        });
                    }
                    ui.separator();
                    if ui.button("Quit  Ctrl+Q").clicked() {
                        if self.dirty {
                            self.confirm_quit = true;
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let can_undo = !self.undo_stack.is_empty();
                    let can_redo = !self.redo_stack.is_empty();
                    if ui
                        .add_enabled(can_undo, egui::Button::new("Undo  Ctrl+Z"))
                        .clicked()
                    {
                        self.undo();
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(can_redo, egui::Button::new("Redo  Ctrl+Y"))
                        .clicked()
                    {
                        self.redo();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Scene", |ui| {
                    if ui.button("Add Scene").clicked() {
                        self.add_scene();
                        ui.close_menu();
                    }
                    if ui.button("Add Entity").clicked() {
                        self.add_entity();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Launch Game").clicked() {
                        self.launch_binary("vibe-engine");
                        ui.close_menu();
                    }
                    if ui.button("Launch Launcher").clicked() {
                        self.launch_binary("vibe-launcher");
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    ui.label("Shortcuts:");
                    ui.label("  Ctrl+N  New project");
                    ui.label("  Ctrl+O  Open project");
                    ui.label("  Ctrl+S  Save");
                    ui.label("  Ctrl+Z  Undo");
                    ui.label("  Ctrl+Y  Redo");
                    ui.label("  Delete  Remove entity");
                    ui.label("  F2      Rename entity");
                    ui.label("  1-4     Switch tool");
                    ui.label("  Ctrl+Q  Quit");
                });

                ui.separator();
                let (dirty_text, dirty_color) = if self.dirty {
                    ("● Unsaved", Color32::from_rgb(245, 200, 80))
                } else {
                    ("○ Saved", Color32::from_rgb(130, 190, 130))
                };
                ui.colored_label(dirty_color, dirty_text);
                ui.separator();
                ui.label(format!(
                    "Project: {}  |  Scene: {}",
                    self.project.name,
                    self.active_scene().name
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Welcome").clicked() {
                        if self.dirty {
                            self.save_before_action = Some(SaveBeforeAction::Welcome);
                        } else {
                            self.show_welcome = true;
                        }
                    }
                });
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Tool:");
                for mode in ToolMode::all() {
                    let label = match mode {
                        ToolMode::Select => "Select [1]",
                        ToolMode::Move => "Move [2]",
                        ToolMode::Rotate => "Rotate [3]",
                        ToolMode::Scale => "Scale [4]",
                    };
                    if ui
                        .selectable_label(self.tool_mode == mode, label)
                        .clicked()
                    {
                        self.tool_mode = mode;
                    }
                }

                ui.separator();
                let play_label = if self.play_mode {
                    "⏹ Stop"
                } else {
                    "▶ Play"
                };
                if ui.button(play_label).clicked() {
                    self.play_mode = !self.play_mode;
                    self.push_log(if self.play_mode {
                        "Preview mode on"
                    } else {
                        "Preview mode off"
                    });
                }

                ui.separator();
                ui.checkbox(&mut self.show_grid, "Grid");
                ui.checkbox(&mut self.snap_enabled, "Snap");
                ui.add(
                    egui::DragValue::new(&mut self.snap_size)
                        .range(0.1..=5.0)
                        .speed(0.05)
                        .prefix("Step "),
                );

                ui.separator();
                if ui.button("+ Entity").clicked() {
                    self.add_entity();
                }
                if ui.button("Import Asset").clicked() {
                    self.import_asset_dialog();
                }
            });
        });
    }

    fn show_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("hierarchy_panel")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("Hierarchy");
                ui.horizontal(|ui| {
                    if ui.button("+ Scene").clicked() {
                        self.add_scene();
                    }
                    if ui.button("- Scene").clicked() {
                        if self.project.scenes.len() > 1 {
                            self.confirm_delete_scene = Some(self.selected_scene);
                        } else {
                            self.status_line = "Need at least one scene".to_string();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("+ Entity").clicked() {
                        self.add_entity();
                    }
                    if ui.button("Duplicate").clicked() {
                        self.duplicate_selected_entity();
                    }
                    if ui.button("Delete").clicked() {
                        if self.selected_entity.is_some() {
                            self.confirm_delete_entity = self.selected_entity;
                        }
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.hierarchy_filter);
                });
                ui.separator();

                let filter = self.hierarchy_filter.to_lowercase();
                scenes_hierarchy_ui(ui, self, &filter);
            });
    }

    fn show_inspector_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(350.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Inspector");
                    ui.separator();

                    if let Some(entity) = self.selected_entity() {
                        let before = EntitySnapshot::from(entity);
                        self.draw_entity_inspector(ui);
                        if let Some(entity) = self.selected_entity() {
                            let after = EntitySnapshot::from(entity);
                            if before != after {
                                self.push_or_replace_entity_undo(UndoCommand::ModifyEntity {
                                    scene_index: self.selected_scene,
                                    entity_id: entity.id,
                                    before,
                                    after,
                                });
                                self.dirty = true;
                            }
                        }
                    } else {
                        self.draw_scene_and_project_inspector(ui);
                    }

                    ui.separator();
                    ui.heading("Asset");
                    self.draw_selected_asset(ui);
                });
            });
    }

    fn show_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("assets_console_panel")
            .resizable(true)
            .default_height(220.0)
            .min_height(100.0)
            .show(ctx, |ui| {
                ui.columns(2, |columns| {
                    self.draw_asset_browser(&mut columns[0]);
                    self.draw_console(&mut columns[1]);
                });
            });
    }

    fn show_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            let scene = self.active_scene().clone();
            painter.rect_filled(rect, 0.0, color32_from_rgb(scene.clear_color));

            if self.show_grid {
                self.draw_grid(&painter, rect, scene.grid_size);
            }
            self.draw_entities(&painter, rect);

            if response.hovered() {
                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                if scroll.abs() > f32::EPSILON {
                    let zoom_factor = 1.0 + scroll * 0.0015;
                    self.viewport_zoom =
                        (self.viewport_zoom * zoom_factor).clamp(14.0, 280.0);
                }
            }

            if response.clicked_by(egui::PointerButton::Primary) {
                self.commit_pending_entity_edit();
                if let Some(pointer) = response.interact_pointer_pos() {
                    self.select_entity_at_screen(rect, pointer);
                }
            }

            if self.tool_mode == ToolMode::Move
                && response.dragged_by(egui::PointerButton::Primary)
            {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let (world_x, world_z) = self.screen_to_world(rect, pointer);
                    let x = self.snap_value(world_x);
                    let z = self.snap_value(world_z);
                    if let Some(entity) = self.selected_entity_mut() {
                        entity.transform.position[0] = x;
                        entity.transform.position[2] = z;
                        self.dirty = true;
                    }
                }
            }

            if response.double_clicked() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let (world_x, world_z) = self.screen_to_world(rect, pointer);
                    self.commit_pending_entity_edit();
                    self.add_entity_at(world_x, world_z);
                }
            }

            if response.hovered()
                && ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle))
            {
                let delta = ui.input(|i| i.pointer.delta());
                self.viewport_pan += delta;
            }

            let overlay = format!(
                "{} | {} entities | Zoom {:.0}%",
                scene.name,
                scene.entities.len(),
                self.viewport_zoom / 48.0 * 100.0
            );
            painter.text(
                rect.left_top() + Vec2::new(8.0, 8.0),
                egui::Align2::LEFT_TOP,
                overlay,
                FontId::monospace(13.0),
                Color32::from_rgb(210, 224, 245),
            );
        });
    }

    fn show_dialogs(&mut self, ctx: &egui::Context) {
        if let Some(id) = self.confirm_delete_entity.take() {
            let ent_name = self
                .active_scene()
                .entities
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "Entity".to_string());
            let mut delete_confirmed = false;
            let mut delete_cancelled = false;
            egui::Window::new("Delete Entity")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Delete \"{}\"?", ent_name));
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            delete_confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            delete_cancelled = true;
                        }
                    });
                });
            if delete_confirmed {
                self.remove_entity_by_id(id);
            }
            if !delete_confirmed && !delete_cancelled {
                self.confirm_delete_entity = Some(id);
            }
        }

        if let Some(index) = self.confirm_delete_scene.take() {
            let scene_name = self
                .project
                .scenes
                .get(index)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "Scene".to_string());
            let mut delete_confirmed = false;
            let mut delete_cancelled = false;
            egui::Window::new("Delete Scene")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Delete scene \"{}\"?", scene_name));
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            delete_confirmed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            delete_cancelled = true;
                        }
                    });
                });
            if delete_confirmed {
                let scene_data = self.project.scenes[index].clone();
                self.project.scenes.remove(index);
                self.push_undo(UndoCommand::RemoveScene {
                    scene: scene_data,
                    index,
                });
                self.selected_scene = self.selected_scene.min(
                    self.project.scenes.len().saturating_sub(1),
                );
                self.selected_entity = None;
                self.dirty = true;
                self.push_log(format!("Deleted scene \"{}\"", scene_name));
            }
            if !delete_confirmed && !delete_cancelled {
                self.confirm_delete_scene = Some(index);
            }
        }

        if self.confirm_new_project {
            let mut action = None;
            egui::Window::new("New Project")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Save current project first?");
                    ui.horizontal(|ui| {
                        if ui.button("Save & New").clicked() {
                            action = Some(true);
                        }
                        if ui.button("Discard & New").clicked() {
                            action = Some(false);
                        }
                        if ui.button("Cancel").clicked() {
                            action = Some(false);
                            self.confirm_new_project = false;
                        }
                    });
                });
            match action {
                Some(true) => {
                    self.save_project(false);
                    self.new_project();
                    self.confirm_new_project = false;
                }
                Some(false) => {
                    self.confirm_new_project = false;
                }
                None => {}
            }
        }

        if self.confirm_quit {
            let mut action = None;
            egui::Window::new("Unsaved Changes")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Save changes before quitting?");
                    ui.horizontal(|ui| {
                        if ui.button("Save & Quit").clicked() {
                            action = Some(true);
                        }
                        if ui.button("Discard & Quit").clicked() {
                            action = Some(false);
                        }
                        if ui.button("Cancel").clicked() {
                            action = Some(false);
                            self.confirm_quit = false;
                        }
                    });
                });
            match action {
                Some(true) => {
                    self.save_project(false);
                    self.confirm_quit = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Some(false) => {
                    self.confirm_quit = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                None => {}
            }
        }

        if let Some(action) = self.save_before_action.take() {
            let mut chosen: Option<bool> = None;
            egui::Window::new("Unsaved Changes")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Save current project first?");
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            chosen = Some(true);
                        }
                        if ui.button("Discard").clicked() {
                            chosen = Some(false);
                        }
                        if ui.button("Cancel").clicked() {
                            chosen = Some(false);
                        }
                    });
                });
            match (chosen, action.clone()) {
                (Some(true), SaveBeforeAction::NewProject) => {
                    self.save_project(false);
                    self.new_project();
                }
                (Some(false), SaveBeforeAction::NewProject) => {
                    self.new_project();
                }
                (Some(true), SaveBeforeAction::OpenProject) => {
                    self.save_project(false);
                    self.open_project_dialog();
                }
                (Some(false), SaveBeforeAction::OpenProject) => {
                    self.open_project_dialog();
                }
                (Some(true), SaveBeforeAction::Welcome) => {
                    self.save_project(false);
                    self.show_welcome = true;
                }
                (Some(false), SaveBeforeAction::Welcome) => {
                    self.show_welcome = true;
                }
                (None, _) => {
                    self.save_before_action = Some(action);
                }
            }
        }
    }

    fn draw_scene_and_project_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scene");
        ui.separator();

        let scene = self.active_scene_mut();
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut scene.name);
        });
        ui.add(
            egui::DragValue::new(&mut scene.grid_size)
                .range(0.1..=8.0)
                .speed(0.05)
                .prefix("Grid: "),
        );
        color_row_rgb(ui, "Ambient:", &mut scene.ambient_light);
        color_row_rgb(ui, "Clear:", &mut scene.clear_color);

        ui.separator();
        ui.heading("Project");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.project.name);
        });
        ui.horizontal(|ui| {
            ui.label("Author:");
            ui.text_edit_singleline(&mut self.project.author);
        });
        ui.horizontal(|ui| {
            ui.label("Startup Scene:");
            ui.text_edit_singleline(&mut self.project.settings.startup_scene);
        });
        ui.add(
            egui::DragValue::new(&mut self.project.settings.gravity)
                .speed(0.05)
                .prefix("Gravity: "),
        );
        ui.add(
            egui::DragValue::new(&mut self.project.settings.fixed_timestep)
                .speed(0.0005)
                .range(0.001..=0.1)
                .prefix("Fixed dt: "),
        );
        ui.horizontal(|ui| {
            ui.label("Lighting Quality:");
            ui.text_edit_singleline(&mut self.project.settings.lighting_quality);
        });
        ui.label("Notes:");
        ui.add(
            egui::TextEdit::multiline(&mut self.project.notes)
                .desired_rows(3)
                .hint_text("Project notes"),
        );
    }

    fn draw_entity_inspector(&mut self, ui: &mut egui::Ui) {
        let entity = match self.selected_entity_mut() {
            Some(e) => e,
            None => return,
        };

        ui.heading("Entity");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut entity.name);
        });
        ui.checkbox(&mut entity.enabled, "Enabled");
        ui.horizontal(|ui| {
            ui.label("Tags:");
            ui.text_edit_singleline(&mut entity.tags);
        });

        ui.separator();
        ui.heading("Transform");
        edit_vec3(ui, "Position", &mut entity.transform.position);
        edit_vec3(ui, "Rotation", &mut entity.transform.rotation);
        edit_vec3(ui, "Scale", &mut entity.transform.scale);

        ui.separator();
        ui.heading("Render");
        let mut has_render = entity.render.is_some();
        if ui.checkbox(&mut has_render, "Enabled").changed() {
            if has_render {
                entity.render = Some(RenderComponent::default());
            } else {
                entity.render = None;
            }
        }
        if let Some(render) = entity.render.as_mut() {
            egui::ComboBox::from_label("Kind")
                .selected_text(render.kind.label())
                .show_ui(ui, |ui| {
                    for kind in RenderKind::all() {
                        ui.selectable_value(&mut render.kind, kind, kind.label());
                    }
                });
            ui.horizontal(|ui| {
                ui.label("Mesh:");
                ui.text_edit_singleline(&mut render.mesh);
            });
            ui.horizontal(|ui| {
                ui.label("Material:");
                ui.text_edit_singleline(&mut render.material);
            });
            ui.add(egui::DragValue::new(&mut render.layer).prefix("Layer: "));
            ui.color_edit_button_rgba_unmultiplied(&mut render.color);
        }

        ui.separator();
        ui.heading("Collider");
        let mut has_collider = entity.collider.is_some();
        if ui.checkbox(&mut has_collider, "Enabled").changed() {
            if has_collider {
                entity.collider = Some(ColliderComponent::default());
            } else {
                entity.collider = None;
            }
        }
        if let Some(collider) = entity.collider.as_mut() {
            egui::ComboBox::from_label("Shape")
                .selected_text(collider.shape.label())
                .show_ui(ui, |ui| {
                    for shape in ColliderShape::all() {
                        ui.selectable_value(
                            &mut collider.shape,
                            shape,
                            shape.label(),
                        );
                    }
                });
            edit_vec3(ui, "Size", &mut collider.size);
            ui.checkbox(&mut collider.is_trigger, "Is Trigger");
        }

        ui.separator();
        ui.heading("Script");
        let mut has_script = entity.script.is_some();
        if ui.checkbox(&mut has_script, "Enabled").changed() {
            if has_script {
                entity.script = Some(ScriptComponent {
                    script_path: "scripts/new_script.rs".to_string(),
                    entry: "update".to_string(),
                });
            } else {
                entity.script = None;
            }
        }
        if let Some(script) = entity.script.as_mut() {
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut script.script_path);
            });
            ui.horizontal(|ui| {
                ui.label("Entry:");
                ui.text_edit_singleline(&mut script.entry);
            });
        }
    }

    fn draw_asset_browser(&mut self, ui: &mut egui::Ui) {
        ui.heading("Asset Browser");
        ui.horizontal(|ui| {
            if ui.button("Import File...").clicked() {
                self.import_asset_dialog();
            }
            if ui.button("Remove").clicked() {
                self.remove_selected_asset();
            }
        });

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.import_path_buffer);
            if ui.button("Add").clicked() {
                let path = self.import_path_buffer.trim().to_string();
                if !path.is_empty() {
                    self.import_asset_from_path(PathBuf::from(path));
                    self.import_path_buffer.clear();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.asset_filter);
        });
        ui.separator();

        let filter = self.asset_filter.to_lowercase();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for asset in &self.project.assets {
                if !filter.is_empty()
                    && !asset.name.to_lowercase().contains(&filter)
                {
                    continue;
                }
                let selected = self.selected_asset == Some(asset.id);
                let label = format!("{} [{}]", asset.name, asset.kind.label());
                if ui.selectable_label(selected, label).clicked() {
                    self.selected_asset = Some(asset.id);
                }
            }
            if self.project.assets.is_empty() {
                ui.label("No assets imported.");
            }
        });
    }

    fn draw_console(&mut self, ui: &mut egui::Ui) {
        ui.heading("Console");
        ui.horizontal(|ui| {
            ui.label(&self.status_line);
            if ui.button("Clear").clicked() {
                self.console.clear();
            }
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &self.console {
                    ui.label(line);
                }
            });
    }

    fn draw_selected_asset(&mut self, ui: &mut egui::Ui) {
        let selected = self.selected_asset.and_then(|id| {
            self.project
                .assets
                .iter_mut()
                .find(|asset| asset.id == id)
        });

        if let Some(asset) = selected {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut asset.name);
            });
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut asset.path);
            });

            egui::ComboBox::from_label("Kind")
                .selected_text(asset.kind.label())
                .show_ui(ui, |ui| {
                    for kind in [
                        AssetKind::Texture,
                        AssetKind::Mesh,
                        AssetKind::Material,
                        AssetKind::Audio,
                        AssetKind::Script,
                        AssetKind::Other,
                    ] {
                        ui.selectable_value(&mut asset.kind, kind, kind.label());
                    }
                });
        } else {
            ui.label("No asset selected.");
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, grid_size: f32) {
        let half = 24;
        let minor = Color32::from_rgba_premultiplied(92, 104, 124, 46);
        let major = Color32::from_rgba_premultiplied(158, 178, 212, 82);

        for i in -half..=half {
            let x = i as f32 * grid_size;
            let from = self.world_to_screen(rect, [x, 0.0, -half as f32 * grid_size]);
            let to = self.world_to_screen(rect, [x, 0.0, half as f32 * grid_size]);
            let stroke = if i == 0 {
                Stroke::new(1.3, major)
            } else {
                Stroke::new(1.0, minor)
            };
            painter.line_segment([from, to], stroke);
        }

        for i in -half..=half {
            let z = i as f32 * grid_size;
            let from =
                self.world_to_screen(rect, [-half as f32 * grid_size, 0.0, z]);
            let to =
                self.world_to_screen(rect, [half as f32 * grid_size, 0.0, z]);
            let stroke = if i == 0 {
                Stroke::new(1.3, major)
            } else {
                Stroke::new(1.0, minor)
            };
            painter.line_segment([from, to], stroke);
        }
    }

    fn draw_entities(&self, painter: &egui::Painter, rect: Rect) {
        let scene = self.active_scene();
        for entity in &scene.entities {
            let pos = self.world_to_screen(rect, entity.transform.position);
            let base_color = entity
                .render
                .as_ref()
                .map(|render| color32_from_rgba(render.color))
                .unwrap_or_else(|| Color32::from_rgb(180, 192, 220));
            let color = if entity.enabled {
                base_color
            } else {
                Color32::from_gray(90)
            };
            let radius = if self.selected_entity == Some(entity.id) {
                8.5
            } else {
                6.2
            };

            painter.circle_filled(pos, radius, color);
            if self.selected_entity == Some(entity.id) {
                painter.circle_stroke(
                    pos,
                    radius + 2.4,
                    Stroke::new(1.8, Color32::from_rgb(255, 220, 120)),
                );
            }
            painter.text(
                pos + Vec2::new(10.0, -14.0),
                egui::Align2::LEFT_TOP,
                &entity.name,
                FontId::monospace(12.0),
                Color32::from_rgb(220, 232, 250),
            );
        }
    }

    fn select_entity_at_screen(&mut self, rect: Rect, pointer: Pos2) {
        let mut best = None;
        let mut best_distance = f32::MAX;
        for entity in &self.active_scene().entities {
            let screen = self.world_to_screen(rect, entity.transform.position);
            let distance = screen.distance(pointer);
            if distance < 14.0 && distance < best_distance {
                best = Some(entity.id);
                best_distance = distance;
            }
        }
        self.selected_entity = best;
        if best.is_none() {
            self.status_line = "Scene selected".to_string();
        }
    }

    fn world_to_screen(&self, rect: Rect, world: [f32; 3]) -> Pos2 {
        let center = rect.center() + self.viewport_pan;
        Pos2::new(
            center.x + world[0] * self.viewport_zoom,
            center.y - world[2] * self.viewport_zoom,
        )
    }

    fn screen_to_world(&self, rect: Rect, screen: Pos2) -> (f32, f32) {
        let center = rect.center() + self.viewport_pan;
        (
            (screen.x - center.x) / self.viewport_zoom,
            -(screen.y - center.y) / self.viewport_zoom,
        )
    }

    fn snap_value(&self, value: f32) -> f32 {
        if self.snap_enabled {
            (value / self.snap_size).round() * self.snap_size
        } else {
            value
        }
    }

    fn add_scene(&mut self) {
        let id = self.allocate_id();
        let scene = SceneDocument {
            id,
            name: format!("Scene {}", self.project.scenes.len() + 1),
            ambient_light: [0.22, 0.24, 0.27],
            clear_color: [0.06, 0.07, 0.09],
            grid_size: 1.0,
            entities: vec![EntityDocument::default_camera()],
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
        self.push_undo(UndoCommand::AddEntity {
            scene_index,
            entity,
        });
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
        self.push_undo(UndoCommand::AddEntity {
            scene_index,
            entity,
        });
        self.selected_entity = Some(id);
        self.dirty = true;
        self.push_log("Added entity at viewport position");
    }

    fn remove_entity_by_id(&mut self, id: u64) {
        let scene_index = self.selected_scene;
        let scene = self.active_scene_mut();
        if let Some(index) = scene.entities.iter().position(|e| e.id == id) {
            let entity = scene.entities.remove(index);
            self.push_undo(UndoCommand::RemoveEntity {
                scene_index,
                entity,
                index,
            });
            self.selected_entity = None;
            self.dirty = true;
            self.push_log("Removed entity");
        }
    }

    fn duplicate_selected_entity(&mut self) {
        let id = match self.selected_entity {
            Some(id) => id,
            None => return,
        };
        let template = self
            .active_scene()
            .entities
            .iter()
            .find(|e| e.id == id)
            .cloned();
        if let Some(mut entity) = template {
            entity.id = self.allocate_id();
            entity.name = format!("{} Copy", entity.name);
            entity.transform.position[0] += self.snap_size.max(0.2);
            entity.transform.position[2] += self.snap_size.max(0.2);
            let new_id = entity.id;
            let scene_index = self.selected_scene;
            self.active_scene_mut().entities.push(entity.clone());
            self.push_undo(UndoCommand::DuplicateEntity {
                scene_index,
                entity,
            });
            self.selected_entity = Some(new_id);
            self.dirty = true;
            self.push_log("Duplicated entity");
        }
    }

    fn get_entity(&self, id: Option<u64>) -> Option<&EntityDocument> {
        let id = id?;
        self.active_scene().entities.iter().find(|e| e.id == id)
    }

    fn selected_entity(&self) -> Option<&EntityDocument> {
        self.get_entity(self.selected_entity)
    }

    fn selected_entity_mut(&mut self) -> Option<&mut EntityDocument> {
        let id = self.selected_entity?;
        self.active_scene_mut()
            .entities
            .iter_mut()
            .find(|e| e.id == id)
    }

    fn active_scene(&self) -> &SceneDocument {
        &self.project.scenes[self.selected_scene]
    }

    fn active_scene_mut(&mut self) -> &mut SceneDocument {
        &mut self.project.scenes[self.selected_scene]
    }

    fn ensure_valid_selection(&mut self) {
        if self.project.scenes.is_empty() {
            self.project.scenes.push(SceneDocument::default());
        }
        if self.selected_scene >= self.project.scenes.len() {
            self.selected_scene = self.project.scenes.len() - 1;
        }
        if let Some(entity_id) = self.selected_entity {
            let exists = self
                .active_scene()
                .entities
                .iter()
                .any(|e| e.id == entity_id);
            if !exists {
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
        if let Some(path) = FileDialog::new()
            .add_filter("Vibe Project", &["json"])
            .pick_file()
        {
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
                Err(err) => {
                    self.status_line = format!("Parse error: {}", err);
                    self.push_log(self.status_line.clone());
                }
            },
            Err(err) => {
                self.status_line = format!("Open error: {}", err);
                self.push_log(self.status_line.clone());
            }
        }
    }

    fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    fn save_project(&mut self, save_as: bool) {
        let path = if save_as || self.project_path.is_none() {
            FileDialog::new()
                .add_filter("Vibe Project", &["json"])
                .set_file_name("project.vibe.json")
                .save_file()
        } else {
            self.project_path.clone()
        };

        if let Some(path) = path {
            self.write_project(path);
        }
    }

    fn write_project(&mut self, path: PathBuf) {
        self.project.modified_unix = unix_now();
        match serde_json::to_string_pretty(&self.project) {
            Ok(serialized) => match fs::write(&path, serialized) {
                Ok(()) => {
                    let name = display_name(&path);
                    self.project_path = Some(path);
                    self.dirty = false;
                    self.status_line = format!("Saved {}", name);
                    self.push_log(self.status_line.clone());
                }
                Err(err) => {
                    self.status_line = format!("Save error: {}", err);
                    self.push_log(self.status_line.clone());
                }
            },
            Err(err) => {
                self.status_line = format!("Serialize error: {}", err);
                self.push_log(self.status_line.clone());
            }
        }
    }

    fn export_runtime_dialog(&mut self) {
        let Some(path) = FileDialog::new()
            .add_filter("Runtime Pack", &["json"])
            .set_file_name("runtime_pack.json")
            .save_file()
        else {
            return;
        };
        self.export_runtime(path);
    }

    fn export_runtime(&mut self, path: PathBuf) {
        let runtime = RuntimeExport {
            project_name: self.project.name.clone(),
            exported_unix: unix_now(),
            scenes: self
                .project
                .scenes
                .iter()
                .map(|scene| RuntimeScene {
                    name: scene.name.clone(),
                    ambient_light: scene.ambient_light,
                    clear_color: scene.clear_color,
                    entities: scene
                        .entities
                        .iter()
                        .map(|entity| RuntimeEntity {
                            name: entity.name.clone(),
                            enabled: entity.enabled,
                            tags: entity
                                .tags
                                .split(',')
                                .map(str::trim)
                                .filter(|t| !t.is_empty())
                                .map(ToOwned::to_owned)
                                .collect(),
                            transform: entity.transform.clone(),
                            render: entity.render.clone(),
                            collider: entity.collider.clone(),
                            script: entity.script.clone(),
                        })
                        .collect(),
                })
                .collect(),
        };

        match serde_json::to_string_pretty(&runtime) {
            Ok(serialized) => match fs::write(&path, serialized) {
                Ok(()) => {
                    self.status_line =
                        format!("Exported to {}", display_name(&path));
                    self.push_log(self.status_line.clone());
                }
                Err(err) => {
                    self.status_line = format!("Export error: {}", err);
                    self.push_log(self.status_line.clone());
                }
            },
            Err(err) => {
                self.status_line = format!("Export serialize error: {}", err);
                self.push_log(self.status_line.clone());
            }
        }
    }

    fn import_asset_dialog(&mut self) {
        if let Some(path) = FileDialog::new().pick_file() {
            self.import_asset_from_path(path);
        }
    }

    fn import_asset_from_path(&mut self, path: PathBuf) {
        let id = self.allocate_id();
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("asset")
            .to_string();
        let kind = infer_asset_kind(&path);
        let asset = AssetDocument {
            id,
            name: name.clone(),
            path: path.to_string_lossy().to_string(),
            kind,
        };
        self.project.assets.push(asset);
        self.selected_asset = Some(id);
        self.dirty = true;
        self.push_log(format!("Imported asset {}", name));
    }

    fn remove_selected_asset(&mut self) {
        let id = match self.selected_asset {
            Some(id) => id,
            None => return,
        };
        if let Some(index) = self
            .project
            .assets
            .iter()
            .position(|a| a.id == id)
        {
            self.project.assets.remove(index);
            self.selected_asset = None;
            self.dirty = true;
            self.push_log("Removed asset");
        }
    }

    fn launch_binary(&mut self, binary_name: &str) {
        match spawn_binary(binary_name) {
            Ok(()) => {
                self.status_line = format!("Launched {}", binary_name);
                self.push_log(self.status_line.clone());
            }
            Err(err) => {
                self.status_line = err;
                self.push_log(self.status_line.clone());
            }
        }
    }

    fn push_log(&mut self, message: impl Into<String>) {
        let entry = format!("[{}] {}", unix_now(), message.into());
        self.status_line = entry.clone();
        self.console.push(entry);
    }

    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn recompute_next_id(&mut self) {
        let mut max_id = 1u64;
        for scene in &self.project.scenes {
            max_id = max_id.max(scene.id);
            for entity in &scene.entities {
                max_id = max_id.max(entity.id);
            }
        }
        for asset in &self.project.assets {
            max_id = max_id.max(asset.id);
        }
        self.next_id = max_id + 1;
    }
}

fn scenes_hierarchy_ui(ui: &mut egui::Ui, app: &mut EditorApp, filter: &str) {
    let mut pending: Option<(usize, Option<u64>)> = None;
    let mut delete_id: Option<u64> = None;
    let mut rename_id: Option<u64> = None;
    let mut duplicate_id: Option<u64> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (scene_index, scene) in app.project.scenes.iter().enumerate() {
            let is_open = scene_index == app.selected_scene;
            let header_text = format!("{} ({})", scene.name, scene.entities.len());

            egui::CollapsingHeader::new(&header_text)
                .default_open(is_open)
                .show(ui, |ui| {
                    let scene_selected =
                        app.selected_scene == scene_index && app.selected_entity.is_none();
                    if ui
                        .selectable_label(scene_selected, "⚙ Scene Settings")
                        .clicked()
                    {
                        pending = Some((scene_index, None));
                    }

                    for entity in &scene.entities {
                        if !filter.is_empty()
                            && !entity.name.to_lowercase().contains(&filter)
                        {
                            continue;
                        }
                        let selected = app.selected_scene == scene_index
                            && app.selected_entity == Some(entity.id);
                        let label = if entity.enabled {
                            entity.name.clone()
                        } else {
                            format!("☐ {}", entity.name)
                        };
                        let resp = ui.selectable_label(selected, label);
                        if resp.clicked() {
                            pending = Some((scene_index, Some(entity.id)));
                        }
                        resp.context_menu(|ui| {
                            if ui.button("Rename  F2").clicked() {
                                rename_id = Some(entity.id);
                                ui.close_menu();
                            }
                            if ui.button("Duplicate").clicked() {
                                duplicate_id = Some(entity.id);
                                ui.close_menu();
                            }
                            if ui.button("Delete  Del").clicked() {
                                delete_id = Some(entity.id);
                                ui.close_menu();
                            }
                        });
                    }
                });
        }
    });

    if let Some(id) = delete_id {
        app.confirm_delete_entity = Some(id);
    }
    if let Some(id) = rename_id {
        app.selected_entity = Some(id);
        app.rename_target = Some(id);
        if let Some(entity) = app.selected_entity() {
            app.rename_buffer = entity.name.clone();
        }
    }
    if let Some(id) = duplicate_id {
        app.selected_entity = Some(id);
        app.duplicate_selected_entity();
    }
    if let Some((si, eid)) = pending {
        app.selected_scene = si;
        app.selected_entity = eid;
    }

    if app.rename_target.is_some() {
        ui.separator();
        ui.label("Rename:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut app.rename_buffer);
            if ui.button("✓").clicked() {
                let target = app.rename_target.take();
                let new_name = app.rename_buffer.clone();
                if target.is_some() {
                    if let Some(entity) = app.selected_entity_mut() {
                        entity.name = new_name;
                    }
                }
            }
        });
    }
}

fn edit_vec3(ui: &mut egui::Ui, label: &str, value: &mut [f32; 3]) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(&mut value[0]).speed(0.05));
        ui.add(egui::DragValue::new(&mut value[1]).speed(0.05));
        ui.add(egui::DragValue::new(&mut value[2]).speed(0.05));
    });
}

fn color_row_rgb(ui: &mut egui::Ui, label: &str, value: &mut [f32; 3]) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(&mut value[0])
                .range(0.0..=1.0)
                .speed(0.01),
        );
        ui.add(
            egui::DragValue::new(&mut value[1])
                .range(0.0..=1.0)
                .speed(0.01),
        );
        ui.add(
            egui::DragValue::new(&mut value[2])
                .range(0.0..=1.0)
                .speed(0.01),
        );
    });
}

fn infer_asset_kind(path: &Path) -> AssetKind {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tga" => AssetKind::Texture,
        "obj" | "fbx" | "gltf" | "glb" => AssetKind::Mesh,
        "wav" | "ogg" | "mp3" | "flac" => AssetKind::Audio,
        "mat" | "material" => AssetKind::Material,
        "rs" | "lua" | "js" | "ts" => AssetKind::Script,
        _ => AssetKind::Other,
    }
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file")
        .to_string()
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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
    let current_exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let mut candidates = Vec::new();

    if let Some(parent) = current_exe.parent() {
        candidates.push(parent.join(binary_name));
        candidates.push(parent.join(format!("{}.exe", binary_name)));
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest.join("target").join("debug").join(binary_name));
    candidates.push(
        manifest
            .join("target")
            .join("debug")
            .join(format!("{}.exe", binary_name)),
    );
    candidates.push(
        manifest
            .join("target")
            .join("release")
            .join(binary_name),
    );
    candidates.push(
        manifest
            .join("target")
            .join("release")
            .join(format!("{}.exe", binary_name)),
    );

    for candidate in candidates {
        if candidate.exists() {
            Command::new(&candidate)
                .spawn()
                .map_err(|err| format!("Failed to launch {}: {}", binary_name, err))?;
            return Ok(());
        }
    }

    Command::new("cargo")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["run", "--bin", binary_name])
        .spawn()
        .map_err(|err| format!("Failed to fallback-launch {}: {}", binary_name, err))?;

    Ok(())
}
