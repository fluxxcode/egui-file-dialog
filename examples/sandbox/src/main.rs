use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::{DialogMode, DialogState, FileDialog};

struct MyApp {
    file_explorer: FileDialog,

    selected_directory: Option<PathBuf>,
    selected_file: Option<PathBuf>,
    saved_file: Option<PathBuf>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            file_explorer: FileDialog::new(),

            selected_directory: None,
            selected_file: None,
            saved_file: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui application");
            egui::widgets::global_dark_light_mode_buttons(ui);

            ui.add_space(5.0);

            if ui.button("Select directory").clicked() {
                self.file_explorer.select_directory();
            }
            ui.label(format!("Selected directory: {:?}", self.selected_directory));

            ui.add_space(5.0);

            if ui.button("Select file").clicked() {
                self.file_explorer.select_file();
            }
            ui.label(format!("Selected file: {:?}", self.selected_file));

            ui.add_space(5.0);

            if ui.button("Save file").clicked() {
                self.file_explorer.save_file();
            }
            ui.label(format!("File to save: {:?}", self.saved_file));

            match self.file_explorer.update(ctx).state() {
                DialogState::Selected(path) => match self.file_explorer.mode() {
                    DialogMode::SelectDirectory => self.selected_directory = Some(path),
                    DialogMode::SelectFile => self.selected_file = Some(path),
                    DialogMode::SaveFile => self.saved_file = Some(path),
                },
                _ => {}
            };
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
