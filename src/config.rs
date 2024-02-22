use std::path::PathBuf;

/// Contains configuration values of a file dialog.
///
/// The configuration of a file dialog can be overwritten with `FileDialog::overwrite_config`. \
/// If you only need to configure a single file dialog, you don't need to
/// manually use a `FileDialogConfig` object. `FileDialog` provides setter methods for
/// each of these configuration options, for example: `FileDialog::initial_directory`
/// or `FileDialog::default_size`. \
/// `FileDialogConfig` is useful when you need to configure multiple `FileDialog` objects with the
/// same or almost the same options.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileDialogConfig {
    // ------------------------------------------------------------------------
    // General options:
    /// The first directory that will be opened when the dialog opens.
    pub initial_directory: PathBuf,
    /// The default filename when opening the dialog in `DialogMode::SaveFile` mode.
    pub default_file_name: String,

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
    fn default() -> Self {
        Self {
            initial_directory: std::env::current_dir().unwrap_or_default(),
            default_file_name: String::new(),

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
            show_reload_button: true,
            show_search: true,

            show_left_panel: true,
            show_places: true,
            show_devices: true,
            show_removable_devices: true,
        }
    }
}
