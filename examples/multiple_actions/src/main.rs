use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::{DialogMode, FileDialog};

struct MyApp {
    file_dialog: FileDialog,

    picked_file_a: Option<PathBuf>,
    picked_file_b: Option<PathBuf>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            file_dialog: FileDialog::new().id("egui_file_dialog"),

            picked_file_a: None,
            picked_file_b: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Pick file a").clicked() {
                let _ = self
                    .file_dialog
                    .open(DialogMode::SelectFile, true, Some("pick_a"));
            }

            if ui.button("Pick file b").clicked() {
                let _ = self
                    .file_dialog
                    .open(DialogMode::SelectFile, true, Some("pick_b"));
            }

            ui.label(format!("Pick file a: {:?}", self.picked_file_a));
            ui.label(format!("Pick file b: {:?}", self.picked_file_b));

            self.file_dialog.update(ctx);

            if let Some(path) = self.file_dialog.picked() {
                if self.file_dialog.operation_id() == Some("pick_a") {
                    self.picked_file_a = Some(path.to_path_buf());
                }

                if self.file_dialog.operation_id() == Some("pick_b") {
                    self.picked_file_b = Some(path.to_path_buf());
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1080.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui application",
        options,
        Box::new(|ctx| Ok(Box::new(MyApp::new(ctx)))),
    )
}
