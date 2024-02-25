//! # egui-file-dialog
//!
//! An easy-to-use file dialog (a.k.a. file explorer, file picker) for [egui](https://github.com/emilk/egui).
//!
//! The project is currently in a very early version. Some planned features are still missing and some improvements still need to be made.
//!
//! **Currently only tested on Linux and Windows!**
//!
//! Read more about the project: <https://github.com/fluxxcode/egui-file-dialog>
//!
//! ### Features
//! - Select a file or a directory
//! - Save a file (Prompt user for a destination path)
//! - Create a new folder
//! - Navigation buttons to open the parent or previous directories
//! - Search for items in a directory
//! - Shortcut for user directories (Home, Documents, ...) and system disks
//! - Resizable window
//!
//! ### A simple example
//!
//! The following example shows of how you can use the file dialog to let the user select a file. \
//! See the full example at: <https://github.com/fluxxcode/egui-file-dialog/tree/master/examples/select_file>
//!
//! ```
//! use egui_file_dialog::FileDialog;
//!
//! struct MyApp {
//!     file_dialog: FileDialog,
//! }
//!
//! impl MyApp {
//!     pub fn new() -> Self {
//!         Self {
//!             // Create a new FileDialog instance
//!             file_dialog: FileDialog::new(),
//!         }
//!     }
//! }
//!
//! impl MyApp {
//!     fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
//!         if ui.button("Select file").clicked() {
//!             // Open the file dialog to select a file
//!             self.file_dialog.select_file();
//!         }
//!
//!         // Update the dialog and check if the user selected a file
//!         if let Some(path) = self.file_dialog.update(ctx).selected() {
//!             println!("Selected file: {:?}", path);
//!         }
//!     }
//! }
//! ```
//!
//! ### Multilingual support
//! For desktop applications it is often necessary to offer different languages.
//! While the dialog currently only offers English labels by default, the labels are
//! fully customizable. This makes it possible to adapt the labels to different languages.
//! 
//! The following example shows how the labels can be changed to display the file dialog in
//! English or German. Checkout `examples/multilingual` for the full example.
//! 
//! ```
//! use egui_file_dialog::{FileDialog, FileDialogLabels};
//! 
//! enum Language {
//!     English,
//!     German,
//! }
//! 
//! fn get_german_labels() -> FileDialogLabels {
//!     FileDialogLabels {
//!         title_select_directory: "📁 Ordner Öffnen".to_string(),
//!         title_select_file: "📂 Datei Öffnen".to_string(),
//!         title_save_file: "📥 Datei Speichern".to_string(),
//! 
//!         // ... See examples/multilingual for the other labels
//!
//!         ..Default::default()
//!     }
//! }
//! 
//! /// Updates the labels of the file dialog.
//! /// Should be called every time the user selects a different language.
//! fn update_labels(language: &Language, file_dialog: &mut FileDialog) {
//!     *file_dialog.labels_mut() = match language {
//!         // English labels are used by default
//!         Language::English => FileDialogLabels::default(),
//!         // Use custom labels for German
//!         Language::German => get_german_labels(),
//!     };
//! }
//! ```

#![warn(missing_docs)] // Let's keep the public API well documented!

mod config;
mod create_directory_dialog;
mod data;
mod file_dialog;

pub use config::{FileDialogConfig, FileDialogLabels};
pub use file_dialog::{DialogMode, DialogState, FileDialog};
