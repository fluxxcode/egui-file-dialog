#![cfg(feature = "information_view")]

use crate::utils::format_bytes;
use crate::{DirectoryEntry, FileDialog, FileSystem, NativeFileSystem};
use chrono::{DateTime, Local};
use egui::ahash::{HashMap, HashMapExt};
use egui::{Direction, Layout, Ui, Vec2};
use indexmap::{IndexMap, IndexSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

type SupportedPreviewFilesMap = HashMap<String, Box<dyn FnMut(&mut Ui, &InfoPanelEntry)>>;
type SupportedPreviewImagesMap =
    HashMap<String, Box<dyn FnMut(&mut Ui, &InfoPanelEntry, &mut IndexSet<String>)>>;
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
#[derive(Debug)]
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
    /// Path of the current item that is selected
    loaded_file_name: PathBuf,
    /// Map that contains the handler for specific file types (by file extension)
    supported_preview_files: SupportedPreviewFilesMap,
    /// Map that contains the handler for image types (by file extension)
    supported_preview_images: SupportedPreviewImagesMap,
    /// Map that contains the additional metadata loader for specific file types (by file extension)
    additional_meta_files: SupportedAdditionalMetaFilesMap,
    /// Other metadata (loaded by the loader in `additional_meta_files`)
    other_meta_data: IndexMap<String, String>,
    /// Stores the images already loaded by the egui loaders.
    stored_images: IndexSet<String>,

    file_system: Arc<dyn FileSystem + Send + Sync>,
}

impl Default for InformationPanel {
    /// Creates a new `InformationPanel` instance with default configurations.
    /// Pre-configures support for several text-based and image file extensions.
    ///
    /// # Returns
    /// A new instance of `InformationPanel`.
    fn default() -> Self {
        let mut supported_files = HashMap::new();
        let mut supported_images = HashMap::new();
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
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut content).code_editor());
                            });
                    }
                }) as Box<dyn FnMut(&mut Ui, &InfoPanelEntry)>,
            );
        }

        // Add preview support for JPEG and PNG image files
        supported_images.insert(
            "jpg".to_string(),
            Box::new(
                |ui: &mut Ui, item: &InfoPanelEntry, stored_images: &mut IndexSet<String>| {
                    Self::show_image_preview(ui, item, stored_images);
                },
            ) as Box<dyn FnMut(&mut Ui, &InfoPanelEntry, &mut IndexSet<String>)>,
        );
        supported_images.insert(
            "jpeg".to_string(),
            Box::new(
                |ui: &mut Ui, item: &InfoPanelEntry, stored_images: &mut IndexSet<String>| {
                    Self::show_image_preview(ui, item, stored_images);
                },
            ) as Box<dyn FnMut(&mut Ui, &InfoPanelEntry, &mut IndexSet<String>)>,
        );
        supported_images.insert(
            "png".to_string(),
            Box::new(
                |ui: &mut Ui, item: &InfoPanelEntry, stored_images: &mut IndexSet<String>| {
                    Self::show_image_preview(ui, item, stored_images);
                },
            ) as Box<dyn FnMut(&mut Ui, &InfoPanelEntry, &mut IndexSet<String>)>,
        );

        Self {
            panel_entry: None,
            load_text_content: true,
            text_content_max_chars: 1000,
            loaded_file_name: PathBuf::new(),
            supported_preview_files: supported_files,
            supported_preview_images: supported_images,
            additional_meta_files,
            other_meta_data: IndexMap::default(),
            stored_images: IndexSet::default(),
            file_system: Arc::new(NativeFileSystem),
        }
    }
}

impl InformationPanel {
    fn show_image_preview(
        ui: &mut Ui,
        item: &InfoPanelEntry,
        stored_images: &mut IndexSet<String>,
    ) {
        stored_images.insert(format!("{}", item.directory_entry.as_path().display()));
        let image = egui::Image::new(format!(
            "file://{}",
            item.directory_entry.as_path().display()
        ));
        ui.add(image);
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

    fn load_content(&self, path: &Path) -> Option<String> {
        if self.load_text_content {
            self.file_system
                .load_text_file_preview(path, self.text_content_max_chars)
                .ok()
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

        if let Some(item) = file_dialog.selected_entry() {
            // load file content and additional metadata if it's a new file
            self.load_meta_data(item);

            // show preview of selected item
            self.display_preview(ui, item);

            let spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
            ui.separator();

            ui.add_space(spacing);

            // show all metadata
            self.display_meta_data(ui, file_dialog.get_window_id(), width, item);
        }
    }

    fn display_preview(&mut self, ui: &mut Ui, item: &DirectoryEntry) {
        let size = Vec2 {
            x: ui.available_width(),
            y: ui.available_width() / 4.0 * 3.0,
        };
        ui.allocate_ui_with_layout(
            size,
            Layout::centered_and_justified(Direction::TopDown),
            |ui| {
                if item.is_dir() {
                    // show folder icon
                    ui.label(egui::RichText::from(item.icon()).size(ui.available_width() / 3.0));
                } else {
                    // Display file content preview based on its extension
                    if let Some(ext) = item.as_path().extension().and_then(|ext| ext.to_str()) {
                        if let Some(panel_entry) = &self.panel_entry {
                            if let Some(preview_handler) =
                                self.supported_preview_files.get_mut(&ext.to_lowercase())
                            {
                                preview_handler(ui, panel_entry);
                            } else if let Some(preview_handler) =
                                self.supported_preview_images.get_mut(&ext.to_lowercase())
                            {
                                preview_handler(ui, panel_entry, &mut self.stored_images);
                                let number_of_stored_images = self.stored_images.len();
                                if number_of_stored_images > 10 {
                                    self.forget_last_stored_image(ui);
                                }
                            } else if let Some(mut content) = panel_entry.content() {
                                egui::ScrollArea::vertical()
                                    .max_height(ui.available_height())
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(&mut content).code_editor(),
                                        );
                                    });
                            } else {
                                // if no preview is available, show icon
                                ui.label(
                                    egui::RichText::from(item.icon())
                                        .size(ui.available_width() / 3.0),
                                );
                            }
                        }
                    } else {
                        // if no ext is available, show icon anyway
                        ui.label(
                            egui::RichText::from(item.icon()).size(ui.available_width() / 3.0),
                        );
                    }
                }
            },
        );
    }

    fn forget_last_stored_image(&mut self, ui: &Ui) {
        if let Some(last_image) = self.stored_images.first() {
            ui.ctx()
                .forget_image(format!("file://{last_image}").as_str());
        }
        self.stored_images.shift_remove_index(0);
    }

    /// removes all loaded preview images from the egui-loaders to reduce memory usage.
    pub fn forget_all_stored_images(&mut self, ui: &Ui) {
        for image in &self.stored_images {
            ui.ctx().forget_image(format!("file://{image}").as_str());
        }
        self.stored_images.clear();
    }

    fn load_meta_data(&mut self, item: &DirectoryEntry) {
        let path_buf = item.to_path_buf();
        if self.loaded_file_name != path_buf {
            self.loaded_file_name.clone_from(&path_buf);
            // clear previous meta data
            self.other_meta_data = IndexMap::default();
            if let Some(ext) = path_buf.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if let Some(load_meta_data) = self.additional_meta_files.get_mut(ext_str) {
                        // load metadata
                        load_meta_data(&mut self.other_meta_data, &path_buf);
                    }
                }
            }
            let content = self.load_content(&path_buf);
            self.panel_entry = Some(InfoPanelEntry::new(item.clone()));
            if let Some(panel_entry) = &mut self.panel_entry {
                // load content
                if panel_entry.content().is_none() {
                    *panel_entry.content_mut() = content;
                }
            }
        }
    }

    fn display_meta_data(&self, ui: &mut Ui, id: egui::Id, width: f32, item: &DirectoryEntry) {
        egui::ScrollArea::vertical()
            .id_salt(id.with("meta_data_scroll"))
            .show(ui, |ui| {
                egui::Grid::new(id.with("meta_data_grid"))
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
