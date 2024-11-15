#![cfg(feature = "info_panel")]

use crate::{DirectoryEntry, FileDialog};
use chrono::{DateTime, Local};
use egui::ahash::{HashMap, HashMapExt};
use egui::Ui;

/// The `InformationPanel` struct provides a panel to display metadata and previews of files.
/// It supports text-based file previews, image previews, and displays file metadata.
///
/// # Fields
/// - `meta_data`: Stores metadata information about the current file.
/// - `load_text_content`: Flag to control whether text content should be loaded for preview.
/// - `supported_files`: A hashmap mapping file extensions to their respective preview rendering functions.
pub struct InformationPanel {
    /// Flag to control whether text content should be loaded for preview.
    pub load_text_content: bool,
    supported_files: HashMap<String, Box<dyn FnMut(&mut Ui, &DirectoryEntry)>>,
}

impl InformationPanel {
    /// Creates a new `InformationPanel` instance with default configurations.
    /// Pre-configures support for several text-based and image file extensions.
    ///
    /// # Returns
    /// A new instance of `InformationPanel`.
    pub fn new() -> Self {
        let mut supported_files = HashMap::new();

        // Add preview support for common text file extensions
        for text_extension in [
            "txt", "json", "md", "toml", "rtf", "xml", "rs", "py", "c", "h", "cpp", "hpp",
        ] {
            supported_files.insert(
                text_extension.to_string(),
                Box::new(|ui: &mut Ui, item: &DirectoryEntry| {
                    if let Some(content) = item.content() {
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut content.clone()).code_editor(),
                                );
                            });
                    }
                }) as Box<dyn FnMut(&mut Ui, &DirectoryEntry)>,
            );
        }

        // Add preview support for JPEG and PNG image files
        supported_files.insert(
            "jpg".to_string(),
            Box::new(|ui: &mut Ui, item: &DirectoryEntry| {
                ui.label("Image");
                ui.image(format!("file://{}", item.as_path().display()));
            }) as Box<dyn FnMut(&mut Ui, &DirectoryEntry)>,
        );
        supported_files.insert(
            "jpeg".to_string(),
            Box::new(|ui: &mut Ui, item: &DirectoryEntry| {
                ui.label("Image");
                ui.image(format!("file://{}", item.as_path().display()));
            }) as Box<dyn FnMut(&mut Ui, &DirectoryEntry)>,
        );
        supported_files.insert(
            "png".to_string(),
            Box::new(|ui: &mut Ui, item: &DirectoryEntry| {
                ui.label("Image");
                ui.image(format!("file://{}", item.as_path().display()));
            }) as Box<dyn FnMut(&mut Ui, &DirectoryEntry)>,
        );

        Self {
            load_text_content: true,
            supported_files,
        }
    }

    /// Adds support for previewing a custom file type.
    ///
    /// # Arguments
    /// - `extension`: The file extension to support (e.g., "csv", "html").
    /// - `add_contents`: A closure defining how the file should be rendered in the UI.
    ///
    /// # Returns
    /// The modified `InformationPanel` instance.
    pub fn add_file_preview(
        mut self,
        extension: &str,
        add_contents: impl FnMut(&mut Ui, &DirectoryEntry) + 'static,
    ) -> Self {
        self.supported_files
            .insert(extension.to_string(), Box::new(add_contents));
        self
    }

    /// Renders the Information Panel in the provided UI context.
    ///
    /// # Arguments
    /// - `ui`: The UI context where the panel will be rendered.
    /// - `file_dialog`: A reference to the current file dialog, which provides the active file entry.
    pub fn ui(&mut self, ui: &mut Ui, file_dialog: &mut FileDialog) {
        const SPACING_MULTIPLIER: f32 = 4.0;

        ui.label("Information");
        ui.separator();

        // Display metadata in a grid format
        let width = file_dialog.config_mut().right_panel_width.unwrap_or(100.0) / 2.0;

        if let Some(item) = file_dialog.active_entry() {
            if item.is_dir() {
                // show folder icon
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::from("📁").size(120.0));
                });
            } else {
                // Display file content preview based on its extension
                if let Some(ext) = item.as_path().extension().and_then(|ext| ext.to_str()) {
                    if let Some(show_preview) = self.supported_files.get_mut(&ext.to_lowercase()) {
                        show_preview(ui, item);
                    } else {
                        // if now preview is available, show icon
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::from(item.icon()).size(120.0));
                        });
                    }
                }
            }

            let spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
            ui.separator();

            ui.add_space(spacing);

            // show all metadata
            egui::ScrollArea::vertical()
                .id_salt("meta_data_scroll")
                .show(ui, |ui| {
                    egui::Grid::new("meta_data")
                        .num_columns(2)
                        .striped(true)
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

                            // show additional metadata, if present
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

/// Formats a file size (in bytes) into a human-readable string (e.g., KB, MB).
///
/// # Arguments
/// - `bytes`: The file size in bytes.
///
/// # Returns
/// A string representing the file size in an appropriate unit.
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
