use std::{path::PathBuf, sync::Arc};

use eframe::egui;
use egui_file_dialog::{DialogMode, FileDialog};

struct MyApp {
    file_dialog: FileDialog,

    picked_directory: Option<PathBuf>,
    picked_file: Option<PathBuf>,
    picked_multiple: Option<Vec<PathBuf>>,
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
            .add_file_filter_extensions("Pictures", vec!["png", "jpg", "dds"])
            .add_file_filter(
                "RS files",
                Arc::new(|p| p.extension().unwrap_or_default() == "rs"),
            )
            .add_file_filter(
                "TOML files",
                Arc::new(|p| p.extension().unwrap_or_default() == "toml"),
            )
            .add_save_extension("Picture", "png")
            .add_save_extension("Rust Source File", "rs")
            .add_save_extension("Configuration File", "toml")
            .default_save_extension("Picture")
            .id("egui_file_dialog");

        if let Some(storage) = cc.storage {
            *file_dialog.storage_mut() =
                eframe::get_value(storage, "file_dialog_storage").unwrap_or_default();
        }

        Self {
            file_dialog,

            picked_directory: None,
            picked_file: None,
            picked_multiple: None,
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
            egui::widgets::global_theme_preference_buttons(ui);

            ui.add_space(5.0);

            if ui.button("Pick directory").clicked() {
                self.file_dialog.pick_directory();
            }
            ui.label(format!("Picked directory: {:?}", self.picked_directory));

            ui.add_space(5.0);

            if ui.button("Pick file").clicked() {
                self.file_dialog.pick_file();
            }
            ui.label(format!("Selected file: {:?}", self.picked_file));

            if ui.button("Pick multiple").clicked() {
                self.file_dialog.pick_multiple();
            }
            ui.label("Picked multiple:");

            if let Some(items) = &self.picked_multiple {
                for item in items {
                    ui.label(format!("{item:?}"));
                }
            } else {
                ui.label("None");
            }

            ui.add_space(5.0);

            if ui.button("Save file").clicked() {
                self.file_dialog.save_file();
            }
            ui.label(format!("File to save: {:?}", self.saved_file));

            self.file_dialog.update(ctx);

            if let Some(path) = self.file_dialog.take_picked() {
                match self.file_dialog.mode() {
                    DialogMode::PickDirectory => self.picked_directory = Some(path),
                    DialogMode::PickFile => self.picked_file = Some(path),
                    DialogMode::SaveFile => self.saved_file = Some(path),
                    DialogMode::PickMultiple => {}
                }
            }

            if let Some(items) = self.file_dialog.take_picked_multiple() {
                self.picked_multiple = Some(items);
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
