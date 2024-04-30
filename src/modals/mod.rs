use std::path::PathBuf;

mod overwrite_file_modal;

pub use overwrite_file_modal::OverwriteFileModal;

pub enum ModalAction {
    SaveFile(PathBuf),
}

pub enum ModalState {
    // If the modal is currently open and still waiting for user input
    Pending,
    // If the modal should be closed in the next frame
    Close(ModalAction),
}

pub trait FileDialogModal {
    fn update(&mut self, ui: &mut egui::Ui) -> ModalState;
}
