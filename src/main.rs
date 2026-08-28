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
use config::{reported_xp_memory_mb, LayerConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Profile {
    WindowsXp,
    Windows2000,
    WindowsVista,
}

impl Profile {
    fn label(self) -> &'static str {
        match self {
            Profile::WindowsXp => "Windows XP",
            Profile::Windows2000 => "Windows 2000 (not yet implemented)",
            Profile::WindowsVista => "Windows Vista (not yet implemented)",
        }
    }

    fn is_available(self) -> bool {
        matches!(self, Profile::WindowsXp)
    }
}

fn load_icon() -> Option<egui::IconData> {
    let path = PathBuf::from("assets/logo.png");
    let bytes = fs::read(&path).ok()?;
    eframe::icon_data::from_png_bytes(&bytes).ok()
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([560.0, 620.0])
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
    apps_dir_input: String,
    apps: Vec<PathBuf>,
    selected: Option<usize>,
    status: String,
    config: LayerConfig,

    // Options
    show_options: bool,
    profile: Profile,
    memory_override_enabled: bool,
    memory_override_mb: u64,
    fullscreen: bool,
    debug_logging: bool,
}

impl AppPicker {
    fn new() -> Self {
        let apps_dir = PathBuf::from("apps");
        let config = LayerConfig::detect();
        let mut picker = Self {
            apps_dir_input: apps_dir.display().to_string(),
            apps_dir,
            apps: Vec::new(),
            selected: None,
            status: String::from("Place .exe files in the apps/ directory, then click Refresh."),
            config,
            show_options: false,
            profile: Profile::WindowsXp,
            memory_override_enabled: false,
            memory_override_mb: 512,
            fullscreen: false,
            debug_logging: false,
        };
        picker.refresh_apps();
        picker
    }

    fn effective_reported_mb(&self) -> u64 {
        if self.memory_override_enabled {
            self.memory_override_mb.max(64)
        } else {
            self.config.reported_memory_mb
        }
    }

    fn effective_config(&self) -> LayerConfig {
        LayerConfig {
            host_memory_mb: self.config.host_memory_mb,
            reported_memory_mb: self.effective_reported_mb(),
        }
    }

    fn refresh_apps(&mut self) {
        self.apps.clear();
        self.selected = None;

        let path = PathBuf::from(self.apps_dir_input.trim());
        self.apps_dir = path;

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
        if !self.profile.is_available() {
            self.status = format!(
                "WARNING: The selected profile ({}) is not implemented yet.\n\
Only the Windows XP profile is available at this time.\n\
Please switch back to Windows XP in Options.",
                self.profile.label()
            );
            return;
        }

        let Some(idx) = self.selected else {
            self.status = "No application selected.".into();
            return;
        };
        let path = &self.apps[idx];
        let cfg = self.effective_config();
        let mem = kernel32::global_memory_status_ex(&cfg);

        let mut msg = format!(
            "Starting translation layer for:\n{}\n\n\
Profile: {}\n\
GlobalMemoryStatusEx (as seen by guest apps):\n\
  TotalPhys : {} MB\n\
  AvailPhys : {} MB\n\
  MemoryLoad: {}%\n",
            path.display(),
            self.profile.label(),
            mem.total_phys / (1024 * 1024),
            mem.avail_phys / (1024 * 1024),
            mem.memory_load
        );

        if self.debug_logging {
            msg.push_str(&format!(
                "\n[Debug] Host memory: {} MB | Override active: {}\n",
                self.config.host_memory_mb, self.memory_override_enabled
            ));
        }

        msg.push_str("\nPE loading and further API translation are not yet implemented.");
        self.status = msg;
    }
}

impl eframe::App for AppPicker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply fullscreen request
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("xp-layer");
            ui.label("Windows XP compatibility layer");
            ui.separator();

            // Top bar
            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() {
                    self.refresh_apps();
                }
                if ui
                    .button(if self.show_options {
                        "Hide Options"
                    } else {
                        "Options"
                    })
                    .clicked()
                {
                    self.show_options = !self.show_options;
                }
            });

            // Options panel
            if self.show_options {
                ui.add_space(6.0);
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.heading("Options");

                    // Profile
                    ui.label("Compatibility profile:");
                    ui.horizontal(|ui| {
                        for p in [Profile::WindowsXp, Profile::Windows2000, Profile::WindowsVista] {
                            ui.radio_value(&mut self.profile, p, p.label());
                        }
                    });
                    if !self.profile.is_available() {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 80, 60),
                            "WARNING: Windows 2000 and Windows Vista profiles are not implemented yet. Only Windows XP is available.",
                        );
                    }

                    ui.add_space(6.0);
                    ui.separator();

                    // Memory override
                    ui.checkbox(
                        &mut self.memory_override_enabled,
                        "Override reported memory",
                    );
                    if self.memory_override_enabled {
                        ui.horizontal(|ui| {
                            ui.label("Reported memory (MB):");
                            ui.add(
                                egui::DragValue::new(&mut self.memory_override_mb)
                                    .range(64..=4096)
                                    .speed(16.0),
                            );
                        });
                        ui.weak(format!(
                            "Automatic mapping would report {} MB",
                            reported_xp_memory_mb(self.config.host_memory_mb)
                        ));
                    }

                    ui.add_space(4.0);

                    // Apps directory
                    ui.label("Apps directory:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.apps_dir_input);
                        if ui.button("Apply").clicked() {
                            self.refresh_apps();
                        }
                    });

                    ui.add_space(4.0);

                    // Window
                    ui.checkbox(&mut self.fullscreen, "Fullscreen");

                    // Debug
                    ui.checkbox(&mut self.debug_logging, "Debug logging (extra status detail)");
                });
                ui.add_space(6.0);
            }

            ui.separator();
            ui.label("Applications:");

            egui::ScrollArea::vertical()
                .max_height(160.0)
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

            ui.add_space(10.0);
            ui.separator();
            ui.label("Status:");
            ui.label(&self.status);

            ui.add_space(6.0);
            ui.weak(format!(
                "Host: {} MB  |  Reported to guest: {} MB  |  Profile: {}",
                self.config.host_memory_mb,
                self.effective_reported_mb(),
                self.profile.label()
            ));
        });
    }
}
