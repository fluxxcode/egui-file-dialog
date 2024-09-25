use std::path::PathBuf;

use super::{FileDialogModal, ModalAction, ModalState};
use crate::config::{FileDialogConfig, FileDialogKeyBindings};

/// The modal that is used to ask the user if the selected path should be
/// overwritten.
pub struct OverwriteFileModal {
    /// The current state of the modal.
    state: ModalState,
    /// The path selected for overwriting.
    path: PathBuf,
}

impl OverwriteFileModal {
    /// Creates a new modal object.
    ///
    /// # Arguments
    ///
    /// * `path` - The path selected for overwriting.
    pub const fn new(path: PathBuf) -> Self {
        Self {
            state: ModalState::Pending,
            path,
        }
    }
}

impl OverwriteFileModal {
    /// Submits the modal and triggers the action to save the file.
    fn submit(&mut self) {
        self.state = ModalState::Close(ModalAction::SaveFile(self.path.clone()));
    }

    /// Closes the modal without overwriting the file.
    fn cancel(&mut self) {
        self.state = ModalState::Close(ModalAction::None);
    }
}

impl FileDialogModal for OverwriteFileModal {
    fn update(&mut self, config: &FileDialogConfig, ui: &mut egui::Ui) -> ModalState {
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
                let required_width = BUTTON_SIZE
                    .x
                    .mul_add(2.0, ui.style().spacing.item_spacing.x);
                let padding = (ui.available_width() - required_width) / 2.0;

                ui.add_space(padding);

                if ui
                    .add_sized(BUTTON_SIZE, egui::Button::new(&config.labels.cancel))
                    .clicked()
                {
                    self.cancel();
                }

                ui.add_space(ui.style().spacing.item_spacing.x);

                if ui
                    .add_sized(BUTTON_SIZE, egui::Button::new(&config.labels.overwrite))
                    .clicked()
                {
                    self.submit();
                }
            });
        });

        self.state.clone()
    }

    fn update_keybindings(&mut self, config: &FileDialogConfig, ctx: &egui::Context) {
        if FileDialogKeyBindings::any_pressed(ctx, &config.keybindings.submit, true) {
            self.submit();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &config.keybindings.cancel, true) {
            self.cancel();
        }
    }
}
