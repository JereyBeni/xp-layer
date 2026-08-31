//! xp-layer - Windows XP API translation / compatibility layer
//!
//! Graphical app picker modelled on touchHLE:
//! place `.exe` files in the `apps/` directory, then select and run them.

mod api;
mod config;
mod cpu;
mod pe;

use eframe::egui;
use std::fs;
use std::path::PathBuf;

use api::kernel32;
use config::{reported_xp_memory_mb, LayerConfig};
use cpu::{check_host_cpu, Emulator, HostCpuInfo};
use pe::LoadedImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Profile {
    WindowsXp,
    Windows2000,
    WindowsVista,
}

impl Profile {
    fn short_label(self) -> &'static str {
        match self {
            Profile::WindowsXp => "Windows XP",
            Profile::Windows2000 => "Windows 2000",
            Profile::WindowsVista => "Windows Vista",
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
        .with_inner_size([720.0, 560.0])
        .with_min_inner_size([560.0, 420.0])
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
    host_cpu: HostCpuInfo,

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
        let host_cpu = check_host_cpu();
        let cpu_line = format!(
            "Host CPU: {} | {}\n{}",
            host_cpu.brand,
            host_cpu.vendor,
            host_cpu.status.message()
        );
        let mut picker = Self {
            apps_dir_input: apps_dir.display().to_string(),
            apps_dir,
            apps: Vec::new(),
            selected: None,
            status: format!(
                "Welcome to xp-layer.\n\n{}\n\nPlace 32-bit Windows XP .exe files in the apps/ folder,\nthen click Refresh and select one to load.",
                cpu_line
            ),
            config,
            host_cpu,
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
                "Created {} directory.\nPlace Windows XP .exe files here, then click Refresh.",
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
                self.status = format!(
                    "Found {} application(s) in {}.\nSelect one and click Run.\n\nHost CPU: {}\n{}",
                    self.apps.len(),
                    self.apps_dir.display(),
                    self.host_cpu.brand,
                    self.host_cpu.status.message()
                );
            }
            Err(e) => {
                self.status = format!("Failed to read apps directory:\n{}", e);
            }
        }
    }

    fn run_selected(&mut self) {
        if !self.host_cpu.status.is_ok() {
            self.status = format!(
                "CPU CHECK FAILED\n\n{}\nVendor: {}\nBrand: {}\nFamily {} Model {} Stepping {}\n\nExecution is blocked until an accepted Intel host is used.\nAMD support is not included yet.",
                self.host_cpu.status.message(),
                self.host_cpu.vendor,
                self.host_cpu.brand,
                self.host_cpu.family,
                self.host_cpu.model,
                self.host_cpu.stepping
            );
            return;
        }

        if !self.profile.is_available() {
            self.status = format!(
                "WARNING\n\nThe selected profile ({}) is not implemented yet.\nOnly Windows XP is available.\n\nSwitch back to Windows XP in Options.",
                self.profile.short_label()
            );
            return;
        }

        let Some(idx) = self.selected else {
            self.status = "No application selected.".into();
            return;
        };
        let path = self.apps[idx].clone();

        let loaded = match LoadedImage::load(&path) {
            Ok(img) => img,
            Err(e) => {
                self.status = format!("Failed to load PE\n\n{}\n\n{}", path.display(), e);
                return;
            }
        };

        let cfg = self.effective_config();
        let mem = kernel32::global_memory_status_ex(&cfg);

        // Enter the CPU emulator at the PE entry point (limited steps).
        let mut emu = Emulator::from_image(&loaded);
        let report = emu.run(64);

        let mut msg = format!(
            "Loaded: {}\n\n{}\nProfile: {}\n\nHost CPU: {} ({})\n{}\n\nGlobalMemoryStatusEx:\n  TotalPhys : {} MB\n  AvailPhys : {} MB\n  MemoryLoad: {}%\n\nCPU emulator:\n  Steps      : {}\n  Stopped    : {}\n  EIP        : 0x{:08X}\n  EAX        : 0x{:08X}\n",
            path.display(),
            loaded.summary(),
            self.profile.short_label(),
            self.host_cpu.brand,
            self.host_cpu.vendor,
            self.host_cpu.status.message(),
            mem.total_phys / (1024 * 1024),
            mem.avail_phys / (1024 * 1024),
            mem.memory_load,
            report.steps,
            report.stopped_reason,
            report.eip,
            report.eax
        );

        if self.debug_logging {
            msg.push_str(&format!(
                "\n[Debug] Host RAM: {} MB | Override: {} | Family {} Model {}\n",
                self.config.host_memory_mb,
                self.memory_override_enabled,
                self.host_cpu.family,
                self.host_cpu.model
            ));
        }

        msg.push_str(
            "\nNote: only a tiny opcode set is implemented so far; real apps will stop on the first unknown instruction.",
        );
        self.status = msg;
    }
}

impl eframe::App for AppPicker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));

        let title_blue = egui::Color32::from_rgb(0, 0, 160);
        let accent_magenta = egui::Color32::from_rgb(200, 0, 200);

        egui::TopBottomPanel::top("header")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.painter().rect_filled(ui.max_rect(), 0.0, title_blue);
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    ui.heading(
                        egui::RichText::new("xp-layer")
                            .color(egui::Color32::WHITE)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("  Windows XP compatibility layer")
                            .color(egui::Color32::from_rgb(200, 200, 255))
                            .small(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
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
                        if ui.button("Refresh").clicked() {
                            self.refresh_apps();
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("footer")
            .exact_height(28.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    let cpu_ok = self.host_cpu.status.is_ok();
                    ui.label(
                        egui::RichText::new(format!(
                            "Host: {} MB  |  Guest: {} MB  |  {}  |  CPU: {}  |  Apps: {}",
                            self.config.host_memory_mb,
                            self.effective_reported_mb(),
                            self.profile.short_label(),
                            if cpu_ok { "Intel OK" } else { "CPU blocked" },
                            self.apps.len()
                        ))
                        .small()
                        .color(if cpu_ok {
                            egui::Color32::GRAY
                        } else {
                            egui::Color32::from_rgb(220, 100, 80)
                        }),
                    );
                });
            });

        if self.show_options {
            egui::SidePanel::right("options")
                .resizable(true)
                .default_width(280.0)
                .show(ctx, |ui| {
                    ui.heading("Options");
                    ui.separator();

                    ui.label("Compatibility profile");
                    for p in [
                        Profile::WindowsXp,
                        Profile::Windows2000,
                        Profile::WindowsVista,
                    ] {
                        ui.radio_value(&mut self.profile, p, p.short_label());
                    }
                    if !self.profile.is_available() {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 80, 60),
                            "Not implemented yet. Only Windows XP works.",
                        );
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.label("Host CPU");
                    ui.monospace(format!("{}", self.host_cpu.brand));
                    ui.weak(format!(
                        "{} | family {} model {}",
                        self.host_cpu.vendor, self.host_cpu.family, self.host_cpu.model
                    ));
                    if self.host_cpu.status.is_ok() {
                        ui.colored_label(egui::Color32::from_rgb(80, 160, 80), "Accepted (Intel)");
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 80, 60),
                            self.host_cpu.status.message(),
                        );
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    ui.checkbox(
                        &mut self.memory_override_enabled,
                        "Override reported memory",
                    );
                    if self.memory_override_enabled {
                        ui.horizontal(|ui| {
                            ui.label("MB:");
                            ui.add(
                                egui::DragValue::new(&mut self.memory_override_mb)
                                    .range(64..=4096)
                                    .speed(16.0),
                            );
                        });
                        ui.weak(format!(
                            "Auto would be {} MB",
                            reported_xp_memory_mb(self.config.host_memory_mb)
                        ));
                    }

                    ui.add_space(8.0);
                    ui.label("Apps directory");
                    ui.text_edit_singleline(&mut self.apps_dir_input);
                    if ui.button("Apply path").clicked() {
                        self.refresh_apps();
                    }

                    ui.add_space(8.0);
                    ui.checkbox(&mut self.fullscreen, "Fullscreen");
                    ui.checkbox(&mut self.debug_logging, "Debug logging");

                    ui.add_space(12.0);
                    ui.separator();
                    ui.weak("Made for Linux and Windows.");
                    ui.weak("CPU: Intel 2006+ only (AMD later).");
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);

            let mut run_after_list = false;

            ui.columns(2, |cols| {
                cols[0].group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Applications");
                        ui.label(
                            egui::RichText::new(format!("({})", self.apps.len()))
                                .color(egui::Color32::GRAY),
                        );
                    });
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("app_list")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.apps.is_empty() {
                                ui.label(
                                    egui::RichText::new("(no .exe files found)")
                                        .italics()
                                        .color(egui::Color32::GRAY),
                                );
                                ui.add_space(8.0);
                                ui.label("Drop 32-bit XP executables into the apps/ folder.");
                            } else {
                                for (i, path) in self.apps.iter().enumerate() {
                                    let name = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("<unknown>");
                                    let selected = self.selected == Some(i);
                                    let response = ui.selectable_label(selected, name);
                                    if response.clicked() {
                                        self.selected = Some(i);
                                    }
                                    if response.double_clicked() {
                                        self.selected = Some(i);
                                        run_after_list = true;
                                    }
                                }
                            }
                        });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let can_run = self.selected.is_some();
                    let run_btn = egui::Button::new(
                        egui::RichText::new("  Run selected application  ").strong(),
                    )
                    .fill(if can_run {
                        egui::Color32::from_rgb(0, 90, 180)
                    } else {
                        egui::Color32::DARK_GRAY
                    })
                    .min_size(egui::vec2(0.0, 32.0));

                    if ui.add_enabled(can_run, run_btn).clicked() {
                        run_after_list = true;
                    }

                    if let Some(idx) = self.selected {
                        if let Some(path) = self.apps.get(idx) {
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            ui.label(
                                egui::RichText::new(format!("Selected: {}", name))
                                    .small()
                                    .color(accent_magenta),
                            );
                        }
                    }
                });

                cols[1].group(|ui| {
                    ui.heading("Status");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("status_view")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&self.status).monospace().size(12.5),
                                )
                                .wrap(),
                            );
                        });
                });
            });

            if run_after_list {
                self.run_selected();
            }
        });
    }
}
