use std::path::PathBuf;

use super::{FileDialogModal, ModalAction, ModalState};

/// The modal that is used to ask the user if the selected path should be
/// overwritten.
pub struct OverwriteFileModal {
    /// The path selected for overwriting.
    path: PathBuf,
}

impl OverwriteFileModal {
    /// Creates a new modal objects.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The path selected for overwriting.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
        }
    }
}

impl FileDialogModal for OverwriteFileModal {
    fn update(&mut self, ui: &mut egui::Ui) -> ModalState {
        // The UI is currently only used for testing purposes.
        ui.label("Overwrite file modal");
        ui.label(format!("Path: {:?}", self.path));

        if ui.button("Close").clicked() {
            return ModalState::Close(ModalAction::None);
        }

        if ui.button("Save file").clicked() {
            return ModalState::Close(ModalAction::SaveFile(self.path.clone()));
        }

        ModalState::Pending
    }
}
