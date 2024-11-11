use crate::{DialogState, FileDialog};
use chrono::{DateTime, Local};
use egui::ahash::{HashMap, HashMapExt};
use egui::Ui;
use image::{DynamicImage, GenericImageView, RgbaImage};
use std::fs;

pub struct InformationPanel {
    pub meta_data: MetaData,
    pub load_text_content: bool,
    pub load_image_content: bool,
    supported_files: HashMap<String, Box<dyn FnMut(&mut Ui, Option<String>, Option<RgbaImage>)>>,
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
                    |ui: &mut Ui, text: Option<String>, image: Option<RgbaImage>| {
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
                ) as Box<dyn FnMut(&mut Ui, Option<String>, Option<RgbaImage>)>,
            );
        }
        // supported_files.insert(
        //     "png".to_string(),
        //     Box::new(
        //         |ui: &mut Ui, text: Option<String>, image: Option<RgbaImage>| {
        //             ui.label("Image");
        //             if let Some(img) = image {
        //                 dbg!(&img.height());
        //                 dbg!(&img.width());
        //                 let color_image = egui::ColorImage::from_rgba_unmultiplied(
        //                     [img.width() as usize, img.height() as usize],
        //                     img.as_flat_samples().as_slice(),
        //                 );
        // 
        //                 // Load the image as a texture in `egui`
        //                 let texture = ui.ctx().load_texture(
        //                     "loaded_image",
        //                     color_image,
        //                     egui::TextureOptions::default(),
        //                 );
        // 
        //                 ui.vertical_centered(|ui| {
        //                     // Display the image
        //                     ui.image(&texture);
        //                 });
        //             }
        //         },
        //     ) as Box<dyn FnMut(&mut Ui, Option<String>, Option<RgbaImage>)>,
        // );
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
        add_contents: impl FnMut(&mut Ui, Option<String>, Option<RgbaImage>) + 'static,
    ) -> Self {
        self.supported_files
            .insert(extension.to_string(), Box::new(add_contents));
        self
    }

    pub fn ui(&mut self, ui: &mut Ui, file_dialog: &mut FileDialog) {
        ui.label("Information");

        if let Some(item) = &file_dialog.selected_item {
            let path = item.as_path();
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                if let Some(content) = self.supported_files.get_mut(ext) {
                    let text = if self.load_text_content {
                        fs::read_to_string(path).ok()
                    } else {
                        None
                    };
                    let image = if self.load_image_content {
                        image::open(path)
                            .map(|img| {
                                let new_width = 100;
                                let (original_width, original_height) = img.dimensions();
                                let new_height = (original_height as f32 * new_width as f32
                                    / original_width as f32)
                                    as u32;

                                // Resize the image to the new dimensions (100px width)
                                let img = img.resize(
                                    new_width,
                                    new_height,
                                    image::imageops::FilterType::Lanczos3,
                                );
                                img.into_rgba8()
                            })
                            .ok()
                    } else {
                        None
                    };
                    content(ui, text, image)
                }
            }
        }

        // // Spacing multiplier used between sections in the right sidebar
        // const SPACING_MULTIPLIER: f32 = 4.0;
        //
        // egui::containers::ScrollArea::vertical()
        //     .auto_shrink([false, false])
        //     .show(ui, |ui| {
        //         // Spacing for the first section in the right sidebar
        //         let mut spacing = ui.ctx().style().spacing.item_spacing.y * 2.0;
        //
        //         // Update paths pinned to the left sidebar by the user
        //         if file_dialog.config.show_pinned_folders
        //             && file_dialog.ui_update_pinned_paths(ui, spacing)
        //         {
        //             spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
        //         }
        //
        //         ui.add_space(spacing);
        //         ui.label(file_dialog.config.labels.heading_meta.as_str());
        //         ui.add_space(spacing);
        //
        //         if let Some(item) = &file_dialog.selected_item {
        //             let file_name = item.file_name();
        //             if file_dialog.metadata.file_name != file_name {
        //                 // update metadata
        //                 let metadata = fs::metadata(item.as_path()).unwrap();
        //                 // Display creation and last modified dates
        //                 if let Ok(created) = metadata.created() {
        //                     let created: DateTime<Local> = created.into();
        //                     file_dialog.metadata.file_created =
        //                         Some(created.format("%Y-%m-%d %H:%M:%S").to_string());
        //                 }
        //                 if let Ok(modified) = metadata.modified() {
        //                     let modified: DateTime<Local> = modified.into();
        //                     file_dialog.metadata.file_modified =
        //                         Some(modified.format("%Y-%m-%d %H:%M:%S").to_string());
        //                 }
        //                 file_dialog.metadata.file_size = Some(format_bytes(metadata.len()));
        //                 file_dialog.metadata.file_name = file_name.to_string();
        //
        //                 // Determine the file type and display relevant metadata
        //                 if let Some(ext) = item.as_path().extension().and_then(|e| e.to_str()) {
        //                     match ext.to_lowercase().as_str() {
        //                         "png" | "jpg" | "jpeg" | "bmp" | "gif" | "tiff" => {
        //                             file_dialog.metadata.file_type = Some("Image".to_string());
        //
        //                             // For image files, show dimensions and color space
        //                             if let Ok(img) = image::open(item.as_path()) {
        //                                 let (width, height) = img.dimensions();
        //                                 file_dialog.metadata.dimensions =
        //                                     Some((width as usize, height as usize));
        //
        //                                 let new_width = 100;
        //                                 let (original_width, original_height) = img.dimensions();
        //                                 let new_height = (original_height as f32 * new_width as f32
        //                                     / original_width as f32)
        //                                     as u32;
        //
        //                                 // Resize the image to the new dimensions (100px width)
        //                                 let img = img.resize(
        //                                     new_width,
        //                                     new_height,
        //                                     image::imageops::FilterType::Lanczos3,
        //                                 );
        //                                 file_dialog.metadata.scaled_dimensions =
        //                                     Some((img.width() as usize, img.height() as usize));
        //
        //                                 file_dialog.metadata.preview_image_bytes =
        //                                     Some(img.into_rgba8());
        //                                 file_dialog.metadata.preview_text = None;
        //                             }
        //                         }
        //                         "txt" | "json" | "md" | "toml" | "csv" | "rtf" | "xml" | "rs"
        //                         | "py" | "c" | "h" | "cpp" | "hpp" => {
        //                             file_dialog.metadata.file_type = Some("Textfile".to_string());
        //
        //                             // For text files, show content
        //                             if let Ok(content) = fs::read_to_string(item.as_path()) {
        //                                 file_dialog.metadata.preview_text = Some(content);
        //                             }
        //                             file_dialog.metadata.preview_image_bytes = None;
        //                             file_dialog.metadata.dimensions = None;
        //                         }
        //                         _ => {
        //                             file_dialog.metadata.file_type = Some("Unknown".to_string());
        //
        //                             file_dialog.metadata.preview_image_bytes = None;
        //                             file_dialog.metadata.dimensions = None;
        //                             file_dialog.metadata.preview_text = None;
        //                         }
        //                     }
        //                 } else {
        //                     file_dialog.metadata.file_type = Some("Unknown".to_string());
        //
        //                     file_dialog.metadata.preview_image_bytes = None;
        //                     file_dialog.metadata.dimensions = None;
        //                     file_dialog.metadata.preview_text = None;
        //                 }
        //             }
        //
        //             file_dialog.metadata.file_name = file_name.to_string();
        //             ui.add_space(spacing);
        //             if let Some(content) = &file_dialog.metadata.preview_text {
        //                 egui::ScrollArea::vertical()
        //                     .max_height(100.0)
        //                     .show(ui, |ui| {
        //                         ui.add(
        //                             egui::TextEdit::multiline(&mut content.clone()).code_editor(),
        //                         );
        //                     });
        //             } else if let Some(img) = &file_dialog.metadata.preview_image_bytes {
        //                 if let Some((width, height)) = &file_dialog.metadata.scaled_dimensions {
        //                     // Convert image into `egui::ColorImage`
        //                     let color_image = egui::ColorImage::from_rgba_unmultiplied(
        //                         [*width, *height],
        //                         img.as_flat_samples().as_slice(),
        //                     );
        //
        //                     // Load the image as a texture in `egui`
        //                     let texture = ui.ctx().load_texture(
        //                         "loaded_image",
        //                         color_image,
        //                         egui::TextureOptions::default(),
        //                     );
        //
        //                     ui.vertical_centered(|ui| {
        //                         // Display the image
        //                         ui.image(&texture);
        //                     });
        //                 }
        //             } else {
        //                 ui.vertical_centered(|ui| {
        //                     ui.label(egui::RichText::from("📁").size(120.0));
        //                 });
        //             }
        //             ui.add_space(spacing);
        //             egui::Grid::new("meta_data")
        //                 .num_columns(2)
        //                 .striped(true)
        //                 .min_col_width(200.0 / 2.0)
        //                 .max_col_width(200.0 / 2.0)
        //                 .show(ui, |ui| {
        //                     ui.label("File name: ");
        //                     ui.label(format!("{}", file_dialog.metadata.file_name));
        //                     ui.end_row();
        //                     ui.label("File type: ");
        //                     ui.label(format!(
        //                         "{}",
        //                         file_dialog
        //                             .metadata
        //                             .file_type
        //                             .clone()
        //                             .unwrap_or("None".to_string())
        //                     ));
        //                     ui.end_row();
        //                     ui.label("File size: ");
        //                     ui.label(format!(
        //                         "{}",
        //                         file_dialog
        //                             .metadata
        //                             .file_size
        //                             .clone()
        //                             .unwrap_or("NAN".to_string())
        //                     ));
        //                     ui.end_row();
        //                     if let Some((width, height)) = file_dialog.metadata.dimensions {
        //                         ui.label("Dimensions: ");
        //
        //                         ui.label(format!("{} x {}", width, height));
        //                         ui.end_row();
        //                         ui.label("Pixel count: ");
        //
        //                         ui.label(format!("{}", format_pixels(width * height)));
        //                         ui.end_row()
        //                     }
        //                 });
        //         }
        //     });
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

pub fn format_bytes(bytes: u64) -> String {
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

pub fn format_pixels(pixels: usize) -> String {
    const K: usize = 1_000;
    const M: usize = K * 1_000;
    const G: usize = M * 1_000;

    if pixels >= K {
        format!("{:.2} MPx", pixels as f64 / M as f64)
    } else {
        format!("{} Px", pixels)
    }
}
