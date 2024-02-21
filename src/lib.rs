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

mod file_dialog;
pub use file_dialog::{DialogMode, DialogState, FileDialog, FileDialogConfig};

mod create_directory_dialog;
mod data;
