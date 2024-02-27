use std::path::{Path, PathBuf};
use std::sync::Arc;

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

/// Contains configuration values of a file dialog.
///
/// The configuration of a file dialog can be set using
/// `FileDialog::with_config` or `FileDialog::overwrite_config`.
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
    // General options:
    /// The labels that the dialog uses.
    pub labels: FileDialogLabels,
    /// The first directory that will be opened when the dialog opens.
    pub initial_directory: PathBuf,
    /// The default filename when opening the dialog in `DialogMode::SaveFile` mode.
    pub default_file_name: String,
    /// Sets the separator of the directories when displaying a path.
    /// Currently only used when the current path is displayed in the top panel.
    pub directory_separator: String,
    /// If the paths in the file dialog should be canonicalized before use.
    pub canonicalize_paths: bool,

    /// The icon that is used to display error messages.
    pub err_icon: String,
    /// The default icon used to display files.
    pub default_file_icon: String,
    /// The default icon used to display folders.
    pub default_folder_icon: String,
    /// The icon used to display devices in the left panel.
    pub device_icon: String,
    /// The icon used to display removable devices in the left panel.
    pub removable_device_icon: String,

    /// Sets custom icons for different files or folders.
    /// Use `FileDialogConfig::set_file_icon` to add a new icon to this list.
    pub file_icon_filters: Vec<IconFilter>,

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
            labels: FileDialogLabels::default(),
            initial_directory: std::env::current_dir().unwrap_or_default(),
            default_file_name: String::new(),
            directory_separator: String::from(">"),
            canonicalize_paths: true,

            err_icon: String::from("⚠"),
            default_file_icon: String::from("🗋"),
            default_folder_icon: String::from("🗀"),
            device_icon: String::from("🖴"),
            removable_device_icon: String::from("💾"),

            file_icon_filters: Vec::new(),

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
            show_places: true,
            show_devices: true,
            show_removable_devices: true,
        }
    }
}

impl FileDialogConfig {
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
}

/// Contains the text labels that the file dialog uses.
///
/// This is used to enable multiple language support.
///
/// # Example
///
/// The following example shows how the default title of the dialog can be displayed
/// in German instead of English.
///
/// ```
/// use egui_file_dialog::{FileDialog, FileDialogLabels};
///
/// let labels_german = FileDialogLabels {
///     title_select_directory: "📁 Ordner Öffnen".to_string(),
///     title_select_file: "📂 Datei Öffnen".to_string(),
///     title_save_file: "📥 Datei Speichern".to_string(),
///     ..Default::default()
/// };
///
/// let file_dialog = FileDialog::new().labels(labels_german);
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileDialogLabels {
    // ------------------------------------------------------------------------
    // General:
    /// The default window title used when the dialog is in `DialogMode::SelectDirectory` mode.
    pub title_select_directory: String,
    /// The default window title used when the dialog is in `DialogMode::SelectFile` mode.
    pub title_select_file: String,
    /// The default window title used when the dialog is in `DialogMode::SaveFile` mode.
    pub title_save_file: String,

    // ------------------------------------------------------------------------
    // Left panel:
    /// Heading of the "Places" section in the left panel
    pub heading_places: String,
    /// Heading of the "Devices" section in the left panel
    pub heading_devices: String,
    /// Heading of the "Removable Devices" section in the left panel
    pub heading_removable_devices: String,

    /// Name of the home directory
    pub home_dir: String,
    /// Name of the desktop directory
    pub desktop_dir: String,
    /// Name of the documents directory
    pub documents_dir: String,
    /// Name of the downloads directory
    pub downloads_dir: String,
    /// Name of the audio directory
    pub audio_dir: String,
    /// Name of the pictures directory
    pub pictures_dir: String,
    /// Name of the videos directory
    pub videos_dir: String,

    // ------------------------------------------------------------------------
    // Bottom panel:
    /// Text that appears in front of the selected folder preview in the bottom panel.
    pub selected_directory: String,
    /// Text that appears in front of the selected file preview in the bottom panel.
    pub selected_file: String,
    /// Text that appears in front of the file name input in the bottom panel.
    pub file_name: String,

    /// Button text to open the selected item.
    pub open_button: String,
    /// Button text to save the file.
    pub save_button: String,
    /// Button text to cancel the dialog.
    pub cancel_button: String,

    // ------------------------------------------------------------------------
    // Error message:
    /// Error if no folder name was specified.
    pub err_empty_folder_name: String,
    /// Error if no file name was specified.
    pub err_empty_file_name: String,
    /// Error if the directory already exists.
    pub err_directory_exists: String,
    /// Error if the file already exists.
    pub err_file_exists: String,
}

impl Default for FileDialogLabels {
    /// Creates a new object with the default english labels.
    fn default() -> Self {
        Self {
            title_select_directory: "📁 Select Folder".to_string(),
            title_select_file: "📂 Open File".to_string(),
            title_save_file: "📥 Save File".to_string(),

            heading_places: "Places".to_string(),
            heading_devices: "Devices".to_string(),
            heading_removable_devices: "Removable Devices".to_string(),

            home_dir: "🏠  Home".to_string(),
            desktop_dir: "🖵  Desktop".to_string(),
            documents_dir: "🗐  Documents".to_string(),
            downloads_dir: "📥  Downloads".to_string(),
            audio_dir: "🎵  Audio".to_string(),
            pictures_dir: "🖼  Pictures".to_string(),
            videos_dir: "🎞  Videos".to_string(),

            selected_directory: "Selected directory:".to_string(),
            selected_file: "Selected file:".to_string(),
            file_name: "File name:".to_string(),

            open_button: "🗀  Open".to_string(),
            save_button: "📥  Save".to_string(),
            cancel_button: "🚫 Cancel".to_string(),

            err_empty_folder_name: "Name of the folder cannot be empty".to_string(),
            err_empty_file_name: "The file name cannot be empty".to_string(),
            err_directory_exists: "A directory with the name already exists".to_string(),
            err_file_exists: "A file with the name already exists".to_string(),
        }
    }
}
