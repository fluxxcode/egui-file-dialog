use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::{DialogMode, FileDialog};

struct MyApp {
    file_dialog: FileDialog,

    selected_directory: Option<PathBuf>,
    selected_file: Option<PathBuf>,
    saved_file: Option<PathBuf>,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut file_dialog = FileDialog::new()
            .add_quick_access("Project", |s| {
                s.add_path("☆  Examples", "examples");
                s.add_path("📷  Media", "media");
                s.add_path("📂  Source", "src");
            })
            .id("egui_file_dialog");

        if let Some(storage) = cc.storage {
            *file_dialog.storage_mut() =
                eframe::get_value(storage, "file_dialog_storage").unwrap_or_default()
        }

        Self {
            file_dialog: file_dialog,

            selected_directory: None,
            selected_file: None,
            saved_file: None,
        }
    }
}

impl eframe::App for MyApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            "file_dialog_storage",
            self.file_dialog.storage_mut(),
        );
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui application");
            egui::widgets::global_dark_light_mode_buttons(ui);

            ui.add_space(5.0);

            if ui.button("Select directory").clicked() {
                self.file_dialog.select_directory();
            }
            ui.label(format!("Selected directory: {:?}", self.selected_directory));

            ui.add_space(5.0);

            if ui.button("Select file").clicked() {
                self.file_dialog.select_file();
            }
            ui.label(format!("Selected file: {:?}", self.selected_file));

            ui.add_space(5.0);

            if ui.button("Save file").clicked() {
                self.file_dialog.save_file();
            }
            ui.label(format!("File to save: {:?}", self.saved_file));

            self.file_dialog.update(ctx);

            if let Some(path) = self.file_dialog.take_selected() {
                match self.file_dialog.mode() {
                    DialogMode::SelectDirectory => self.selected_directory = Some(path),
                    DialogMode::SelectFile => self.selected_file = Some(path),
                    DialogMode::SaveFile => self.saved_file = Some(path),
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
