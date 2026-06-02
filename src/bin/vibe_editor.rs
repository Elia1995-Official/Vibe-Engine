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
        Box::new(|_cc| Ok(Box::new(EditorApp::default()))),
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
    fn label(self) -> &'static str {
        match self {
            ToolMode::Select => "Select",
            ToolMode::Move => "Move",
            ToolMode::Rotate => "Rotate",
            ToolMode::Scale => "Scale",
        }
    }

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
            notes: "Project notes...".to_string(),
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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
}

impl Default for EditorApp {
    fn default() -> Self {
        let project = ProjectDocument::default();
        let mut app = Self {
            project,
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
        };
        app.recompute_next_id();
        app
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_valid_selection();
        self.show_top_bar(ctx);
        self.show_bottom_panel(ctx);
        self.show_hierarchy_panel(ctx);
        self.show_inspector_panel(ctx);
        self.show_viewport(ctx);
    }
}

impl EditorApp {
    fn show_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("editor_top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        self.new_project();
                        ui.close_menu();
                    }
                    if ui.button("Open Project...").clicked() {
                        self.open_project_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        self.save_project(false);
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_project(true);
                        ui.close_menu();
                    }
                    if ui.button("Export Runtime Pack...").clicked() {
                        self.export_runtime_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Project", |ui| {
                    if ui.button("Launch Game Collection").clicked() {
                        self.launch_binary("vibe-engine");
                        ui.close_menu();
                    }
                    if ui.button("Launch Launcher App").clicked() {
                        self.launch_binary("vibe-launcher");
                        ui.close_menu();
                    }
                    if ui.button("Create Scene").clicked() {
                        self.add_scene();
                        ui.close_menu();
                    }
                    if ui.button("Create Entity").clicked() {
                        self.add_entity();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    ui.label("Vibe Editor gives you a full project GUI:");
                    ui.label("- Hierarchy and scene organization");
                    ui.label("- Viewport with selection and move tools");
                    ui.label("- Inspector for components");
                    ui.label("- Asset browser and import");
                    ui.label("- Save/load/export runtime packs");
                });

                ui.separator();
                ui.label(if self.dirty { "Unsaved changes" } else { "Saved" });
                ui.label(format!("Project: {}", self.project.name));
            });

            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.label("Tool:");
                for mode in ToolMode::all() {
                    if ui
                        .selectable_label(self.tool_mode == mode, mode.label())
                        .clicked()
                    {
                        self.tool_mode = mode;
                    }
                }

                ui.separator();
                if ui
                    .button(if self.play_mode { "Stop Preview" } else { "Play Preview" })
                    .clicked()
                {
                    self.play_mode = !self.play_mode;
                    self.push_log(if self.play_mode {
                        "Preview mode enabled"
                    } else {
                        "Preview mode disabled"
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
                        self.remove_selected_scene();
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("+ Entity").clicked() {
                        self.add_entity();
                    }
                    if ui
                        .add_enabled(self.selected_entity.is_some(), egui::Button::new("Duplicate"))
                        .clicked()
                    {
                        self.duplicate_selected_entity();
                    }
                    if ui
                        .add_enabled(self.selected_entity.is_some(), egui::Button::new("Delete"))
                        .clicked()
                    {
                        self.remove_selected_entity();
                    }
                });

                ui.separator();
                ui.label("Filter");
                ui.text_edit_singleline(&mut self.hierarchy_filter);
                ui.separator();

                let filter = self.hierarchy_filter.to_lowercase();
                let mut pending_scene = None;
                let mut pending_entity = None;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (scene_index, scene) in self.project.scenes.iter().enumerate() {
                        egui::CollapsingHeader::new(format!("{} ({})", scene.name, scene.entities.len()))
                            .default_open(scene_index == self.selected_scene)
                            .show(ui, |ui| {
                                let scene_selected =
                                    self.selected_scene == scene_index && self.selected_entity.is_none();
                                if ui
                                    .selectable_label(scene_selected, "Scene Settings")
                                    .clicked()
                                {
                                    pending_scene = Some(scene_index);
                                    pending_entity = Some(None);
                                }

                                for entity in &scene.entities {
                                    if !filter.is_empty()
                                        && !entity.name.to_lowercase().contains(&filter)
                                    {
                                        continue;
                                    }
                                    let selected = self.selected_scene == scene_index
                                        && self.selected_entity == Some(entity.id);
                                    if ui
                                        .selectable_label(selected, &entity.name)
                                        .clicked()
                                    {
                                        pending_scene = Some(scene_index);
                                        pending_entity = Some(Some(entity.id));
                                    }
                                }
                            });
                    }
                });

                if let Some(scene_index) = pending_scene {
                    self.selected_scene = scene_index;
                }
                if let Some(entity) = pending_entity {
                    self.selected_entity = entity;
                }
            });
    }

    fn show_inspector_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(350.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();

                if self.selected_entity.is_some() {
                    self.draw_entity_inspector(ui);
                } else {
                    self.draw_scene_and_project_inspector(ui);
                }

                ui.separator();
                ui.heading("Selected Asset");
                self.draw_selected_asset(ui);
            });
    }

    fn show_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("assets_console_panel")
            .resizable(true)
            .default_height(250.0)
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
                    self.viewport_zoom = (self.viewport_zoom * zoom_factor).clamp(14.0, 280.0);
                }
            }

            if response.clicked_by(egui::PointerButton::Primary) {
                if let Some(pointer) = response.interact_pointer_pos() {
                    self.select_entity_at_screen(rect, pointer);
                }
            }

            if self.tool_mode == ToolMode::Move && response.dragged_by(egui::PointerButton::Primary)
            {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let (world_x, world_z) = self.screen_to_world(rect, pointer);
                    let x = self.snap_value(world_x);
                    let z = self.snap_value(world_z);
                    if let Some(entity) = self.selected_entity_mut() {
                        entity.transform.position[0] = x;
                        entity.transform.position[2] = z;
                        self.mark_dirty("Moved entity in viewport");
                    }
                }
            }

            if response.double_clicked() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let (world_x, world_z) = self.screen_to_world(rect, pointer);
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
                "Scene: {} | Entities: {} | Zoom: {:.0}%",
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

    fn draw_scene_and_project_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scene");
        let mut changed = false;
        {
            let scene = self.active_scene_mut();
            changed |= ui.text_edit_singleline(&mut scene.name).changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut scene.grid_size)
                        .range(0.1..=8.0)
                        .speed(0.05)
                        .prefix("Grid "),
                )
                .changed();
            changed |= color_row_rgb(ui, "Ambient", &mut scene.ambient_light);
            changed |= color_row_rgb(ui, "Clear", &mut scene.clear_color);
        }
        if changed {
            self.mark_dirty("Updated scene settings");
        }

        ui.separator();
        ui.heading("Project");
        let mut project_changed = false;
        project_changed |= ui.text_edit_singleline(&mut self.project.name).changed();
        project_changed |= ui.text_edit_singleline(&mut self.project.author).changed();
        project_changed |= ui
            .text_edit_singleline(&mut self.project.settings.startup_scene)
            .changed();
        project_changed |= ui
            .add(
                egui::DragValue::new(&mut self.project.settings.gravity)
                    .speed(0.05)
                    .prefix("Gravity "),
            )
            .changed();
        project_changed |= ui
            .add(
                egui::DragValue::new(&mut self.project.settings.fixed_timestep)
                    .speed(0.0005)
                    .range(0.001..=0.1)
                    .prefix("Fixed dt "),
            )
            .changed();
        project_changed |= ui
            .text_edit_singleline(&mut self.project.settings.lighting_quality)
            .changed();
        project_changed |= ui
            .add(
                egui::TextEdit::multiline(&mut self.project.notes)
                    .desired_rows(4)
                    .hint_text("Project notes"),
            )
            .changed();
        if project_changed {
            self.mark_dirty("Updated project settings");
        }
    }

    fn draw_entity_inspector(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;

        if let Some(entity) = self.selected_entity_mut() {
            ui.heading("Entity");
            changed |= ui.text_edit_singleline(&mut entity.name).changed();
            changed |= ui.checkbox(&mut entity.enabled, "Enabled").changed();
            changed |= ui.text_edit_singleline(&mut entity.tags).changed();

            ui.separator();
            ui.heading("Transform");
            changed |= edit_vec3(ui, "Position", &mut entity.transform.position);
            changed |= edit_vec3(ui, "Rotation", &mut entity.transform.rotation);
            changed |= edit_vec3(ui, "Scale", &mut entity.transform.scale);

            ui.separator();
            ui.heading("Render Component");
            let mut has_render = entity.render.is_some();
            if ui.checkbox(&mut has_render, "Enabled").changed() {
                if has_render {
                    entity.render = Some(RenderComponent::default());
                } else {
                    entity.render = None;
                }
                changed = true;
            }
            if let Some(render) = entity.render.as_mut() {
                egui::ComboBox::from_label("Kind")
                    .selected_text(render.kind.label())
                    .show_ui(ui, |ui| {
                        for kind in RenderKind::all() {
                            changed |= ui
                                .selectable_value(&mut render.kind, kind, kind.label())
                                .changed();
                        }
                    });
                changed |= ui.text_edit_singleline(&mut render.mesh).changed();
                changed |= ui.text_edit_singleline(&mut render.material).changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut render.layer).prefix("Layer "))
                    .changed();
                changed |= ui
                    .color_edit_button_rgba_unmultiplied(&mut render.color)
                    .changed();
            }

            ui.separator();
            ui.heading("Collider Component");
            let mut has_collider = entity.collider.is_some();
            if ui.checkbox(&mut has_collider, "Enabled").changed() {
                if has_collider {
                    entity.collider = Some(ColliderComponent::default());
                } else {
                    entity.collider = None;
                }
                changed = true;
            }
            if let Some(collider) = entity.collider.as_mut() {
                egui::ComboBox::from_label("Shape")
                    .selected_text(collider.shape.label())
                    .show_ui(ui, |ui| {
                        for shape in ColliderShape::all() {
                            changed |= ui
                                .selectable_value(&mut collider.shape, shape, shape.label())
                                .changed();
                        }
                    });
                changed |= edit_vec3(ui, "Size", &mut collider.size);
                changed |= ui.checkbox(&mut collider.is_trigger, "Is Trigger").changed();
            }

            ui.separator();
            ui.heading("Script Component");
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
                changed = true;
            }
            if let Some(script) = entity.script.as_mut() {
                changed |= ui.text_edit_singleline(&mut script.script_path).changed();
                changed |= ui.text_edit_singleline(&mut script.entry).changed();
            }
        }

        if changed {
            self.mark_dirty("Updated entity");
        }
    }

    fn draw_asset_browser(&mut self, ui: &mut egui::Ui) {
        ui.heading("Asset Browser");
        ui.horizontal(|ui| {
            if ui.button("Import File...").clicked() {
                self.import_asset_dialog();
            }
            if ui
                .add_enabled(self.selected_asset.is_some(), egui::Button::new("Remove Selected"))
                .clicked()
            {
                self.remove_selected_asset();
            }
        });

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.import_path_buffer);
            if ui.button("Add Path").clicked() {
                let path = self.import_path_buffer.trim().to_string();
                if !path.is_empty() {
                    self.import_asset_from_path(PathBuf::from(path));
                    self.import_path_buffer.clear();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.asset_filter);
        });
        ui.separator();

        let filter = self.asset_filter.to_lowercase();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for asset in &self.project.assets {
                if !filter.is_empty() && !asset.name.to_lowercase().contains(&filter) {
                    continue;
                }

                let selected = self.selected_asset == Some(asset.id);
                let label = format!("{} [{}]", asset.name, asset.kind.label());
                if ui.selectable_label(selected, label).clicked() {
                    self.selected_asset = Some(asset.id);
                }
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
        let selected = self
            .selected_asset
            .and_then(|id| self.project.assets.iter_mut().find(|asset| asset.id == id));

        if let Some(asset) = selected {
            let mut changed = false;
            changed |= ui.text_edit_singleline(&mut asset.name).changed();
            changed |= ui.text_edit_singleline(&mut asset.path).changed();

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
                        changed |= ui
                            .selectable_value(&mut asset.kind, kind, kind.label())
                            .changed();
                    }
                });

            if changed {
                self.mark_dirty("Updated asset");
            }
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
            let from = self.world_to_screen(rect, [-half as f32 * grid_size, 0.0, z]);
            let to = self.world_to_screen(rect, [half as f32 * grid_size, 0.0, z]);
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
                painter.circle_stroke(pos, radius + 2.4, Stroke::new(1.8, Color32::from_rgb(255, 220, 120)));
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
        self.project.scenes.push(scene);
        self.selected_scene = self.project.scenes.len() - 1;
        self.selected_entity = None;
        self.mark_dirty("Added scene");
    }

    fn remove_selected_scene(&mut self) {
        if self.project.scenes.len() <= 1 {
            self.status_line = "At least one scene must exist".to_string();
            return;
        }

        self.project.scenes.remove(self.selected_scene);
        self.selected_scene = self.selected_scene.min(self.project.scenes.len() - 1);
        self.selected_entity = None;
        self.mark_dirty("Removed scene");
    }

    fn add_entity(&mut self) {
        let id = self.allocate_id();
        let entity = EntityDocument::default_cube(id);
        self.active_scene_mut().entities.push(entity);
        self.selected_entity = Some(id);
        self.mark_dirty("Added entity");
    }

    fn add_entity_at(&mut self, world_x: f32, world_z: f32) {
        let id = self.allocate_id();
        let mut entity = EntityDocument::default_cube(id);
        entity.transform.position[0] = self.snap_value(world_x);
        entity.transform.position[2] = self.snap_value(world_z);
        self.active_scene_mut().entities.push(entity);
        self.selected_entity = Some(id);
        self.mark_dirty("Added entity from viewport");
    }

    fn remove_selected_entity(&mut self) {
        let Some(entity_id) = self.selected_entity else {
            return;
        };
        let scene = self.active_scene_mut();
        if let Some(index) = scene.entities.iter().position(|entity| entity.id == entity_id) {
            scene.entities.remove(index);
            self.selected_entity = None;
            self.mark_dirty("Removed entity");
        }
    }

    fn duplicate_selected_entity(&mut self) {
        let Some(entity_id) = self.selected_entity else {
            return;
        };

        let template = self
            .active_scene()
            .entities
            .iter()
            .find(|entity| entity.id == entity_id)
            .cloned();

        if let Some(mut entity) = template {
            entity.id = self.allocate_id();
            entity.name = format!("{} Copy", entity.name);
            entity.transform.position[0] += self.snap_size.max(0.2);
            entity.transform.position[2] += self.snap_size.max(0.2);
            let new_id = entity.id;
            self.active_scene_mut().entities.push(entity);
            self.selected_entity = Some(new_id);
            self.mark_dirty("Duplicated entity");
        }
    }

    fn selected_entity_mut(&mut self) -> Option<&mut EntityDocument> {
        let entity_id = self.selected_entity?;
        self.active_scene_mut()
            .entities
            .iter_mut()
            .find(|entity| entity.id == entity_id)
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
                .any(|entity| entity.id == entity_id);
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
                    self.dirty = false;
                    self.recompute_next_id();
                    self.push_log(format!("Opened project {}", display_name(&path)));
                }
                Err(err) => {
                    self.status_line = format!("Failed to parse project: {}", err);
                    self.push_log(self.status_line.clone());
                }
            },
            Err(err) => {
                self.status_line = format!("Failed to open file: {}", err);
                self.push_log(self.status_line.clone());
            }
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
                    self.project_path = Some(path.clone());
                    self.dirty = false;
                    self.status_line = format!("Saved {}", display_name(&path));
                    self.push_log(self.status_line.clone());
                }
                Err(err) => {
                    self.status_line = format!("Failed to save: {}", err);
                    self.push_log(self.status_line.clone());
                }
            },
            Err(err) => {
                self.status_line = format!("Serialization failed: {}", err);
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
                                .filter(|tag| !tag.is_empty())
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
                    self.status_line = format!("Exported runtime pack to {}", display_name(&path));
                    self.push_log(self.status_line.clone());
                }
                Err(err) => {
                    self.status_line = format!("Failed to export runtime pack: {}", err);
                    self.push_log(self.status_line.clone());
                }
            },
            Err(err) => {
                self.status_line = format!("Runtime export serialization failed: {}", err);
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
        self.mark_dirty(format!("Imported asset {}", name));
    }

    fn remove_selected_asset(&mut self) {
        let Some(asset_id) = self.selected_asset else {
            return;
        };
        if let Some(index) = self.project.assets.iter().position(|asset| asset.id == asset_id) {
            let removed = self.project.assets.remove(index);
            self.selected_asset = None;
            self.mark_dirty(format!("Removed asset {}", removed.name));
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

    fn mark_dirty(&mut self, message: impl Into<String>) {
        self.dirty = true;
        self.push_log(message.into());
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

fn edit_vec3(ui: &mut egui::Ui, label: &str, value: &mut [f32; 3]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        changed |= ui.add(egui::DragValue::new(&mut value[0]).speed(0.05)).changed();
        changed |= ui.add(egui::DragValue::new(&mut value[1]).speed(0.05)).changed();
        changed |= ui.add(egui::DragValue::new(&mut value[2]).speed(0.05)).changed();
    });
    changed
}

fn color_row_rgb(ui: &mut egui::Ui, label: &str, value: &mut [f32; 3]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        changed |= ui
            .add(egui::DragValue::new(&mut value[0]).range(0.0..=1.0).speed(0.01))
            .changed();
        changed |= ui
            .add(egui::DragValue::new(&mut value[1]).range(0.0..=1.0).speed(0.01))
            .changed();
        changed |= ui
            .add(egui::DragValue::new(&mut value[2]).range(0.0..=1.0).speed(0.01))
            .changed();
    });
    changed
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
        .map(|duration| duration.as_secs())
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
    candidates.push(manifest.join("target").join("release").join(binary_name));
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
