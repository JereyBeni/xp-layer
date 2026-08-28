//! xp-layer — Windows XP API translation / compatibility layer
//!
//! Graphical app picker modelled on touchHLE:
//! place `.exe` files in the `apps/` directory, then select and run them.

mod config;

use eframe::egui;
use std::fs;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 420.0])
            .with_title("xp-layer — Windows XP Compatibility Layer"),
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
    reported_memory_mb: u64,
}

impl AppPicker {
    fn new() -> Self {
        let apps_dir = PathBuf::from("apps");
        let mut picker = Self {
            apps_dir,
            apps: Vec::new(),
            selected: None,
            status: String::from("Place .exe files in the apps/ directory, then click Refresh."),
            // Placeholder until real host detection is added
            reported_memory_mb: config::reported_xp_memory_mb(8 * 1024),
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
        // Placeholder: later this will load the PE and start the translation layer.
        self.status = format!(
            "Starting translation layer for:\n{}\n\n(Reported XP memory: {} MB)\n\nPE loading and API translation are not yet implemented.",
            path.display(),
            self.reported_memory_mb
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
                "Memory policy active — applications will see {} MB RAM",
                self.reported_memory_mb
            ));
        });
    }
}
