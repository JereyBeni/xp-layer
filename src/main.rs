//! xp-layer - Windows XP API translation / compatibility layer
//!
//! Graphical app picker modelled on touchHLE:
//! place `.exe` files in the `apps/` directory, then select and run them.

mod api;
mod config;

use eframe::egui;
use std::fs;
use std::path::PathBuf;

use api::kernel32;
use config::LayerConfig;

fn load_icon() -> Option<egui::IconData> {
    let path = PathBuf::from("assets/logo.png");
    let bytes = fs::read(&path).ok()?;
    eframe::icon_data::from_png_bytes(&bytes).ok()
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([520.0, 460.0])
        .with_title("xp-layer - Windows XP Compatibility Layer");

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "xp-layer",
        options,
        Box::new(|_cc| Ok(Box::new(AppPicker::new()))),
    )
}

struct AppPicker {
    apps_dir: PathBuf,
    apps: Vec<PathBuf>,
    selected: Option<usize>,
    status: String,
    config: LayerConfig,
}

impl AppPicker {
    fn new() -> Self {
        let apps_dir = PathBuf::from("apps");
        let config = LayerConfig::detect();
        let mut picker = Self {
            apps_dir,
            apps: Vec::new(),
            selected: None,
            status: String::from("Place .exe files in the apps/ directory, then click Refresh."),
            config,
        };
        picker.refresh_apps();
        picker
    }

    fn refresh_apps(&mut self) {
        self.apps.clear();
        self.selected = None;

        if !self.apps_dir.exists() {
            let _ = fs::create_dir_all(&self.apps_dir);
            self.status = format!(
                "Created {} directory. Place Windows XP .exe files here.",
                self.apps_dir.display()
            );
            return;
        }

        match fs::read_dir(&self.apps_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("exe") {
                        self.apps.push(path);
                    }
                }
                self.apps.sort();
                self.status = format!("Found {} application(s).", self.apps.len());
            }
            Err(e) => {
                self.status = format!("Failed to read apps directory: {}", e);
            }
        }
    }

    fn run_selected(&mut self) {
        let Some(idx) = self.selected else {
            self.status = "No application selected.".into();
            return;
        };
        let path = &self.apps[idx];

        // Demonstrate the memory-status API that guest applications will see.
        let mem = kernel32::global_memory_status_ex(&self.config);

        self.status = format!(
            "Starting translation layer for:\n{}\n\n\
GlobalMemoryStatusEx (as seen by XP apps):\n\
  TotalPhys : {} MB\n\
  AvailPhys : {} MB\n\
  MemoryLoad: {}%\n\n\
PE loading and further API translation are not yet implemented.",
            path.display(),
            mem.total_phys / (1024 * 1024),
            mem.avail_phys / (1024 * 1024),
            mem.memory_load
        );
    }
}

impl eframe::App for AppPicker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("xp-layer");
            ui.label("Windows XP compatibility layer (early skeleton)");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() {
                    self.refresh_apps();
                }
                ui.label(format!("Apps directory: {}", self.apps_dir.display()));
            });

            ui.add_space(8.0);
            ui.label("Applications:");

            egui::ScrollArea::vertical()
                .max_height(180.0)
                .show(ui, |ui| {
                    if self.apps.is_empty() {
                        ui.label("(no .exe files found)");
                    } else {
                        for (i, path) in self.apps.iter().enumerate() {
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("<unknown>");
                            let selected = self.selected == Some(i);
                            if ui.selectable_label(selected, name).clicked() {
                                self.selected = Some(i);
                            }
                        }
                    }
                });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let can_run = self.selected.is_some();
                if ui
                    .add_enabled(can_run, egui::Button::new("Run selected application"))
                    .clicked()
                {
                    self.run_selected();
                }
            });

            ui.add_space(12.0);
            ui.separator();
            ui.label("Status:");
            ui.label(&self.status);

            ui.add_space(8.0);
            ui.weak(format!(
                "Host: {} MB  |  Reported to XP apps: {} MB",
                self.config.host_memory_mb, self.config.reported_memory_mb
            ));
        });
    }
}
