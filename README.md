# egui-file-dialog
[![Latest version](https://img.shields.io/crates/v/egui-file-dialog.svg)](https://crates.io/crates/egui-file-dialog)
[![Documentation](https://img.shields.io/docsrs/egui-file-dialog)](https://docs.rs/egui-file-dialog)
[![Dependency status](https://deps.rs/repo/github/fluxxcode/egui-file-dialog/status.svg)](https://deps.rs/repo/github/fluxxcode/egui-file-dialog)
![Crates.io Total Downloads](https://img.shields.io/crates/d/egui-file-dialog)
[![Total lines of code ](https://sloc.xyz/github/fluxxcode/egui-file-dialog/)](https://github.com/fluxxcode/egui-file-dialog/)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/fluxxcode/egui-file-dialog/blob/master/LICENSE)

This repository provides an easy-to-use and customizable file dialog (a.k.a. file explorer, file picker) for [egui](https://github.com/emilk/egui).

The file dialog is intended for use by desktop applications, thus allowing the use of a file dialog directly within the egui application without relying on the operating system's file explorer. This also ensures that the file dialog looks the same and has the same functionality on all platforms.

<img src="media/demo.png">

The project is currently in a very early version. Some planned features are still missing and some improvements still need to be made. See the [Planned features](#Planned-features) section for some of the features to be implemented in the future.

The latest changes included in the next release can be found in the [CHANGELOG.md](https://github.com/fluxxcode/egui-file-dialog/blob/develop/CHANGELOG.md) file on the develop branch.

**Currently only tested on Linux and Windows!**

## Features
- Select a file or a directory
- Save a file (Prompt user for a destination path)
- Create a new folder
- Navigation buttons to open the parent or previous directories
- Search for items in a directory
- Shortcut for user directories (Home, Documents, ...) and system disks
- Customization highlights:
  - Customize which areas and functions of the dialog are visible
  - Multilingual support: Customize the text labels that the dialog uses
  - Customize file and folder icons
  - _More options can be found in the documentation on [docs.rs](https://docs.rs/egui-file-dialog/latest/egui_file_dialog/index.html)_

## Planned features
The following lists some of the features that are currently missing but are planned for the future!
- Selection of multiple directory items at once
- Pinnable folders for quick access [#42](https://github.com/fluxxcode/egui-file-dialog/issues/42)
- Only show files with a specific file extension (The user can already filter files by file extension using the search, but there is currently no backend method for this or a dropdown to be able to select from predefined file extensions.)
- Keyboard input
- Context menus, for example for renaming, deleting or copying files or directories.
- Option to show or hide hidden files and folders

## Example
Detailed examples that can be run can be found in the [examples](https://github.com/fluxxcode/egui-file-dialog/tree/master/examples) folder.

The following example shows the basic use of the file dialog with [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) to select a file.

Cargo.toml:
```toml
[dependencies]
eframe = "0.26.0"
egui-file-dialog = "0.3.1"
```

main.rs:
```rust
use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::FileDialog;

struct MyApp {
    file_dialog: FileDialog,
    selected_file: Option<PathBuf>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            // Create a new file dialog object
            file_dialog: FileDialog::new(),
            selected_file: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Select file").clicked() {
                // Open the file dialog to select a file.
                self.file_dialog.select_file();
            }

            ui.label(format!("Selected file: {:?}", self.selected_file));

            // Update the dialog and check if the user selected a file
            if let Some(path) = self.file_dialog.update(ctx).selected() {
                self.selected_file = Some(path.to_path_buf());
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "File dialog demo",
        eframe::NativeOptions::default(),
        Box::new(|ctx| Box::new(MyApp::new(ctx))),
    )
}
```

## Customization
Many things can be customized so that the dialog can be used in different situations. \
A few highlights of the customization are listed below. For all possible customization options, see the documentation on [docs.rs](https://docs.rs/egui-file-dialog/latest/egui_file_dialog/struct.FileDialog.html). (More customization will be implemented in the future!)

- Set which areas and functions of the dialog are visible using `FileDialog::show_*` methods
- Update the text labels that the dialog uses. See [Multilingual support](#multilingual-support)
- Customize file and folder icons using `FileDialog::set_file_icon` (Currently only unicode is supported)

Since the dialog uses the egui style to look like the rest of the application, the appearance can be customized with `egui::Style`.

The following example shows how a file dialog can be customized. If you need to configure multiple file dialog objects with the same or almost the same options, it is a good idea to use `FileDialogConfig` and `FileDialog::with_config` (See `FileDialogConfig` on [docs.rs](https://docs.rs/egui-file-dialog/latest/egui_file_dialog/struct.FileDialogConfig.html)).
```rust
use std::path::PathBuf;
use std::sync::Arc;

use egui_file_dialog::FileDialog;

FileDialog::new()
    .initial_directory(PathBuf::from("/path/to/app"))
    .default_file_name("app.cfg")
    .default_size([600.0, 400.0])
    .resizable(false)
    .show_new_folder_button(false)
    .show_search(false)
    // Markdown and text files should use the "document with text (U+1F5B9)" icon
    .set_file_icon(
        "🖹",
        Arc::new(|path| {
            match path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
            {
                "md" => true,
                "txt" => true,
                _ => false,
            }
        }),
    )
    // .gitignore files should use the "web-github (U+E624)" icon
    .set_file_icon(
        "",
        Arc::new(|path| path.file_name().unwrap_or_default() == ".gitignore"),
    );
```
With the options the dialog then looks like this:
<img src="media/customization_demo.png">

The smallest possible dialog can be generated with the following configuration:

```rust
FileDialog::new()
    .title_bar(false)
    .show_top_panel(false)
    .show_left_panel(false)
```
<img src="media/customization_demo_2.png">

## Multilingual support
For desktop applications it is often necessary to offer different languages. While the dialog currently only offers English labels by default, the labels are fully customizable. This makes it possible to adapt the labels to different languages.

The following example shows how the labels can be changed to display the file dialog in English or German. \
Checkout `examples/multilingual` for the full example.

```rust
use egui_file_dialog::{FileDialog, FileDialogLabels};

enum Language {
    English,
    German,
}

fn get_labels_german() -> FileDialogLabels {
    FileDialogLabels {
        title_select_directory: "📁 Ordner Öffnen".to_string(),
        title_select_file: "📂 Datei Öffnen".to_string(),
        title_save_file: "📥 Datei Speichern".to_string(),

        // ... See examples/multilingual for the other labels

        ..Default::default()
    }
}

/// Updates the labels of the file dialog.
/// Should be called every time the user selects a different language.
fn update_labels(language: &Language, file_dialog: &mut FileDialog) {
    *file_dialog.labels_mut() = match language {
        // English labels are used by default
        Language::English => FileDialogLabels::default(),
        // Use custom labels for German
        Language::German => get_labels_german(),
    };
}
```
