use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::FileDialog;

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

            self.file_dialog.update(ctx);

            if let Some(items) = self.file_dialog.take_picked_multiple() {
                self.picked_items = Some(items);
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
