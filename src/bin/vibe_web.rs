use eframe::{egui, App};
use egui::{Color32, FontId, Pos2, Rect, Stroke, Vec2};
use serde::Deserialize;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vibe Web")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([860.0, 580.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vibe Web",
        options,
        Box::new(|_cc| Ok(Box::new(VibeWebApp::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("window should exist in browser");
        let document = window
            .document()
            .expect("document should exist in browser");
        let canvas = document
            .get_element_by_id("vibe_web_canvas")
            .expect("missing #vibe_web_canvas in index.html")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#vibe_web_canvas is not a canvas element");

        let web_options = eframe::WebOptions::default();
        if let Err(err) = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(VibeWebApp::default()))),
            )
            .await
        {
            eprintln!("Failed to start web app: {:?}", err);
        }
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum WebTab {
    #[default]
    Home,
    RuntimePack,
    Build,
}

impl WebTab {
    fn label(self) -> &'static str {
        match self {
            WebTab::Home => "Home",
            WebTab::RuntimePack => "Runtime Pack",
            WebTab::Build => "Build",
        }
    }

    fn all() -> [WebTab; 3] {
        [WebTab::Home, WebTab::RuntimePack, WebTab::Build]
    }
}

#[derive(Default)]
struct VibeWebApp {
    tab: WebTab,
    selected_game: usize,
    runtime_pack_text: String,
    runtime_pack_info: Option<RuntimePackInfo>,
    runtime_pack_error: Option<String>,
    time: f32,
}

impl App for VibeWebApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|input| input.stable_dt).max(1.0 / 120.0);
        self.time += dt;

        egui::TopBottomPanel::top("web_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("Vibe Engine Web");
                ui.separator();
                for tab in WebTab::all() {
                    if ui.selectable_label(self.tab == tab, tab.label()).clicked() {
                        self.tab = tab;
                    }
                }
            });
        });

        match self.tab {
            WebTab::Home => self.draw_home(ctx),
            WebTab::RuntimePack => self.draw_runtime_pack(ctx),
            WebTab::Build => self.draw_build_info(ctx),
        }
    }
}

impl VibeWebApp {
    fn draw_home(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let (rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());
            let painter = ui.painter_at(rect);

            draw_stars(&painter, rect, self.time);

            let panel = Rect::from_min_size(
                rect.center() - Vec2::new(420.0, 260.0),
                Vec2::new(840.0, 520.0),
            );
            painter.rect_filled(panel, 14.0, Color32::from_rgba_premultiplied(12, 18, 34, 230));
            painter.rect_stroke(
                panel,
                14.0,
                Stroke::new(1.0, Color32::from_rgb(66, 108, 160)),
            );

            ui.allocate_new_ui(
                egui::UiBuilder::new().max_rect(panel.shrink2(Vec2::new(18.0, 18.0))),
                |ui| {
                ui.heading("Web Runtime Launcher");
                ui.label("Choose a demo profile for browser runtime preview.");
                ui.separator();

                const GAMES: [&str; 5] = [
                    "Space Shooter",
                    "3D Snake",
                    "FPS Arena",
                    "3D Platformer",
                    "Tetris",
                ];

                ui.horizontal_wrapped(|ui| {
                    for (index, game) in GAMES.iter().enumerate() {
                        if ui
                            .add_sized(
                                [145.0, 42.0],
                                egui::Button::new(*game)
                                    .fill(if self.selected_game == index {
                                        Color32::from_rgb(54, 110, 170)
                                    } else {
                                        Color32::from_rgb(36, 54, 90)
                                    }),
                            )
                            .clicked()
                        {
                            self.selected_game = index;
                        }
                    }
                });

                ui.add_space(14.0);
                ui.label(format!("Selected: {}", GAMES[self.selected_game]));
                ui.label("This web client is WASM-native and can be hosted as static files.");
                ui.label("Use Runtime Pack tab to inspect exported editor packs in browser.");

                ui.add_space(20.0);
                let progress = ((self.time * 0.75).sin() * 0.5 + 0.5) as f32;
                ui.add(
                    egui::ProgressBar::new(progress)
                        .desired_width(420.0)
                        .text("Web runtime heartbeat"),
                );
                ui.add_space(10.0);
                ui.colored_label(
                    Color32::from_rgb(166, 212, 252),
                    "Tip: build with trunk and deploy dist-web/ to any static host.",
                );
                },
            );
        });
    }

    fn draw_runtime_pack(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Runtime Pack Inspector");
            ui.label("Paste exported runtime JSON from the desktop editor, then validate it in browser.");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Load Example JSON").clicked() {
                    self.runtime_pack_text = example_runtime_pack();
                }
                if ui.button("Parse Pack").clicked() {
                    self.parse_runtime_pack();
                }
                if ui.button("Clear").clicked() {
                    self.runtime_pack_text.clear();
                    self.runtime_pack_error = None;
                    self.runtime_pack_info = None;
                }
            });

            ui.add(
                egui::TextEdit::multiline(&mut self.runtime_pack_text)
                    .desired_rows(20)
                    .font(FontId::monospace(13.0))
                    .hint_text("Paste runtime pack JSON here..."),
            );

            ui.separator();

            if let Some(error) = &self.runtime_pack_error {
                ui.colored_label(Color32::from_rgb(255, 122, 122), error);
            }

            if let Some(info) = &self.runtime_pack_info {
                ui.colored_label(Color32::from_rgb(148, 222, 170), "Runtime pack parsed successfully");
                ui.label(format!("Project: {}", info.project_name));
                ui.label(format!("Scenes: {}", info.scene_count));
                ui.label(format!("Entities: {}", info.entity_count));
                ui.label(format!("Scripts: {}", info.script_count));
                ui.label(format!("Render Components: {}", info.render_component_count));
                ui.label(format!("Collider Components: {}", info.collider_component_count));
                ui.separator();
                ui.label("Scene names:");
                for name in &info.scene_names {
                    ui.label(format!("- {}", name));
                }
            }
        });
    }

    fn draw_build_info(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Web Build Instructions");
            ui.separator();
            ui.label("1. Add wasm target:");
            ui.monospace("rustup target add wasm32-unknown-unknown");
            ui.add_space(8.0);
            ui.label("2. Install trunk:");
            ui.monospace("cargo install trunk");
            ui.add_space(8.0);
            ui.label("3. Serve locally:");
            ui.monospace("trunk serve --config Trunk.toml");
            ui.add_space(8.0);
            ui.label("4. Build static output:");
            ui.monospace("trunk build --release --config Trunk.toml");
            ui.add_space(14.0);
            ui.label("The output goes to dist-web/ and can be hosted on any static hosting provider.");
        });
    }

    fn parse_runtime_pack(&mut self) {
        let parsed = serde_json::from_str::<RuntimePack>(&self.runtime_pack_text);
        match parsed {
            Ok(pack) => {
                let mut entity_count = 0usize;
                let mut script_count = 0usize;
                let mut render_component_count = 0usize;
                let mut collider_component_count = 0usize;
                let mut scene_names = Vec::new();

                for scene in &pack.scenes {
                    scene_names.push(scene.name.clone());
                    entity_count += scene.entities.len();
                    for entity in &scene.entities {
                        if entity.script.is_some() {
                            script_count += 1;
                        }
                        if entity.render.is_some() {
                            render_component_count += 1;
                        }
                        if entity.collider.is_some() {
                            collider_component_count += 1;
                        }
                    }
                }

                self.runtime_pack_info = Some(RuntimePackInfo {
                    project_name: pack.project_name,
                    scene_count: pack.scenes.len(),
                    entity_count,
                    script_count,
                    render_component_count,
                    collider_component_count,
                    scene_names,
                });
                self.runtime_pack_error = None;
            }
            Err(err) => {
                self.runtime_pack_info = None;
                self.runtime_pack_error = Some(format!("Runtime pack parse failed: {err}"));
            }
        }
    }
}

fn draw_stars(painter: &egui::Painter, rect: Rect, time: f32) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(6, 10, 22));

    let w = rect.width();
    let h = rect.height();
    let stars = 210;

    for i in 0..stars {
        let seed = i as f32 * 17.37;
        let speed = 0.06 + (seed.sin().abs() * 0.22);
        let x_phase = (seed * 1.71).sin() * 0.5 + 0.5;
        let y_phase = ((seed * 2.31 + time * speed * 8.0).sin() * 0.5 + 0.5) * h;

        let x = rect.left() + x_phase * w;
        let y = rect.top() + y_phase;
        let alpha = (100.0 + (seed.cos().abs() * 120.0)) as u8;
        let radius = 0.7 + seed.sin().abs() * 1.7;
        painter.circle_filled(
            Pos2::new(x, y),
            radius,
            Color32::from_rgba_premultiplied(170, 210, 255, alpha),
        );
    }
}

#[derive(Deserialize)]
struct RuntimePack {
    project_name: String,
    scenes: Vec<RuntimeScene>,
}

#[derive(Deserialize)]
struct RuntimeScene {
    name: String,
    entities: Vec<RuntimeEntity>,
}

#[derive(Deserialize)]
struct RuntimeEntity {
    render: Option<serde_json::Value>,
    collider: Option<serde_json::Value>,
    script: Option<serde_json::Value>,
}

struct RuntimePackInfo {
    project_name: String,
    scene_count: usize,
    entity_count: usize,
    script_count: usize,
    render_component_count: usize,
    collider_component_count: usize,
    scene_names: Vec<String>,
}

fn example_runtime_pack() -> String {
    r#"{
  "project_name": "Web Demo Project",
  "exported_unix": 1735689600,
  "scenes": [
    {
      "name": "Main Scene",
      "ambient_light": [0.2, 0.24, 0.28],
      "clear_color": [0.05, 0.07, 0.11],
      "entities": [
        {
          "name": "Camera",
          "enabled": true,
          "tags": ["camera"],
          "transform": {
            "position": [0.0, 2.0, 7.0],
            "rotation": [-10.0, 0.0, 0.0],
            "scale": [1.0, 1.0, 1.0]
          },
          "render": null,
          "collider": null,
          "script": {
            "script_path": "scripts/camera_controller.rs",
            "entry": "update"
          }
        },
        {
          "name": "Crate",
          "enabled": true,
          "tags": ["prop"],
          "transform": {
            "position": [1.0, 0.0, 0.0],
            "rotation": [0.0, 0.0, 0.0],
            "scale": [1.0, 1.0, 1.0]
          },
          "render": {
            "kind": "Mesh",
            "mesh": "meshes/crate.glb"
          },
          "collider": {
            "shape": "Box",
            "size": [1.0, 1.0, 1.0],
            "is_trigger": false
          },
          "script": null
        }
      ]
    }
  ]
}"#
    .to_string()
}
