use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::FileDialog;

struct MyApp {
    file_dialog: FileDialog,
    selected_file: Option<PathBuf>,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut file_dialog = FileDialog::default();

        // Load the persistent data of the file dialog.
        // Alternatively, you can also use the `FileDialog::storage` builder method.
        if let Some(storage) = cc.storage {
            *file_dialog.storage_mut() =
                eframe::get_value(storage, "file_dialog_storage").unwrap_or_default()
        }

        Self {
            file_dialog,
            selected_file: None,
        }
    }
}

impl eframe::App for MyApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Save the persistent data of the file dialog
        eframe::set_value(
            storage,
            "file_dialog_storage",
            self.file_dialog.storage_mut(),
        );
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Select file").clicked() {
                self.file_dialog.select_file();
            }

            ui.label(format!("Selected file: {:?}", self.selected_file));

            self.file_dialog.update(ctx);

            if let Some(path) = self.file_dialog.take_selected() {
                self.selected_file = Some(path);
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
