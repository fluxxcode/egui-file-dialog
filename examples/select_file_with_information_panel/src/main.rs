use std::path::PathBuf;

use eframe::egui;
use egui_commonmark::*;
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
            information_panel: InformationPanel::new()
                .add_file_preview("csv", |ui, text, _item| {
                    ui.label("CSV preview:");
                    if let Some(content) = text {
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut content.clone()).code_editor(),
                                );
                            });
                    }
                })
                // you can also override existing preview handlers
                .add_file_preview("md", |ui, text, _item| {
                    let mut cache = CommonMarkCache::default();
                    if let Some(content) = text {
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .max_width(300.0)
                            .show(ui, |ui| {
                                CommonMarkViewer::new().show(ui, &mut cache, &content);
                            });
                    }
                }),
            selected_file: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Select file").clicked() {
                self.file_dialog.select_file();
            }

            self.file_dialog.set_right_panel_width(300.0);

            self.file_dialog
                .update_with_right_panel_ui(ctx, &mut |ui, dia| {
                    self.information_panel.ui(ui, dia);
                });

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
