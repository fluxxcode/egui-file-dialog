use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::{FileDialog, FileDialogLabels};

#[derive(Debug, Clone, PartialEq)]
enum Language {
    English,
    German,
}

fn get_labels_german() -> FileDialogLabels {
    FileDialogLabels {
        title_select_directory: "📁 Ordner Öffnen".to_string(),
        title_select_file: "📂 Datei Öffnen".to_string(),
        title_save_file: "📥 Datei Speichern".to_string(),

        heading_places: "Orte".to_string(),
        heading_devices: "Medien".to_string(),
        heading_removable_devices: "Wechselmedien".to_string(),

        home_dir: "🏠  Zuhause".to_string(),
        desktop_dir: "🖵  Desktop".to_string(),
        documents_dir: "🗐  Dokumente".to_string(),
        downloads_dir: "📥  Downloads".to_string(),
        audio_dir: "🎵  Audio".to_string(),
        pictures_dir: "🖼  Fotos".to_string(),
        videos_dir: "🎞  Videos".to_string(),

        selected_directory: "Ausgewählter Ordner:".to_string(),
        selected_file: "Ausgewählte Datei:".to_string(),
        file_name: "Dateiname:".to_string(),

        open_button: "🗀  Öffnen".to_string(),
        save_button: "📥  Speichern".to_string(),
        cancel_button: "🚫 Abbrechen".to_string(),

        err_empty_folder_name: "Der Ordnername darf nicht leer sein".to_string(),
        err_empty_file_name: "Der Dateiname darf nicht leer sein".to_string(),
        err_directory_exists: "Ein Ordner mit diesem Namen existiert bereits".to_string(),
        err_file_exists: "Eine Datei mit diesem Namen existiert bereits".to_string(),

        ..Default::default()
    }
}

struct MyApp {
    file_dialog: FileDialog,
    language: Language,

    selected_file: Option<PathBuf>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            file_dialog: FileDialog::new().id("egui_file_dialog"),
            language: Language::English,

            selected_file: None,
        }
    }

    fn update_labels(&mut self) {
        *self.file_dialog.labels_mut() = match self.language {
            Language::English => FileDialogLabels::default(),
            Language::German => get_labels_german(),
        };
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let language_before = self.language.clone();

            egui::ComboBox::from_label("Language")
                .selected_text(format!("{:?}", self.language))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.language, Language::English, "English");
                    ui.selectable_value(&mut self.language, Language::German, "German");
                })
                .inner;

            if language_before != self.language {
                self.update_labels();
            }

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
