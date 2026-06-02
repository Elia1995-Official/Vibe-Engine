use std::path::PathBuf;
use std::process::Command;

use eframe::{egui, App};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vibe Launcher")
            .with_inner_size([860.0, 540.0])
            .with_min_inner_size([720.0, 460.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vibe Launcher",
        options,
        Box::new(|_cc| Ok(Box::new(LauncherApp::default()))),
    )
}

#[derive(Default)]
struct LauncherApp {
    status: String,
    logs: Vec<String>,
}

impl App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("launcher_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Vibe Engine Launcher");
                ui.separator();
                if ui.button("Open Repository Folder").clicked() {
                    self.open_project_folder();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);
            ui.label(
                "Launch the runtime game collection or the full editor suite from this standalone launcher.",
            );
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized([270.0, 56.0], egui::Button::new("Launch Game Collection"))
                    .clicked()
                {
                    self.launch_binary("vibe-engine");
                }

                if ui
                    .add_sized([270.0, 56.0], egui::Button::new("Launch Editor"))
                    .clicked()
                {
                    self.launch_binary("vibe-editor");
                }
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui
                    .add_sized([270.0, 44.0], egui::Button::new("Build Runtime (cargo build)"))
                    .clicked()
                {
                    self.build_runtime();
                }

                if ui
                    .add_sized([270.0, 44.0], egui::Button::new("Quit Launcher"))
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.add_space(18.0);
            if self.status.is_empty() {
                ui.label("Status: ready");
            } else {
                ui.label(format!("Status: {}", self.status));
            }

            ui.separator();
            ui.label("Activity Log");
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.logs {
                        ui.label(line);
                    }
                });
        });
    }
}

impl LauncherApp {
    fn launch_binary(&mut self, binary_name: &str) {
        match spawn_binary(binary_name) {
            Ok(()) => self.push_log(format!("Launched {}", binary_name)),
            Err(err) => self.push_log(err),
        }
    }

    fn build_runtime(&mut self) {
        let result = Command::new("cargo")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["build"])
            .spawn();

        match result {
            Ok(_) => self.push_log("Started cargo build".to_string()),
            Err(err) => self.push_log(format!("Failed to start cargo build: {}", err)),
        }
    }

    fn open_project_folder(&mut self) {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let result = if cfg!(target_os = "windows") {
            Command::new("explorer").arg(&manifest).spawn()
        } else if cfg!(target_os = "macos") {
            Command::new("open").arg(&manifest).spawn()
        } else {
            Command::new("xdg-open").arg(&manifest).spawn()
        };

        match result {
            Ok(_) => self.push_log("Opened repository folder".to_string()),
            Err(err) => self.push_log(format!("Failed to open repository folder: {}", err)),
        }
    }

    fn push_log(&mut self, message: String) {
        self.status = message.clone();
        self.logs.push(message);
    }
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
