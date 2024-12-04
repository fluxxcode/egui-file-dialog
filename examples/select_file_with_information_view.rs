use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::information_panel::InformationPanel;
use egui_file_dialog::FileDialog;

struct MyApp {
    file_dialog: FileDialog,
    information_panel: InformationPanel,
    selected_file: Option<PathBuf>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            file_dialog: FileDialog::new(),
            information_panel: InformationPanel::default()
                .add_file_preview("csv", |ui, item| {
                    ui.label("CSV preview:");
                    if let Some(mut content) = item.content() {
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut content).code_editor());
                            });
                    }
                })
                // add additional metadata loader
                .add_metadata_loader("pdf", |other_meta_data, path| {
                    // as a simple example, just show the Filename of the PDF
                    other_meta_data.insert("PDF Filename".to_string(), format!("{:?}", path));
                }),
            selected_file: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Select file").clicked() {
                self.file_dialog.pick_file();
            }

            self.file_dialog.set_right_panel_width(300.0);

            if let Some(path) = self
                .file_dialog
                .update_with_right_panel_ui(ctx, &mut |ui, dia| {
                    self.information_panel.ui(ui, dia);
                })
                .picked()
            {
                self.selected_file = Some(path.to_path_buf());
            }

            ui.label(format!("Selected file: {:?}", self.selected_file));
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "File dialog example",
        eframe::NativeOptions::default(),
        Box::new(|ctx| {
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            Ok(Box::new(MyApp::new(ctx)))
        }),
    )
}
