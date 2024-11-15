#![cfg(feature = "info_panel")]
use crate::{DirectoryEntry, FileDialog};
use chrono::{DateTime, Local};
use egui::ahash::{HashMap, HashMapExt};
use egui::Ui;
use image::RgbaImage;
use std::fs;
use std::io::Read;

pub struct InformationPanel {
    pub meta_data: MetaData,
    pub load_text_content: bool,
    pub load_image_content: bool,
    supported_files: HashMap<String, Box<dyn FnMut(&mut Ui, Option<String>, &DirectoryEntry)>>,
}

impl InformationPanel {
    pub fn new() -> Self {
        let mut supported_files = HashMap::new();
        for text_extension in [
            "txt", "json", "md", "toml", "rtf", "xml", "rs", "py", "c", "h", "cpp", "hpp",
        ] {
            supported_files.insert(
                text_extension.to_string(),
                Box::new(
                    |ui: &mut Ui, text: Option<String>, active_entry: &DirectoryEntry| {
                        if let Some(content) = text {
                            egui::ScrollArea::vertical()
                                .max_height(100.0)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut content.clone())
                                            .code_editor(),
                                    );
                                });
                        }
                    },
                ) as Box<dyn FnMut(&mut Ui, Option<String>, &DirectoryEntry)>,
            );
        }
        supported_files.insert(
            "jpg".to_string(),
            Box::new(|ui: &mut Ui, text: Option<String>, item: &DirectoryEntry| {
                ui.label("Image");
                ui.image(format!("file://{}", item.as_path().display()));
            }) as Box<dyn FnMut(&mut Ui, Option<String>, &DirectoryEntry)>,
        );
        supported_files.insert(
            "png".to_string(),
            Box::new(|ui: &mut Ui, text: Option<String>, item: &DirectoryEntry| {
                ui.label("Image");
                ui.image(format!("file://{}", item.as_path().display()));
            }) as Box<dyn FnMut(&mut Ui, Option<String>, &DirectoryEntry)>,
        );
        Self {
            meta_data: MetaData::default(),
            load_text_content: true,
            load_image_content: true,
            supported_files,
        }
    }

    pub fn add_file_preview(
        mut self,
        extension: &str,
        add_contents: impl FnMut(&mut Ui, Option<String>, &DirectoryEntry) + 'static,
    ) -> Self {
        self.supported_files
            .insert(extension.to_string(), Box::new(add_contents));
        self
    }

    pub fn ui(&mut self, ui: &mut Ui, file_dialog: &mut FileDialog) {
        const SPACING_MULTIPLIER: f32 = 4.0;

        ui.label("Information");
        ui.separator();
        if let Some(item) = file_dialog.active_entry() {
            let path = item.as_path();
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                if let Some(content) = self.supported_files.get_mut(ext) {
                    let text = if self.load_text_content {
                        // only show the first 1000 characters of the file
                        fs::read_to_string(path)
                            .ok()
                            .map(|s| s[0..1000].to_string())
                    } else {
                        None
                    };
                    content(ui, text, item)
                }
            }
            let spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
            ui.separator();

            let width = file_dialog.config_mut().right_panel_width.unwrap_or(100.0) / 2.0;

            if let Some(item) = file_dialog.active_entry() {
                ui.add_space(spacing);
                egui::ScrollArea::vertical()
                    .id_salt("meta_data_scroll")
                    .show(ui, |ui| {
                        egui::Grid::new("meta_data")
                            .num_columns(2)
                            .striped(true)
                            // not sure if 100.0 as a default value is a good idea
                            .min_col_width(width)
                            .max_col_width(width)
                            .show(ui, |ui| {
                                ui.label("Filename: ");
                                ui.label(format!("{}", item.file_name()));
                                ui.end_row();

                                if let Some(size) = item.size() {
                                    ui.label("File Size: ");
                                    if item.is_file() {
                                        ui.label(format!("{}", format_bytes(size)));
                                    } else {
                                        ui.label("NAN");
                                    }
                                    ui.end_row();
                                }
                                if let Some(date) = item.created() {
                                    ui.label("Created: ");
                                    let created: DateTime<Local> = date.into();
                                    ui.label(format!("{}", created.format("%Y-%m-%d %H:%M:%S")));
                                    ui.end_row();
                                }
                                if let Some(date) = item.last_modified() {
                                    ui.label("Last Modified: ");
                                    let modified: DateTime<Local> = date.into();
                                    ui.label(format!("{}", modified.format("%Y-%m-%d %H:%M:%S")));
                                    ui.end_row();
                                }

                                for (key, value) in item.other_metadata() {
                                    ui.label(key);
                                    ui.label(value);
                                    ui.end_row();
                                }
                            });
                    });
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct MetaData {
    pub file_name: String,
    pub dimensions: Option<(usize, usize)>,
    pub scaled_dimensions: Option<(usize, usize)>,
    pub preview_image_bytes: Option<RgbaImage>,
    pub preview_text: Option<String>,
    pub file_size: Option<String>,
    pub file_type: Option<String>,
    pub file_modified: Option<String>,
    pub file_created: Option<String>,
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
