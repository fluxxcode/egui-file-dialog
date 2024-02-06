use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::{DialogMode, FileDialog};

struct MyApp {
    file_dialog: FileDialog,

    selected_file_a: Option<PathBuf>,
    selected_file_b: Option<PathBuf>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            file_dialog: FileDialog::new().id("egui_file_dialog"),

            selected_file_a: None,
            selected_file_b: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Select file a").clicked() {
                let _ = self
                    .file_dialog
                    .open(DialogMode::SelectFile, true, Some("select_a"));
            }

            if ui.button("Select file b").clicked() {
                let _ = self
                    .file_dialog
                    .open(DialogMode::SelectFile, true, Some("select_b"));
            }

            ui.label(format!("Selected file a: {:?}", self.selected_file_a));
            ui.label(format!("Selected file b: {:?}", self.selected_file_b));

            self.file_dialog.update(ctx);

            if let Some(path) = self.file_dialog.selected() {
                if self.file_dialog.operation_id() == Some("select_a") {
                    self.selected_file_a = Some(path.to_path_buf());
                }

                if self.file_dialog.operation_id() == Some("select_b") {
                    self.selected_file_b = Some(path.to_path_buf());
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
        Box::new(|ctx| Box::new(MyApp::new(ctx))),
    )
}
