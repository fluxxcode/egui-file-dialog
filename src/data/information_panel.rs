#![cfg(feature = "information_view")]

use crate::{DirectoryEntry, FileDialog};
use chrono::{DateTime, Local};
use egui::ahash::{HashMap, HashMapExt};
use egui::Ui;
use indexmap::IndexMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

type SupportedPreviewFilesMap = HashMap<String, Box<dyn FnMut(&mut Ui, &InfoPanelEntry)>>;
type SupportedAdditionalMetaFilesMap =
    HashMap<String, Box<dyn FnMut(&mut IndexMap<String, String>, &PathBuf)>>;

fn format_pixels(pixels: u32) -> String {
    const K: u32 = 1_000;
    const M: u32 = K * 1_000;

    if pixels >= K {
        format!("{:.2} MPx", f64::from(pixels) / f64::from(M))
    } else {
        format!("{pixels} Px")
    }
}

/// Wrapper for the `DirectoryEntry` struct, that also adds the option to store text content
pub struct InfoPanelEntry {
    /// Directory Item containing info like path
    pub directory_entry: DirectoryEntry,
    /// Optional text content of the file
    pub content: Option<String>,
}

impl InfoPanelEntry {
    /// Create a new `InfoPanelEntry` object
    pub const fn new(item: DirectoryEntry) -> Self {
        Self {
            directory_entry: item,
            content: None,
        }
    }
}

impl InfoPanelEntry {
    /// Returns the content of the directory item, if available
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    /// Mutably borrow content
    pub fn content_mut(&mut self) -> &mut Option<String> {
        &mut self.content
    }
}

/// The `InformationPanel` struct provides a panel to display metadata and previews of files.
/// It supports text-based file previews, image previews, and displays file metadata.
///
/// # Fields
/// - `load_text_content`: Flag to control whether text content should be loaded for preview.
/// - `supported_files`: A hashmap mapping file extensions to their respective preview rendering functions.
pub struct InformationPanel {
    panel_entry: Option<InfoPanelEntry>,
    /// Flag to control whether text content should be loaded for preview.
    pub load_text_content: bool,
    /// Max chars that should be loaded for preview of text files.
    pub text_content_max_chars: usize,
    loaded_file_name: PathBuf,
    supported_preview_files: SupportedPreviewFilesMap,
    additional_meta_files: SupportedAdditionalMetaFilesMap,
    other_meta_data: IndexMap<String, String>,
}

impl Default for InformationPanel {
    /// Creates a new `InformationPanel` instance with default configurations.
    /// Pre-configures support for several text-based and image file extensions.
    ///
    /// # Returns
    /// A new instance of `InformationPanel`.
    fn default() -> Self {
        let mut supported_files = HashMap::new();
        let mut additional_meta_files = HashMap::new();

        for ext in ["png", "jpg", "jpeg", "bmp", "gif"] {
            additional_meta_files.insert(
                ext.to_string(),
                Box::new(
                    |other_meta_data: &mut IndexMap<String, String>, path: &PathBuf| {
                        if let Ok(meta) = image_meta::load_from_file(&path) {
                            let (width, height) = (meta.dimensions.width, meta.dimensions.height);
                            // For image files, show dimensions and color space
                            other_meta_data
                                .insert("Dimensions".to_string(), format!("{width} x {height}"));
                            other_meta_data
                                .insert("Pixel Count".to_string(), format_pixels(width * height));
                            other_meta_data
                                .insert("Colorspace".to_string(), format!("{:?}", meta.color));
                            other_meta_data
                                .insert("Format".to_string(), format!("{:?}", meta.format));
                        }
                    },
                ) as Box<dyn FnMut(&mut IndexMap<String, String>, &PathBuf)>,
            );
        }

        // Add preview support for common text file extensions
        for text_extension in [
            "txt", "json", "md", "toml", "rtf", "xml", "rs", "py", "c", "h", "cpp", "hpp",
        ] {
            supported_files.insert(
                text_extension.to_string(),
                Box::new(|ui: &mut Ui, item: &InfoPanelEntry| {
                    if let Some(mut content) = item.content() {
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut content).code_editor());
                            });
                    }
                }) as Box<dyn FnMut(&mut Ui, &InfoPanelEntry)>,
            );
        }

        // Add preview support for JPEG and PNG image files
        supported_files.insert(
            "jpg".to_string(),
            Box::new(|ui: &mut Ui, item: &InfoPanelEntry| {
                ui.label("Image");
                ui.image(format!(
                    "file://{}",
                    item.directory_entry.as_path().display()
                ));
            }) as Box<dyn FnMut(&mut Ui, &InfoPanelEntry)>,
        );
        supported_files.insert(
            "jpeg".to_string(),
            Box::new(|ui: &mut Ui, item: &InfoPanelEntry| {
                ui.label("Image");
                ui.image(format!(
                    "file://{}",
                    item.directory_entry.as_path().display()
                ));
            }) as Box<dyn FnMut(&mut Ui, &InfoPanelEntry)>,
        );
        supported_files.insert(
            "png".to_string(),
            Box::new(|ui: &mut Ui, item: &InfoPanelEntry| {
                ui.label("Image");
                ui.image(format!(
                    "file://{}",
                    item.directory_entry.as_path().display()
                ));
            }) as Box<dyn FnMut(&mut Ui, &InfoPanelEntry)>,
        );

        Self {
            panel_entry: None,
            load_text_content: true,
            text_content_max_chars: 1000,
            loaded_file_name: PathBuf::new(),
            supported_preview_files: supported_files,
            additional_meta_files,
            other_meta_data: IndexMap::default(),
        }
    }
}

impl InformationPanel {
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
        add_contents: impl FnMut(&mut Ui, &InfoPanelEntry) + 'static,
    ) -> Self {
        self.supported_preview_files
            .insert(extension.to_string(), Box::new(add_contents));
        self
    }

    /// Adds support for an additional metadata loader.
    ///
    /// # Arguments
    /// - `extension`: The file extension to support (e.g., "png", "pdf").
    /// - `load_metadata`: A closure defining how the metadata should be loaded when the file is selected.
    ///
    /// # Returns
    /// The modified `InformationPanel` instance.
    pub fn add_metadata_loader(
        mut self,
        extension: &str,
        load_metadata: impl FnMut(&mut IndexMap<String, String>, &PathBuf) + 'static,
    ) -> Self {
        self.additional_meta_files
            .insert(extension.to_string(), Box::new(load_metadata));
        self
    }

    /// Reads a preview of the file if it is detected as a text file.
    fn load_text_file_preview(path: PathBuf, max_chars: usize) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut chunk = [0; 96]; // Temporary buffer
        let mut buffer = String::new();

        // Add the first chunk to the buffer as text
        let mut total_read = 0;

        // Continue reading if needed
        while total_read < max_chars {
            let bytes_read = file.read(&mut chunk)?;
            if bytes_read == 0 {
                break; // End of file
            }
            let chars_read: String = String::from_utf8(chunk[..bytes_read].to_vec())
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
            total_read += chars_read.len();
            buffer.push_str(&chars_read);
        }

        Ok(buffer.to_string())
    }

    fn load_content(&self, path: PathBuf) -> Option<String> {
        if self.load_text_content {
            Self::load_text_file_preview(path, self.text_content_max_chars).ok()
        } else {
            None
        }
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
            // load file content and additional metadata if it's a new file
            let path_option = file_dialog.active_entry();
            if let Some(path) = path_option {
                if self.loaded_file_name != path.to_path_buf() {
                    self.loaded_file_name = path.to_path_buf();
                    // clear previous meta data
                    self.other_meta_data = IndexMap::default();
                    if let Some(ext) = path.to_path_buf().extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if let Some(load_meta_data) =
                                self.additional_meta_files.get_mut(ext_str)
                            {
                                // load metadata
                                load_meta_data(&mut self.other_meta_data, &path.to_path_buf());
                            }
                        }
                    }
                    let content = self.load_content(path.to_path_buf());
                    self.panel_entry = Some(InfoPanelEntry::new(item.clone()));
                    if let Some(panel_entry) = &mut self.panel_entry {
                        // load content
                        if panel_entry.content().is_none() {
                            *panel_entry.content_mut() = content;
                        }
                    }
                }

                if item.is_dir() {
                    // show folder icon
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::from("📁").size(120.0));
                    });
                } else {
                    // Display file content preview based on its extension
                    if let Some(ext) = item.as_path().extension().and_then(|ext| ext.to_str()) {
                        if let Some(panel_entry) = &self.panel_entry {
                            if let Some(show_preview) =
                                self.supported_preview_files.get_mut(&ext.to_lowercase())
                            {
                                show_preview(ui, panel_entry);
                            } else if let Some(mut content) = panel_entry.content() {
                                egui::ScrollArea::vertical()
                                    .max_height(100.0)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(&mut content).code_editor(),
                                        );
                                    });
                            } else {
                                // if now preview is available, show icon
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::from(item.icon()).size(120.0));
                                });
                            }
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
                                ui.label(item.file_name().to_string());
                                ui.end_row();

                                if let Some(size) = item.metadata().size {
                                    ui.label("File Size: ");
                                    if item.is_file() {
                                        ui.label(format_bytes(size));
                                    } else {
                                        ui.label("NAN");
                                    }
                                    ui.end_row();
                                }

                                if let Some(date) = item.metadata().created {
                                    ui.label("Created: ");
                                    let created: DateTime<Local> = date.into();
                                    ui.label(format!("{}", created.format("%d.%m.%Y, %H:%M:%S")));
                                    ui.end_row();
                                }

                                if let Some(date) = item.metadata().last_modified {
                                    ui.label("Last Modified: ");
                                    let modified: DateTime<Local> = date.into();
                                    ui.label(format!("{}", modified.format("%d.%m.%Y, %H:%M:%S")));
                                    ui.end_row();
                                }

                                // show additional metadata, if present
                                for (key, value) in self.other_meta_data.clone() {
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
        format!("{bytes} B")
    }
}
