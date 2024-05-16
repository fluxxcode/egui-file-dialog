mod labels;
pub use labels::FileDialogLabels;

mod keybindings;
pub use keybindings::FileDialogKeyBindings;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::FileDialogStorage;

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
    /// Persistent data of the file dialog.
    pub storage: FileDialogStorage,
    /// Keybindings used by the file dialog.
    pub keybindings: FileDialogKeyBindings,

    // ------------------------------------------------------------------------
    // General options:
    /// The labels that the dialog uses.
    pub labels: FileDialogLabels,
    /// The first directory that will be opened when the dialog opens.
    pub initial_directory: PathBuf,
    /// The default filename when opening the dialog in `DialogMode::SaveFile` mode.
    pub default_file_name: String,
    /// If the user is allowed to select an already existing file when the dialog is
    /// in `DialogMode::SaveFile` mode.
    pub allow_file_overwrite: bool,
    /// Sets the separator of the directories when displaying a path.
    /// Currently only used when the current path is displayed in the top panel.
    pub directory_separator: String,
    /// If the paths in the file dialog should be canonicalized before use.
    pub canonicalize_paths: bool,

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
    /// If the reload button in the top panel should be visible.
    pub show_reload_button: bool,
    /// If the search input in the top panel should be visible.
    pub show_search: bool,

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
    /// Creates a new configuration with default values
    fn default() -> Self {
        Self {
            storage: FileDialogStorage::default(),
            keybindings: FileDialogKeyBindings::default(),

            labels: FileDialogLabels::default(),
            initial_directory: std::env::current_dir().unwrap_or_default(),
            default_file_name: String::new(),
            allow_file_overwrite: true,
            directory_separator: String::from(">"),
            canonicalize_paths: true,

            err_icon: String::from("⚠"),
            warn_icon: String::from("⚠"),
            default_file_icon: String::from("🗋"),
            default_folder_icon: String::from("🗀"),
            pinned_icon: String::from("📌"),
            device_icon: String::from("🖴"),
            removable_device_icon: String::from("💾"),

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
            show_reload_button: true,
            show_search: true,

            show_left_panel: true,
            show_pinned_folders: true,
            show_places: true,
            show_devices: true,
            show_removable_devices: true,
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
pub type Filter<T> = Arc<dyn Fn(&T) -> bool>;

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
    pub display_name: String,
    pub path: PathBuf,
}

/// Stores a custom quick access section of the file dialog.
#[derive(Debug, Clone)]
pub struct QuickAccess {
    pub canonicalize_paths: bool,
    pub heading: String,
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

        let canonicalized_path = match self.canonicalize_paths {
            true => fs::canonicalize(&path).unwrap_or(path),
            false => path,
        };

        self.paths.push(QuickAccessPath {
            display_name: display_name.to_string(),
            path: canonicalized_path,
        });
    }
}
