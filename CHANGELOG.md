# egui-file-dialog changelog

## [Unreleased] - v0.3.1 - Bug fixes
### 🐛 Bug Fixes
- Fixed not being able to select a shortcut directory like Home or Documents [#43](https://github.com/fluxxcode/egui-file-dialog/pull/43)
- Fixed issue where root directories were not displayed correctly [#44](https://github.com/fluxxcode/egui-file-dialog/pull/44)

### 🔧 Changes
- Updated CI to also run on release branches [#46](https://github.com/fluxxcode/egui-file-dialog/pull/46)

### 📚 Documentation
- `FileDialog::update` has been moved up in the documentation [#47](https://github.com/fluxxcode/egui-file-dialog/pull/47)

## 2024-02-18 - v0.3.0 - UI improvements
### 🖥 UI
- Updated bottom panel so that the dialog can also be resized in `DialogMode::SaveFile` or when selecting a file or directory with a long name [#32](https://github.com/fluxxcode/egui-file-dialog/pull/32)
  - The error when saving a file is now displayed as a tooltip when hovering over the grayed out save button \
    ![preview](media/changelog/v0.3.0/error_tooltip.png)
  - Updated file name input to use all available space
  - Added scroll area around the selected item, so that long file names can be displayed without making the dialog larger
- The default minimum window size has been further reduced to `(340.0, 170.0)` [#32](https://github.com/fluxxcode/egui-file-dialog/pull/32)
- Added an error icon to the error message when creating a new folder [#32](https://github.com/fluxxcode/egui-file-dialog/pull/32) \
  ![preview](media/changelog/v0.3.0/error_icon.png)
- Removable devices are now listed in a separate devices section [#34](https://github.com/fluxxcode/egui-file-dialog/pull/34)
- Added mount point to the disk names on Windows [#38](https://github.com/fluxxcode/egui-file-dialog/pull/38)

### 🔧 Changes
- Restructure `file_dialog.rs` [#36](https://github.com/fluxxcode/egui-file-dialog/pull/36)

### 📚 Documentation
- Fix typos in the documentation [#29](https://github.com/fluxxcode/egui-file-dialog/pull/29)
- Fix eframe version in the example in `README.md` [#30](https://github.com/fluxxcode/egui-file-dialog/pull/30)
- Added "Planned features” section to `README.md` and minor improvements [#31](https://github.com/fluxxcode/egui-file-dialog/pull/31) (Renamed with [#35](https://github.com/fluxxcode/egui-file-dialog/pull/35))
- Updated example screenshot in `README.md` to include new "Removable Devices" section [#34](https://github.com/fluxxcode/egui-file-dialog/pull/34)
- Moved media files from `doc/img/` to `media/` [#37](https://github.com/fluxxcode/egui-file-dialog/pull/37)

## 2024-02-07 - v0.2.0 - API improvements
### 🚨 Breaking Changes
- Rename `FileDialog::default_window_size` to `FileDialog::default_size` [#14](https://github.com/fluxxcode/egui-file-dialog/pull/14)
- Added attribute `operation_id` to `FileDialog::open` [#25](https://github.com/fluxxcode/egui-file-dialog/pull/25)

### ✨ Features
- Implemented `operation_id` so the dialog can be used for multiple different actions in a single view [#25](https://github.com/fluxxcode/egui-file-dialog/pull/25)
- Added `FileDialog::anchor` to overwrite the window anchor [#11](https://github.com/fluxxcode/egui-file-dialog/pull/11)
- Added `FileDialog::title` to overwrite the window title [#12](https://github.com/fluxxcode/egui-file-dialog/pull/12)
- Added `FileDialog::resizable` to set if the window is resizable [#15](https://github.com/fluxxcode/egui-file-dialog/pull/15)
- Added `FileDialog::movable` to set if the window is movable [#15](https://github.com/fluxxcode/egui-file-dialog/pull/15)
- Added `FileDialog::id` to set the ID of the window [#16](https://github.com/fluxxcode/egui-file-dialog/pull/16)
- Added `FileDialog::fixed_pos` and `FileDialog::default_pos` to set the position of the window [#17](https://github.com/fluxxcode/egui-file-dialog/pull/17)
- Added `FileDialog::min_size` and `FileDialog::max_size` to set the minimum and maximum size of the window [#21](https://github.com/fluxxcode/egui-file-dialog/pull/21)
- Added `FileDialog::title_bar` to enable or disable the title bar of the window [#23](https://github.com/fluxxcode/egui-file-dialog/pull/23)

### 🐛 Bug Fixes
- Fixed issue where no error message was displayed when creating a folder [#18](https://github.com/fluxxcode/egui-file-dialog/pull/18)
- Fixed an issue where the same disk can be loaded multiple times in a row on Windows [#26](https://github.com/fluxxcode/egui-file-dialog/pull/26)

### 🔧 Changes
- Removed the version of `egui-file-dialog` in the examples [#8](https://github.com/fluxxcode/egui-file-dialog/pull/8)
- Use `ui.add_enabled` instead of custom `ui.rs` module [#22](https://github.com/fluxxcode/egui-file-dialog/pull/22)

#### Dependency updates:
- Updated egui to version `0.26.0` [#24](https://github.com/fluxxcode/egui-file-dialog/pull/24)

### 📚 Documentation
- Fix syntax highlighting on crates.io [#9](https://github.com/fluxxcode/egui-file-dialog/pull/9)
- Added dependency badge to `README.md` [#10](https://github.com/fluxxcode/egui-file-dialog/pull/10)
- Updated docs badge to use shields.io [#19](https://github.com/fluxxcode/egui-file-dialog/pull/19)

## 2024-02-03 - v0.1.0

Initial release of the file dialog.

The following features are included in this release:
- Select a file or a directory
- Save a file (Prompt user for a destination path)
- Create a new folder
- Navigation buttons to open the parent or previous directories
- Search for items in a directory
- Shortcut for user directories (Home, Documents, ...) and system disks
- Resizable window
