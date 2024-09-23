use std::path::PathBuf;

use crate::FileDialogConfig;

mod overwrite_file_modal;
pub use overwrite_file_modal::OverwriteFileModal;

/// Contains actions that are executed by the file dialog when closing a modal.
#[derive(Clone)]
pub enum ModalAction {
    /// If no action should be executed.
    None,
    /// If the file dialog should save the specified path.
    /// Should only be used if the `FileDialog` is in `FileDialogMode::SaveFile` mode.
    SaveFile(PathBuf),
}

#[derive(Clone)]
pub enum ModalState {
    /// If the modal is currently open and still waiting for user input
    Pending,
    /// If the modal should be closed in the next frame
    Close(ModalAction),
}

pub trait FileDialogModal {
    /// Main update method of the modal.
    /// Should be called every time the modal should be visible.
    fn update(&mut self, config: &FileDialogConfig, ui: &mut egui::Ui) -> ModalState;

    /// Updates the configured keybindings for the modal window.
    fn update_keybindings(&mut self, _config: &FileDialogConfig, _ctx: &egui::Context) {}
}
