//! PenguinWash GUI using egui (pure Rust, no GTK dependency)

use std::collections::HashSet;
use egui::*;
use crate::{run_scan, Config, ScanResult};

/// The main PenguinWash application state
pub struct PenguinWashApp {
    scan_result: Option<ScanResult>,
    is_scanning: bool,
    selected_keys: HashSet<String>,
}

impl Default for PenguinWashApp {
    fn default() -> Self {
        Self {
            scan_result: None,
            is_scanning: false,
            selected_keys: HashSet::new(),
        }
    }
}

impl eframe::App for PenguinWashApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Trigger scan if requested
        if self.is_scanning {
            self.is_scanning = false;
            let result = poll_async_scan();
            self.scan_result = Some(result);
            if let Some(ref scan) = self.scan_result {
                for cat in &scan.categories {
                    if cat.auto_cleanable {
                        self.selected_keys.insert(cat.key.clone());
                    }
                }
            }
        }

        // ── Header panel ─────────────────────────────────────────────
        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🐧 PenguinWash");
                ui.separator();
                if ui.button("🔄 Scan").clicked() {
                    self.is_scanning = true;
                }
                if self.scan_result.is_some() {
                    if ui.button("Clear").clicked() {
                        self.scan_result = None;
                        self.selected_keys.clear();
                    }
                }
            });
        });

        // ── Main content ─────────────────────────────────────────────
        CentralPanel::default().show(ctx, |ui| {
            match &self.scan_result {
                None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.heading("🐧 PenguinWash");
                        ui.label("Free, open-source Linux system cleaner");
                        ui.add_space(30.0);
                        if ui.button("▶  Scan System").clicked() {
                            self.is_scanning = true;
                        }
                    });
                }
                Some(result) => {
                    ui.label(format!(
                        "Total reclaimable: {} | {} categories | scanned in {}ms",
                        result.grand_total_size_formatted(),
                        result.categories.len(),
                        result.scan_duration_ms
                    ));
                    ui.separator();

                    ScrollArea::vertical().show(ui, |ui| {
                        for cat in &result.categories {
                            let key = cat.key.clone();
                            let is_selected = self.selected_keys.contains(&key);

                            ui.horizontal(|ui| {
                                let mut checked = is_selected;
                                ui.checkbox(&mut checked, "");
                                ui.label(&cat.name);
                                ui.separator();
                                ui.label(format!("{} ({})", cat.total_size_formatted(), cat.item_count));
                                if !cat.auto_cleanable {
                                    ui.label("(manual)");
                                }
                                if checked {
                                    self.selected_keys.insert(key.clone());
                                } else {
                                    self.selected_keys.remove(&key);
                                }
                            });
                        }
                    });

                    ui.separator();
                    let selected_count = self.selected_keys.len();
                    let selected_size: u64 = result.categories.iter()
                        .filter(|c| self.selected_keys.contains(&c.key))
                        .map(|c| c.total_size)
                        .sum();

                    ui.horizontal(|ui| {
                        ui.label(format!("{} categories selected ({})", selected_count, humanize_bytes(selected_size)));
                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            if ui.button("🧹 Clean Selected").clicked() {
                                // TODO: implement clean
                            }
                        });
                    });
                }
            }
        });
    }
}

/// Poll async scan result (blocking in egui thread)
fn poll_async_scan() -> ScanResult {
    let config = Config::default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(run_scan(&config)).unwrap_or_else(|_| ScanResult {
        categories: vec![],
        grand_total_size: 0,
        scan_duration_ms: 0,
    })
}

/// Human-readable byte size
fn humanize_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, units[unit_idx])
}

/// Run the egui GUI
pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 600.0])
            .with_title("PenguinWash 🐧"),
        ..Default::default()
    };

    match eframe::run_native(
        "PenguinWash",
        options,
        Box::new(|_cc| Ok(Box::new(PenguinWashApp::default()))),
    ) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("GUI error: {:?}", e);
            Err(Box::new(e))
        }
    }
}
