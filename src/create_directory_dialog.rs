use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{FileDialogConfig, FileDialogLabels, FileSystem};

pub struct CreateDirectoryResponse {
    /// Contains the path to the directory that was created.
    directory: Option<PathBuf>,
}

impl CreateDirectoryResponse {
    /// Creates a new response object with the given directory.
    pub fn new(directory: &Path) -> Self {
        Self {
            directory: Some(directory.to_path_buf()),
        }
    }

    /// Creates a new response with no directory set.
    pub const fn new_empty() -> Self {
        Self { directory: None }
    }

    /// Returns the directory that was created.
    /// None is returned if no directory has been created yet.
    pub fn directory(&self) -> Option<PathBuf> {
        self.directory.clone()
    }
}

/// A dialog to create new folder.
#[derive(Debug)]
pub struct CreateDirectoryDialog {
    /// If the dialog is currently open
    open: bool,
    /// If the update method is called for the first time.
    /// Used to initialize some stuff and scroll to the dialog.
    init: bool,
    /// The directory that is currently open and where the folder is created.
    directory: Option<PathBuf>,

    /// Buffer to hold the data of the folder name input
    input: String,
    /// This contains the error message if the folder name is invalid
    error: Option<String>,
    /// If we should scroll to the error in the next frame
    scroll_to_error: bool,
    /// If the text input should request focus in the next frame
    request_focus: bool,

    file_system: Arc<dyn FileSystem + Send + Sync>,
}

impl CreateDirectoryDialog {
    /// Creates a new dialog with default values
    pub fn from_filesystem(file_system: Arc<dyn FileSystem + Send + Sync>) -> Self {
        Self {
            open: false,
            init: false,
            directory: None,

            input: String::new(),
            error: None,
            scroll_to_error: false,
            request_focus: true,
            file_system,
        }
    }

    /// Resets the dialog and opens it.
    pub fn open(&mut self, directory: PathBuf) {
        self.reset();

        self.open = true;
        self.init = true;
        self.directory = Some(directory);
    }

    /// Closes and resets the dialog without creating the directory.
    pub fn close(&mut self) {
        self.reset();
    }

    /// Tries to create the given folder.
    pub fn submit(&mut self) -> CreateDirectoryResponse {
        // Only necessary in the event of an error
        self.request_focus = true;

        if self.error.is_none() {
            return self.create_directory();
        }

        CreateDirectoryResponse::new_empty()
    }

    /// Main update function of the dialog. Should be called in every frame
    /// in which the dialog is to be displayed.
    pub fn update(
        &mut self,
        ui: &mut egui::Ui,
        config: &FileDialogConfig,
    ) -> CreateDirectoryResponse {
        if !self.open {
            return CreateDirectoryResponse::new_empty();
        }

        let mut result = CreateDirectoryResponse::new_empty();

        ui.horizontal(|ui| {
            ui.label(&config.default_folder_icon);

            let text_edit_response = ui.text_edit_singleline(&mut self.input);

            if self.init {
                text_edit_response.scroll_to_me(Some(egui::Align::Center));
                text_edit_response.request_focus();

                self.error = self.validate_input(&config.labels);
                self.init = false;
                self.request_focus = false;
            }

            if self.request_focus {
                text_edit_response.request_focus();
                self.request_focus = false;
            }

            if text_edit_response.changed() {
                self.error = self.validate_input(&config.labels);
            }

            let apply_button_response =
                ui.add_enabled(self.error.is_none(), egui::Button::new("✔"));

            if apply_button_response.clicked() {
                result = self.submit();
            }

            if ui.button("✖").clicked()
                || (text_edit_response.lost_focus() && !apply_button_response.contains_pointer())
            {
                self.close();
            }
        });

        if let Some(err) = &self.error {
            ui.add_space(5.0);

            let response = ui
                .horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    ui.colored_label(
                        ui.style().visuals.error_fg_color,
                        format!("{} ", config.err_icon),
                    );

                    ui.label(err);
                })
                .response;

            if self.scroll_to_error {
                response.scroll_to_me(Some(egui::Align::Center));
                self.scroll_to_error = false;
            }
        }

        result
    }

    /// Returns if the dialog is currently open
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// Creates a new folder in the current directory.
    /// The variable `input` is used as the folder name.
    /// Might change the `error` variable when an error occurred creating the new folder.
    fn create_directory(&mut self) -> CreateDirectoryResponse {
        if let Some(mut dir) = self.directory.clone() {
            dir.push(self.input.as_str());

            match self.file_system.create_dir(&dir) {
                Ok(()) => {
                    self.close();
                    return CreateDirectoryResponse::new(dir.as_path());
                }
                Err(err) => {
                    self.error = Some(self.create_error(format!("Error: {err}").as_str()));
                    return CreateDirectoryResponse::new_empty();
                }
            }
        }

        // This error should not occur because the create_directory function is only
        // called when the dialog is open and the directory is set.
        // If this error occurs, there is most likely a bug in the code.
        self.error = Some(self.create_error("No directory given"));

        CreateDirectoryResponse::new_empty()
    }

    /// Validates the folder name input.
    /// Returns None if the name is valid. Otherwise returns the error message.
    fn validate_input(&mut self, labels: &FileDialogLabels) -> Option<String> {
        if self.input.is_empty() {
            return Some(self.create_error(&labels.err_empty_file_name));
        }

        if let Some(mut x) = self.directory.clone() {
            x.push(self.input.as_str());

            if x.is_dir() {
                return Some(self.create_error(&labels.err_directory_exists));
            }
            if x.is_file() {
                return Some(self.create_error(&labels.err_file_exists));
            }
        } else {
            // This error should not occur because the validate_input function is only
            // called when the dialog is open and the directory is set.
            // If this error occurs, there is most likely a bug in the code.
            return Some(self.create_error("No directory given"));
        }

        None
    }

    /// Creates the specified error and sets to scroll to the error in the next frame.
    fn create_error(&mut self, error: &str) -> String {
        self.scroll_to_error = true;
        error.to_string()
    }

    /// Resets the dialog.
    /// Configuration variables are not changed.
    fn reset(&mut self) {
        self.open = false;
        self.init = false;
        self.directory = None;
        self.input.clear();
        self.error = None;
        self.scroll_to_error = false;
    }
}
