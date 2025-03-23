mod labels;
pub use labels::FileDialogLabels;

mod keybindings;
pub use keybindings::{FileDialogKeyBindings, KeyBinding};

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{FileSystem, NativeFileSystem};

/// Contains data of the `FileDialog` that should be stored persistently.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FileDialogStorage {
    /// The folders the user pinned to the left sidebar.
    pub pinned_folders: Vec<PathBuf>,
    /// If hidden files and folders should be listed inside the directory view.
    pub show_hidden: bool,
    /// If system files should be listed inside the directory view.
    pub show_system_files: bool,
    /// The last directory the user visited.
    pub last_visited_dir: Option<PathBuf>,
    /// The last directory from which the user picked an item.
    pub last_picked_dir: Option<PathBuf>,
}

impl Default for FileDialogStorage {
    /// Creates a new object with default values
    fn default() -> Self {
        Self {
            pinned_folders: Vec::new(),
            show_hidden: false,
            show_system_files: false,
            last_visited_dir: None,
            last_picked_dir: None,
        }
    }
}

/// Sets which directory is loaded when opening the file dialog.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OpeningMode {
    /// The configured initial directory (`FileDialog::initial_directory`) should always be opened.
    AlwaysInitialDir,
    /// The directory most recently visited by the user should be opened regardless of
    /// whether anything was picked.
    LastVisitedDir,
    /// The last directory from which the user picked an item should be opened.
    LastPickedDir,
}

/// Contains configuration values of a file dialog.
///
/// The configuration of a file dialog can be set using `FileDialog::with_config`.
///
/// If you only need to configure a single file dialog, you don't need to
/// manually use a `FileDialogConfig` object. `FileDialog` provides setter methods for
/// each of these configuration options, for example: `FileDialog::initial_directory`
/// or `FileDialog::default_size`.
///
/// `FileDialogConfig` is useful when you need to configure multiple `FileDialog` objects with the
/// same or almost the same options.
///
/// # Example
///
/// ```
/// use egui_file_dialog::{FileDialog, FileDialogConfig};
///
/// let config = FileDialogConfig {
///     initial_directory: std::path::PathBuf::from("/app/config"),
///     fixed_pos: Some(egui::Pos2::new(40.0, 40.0)),
///     show_left_panel: false,
///     ..Default::default()
/// };
///
/// let file_dialog_a = FileDialog::with_config(config.clone())
///     .id("file-dialog-a");
///
/// let file_dialog_b = FileDialog::with_config(config.clone());
/// ```
#[derive(Debug, Clone)]
pub struct FileDialogConfig {
    // ------------------------------------------------------------------------
    // Core:
    /// File system browsed by the file dialog; may be native or virtual.
    pub file_system: Arc<dyn FileSystem + Send + Sync>,
    /// Persistent data of the file dialog.
    pub storage: FileDialogStorage,
    /// The labels that the dialog uses.
    pub labels: FileDialogLabels,
    /// Keybindings used by the file dialog.
    pub keybindings: FileDialogKeyBindings,

    // ------------------------------------------------------------------------
    // General options:
    /// Sets which directory is loaded when opening the file dialog.
    pub opening_mode: OpeningMode,
    /// If the file dialog should be visible as a modal window.
    /// This means that the input outside the window is not registered.
    pub as_modal: bool,
    /// Color of the overlay that is displayed under the modal to prevent user interaction.
    pub modal_overlay_color: egui::Color32,
    /// The first directory that will be opened when the dialog opens.
    pub initial_directory: PathBuf,
    /// The default filename when opening the dialog in `DialogMode::SaveFile` mode.
    pub default_file_name: String,
    /// If the user is allowed to select an already existing file when the dialog is
    /// in `DialogMode::SaveFile` mode.
    pub allow_file_overwrite: bool,
    /// If the path edit is allowed to select the path as the file to save
    /// if it does not have an extension.
    ///
    /// This can lead to confusion if the user wants to open a directory with the path edit,
    /// types it incorrectly and the dialog tries to select the incorrectly typed folder as
    /// the file to be saved.
    ///
    /// This only affects the `DialogMode::SaveFile` mode.
    pub allow_path_edit_to_save_file_without_extension: bool,
    /// Sets the separator of the directories when displaying a path.
    /// Currently only used when the current path is displayed in the top panel.
    pub directory_separator: String,
    /// If the paths in the file dialog should be canonicalized before use.
    pub canonicalize_paths: bool,
    /// If the directory content should be loaded via a separate thread.
    /// This prevents the application from blocking when loading large directories
    /// or from slow hard drives.
    pub load_via_thread: bool,
    /// If we should truncate the filenames in the middle
    pub truncate_filenames: bool,

    /// The icon that is used to display error messages.
    pub err_icon: String,
    /// The icon that is used to display warning messages.
    pub warn_icon: String,
    /// The default icon used to display files.
    pub default_file_icon: String,
    /// The default icon used to display folders.
    pub default_folder_icon: String,
    /// The icon used to display pinned paths in the left panel.
    pub pinned_icon: String,
    /// The icon used to display devices in the left panel.
    pub device_icon: String,
    /// The icon used to display removable devices in the left panel.
    pub removable_device_icon: String,

    /// File filters presented to the user in a dropdown.
    pub file_filters: Vec<FileFilter>,
    /// Name of the file filter to be selected by default.
    pub default_file_filter: Option<String>,
    /// File extensions presented to the user in a dropdown when saving a file.
    pub save_extensions: Vec<SaveExtension>,
    /// Name of the file extension selected by default.
    pub default_save_extension: Option<String>,
    /// Sets custom icons for different files or folders.
    /// Use `FileDialogConfig::set_file_icon` to add a new icon to this list.
    pub file_icon_filters: Vec<IconFilter>,

    /// Custom sections added to the left sidebar for quick access.
    /// Use `FileDialogConfig::add_quick_access` to add a new section to this list.
    pub quick_accesses: Vec<QuickAccess>,

    // ------------------------------------------------------------------------
    // Window options:
    /// If set, the window title will be overwritten and set to the fixed value instead
    /// of being set dynamically.
    pub title: Option<String>,
    /// The ID of the window.
    pub id: Option<egui::Id>,
    /// The default position of the window.
    pub default_pos: Option<egui::Pos2>,
    /// Sets the window position and prevents it from being dragged around.
    pub fixed_pos: Option<egui::Pos2>,
    /// The default size of the window.
    pub default_size: egui::Vec2,
    /// The maximum size of the window.
    pub max_size: Option<egui::Vec2>,
    /// The minimum size of the window.
    pub min_size: egui::Vec2,
    /// The anchor of the window.
    pub anchor: Option<(egui::Align2, egui::Vec2)>,
    /// If the window is resizable.
    pub resizable: bool,
    /// If the window is movable.
    pub movable: bool,
    /// If the title bar of the window is shown.
    pub title_bar: bool,

    // ------------------------------------------------------------------------
    // Feature options:
    /// If the top panel with the navigation buttons, current path display and search input
    /// should be visible.
    pub show_top_panel: bool,
    /// Whether the parent folder button should be visible at the top.
    pub show_parent_button: bool,
    /// Whether the back button should be visible at the top.
    pub show_back_button: bool,
    /// Whether the forward button should be visible at the top.
    pub show_forward_button: bool,
    /// If the button to create a new folder should be visible at the top.
    pub show_new_folder_button: bool,
    /// If the current path display in the top panel should be visible.
    pub show_current_path: bool,
    /// If the button to text edit the current path should be visible.
    pub show_path_edit_button: bool,
    /// If the menu button containing the reload button and other options should be visible.
    pub show_menu_button: bool,
    /// If the reload button inside the top panel menu should be visible.
    pub show_reload_button: bool,
    /// If the working directory shortcut in the hamburger menu should be visible.
    pub show_working_directory_button: bool,
    /// If the show hidden files and folders option inside the top panel menu should be visible.
    pub show_hidden_option: bool,
    /// If the show system files option inside the top panel menu should be visible.
    pub show_system_files_option: bool,
    /// If the search input in the top panel should be visible.
    pub show_search: bool,

    /// Set the width of the right panel, if used
    pub right_panel_width: Option<f32>,

    /// If the sidebar with the shortcut directories such as
    /// “Home”, “Documents” etc. should be visible.
    pub show_left_panel: bool,
    /// If pinned folders should be listed in the left sidebar.
    /// Disabling this will also disable the functionality to pin a folder.
    pub show_pinned_folders: bool,
    /// If the Places section in the left sidebar should be visible.
    pub show_places: bool,
    /// If the Devices section in the left sidebar should be visible.
    pub show_devices: bool,
    /// If the Removable Devices section in the left sidebar should be visible.
    pub show_removable_devices: bool,
}

impl Default for FileDialogConfig {
    fn default() -> Self {
        Self::default_from_filesystem(Arc::new(NativeFileSystem))
    }
}

impl FileDialogConfig {
    /// Creates a new configuration with default values
    pub fn default_from_filesystem(file_system: Arc<dyn FileSystem + Send + Sync>) -> Self {
        Self {
            storage: FileDialogStorage::default(),
            labels: FileDialogLabels::default(),
            keybindings: FileDialogKeyBindings::default(),

            opening_mode: OpeningMode::LastPickedDir,
            as_modal: true,
            modal_overlay_color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 120),
            initial_directory: file_system.current_dir().unwrap_or_default(),
            default_file_name: String::from("Untitled"),
            allow_file_overwrite: true,
            allow_path_edit_to_save_file_without_extension: false,
            directory_separator: String::from(">"),
            canonicalize_paths: true,

            #[cfg(target_arch = "wasm32")]
            load_via_thread: false,
            #[cfg(not(target_arch = "wasm32"))]
            load_via_thread: true,

            truncate_filenames: true,

            err_icon: String::from("⚠"),
            warn_icon: String::from("⚠"),
            default_file_icon: String::from("🗋"),
            default_folder_icon: String::from("🗀"),
            pinned_icon: String::from("📌"),
            device_icon: String::from("🖴"),
            removable_device_icon: String::from("💾"),

            file_filters: Vec::new(),
            default_file_filter: None,
            save_extensions: Vec::new(),
            default_save_extension: None,
            file_icon_filters: Vec::new(),

            quick_accesses: Vec::new(),

            title: None,
            id: None,
            default_pos: None,
            fixed_pos: None,
            default_size: egui::Vec2::new(650.0, 370.0),
            max_size: None,
            min_size: egui::Vec2::new(340.0, 170.0),
            anchor: None,
            resizable: true,
            movable: true,
            title_bar: true,

            show_top_panel: true,
            show_parent_button: true,
            show_back_button: true,
            show_forward_button: true,
            show_new_folder_button: true,
            show_current_path: true,
            show_path_edit_button: true,
            show_menu_button: true,
            show_reload_button: true,
            show_working_directory_button: true,
            show_hidden_option: true,
            show_system_files_option: true,
            show_search: true,

            right_panel_width: None,
            show_left_panel: true,
            show_pinned_folders: true,
            show_places: true,
            show_devices: true,
            show_removable_devices: true,

            file_system,
        }
    }
}

impl FileDialogConfig {
    /// Sets the storage used by the file dialog.
    /// Storage includes all data that is persistently stored between multiple
    /// file dialog instances.
    pub fn storage(mut self, storage: FileDialogStorage) -> Self {
        self.storage = storage;
        self
    }

    /// Adds a new file filter the user can select from a dropdown widget.
    ///
    /// NOTE: The name must be unique. If a filter with the same name already exists,
    ///       it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name of the filter
    /// * `filter` - Sets a filter function that checks whether a given
    ///   Path matches the criteria for this filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use egui_file_dialog::FileDialogConfig;
    ///
    /// let config = FileDialogConfig::default()
    ///     .add_file_filter(
    ///         "PNG files",
    ///         Arc::new(|path| path.extension().unwrap_or_default() == "png"))
    ///     .add_file_filter(
    ///         "JPG files",
    ///         Arc::new(|path| path.extension().unwrap_or_default() == "jpg"));
    /// ```
    pub fn add_file_filter(mut self, name: &str, filter: Filter<Path>) -> Self {
        let id = egui::Id::new(name);

        // Replace filter if a filter with the same name already exists.
        if let Some(item) = self.file_filters.iter_mut().find(|p| p.id == id) {
            item.filter = filter.clone();
            return self;
        }

        self.file_filters.push(FileFilter {
            id,
            name: name.to_owned(),
            filter,
        });

        self
    }

    /// Adds a new file extension that the user can select in a dropdown widget when
    /// saving a file.
    ///
    /// NOTE: The name must be unique. If an extension with the same name already exists,
    ///       it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name of the save extension.
    /// * `file_extension` - The file extension to use.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use egui_file_dialog::FileDialogConfig;
    ///
    /// let config = FileDialogConfig::default()
    ///     .add_save_extension("PNG files", "png")
    ///     .add_save_extension("JPG files", "jpg");
    /// ```
    pub fn add_save_extension(mut self, name: &str, file_extension: &str) -> Self {
        let id = egui::Id::new(name);

        // Replace extension when an extension with the same name already exists.
        if let Some(item) = self.save_extensions.iter_mut().find(|p| p.id == id) {
            file_extension.clone_into(&mut item.file_extension);
            return self;
        }

        self.save_extensions.push(SaveExtension {
            id,
            name: name.to_owned(),
            file_extension: file_extension.to_owned(),
        });

        self
    }

    /// Sets a new icon for specific files or folders.
    ///
    /// # Arguments
    ///
    /// * `icon` - The icon that should be used.
    /// * `filter` - Sets a filter function that checks whether a given
    ///   Path matches the criteria for this icon.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use egui_file_dialog::FileDialogConfig;
    ///
    /// let config = FileDialogConfig::default()
    ///     // .png files should use the "document with picture (U+1F5BB)" icon.
    ///     .set_file_icon("🖻", Arc::new(|path| path.extension().unwrap_or_default() == "png"))
    ///     // .git directories should use the "web-github (U+E624)" icon.
    ///     .set_file_icon("", Arc::new(|path| path.file_name().unwrap_or_default() == ".git"));
    /// ```
    pub fn set_file_icon(mut self, icon: &str, filter: Filter<Path>) -> Self {
        self.file_icon_filters.push(IconFilter {
            icon: icon.to_string(),
            filter,
        });

        self
    }

    /// Adds a new custom quick access section to the left panel of the file dialog.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_file_dialog::FileDialogConfig;
    ///
    /// FileDialogConfig::default()
    ///     .add_quick_access("My App", |s| {
    ///         s.add_path("Config", "/app/config");
    ///         s.add_path("Themes", "/app/themes");
    ///         s.add_path("Languages", "/app/languages");
    ///     });
    /// ```
    pub fn add_quick_access(
        mut self,
        heading: &str,
        builder: impl FnOnce(&mut QuickAccess),
    ) -> Self {
        let mut obj = QuickAccess {
            canonicalize_paths: self.canonicalize_paths,
            heading: heading.to_string(),
            paths: Vec::new(),
        };
        builder(&mut obj);
        self.quick_accesses.push(obj);
        self
    }
}

/// Function that returns true if the specific item matches the filter.
pub type Filter<T> = Arc<dyn Fn(&T) -> bool + Send + Sync>;

/// Defines a specific file filter that the user can select from a dropdown.
#[derive(Clone)]
pub struct FileFilter {
    /// The ID of the file filter, used internally for identification.
    pub id: egui::Id,
    /// The display name of the file filter
    pub name: String,
    /// Sets a filter function that checks whether a given Path matches the criteria for this file.
    pub filter: Filter<Path>,
}

impl std::fmt::Debug for FileFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileFilter")
            .field("name", &self.name)
            .finish()
    }
}

/// Defines a specific file extension that the user can select when saving a file.
#[derive(Clone, Debug)]
pub struct SaveExtension {
    /// The ID of the file filter, used internally for identification.
    pub id: egui::Id,
    /// The display name of the file filter.
    pub name: String,
    /// The file extension to use.
    pub file_extension: String,
}

impl Display for SaveExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{} (.{})", &self.name, &self.file_extension))
    }
}

/// Sets a specific icon for directory entries.
#[derive(Clone)]
pub struct IconFilter {
    /// The icon that should be used.
    pub icon: String,
    /// Sets a filter function that checks whether a given Path matches the criteria for this icon.
    pub filter: Filter<Path>,
}

impl std::fmt::Debug for IconFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IconFilter")
            .field("icon", &self.icon)
            .finish()
    }
}

/// Stores the display name and the actual path of a quick access link.
#[derive(Debug, Clone)]
pub struct QuickAccessPath {
    /// Name of the path that is shown inside the left panel.
    pub display_name: String,
    /// Absolute or relative path to the folder.
    pub path: PathBuf,
}

/// Stores a custom quick access section of the file dialog.
#[derive(Debug, Clone)]
pub struct QuickAccess {
    /// If the path's inside the quick access section should be canonicalized.
    canonicalize_paths: bool,
    /// Name of the quick access section displayed inside the left panel.
    pub heading: String,
    /// Path's contained inside the quick access section.
    pub paths: Vec<QuickAccessPath>,
}

impl QuickAccess {
    /// Adds a new path to the quick access.
    ///
    /// Since `fs::canonicalize` is used, both absolute paths and relative paths are allowed.
    /// See `FileDialog::canonicalize_paths` for more information.
    ///
    /// See `FileDialogConfig::add_quick_access` for an example.
    pub fn add_path(&mut self, display_name: &str, path: impl Into<PathBuf>) {
        let path = path.into();

        let canonicalized_path = if self.canonicalize_paths {
            dunce::canonicalize(&path).unwrap_or(path)
        } else {
            path
        };

        self.paths.push(QuickAccessPath {
            display_name: display_name.to_string(),
            path: canonicalized_path,
        });
    }
}
