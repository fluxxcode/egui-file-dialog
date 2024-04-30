use super::{FileDialogModal, ModalState};

pub struct OverwriteFileModal;

impl OverwriteFileModal {
    pub fn new() -> Self {
        Self { }
    }
}

impl FileDialogModal for OverwriteFileModal {
    fn update(&mut self, ui: &mut egui::Ui) -> ModalState {
        ui.label("Overwrite file modal");

        ModalState::Pending
    }
}
