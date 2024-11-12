//! # egui-file-dialog
//!
//! An easy-to-use and customizable file dialog (a.k.a. file explorer, file picker) for
//! [egui](https://github.com/emilk/egui).
//!
//! **Currently only tested on Linux and Windows!**
//!
//! Read more about the project: <https://github.com/fluxxcode/egui-file-dialog>
//!
//! ### Features
//! - Select a file or a directory
//! - Save a file (Prompt user for a destination path)
//!   - Dialog to ask the user if the existing file should be overwritten
//! - Select multiple files and folders at once (ctrl/shift + click)
//! - Open the dialog in a normal or modal window
//! - Create a new folder
//! - Keyboard navigation
//! - Option to show or hide hidden files and folders
//! - Navigation buttons to open the parent or previous directories
//! - Search for items in a directory
//! - Add file filters the user can select from a dropdown
//! - Shortcut for user directories (Home, Documents, ...) and system disks
//! - Pin folders to the left sidebar
//! - Manually edit the path via text
//! - Customization highlights:
//!   - Customize which areas and functions of the dialog are visible
//!   - Customize the text labels used by the dialog to enable multilingual support
//!   - Customize file and folder icons
//!   - Add custom quick access sections to the left sidebar
//!   - Customize keybindings used by the file dialog
//!
//! ### A simple example
//!
//! The following example shows of how you can use the file dialog to let the user select a file. \
//! See the full example at
//! <https://github.com/fluxxcode/egui-file-dialog/tree/master/examples/select_file>
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
//! ### Keybindings
//! Keybindings can be used in the file dialog for easier navigation. All keybindings can be configured from the backend with `FileDialogKeyBindings` and `FileDialog::keybindings`. \
//! The following table lists all available keybindings and their default values.
//!
//! | Name | Description | Default |
//! | --- | --- | --- |
//! | `submit` | Submit the current action or open the currently selected folder | `Enter` |
//! | `cancel` | Cancel the current action | `Escape` |
//! | `parent` | Open the parent directory | `ALT` + `↑` |
//! | `back` | Go back | `Mouse button 1` <br/> `ALT` + `←` <br/> `Backspace` |
//! | `forward` | Go forward | `Mouse button 2` <br/> `ALT` + `→` |
//! | `reload` | Reload the file dialog data and the currently open directory | `F5` |
//! | `new_folder` | Open the dialog to create a new folder | `CTRL` + `N` |
//! | `edit_path` | Text edit the current path | `/` |
//! | `home_edit_path` | Open the home directory and start text editing the path | `~` |
//! | `selection_up` | Move the selection one item up | `↑` |
//! | `selection_down` | Move the selection one item down | `↓` |
//! | `select_all` | Select every item in the directory when using the file dialog to select multiple files and folders | `CTRL` + `A` |
//!
//! ### Customization
//! Many things can be customized so that the dialog can be used in different situations. \
//! A few highlights of the customization are listed below. For all possible customization options, see the documentation on [docs.rs](https://docs.rs/egui-file-dialog/latest/egui_file_dialog/struct.FileDialog.html).
//!
//! - Set which areas and functions of the dialog are visible using `FileDialog::show_*` methods
//! - Update the text labels that the dialog uses. See [Multilingual support](#multilingual-support)
//! - Customize file and folder icons using `FileDialog::set_file_icon` (Currently only unicode is supported)
//! - Customize keybindings used by the file dialog using `FileDialog::keybindings`. See [Keybindings](#keybindings)
//!
//! Since the dialog uses the egui style to look like the rest of the application, the appearance can be customized with `egui::Style` and `egui::Context::set_style`.
//!
//! The following example shows how a single file dialog can be customized. \
//! If you need to configure multiple file dialog objects with the same or almost the same options, it is a good idea to use `FileDialogConfig` and `FileDialog::with_config` (See `FileDialogConfig` on [docs.rs](https://docs.rs/egui-file-dialog/latest/egui_file_dialog/struct.FileDialogConfig.html)).
//! ```rust
//! use std::path::PathBuf;
//! use std::sync::Arc;
//!
//! use egui_file_dialog::FileDialog;
//!
//! FileDialog::new()
//!     .initial_directory(PathBuf::from("/path/to/app"))
//!     .default_file_name("app.cfg")
//!     .default_size([600.0, 400.0])
//!     .resizable(false)
//!     .show_new_folder_button(false)
//!     .show_search(false)
//!     .show_path_edit_button(false)
//!     // Add a new quick access section to the left sidebar
//!     .add_quick_access("Project", |s| {
//!         s.add_path("☆  Examples", "examples");
//!         s.add_path("📷  Media", "media");
//!         s.add_path("📂  Source", "src");
//!     })
//!     // Markdown files should use the "document with text (U+1F5B9)" icon
//!     .set_file_icon(
//!         "🖹",
//!         Arc::new(|path| path.extension().unwrap_or_default() == "md"),
//!     )
//!     // .gitignore files should use the "web-github (U+E624)" icon
//!     .set_file_icon(
//!         "",
//!         Arc::new(|path| path.file_name().unwrap_or_default() == ".gitignore"),
//!     )
//!     // Add file filters the user can select in the bottom right
//!     .add_file_filter(
//!         "PNG files",
//!         Arc::new(|p| p.extension().unwrap_or_default() == "png"),
//!     )
//!     .add_file_filter(
//!         "Rust source files",
//!         Arc::new(|p| p.extension().unwrap_or_default() == "rs"),
//!     );
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
//! fn get_labels_german() -> FileDialogLabels {
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
//!         Language::German => get_labels_german(),
//!     };
//! }
//! ```
//!
//! ### Persistent data
//! The file dialog currently requires the following persistent data to be stored across multiple file dialog objects:
//!
//! - Folders the user pinned to the left sidebar (`FileDialog::show_pinned_folders`)
//! - If hidden files and folders should be visible (`FileDialog::show_hidden_option`)
//!
//! If one of the above feature is activated, the data should be saved by the application. Otherwise, frustrating situations could arise for the user and the features would not offer much added value.
//!
//! All data that needs to be stored permanently is contained in the `FileDialogStorage` struct. This struct can be accessed using `FileDialog::storage` or `FileDialog::storage_mut` to save or load the persistent data. \
//! By default the feature `serde` is enabled, which implements `serde::Serialize` and `serde::Deserialize` for the objects to be saved. However, the objects can also be accessed without the feature enabled.
//!
//! Checkout `examples/persistence` for an example.

#![warn(missing_docs)] // Let's keep the public API well documented!

mod config;
mod create_directory_dialog;
mod data;
mod file_dialog;
mod modals;

pub use config::{
    FileDialogConfig, FileDialogKeyBindings, FileDialogLabels, FileDialogStorage, IconFilter,
    KeyBinding, QuickAccess, QuickAccessPath,
};
#[cfg(feature = "info_panel")]
pub use data::information_panel;
pub use data::DirectoryEntry;
pub use file_dialog::{DialogMode, DialogState, FileDialog};
