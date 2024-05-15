use std::path::PathBuf;

use super::{FileDialogModal, ModalAction, ModalState};
use crate::config::FileDialogConfig;

/// The modal that is used to ask the user if the selected path should be
/// overwritten.
pub struct OverwriteFileModal {
    /// The path selected for overwriting.
    path: PathBuf,
}

impl OverwriteFileModal {
    /// Creates a new modal object.
    ///
    /// # Arguments
    ///
    /// * `path` - The path selected for overwriting.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl FileDialogModal for OverwriteFileModal {
    fn update(&mut self, config: &FileDialogConfig, ui: &mut egui::Ui) -> ModalState {
        let mut return_val = ModalState::Pending;

        const SECTION_SPACING: f32 = 15.0;
        const BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(90.0, 20.0);

        ui.vertical_centered(|ui| {
            let warn_icon = egui::RichText::new(&config.warn_icon)
                .color(ui.visuals().warn_fg_color)
                .heading();

            ui.add_space(SECTION_SPACING);

            ui.label(warn_icon);

            ui.add_space(SECTION_SPACING);

            // Used to wrap the path on a single line.
            let mut job = egui::text::LayoutJob::single_section(
                format!("'{}'", self.path.to_str().unwrap_or_default()),
                egui::TextFormat::default(),
            );

            job.wrap = egui::text::TextWrapping {
                max_rows: 1,
                ..Default::default()
            };

            ui.label(job);
            ui.label(&config.labels.overwrite_file_modal_text);

            ui.add_space(SECTION_SPACING);

            ui.horizontal(|ui| {
                let required_width = BUTTON_SIZE.x * 2.0 + ui.style().spacing.item_spacing.x;
                let padding = (ui.available_width() - required_width) / 2.0;

                ui.add_space(padding);

                if ui
                    .add_sized(BUTTON_SIZE, egui::Button::new(&config.labels.cancel))
                    .clicked()
                {
                    return_val = ModalState::Close(ModalAction::None);
                }

                ui.add_space(ui.style().spacing.item_spacing.x);

                if ui
                    .add_sized(BUTTON_SIZE, egui::Button::new(&config.labels.overwrite))
                    .clicked()
                {
                    return_val = ModalState::Close(ModalAction::SaveFile(self.path.to_path_buf()));
                }
            });
        });

        return_val
    }
}
