use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::{DialogMode, FileDialog};

struct MyApp {
    file_dialog: FileDialog,
    picked_items: Option<Vec<PathBuf>>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            file_dialog: FileDialog::new(),
            picked_items: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Pick single").clicked() {
                self.file_dialog.pick_file();
            }
            if ui.button("Pick multiple").clicked() {
                self.file_dialog.pick_multiple();
            }

            ui.label("Picked items:");

            if let Some(items) = &self.picked_items {
                for item in items {
                    ui.label(format!("{}", item.display()));
                }
            } else {
                ui.label("None");
            }

            self.file_dialog
                .update_with_right_panel_ui(ctx, &mut |ui, dia| {
                    if dia.mode() == DialogMode::PickMultiple {
                        ui.heading("Selected items");
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                for item in dia.selected_entries() {
                                    ui.small(format!("{item:#?}"));
                                    ui.separator();
                                }
                            });
                    } else {
                        ui.heading("Active item");
                        ui.small(format!("{:#?}", dia.selected_entry()));
                    }
                });

            match self.file_dialog.mode() {
                DialogMode::PickMultiple => {
                    if let Some(items) = self.file_dialog.take_picked_multiple() {
                        self.picked_items = Some(items);
                    }
                }
                _ => {
                    if let Some(item) = self.file_dialog.take_picked() {
                        self.picked_items = Some(vec![item]);
                    }
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "File dialog example",
        eframe::NativeOptions::default(),
        Box::new(|ctx| Ok(Box::new(MyApp::new(ctx)))),
    )
}
