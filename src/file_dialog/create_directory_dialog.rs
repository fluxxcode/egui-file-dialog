use std::path::{PathBuf, Path};
use std::fs;

use crate::ui::ui_button;

pub struct CreateDirectoryResponse {
    directory: Option<PathBuf>
}

impl CreateDirectoryResponse {
    pub fn new(directory: &Path) -> Self {
        Self {
            directory: Some(directory.to_path_buf())
        }
    }

    pub fn new_empty() -> Self {
        Self {
            directory: None
        }
    }

    pub fn directory(&self) -> Option<PathBuf> {
        self.directory.clone()
    }
}

pub struct CreateDirectoryDialog {
    open: bool,
    init: bool,
    directory: Option<PathBuf>,

    input: String,
    error: Option<String>
}

impl CreateDirectoryDialog {
    pub fn new() -> Self {
        Self {
            open: false,
            init: false,
            directory: None,

            input: String::new(),
            error: None
        }
    }

    pub fn open(&mut self, directory: PathBuf) {
        self.reset();

        self.open = true;
        self.init = true;
        self.directory = Some(directory);
    }

    pub fn close(&mut self) {
        self.reset();
    }

    pub fn update(&mut self, ui: &mut egui::Ui) -> CreateDirectoryResponse {

        if !self.open {
            return CreateDirectoryResponse::new_empty();
        }

        let mut result = CreateDirectoryResponse::new_empty();

        ui.horizontal(|ui| {
            ui.label("🗀");

            let response = ui.text_edit_singleline(&mut self.input);

            if self.init {
                response.scroll_to_me(None);
                response.request_focus();

                self.error = self.validate_input();
                self.init = false;
            }

            if response.changed() {
                self.error = self.validate_input();
            }

            if ui_button(ui, "✔", self.error.is_none()) {
                result = self.create_directory();
            }

            if ui.button("✖").clicked() {
                self.close();
            }

            if let Some(err) = &self.error {
                // TODO: Use error icon instead
                ui.label(err);
            }
        });

        result
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    fn create_directory(&mut self) -> CreateDirectoryResponse {
        if let Some(mut dir) = self.directory.clone() {
            dir.push(self.input.as_str());

            match fs::create_dir(&dir) {
                Ok(()) => {
                    self.close();
                    return CreateDirectoryResponse::new(dir.as_path());
                }
                Err(err) => {
                    self.error = Some(format!("Error: {}", err));
                    return CreateDirectoryResponse::new_empty();
                }
            }
        }

        // This error should not occur because the create_directory function is only
        // called when the dialog is open and the directory is set.
        // If this error occurs, there is most likely a bug in the code.
        self.error = Some("No directory given".to_string());

        CreateDirectoryResponse::new_empty()
    }

    fn validate_input(&mut self) -> Option<String> {
        if self.input.is_empty() {
            return Some("Name of the folder can not be empty".to_string());
        }

        if let Some(mut x) = self.directory.clone() {
            x.push(self.input.as_str());

            if x.is_dir() {
                return Some("A directory with the name already exists".to_string())
            }
        }
        else {
            // This error should not occur because the validate_input function is only
            // called when the dialog is open and the directory is set.
            // If this error occurs, there is most likely a bug in the code.
            return Some("No directory given".to_string())
        }

        None
    }

    fn reset(&mut self) {
        self.open = false;
        self.init = false;
        self.directory = None;
        self.input.clear();
    }
}
