use std::fs;
use std::path::{Path, PathBuf};

use crate::ui;

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
    pub fn new_empty() -> Self {
        Self { directory: None }
    }

    /// Returns the directory that was created.
    /// None is returned if no directory has been created yet.
    pub fn directory(&self) -> Option<PathBuf> {
        self.directory.clone()
    }
}

/// A dialog to create new folder.
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
}

impl CreateDirectoryDialog {
    /// Creates a new dialog with default values
    pub fn new() -> Self {
        Self {
            open: false,
            init: false,
            directory: None,

            input: String::new(),
            error: None,
            scroll_to_error: false,
        }
    }

    /// Resets the dialog and opens it.
    pub fn open(&mut self, directory: PathBuf) {
        self.reset();

        self.open = true;
        self.init = true;
        self.directory = Some(directory);
    }

    /// Closes and resets the dialog.
    pub fn close(&mut self) {
        self.reset();
    }

    /// Main update function of the dialog. Should be called in every frame
    /// in which the dialog is to be displayed.
    pub fn update(&mut self, ui: &mut egui::Ui) -> CreateDirectoryResponse {
        if !self.open {
            return CreateDirectoryResponse::new_empty();
        }

        let mut result = CreateDirectoryResponse::new_empty();

        ui.horizontal(|ui| {
            ui.label("🗀");

            let response = ui.text_edit_singleline(&mut self.input);

            if self.init {
                response.scroll_to_me(Some(egui::Align::Center));
                response.request_focus();

                self.error = self.validate_input();
                self.init = false;
            }

            if response.changed() {
                self.error = self.validate_input();
            }

            if ui::button_enabled_disabled(ui, "✔", self.error.is_none()) {
                result = self.create_directory();
            }

            if ui.button("✖").clicked() {
                self.close();
            }
        });

        if let Some(err) = &self.error {
            ui.add_space(5.0);
            let response = ui.label(err);

            if self.scroll_to_error {
                response.scroll_to_me(Some(egui::Align::Center));
                self.scroll_to_error = false;
            }
        }

        result
    }

    /// Returns if the dialog is currently open
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Creates a new folder in the current directory.
    /// The variable `input` is used as the folder name.
    /// Might change the `error` variable when an error occurred creating the new folder.
    fn create_directory(&mut self) -> CreateDirectoryResponse {
        if let Some(mut dir) = self.directory.clone() {
            dir.push(self.input.as_str());

            match fs::create_dir(&dir) {
                Ok(()) => {
                    self.close();
                    return CreateDirectoryResponse::new(dir.as_path());
                }
                Err(err) => {
                    self.error = self.create_error(format!("Error: {}", err).as_str());
                    return CreateDirectoryResponse::new_empty();
                }
            }
        }

        // This error should not occur because the create_directory function is only
        // called when the dialog is open and the directory is set.
        // If this error occurs, there is most likely a bug in the code.
        self.error = self.create_error("No directory given");

        CreateDirectoryResponse::new_empty()
    }

    /// Validates the folder name input.
    /// Returns None if the name is valid. Otherwise returns the error message.
    fn validate_input(&mut self) -> Option<String> {
        if self.input.is_empty() {
            return self.create_error("Name of the folder can not be empty");
        }

        if let Some(mut x) = self.directory.clone() {
            x.push(self.input.as_str());

            if x.is_dir() {
                return self.create_error("A directory with the name already exists");
            }
            if x.is_file() {
                return self.create_error("A file with the name already exists");
            }
        } else {
            // This error should not occur because the validate_input function is only
            // called when the dialog is open and the directory is set.
            // If this error occurs, there is most likely a bug in the code.
            return self.create_error("No directory given");
        }

        None
    }

    /// Creates the specified error and sets to scroll to the error in the next frame.
    fn create_error(&mut self, error: &str) -> Option<String> {
        self.scroll_to_error = true;
        return Some(error.to_string());
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
