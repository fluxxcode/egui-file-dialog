use std::path::{Path, PathBuf};
use std::{fs, io};

use egui::text::{CCursor, CCursorRange};

use crate::config::{
    FileDialogConfig, FileDialogKeyBindings, FileDialogLabels, FileDialogStorage, FileFilter,
    Filter, QuickAccess,
};
use crate::create_directory_dialog::CreateDirectoryDialog;
use crate::data::{DirectoryContent, DirectoryEntry, Disk, Disks, UserDirectories};
use crate::modals::{FileDialogModal, ModalAction, ModalState, OverwriteFileModal};

/// Represents the mode the file dialog is currently in.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DialogMode {
    /// When the dialog is currently used to select a single file.
    SelectFile,

    /// When the dialog is currently used to select a single directory.
    SelectDirectory,

    /// When the dialog is currently used to select multiple files and directories.
    SelectMultiple,

    /// When the dialog is currently used to save a file.
    SaveFile,
}

/// Represents the state the file dialog is currently in.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DialogState {
    /// The dialog is currently open and the user can perform the desired actions.
    Open,

    /// The dialog is currently closed and not visible.
    Closed,

    /// The user has selected a folder or file or specified a destination path for saving a file.
    Selected(PathBuf),

    /// The user has finished selecting multiple files and folders.
    SelectedMultiple(Vec<PathBuf>),

    /// The user cancelled the dialog and didn't select anything.
    Cancelled,
}

/// Represents a file dialog instance.
///
/// The `FileDialog` instance can be used multiple times and for different actions.
///
/// # Examples
///
/// ```
/// use egui_file_dialog::FileDialog;
///
/// struct MyApp {
///     file_dialog: FileDialog,
/// }
///
/// impl MyApp {
///     fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
///         if ui.button("Select a file").clicked() {
///             self.file_dialog.select_file();
///         }
///
///         if let Some(path) = self.file_dialog.update(ctx).selected() {
///             println!("Selected file: {:?}", path);
///         }
///     }
/// }
/// ```
pub struct FileDialog {
    /// The configuration of the file dialog
    config: FileDialogConfig,

    /// Stack of modal windows to be displayed.
    /// The top element is what is currently being rendered.
    modals: Vec<Box<dyn FileDialogModal>>,

    /// The mode the dialog is currently in
    mode: DialogMode,
    /// The state the dialog is currently in
    state: DialogState,
    /// If files are displayed in addition to directories.
    /// This option will be ignored when mode == DialogMode::SelectFile.
    show_files: bool,
    /// This is an optional ID that can be set when opening the dialog to determine which
    /// operation the dialog is used for. This is useful if the dialog is used multiple times
    /// for different actions in the same view. The ID then makes it possible to distinguish
    /// for which action the user has selected an item.
    /// This ID is not used internally.
    operation_id: Option<String>,

    /// The user directories like Home or Documents.
    /// These are loaded once when the dialog is created or when the refresh() method is called.
    user_directories: Option<UserDirectories>,
    /// The currently mounted system disks.
    /// These are loaded once when the dialog is created or when the refresh() method is called.
    system_disks: Disks,

    /// Contains the directories that the user opened. Every newly opened directory
    /// is pushed to the vector.
    /// Used for the navigation buttons to load the previous or next directory.
    directory_stack: Vec<PathBuf>,
    /// An offset from the back of directory_stack telling which directory is currently open.
    /// If 0, the user is currently in the latest open directory.
    /// If not 0, the user has used the "Previous directory" button and has
    /// opened previously opened directories.
    directory_offset: usize,
    /// The content of the currently open directory
    directory_content: DirectoryContent,
    /// This variable contains the error message if an error occurred while loading the directory.
    directory_error: Option<String>,

    /// The dialog that is shown when the user wants to create a new directory.
    create_directory_dialog: CreateDirectoryDialog,

    /// Whether the text edit is open for editing the current path.
    path_edit_visible: bool,
    /// Buffer holding the text when the user edits the current path.
    path_edit_value: String,
    /// If the path edit should be initialized. Unlike `path_edit_request_focus`,
    /// this also sets the cursor to the end of the text input field.
    path_edit_activate: bool,
    /// If the text edit of the path should request focus in the next frame.
    path_edit_request_focus: bool,

    /// The item that the user currently selected.
    /// Can be a directory or a folder.
    selected_item: Option<DirectoryEntry>,
    /// Buffer for the input of the file name when the dialog is in "SaveFile" mode.
    file_name_input: String,
    /// This variables contains the error message if the file_name_input is invalid.
    /// This can be the case, for example, if a file or folder with the name already exists.
    file_name_input_error: Option<String>,
    /// If the file name input text field should request focus in the next frame.
    file_name_input_request_focus: bool,
    /// The file filter the user selected
    selected_file_filter: Option<egui::Id>,

    /// If we should scroll to the item selected by the user in the next frame.
    scroll_to_selection: bool,
    /// Buffer containing the value of the search input.
    search_value: String,
    /// If the search should be initialized in the next frame.
    init_search: bool,

    /// If any widget was focused in the last frame.
    /// This is used to prevent the dialog from closing when pressing the escape key
    /// inside a text input.
    any_focused_last_frame: bool,
}

impl Default for FileDialog {
    /// Creates a new file dialog instance with default values.
    fn default() -> Self {
        Self::new()
    }
}

impl FileDialog {
    // ------------------------------------------------------------------------
    // Creation:

    /// Creates a new file dialog instance with default values.
    pub fn new() -> Self {
        Self {
            config: FileDialogConfig::default(),

            modals: Vec::new(),

            mode: DialogMode::SelectDirectory,
            state: DialogState::Closed,
            show_files: true,
            operation_id: None,

            user_directories: UserDirectories::new(true),
            system_disks: Disks::new_with_refreshed_list(true),

            directory_stack: Vec::new(),
            directory_offset: 0,
            directory_content: DirectoryContent::new(),
            directory_error: None,

            create_directory_dialog: CreateDirectoryDialog::new(),

            path_edit_visible: false,
            path_edit_value: String::new(),
            path_edit_activate: false,
            path_edit_request_focus: false,

            selected_item: None,
            file_name_input: String::new(),
            file_name_input_error: None,
            file_name_input_request_focus: true,
            selected_file_filter: None,

            scroll_to_selection: false,
            search_value: String::new(),
            init_search: false,

            any_focused_last_frame: false,
        }
    }

    /// Creates a new file dialog object and initializes it with the specified configuration.
    pub fn with_config(config: FileDialogConfig) -> Self {
        let mut obj = Self::new();
        *obj.config_mut() = config;
        obj
    }

    // -------------------------------------------------
    // Open, Update:

    /// Opens the file dialog in the given mode with the given options.
    /// This function resets the file dialog and takes care for the variables that need to be
    /// set when opening the file dialog.
    ///
    /// Returns the result of the operation to load the initial directory.
    ///
    /// If you don't need to set the individual parameters, you can also use the shortcut
    /// methods `select_directory`, `select_file` and `save_file`.
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode in which the dialog should be opened
    /// * `show_files` - If files should also be displayed to the user in addition to directories.
    ///    This is ignored if the mode is `DialogMode::SelectFile`.
    /// * `operation_id` - Sets an ID for which operation the dialog was opened.
    ///    This is useful when the dialog can be used for various operations in a single view.
    ///    The ID can then be used to check which action the user selected an item for.
    ///
    /// # Examples
    ///
    /// The following example shows how the dialog can be used for multiple
    /// actions using the `operation_id`.
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// use egui_file_dialog::{DialogMode, FileDialog};
    ///
    /// struct MyApp {
    ///     file_dialog: FileDialog,
    ///
    ///     selected_file_a: Option<PathBuf>,
    ///     selected_file_b: Option<PathBuf>,
    /// }
    ///
    /// impl MyApp {
    ///     fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
    ///         if ui.button("Select file a").clicked() {
    ///             let _ = self.file_dialog.open(DialogMode::SelectFile, true, Some("select_a"));
    ///         }
    ///
    ///         if ui.button("Select file b").clicked() {
    ///             let _ = self.file_dialog.open(DialogMode::SelectFile, true, Some("select_b"));
    ///         }
    ///
    ///         self.file_dialog.update(ctx);
    ///
    ///         if let Some(path) = self.file_dialog.selected() {
    ///             if self.file_dialog.operation_id() == Some("select_a") {
    ///                 self.selected_file_a = Some(path.to_path_buf());
    ///             }
    ///             if self.file_dialog.operation_id() == Some("select_b") {
    ///                 self.selected_file_b = Some(path.to_path_buf());
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn open(
        &mut self,
        mode: DialogMode,
        mut show_files: bool,
        operation_id: Option<&str>,
    ) -> io::Result<()> {
        self.reset();

        if mode == DialogMode::SelectFile {
            show_files = true;
        }

        if mode == DialogMode::SaveFile {
            self.file_name_input
                .clone_from(&self.config.default_file_name);
        }

        if let Some(name) = &self.config.default_file_filter {
            for filter in &self.config.file_filters {
                if filter.name == name.as_str() {
                    self.selected_file_filter = Some(filter.id);
                }
            }
        }

        self.mode = mode;
        self.state = DialogState::Open;
        self.show_files = show_files;
        self.operation_id = operation_id.map(String::from);

        self.load_directory(&self.gen_initial_directory(&self.config.initial_directory))
    }

    /// Shortcut function to open the file dialog to prompt the user to select a directory.
    /// If used, no files in the directories will be shown to the user.
    /// Use the `open()` method instead, if you still want to display files to the user.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn select_directory(&mut self) {
        let _ = self.open(DialogMode::SelectDirectory, false, None);
    }

    /// Shortcut function to open the file dialog to prompt the user to select a file.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn select_file(&mut self) {
        let _ = self.open(DialogMode::SelectFile, true, None);
    }

    /// Shortcut function to open the file dialog to prompt the user to select multiple
    /// files and folders.
    /// This function resets the file dialog. Configuration variables such as `initial_directory`
    /// are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn select_multiple(&mut self) {
        let _ = self.open(DialogMode::SelectMultiple, true, None);
    }

    /// Shortcut function to open the file dialog to prompt the user to save a file.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn save_file(&mut self) {
        let _ = self.open(DialogMode::SaveFile, true, None);
    }

    /// The main update method that should be called every frame if the dialog is to be visible.
    ///
    /// This function has no effect if the dialog state is currently not `DialogState::Open`.
    pub fn update(&mut self, ctx: &egui::Context) -> &Self {
        if self.state != DialogState::Open {
            return self;
        }

        self.update_keybindings(ctx);
        self.update_ui(ctx);

        self
    }

    // -------------------------------------------------
    // Setter:
    /// Overwrites the configuration of the file dialog.
    ///
    /// This is useful when you want to configure multiple `FileDialog` objects with the
    /// same configuration. If you only want to configure a single object,
    /// it's probably easier to use the setter methods like `FileDialog::initial_directory`
    /// or `FileDialog::default_pos`.
    ///
    /// If you want to create a new FileDialog object with a config,
    /// you probably want to use `FileDialog::with_config`.
    ///
    /// NOTE: Any configuration that was set before `FileDialog::overwrite_config`
    /// will be overwritten! \
    /// This means, for example, that the following code is invalid:
    /// ```
    /// pub use egui_file_dialog::{FileDialog, FileDialogConfig};
    ///
    /// fn create_file_dialog() -> FileDialog {
    ///     FileDialog::new()
    ///        .title("Hello world")
    ///         // This will overwrite `.title("Hello world")`!
    ///        .overwrite_config(FileDialogConfig::default())
    /// }
    ///
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_file_dialog::{FileDialog, FileDialogConfig};
    ///
    /// struct MyApp {
    ///     file_dialog_a: FileDialog,
    ///     file_dialog_b: FileDialog,
    /// }
    ///
    /// impl MyApp {
    ///     pub fn new() -> Self {
    ///         let config = FileDialogConfig {
    ///             default_size: egui::Vec2::new(500.0, 500.0),
    ///             resizable: false,
    ///             movable: false,
    ///             ..Default::default()
    ///         };
    ///
    ///         Self {
    ///             file_dialog_a: FileDialog::new()
    ///                 .overwrite_config(config.clone())
    ///                 .title("File Dialog A")
    ///                 .id("fd_a"),
    ///
    ///             file_dialog_b: FileDialog::new()
    ///                 .overwrite_config(config)
    ///                 .title("File Dialog B")
    ///                 .id("fd_b"),
    ///         }
    ///     }
    /// }
    /// ```
    #[deprecated(
        since = "0.6.0",
        note = "use `FileDialog::with_config` and `FileDialog::config_mut` instead"
    )]
    pub fn overwrite_config(mut self, config: FileDialogConfig) -> Self {
        self.config = config;
        self
    }

    /// Mutably borrow internal `config`.
    pub fn config_mut(&mut self) -> &mut FileDialogConfig {
        &mut self.config
    }

    /// Sets the storage used by the file dialog.
    /// Storage includes all data that is persistently stored between multiple
    /// file dialog instances.
    pub fn storage(mut self, storage: FileDialogStorage) -> Self {
        self.config.storage = storage;
        self
    }

    /// Mutably borrow internal storage.
    pub fn storage_mut(&mut self) -> &mut FileDialogStorage {
        &mut self.config.storage
    }

    /// Sets the keybindings used by the file dialog.
    pub fn keybindings(mut self, keybindings: FileDialogKeyBindings) -> Self {
        self.config.keybindings = keybindings;
        self
    }

    /// Sets the labels the file dialog uses.
    ///
    /// Used to enable multiple language support.
    ///
    /// See `FileDialogLabels` for more information.
    pub fn labels(mut self, labels: FileDialogLabels) -> Self {
        self.config.labels = labels;
        self
    }

    /// Mutably borrow internal `config.labels`.
    pub fn labels_mut(&mut self) -> &mut FileDialogLabels {
        &mut self.config.labels
    }

    /// If the file dialog window should be displayed as a modal.
    ///
    /// If the window is displayed as modal, the area outside the dialog can no longer be
    /// interacted with and an overlay is displayed.
    pub fn as_modal(mut self, as_modal: bool) -> Self {
        self.config.as_modal = as_modal;
        self
    }

    /// Sets the color of the overlay when the dialog is displayed as a modal window.
    pub fn modal_overlay_color(mut self, modal_overlay_color: egui::Color32) -> Self {
        self.config.modal_overlay_color = modal_overlay_color;
        self
    }

    /// Sets the first loaded directory when the dialog opens.
    /// If the path is a file, the file's parent directory is used. If the path then has no
    /// parent directory or cannot be loaded, the user will receive an error.
    /// However, the user directories and system disk allow the user to still select a file in
    /// the event of an error.
    ///
    /// Since `fs::canonicalize` is used, both absolute paths and relative paths are allowed.
    /// See `FileDialog::canonicalize_paths` for more information.
    pub fn initial_directory(mut self, directory: PathBuf) -> Self {
        self.config.initial_directory.clone_from(&directory);
        self
    }

    /// Sets the default file name when opening the dialog in `DialogMode::SaveFile` mode.
    pub fn default_file_name(mut self, name: &str) -> Self {
        self.config.default_file_name = name.to_string();
        self
    }

    /// Sets if the user is allowed to select an already existing file when the dialog is in
    /// `DialogMode::SaveFile` mode.
    ///
    /// If this is enabled, the user will receive a modal asking whether the user really
    /// wants to overwrite an existing file.
    pub fn allow_file_overwrite(mut self, allow_file_overwrite: bool) -> Self {
        self.config.allow_file_overwrite = allow_file_overwrite;
        self
    }

    /// Sets the separator of the directories when displaying a path.
    /// Currently only used when the current path is displayed in the top panel.
    pub fn directory_separator(mut self, separator: &str) -> Self {
        self.config.directory_separator = separator.to_string();
        self
    }

    /// Sets if the paths in the file dialog should be canonicalized before use.
    ///
    /// By default, all paths are canonicalized. This has the advantage that the paths are
    /// all brought to a standard and are therefore compatible with each other.
    ///
    /// On Windows, however, this results in the namespace prefix `\\?\` being set in
    /// front of the path, which may not be compatible with other applications.
    /// In addition, canonicalizing converts all relative paths to absolute ones.
    ///
    /// See: [Rust docs](https://doc.rust-lang.org/std/fs/fn.canonicalize.html)
    /// for more information.
    ///
    /// In general, it is only recommended to disable canonicalization if
    /// you know what you are doing and have a reason for it.
    /// Disabling canonicalization can lead to unexpected behavior, for example if an
    /// already canonicalized path is then set as the initial directory.
    pub fn canonicalize_paths(mut self, canonicalize: bool) -> Self {
        self.config.canonicalize_paths = canonicalize;
        self
    }

    /// Sets the icon that is used to display errors.
    pub fn err_icon(mut self, icon: &str) -> Self {
        self.config.err_icon = icon.to_string();
        self
    }

    /// Sets the default icon that is used to display files.
    pub fn default_file_icon(mut self, icon: &str) -> Self {
        self.config.default_file_icon = icon.to_string();
        self
    }

    /// Sets the default icon that is used to display folders.
    pub fn default_folder_icon(mut self, icon: &str) -> Self {
        self.config.default_folder_icon = icon.to_string();
        self
    }

    /// Sets the icon that is used to display devices in the left panel.
    pub fn device_icon(mut self, icon: &str) -> Self {
        self.config.device_icon = icon.to_string();
        self
    }

    /// Sets the icon that is used to display removable devices in the left panel.
    pub fn removable_device_icon(mut self, icon: &str) -> Self {
        self.config.removable_device_icon = icon.to_string();
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
    /// use egui_file_dialog::FileDialog;
    ///
    /// FileDialog::new()
    ///     .add_file_filter(
    ///         "PNG files",
    ///         Arc::new(|path| path.extension().unwrap_or_default() == "png"))
    ///     .add_file_filter(
    ///         "JPG files",
    ///         Arc::new(|path| path.extension().unwrap_or_default() == "jpg"));
    /// ```
    pub fn add_file_filter(mut self, name: &str, filter: Filter<Path>) -> Self {
        self.config = self.config.add_file_filter(name, filter);
        self
    }

    /// Name of the file filter to be selected by default.
    ///
    /// No file filter is selected if there is no file filter with that name.
    pub fn default_file_filter(mut self, name: &str) -> Self {
        self.config.default_file_filter = Some(name.to_string());
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
    /// use egui_file_dialog::FileDialog;
    ///
    /// FileDialog::new()
    ///     // .png files should use the "document with picture (U+1F5BB)" icon.
    ///     .set_file_icon("🖻", Arc::new(|path| path.extension().unwrap_or_default() == "png"))
    ///     // .git directories should use the "web-github (U+E624)" icon.
    ///     .set_file_icon("", Arc::new(|path| path.file_name().unwrap_or_default() == ".git"));
    /// ```
    pub fn set_file_icon(mut self, icon: &str, filter: Filter<std::path::Path>) -> Self {
        self.config = self.config.set_file_icon(icon, filter);
        self
    }

    /// Adds a new custom quick access section to the left panel.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_file_dialog::FileDialog;
    ///
    /// FileDialog::new()
    ///     .add_quick_access("My App", |s| {
    ///         s.add_path("Config", "/app/config");
    ///         s.add_path("Themes", "/app/themes");
    ///         s.add_path("Languages", "/app/languages");
    ///     });
    /// ```
    // pub fn add_quick_access(mut self, heading: &str, builder: &fn(&mut QuickAccess)) -> Self {
    pub fn add_quick_access(
        mut self,
        heading: &str,
        builder: impl FnOnce(&mut QuickAccess),
    ) -> Self {
        self.config = self.config.add_quick_access(heading, builder);
        self
    }

    /// Overwrites the window title.
    ///
    /// By default, the title is set dynamically, based on the `DialogMode`
    /// the dialog is currently in.
    pub fn title(mut self, title: &str) -> Self {
        self.config.title = Some(title.to_string());
        self
    }

    /// Sets the ID of the window.
    pub fn id(mut self, id: impl Into<egui::Id>) -> Self {
        self.config.id = Some(id.into());
        self
    }

    /// Sets the default position of the window.
    pub fn default_pos(mut self, default_pos: impl Into<egui::Pos2>) -> Self {
        self.config.default_pos = Some(default_pos.into());
        self
    }

    /// Sets the window position and prevents it from being dragged around.
    pub fn fixed_pos(mut self, pos: impl Into<egui::Pos2>) -> Self {
        self.config.fixed_pos = Some(pos.into());
        self
    }

    /// Sets the default size of the window.
    pub fn default_size(mut self, size: impl Into<egui::Vec2>) -> Self {
        self.config.default_size = size.into();
        self
    }

    /// Sets the maximum size of the window.
    pub fn max_size(mut self, max_size: impl Into<egui::Vec2>) -> Self {
        self.config.max_size = Some(max_size.into());
        self
    }

    /// Sets the minimum size of the window.
    ///
    /// Specifying a smaller minimum size than the default can lead to unexpected behavior.
    pub fn min_size(mut self, min_size: impl Into<egui::Vec2>) -> Self {
        self.config.min_size = min_size.into();
        self
    }

    /// Sets the anchor of the window.
    pub fn anchor(mut self, align: egui::Align2, offset: impl Into<egui::Vec2>) -> Self {
        self.config.anchor = Some((align, offset.into()));
        self
    }

    /// Sets if the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.config.resizable = resizable;
        self
    }

    /// Sets if the window is movable.
    ///
    /// Has no effect if an anchor is set.
    pub fn movable(mut self, movable: bool) -> Self {
        self.config.movable = movable;
        self
    }

    /// Sets if the title bar of the window is shown.
    pub fn title_bar(mut self, title_bar: bool) -> Self {
        self.config.title_bar = title_bar;
        self
    }

    /// Sets if the top panel with the navigation buttons, current path display
    /// and search input should be visible.
    pub fn show_top_panel(mut self, show_top_panel: bool) -> Self {
        self.config.show_top_panel = show_top_panel;
        self
    }

    /// Sets whether the parent folder button should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub fn show_parent_button(mut self, show_parent_button: bool) -> Self {
        self.config.show_parent_button = show_parent_button;
        self
    }

    /// Sets whether the back button should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub fn show_back_button(mut self, show_back_button: bool) -> Self {
        self.config.show_back_button = show_back_button;
        self
    }

    /// Sets whether the forward button should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub fn show_forward_button(mut self, show_forward_button: bool) -> Self {
        self.config.show_forward_button = show_forward_button;
        self
    }

    /// Sets whether the button to create a new folder should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub fn show_new_folder_button(mut self, show_new_folder_button: bool) -> Self {
        self.config.show_new_folder_button = show_new_folder_button;
        self
    }

    /// Sets whether the current path should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub fn show_current_path(mut self, show_current_path: bool) -> Self {
        self.config.show_current_path = show_current_path;
        self
    }

    /// Sets whether the button to text edit the current path should be visible in the top panel.
    ///
    /// has no effect when `FileDialog::show_top_panel` is disabled.
    pub fn show_path_edit_button(mut self, show_path_edit_button: bool) -> Self {
        self.config.show_path_edit_button = show_path_edit_button;
        self
    }

    /// Sets whether the menu with the reload button and other options should be visible
    /// inside the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub fn show_menu_button(mut self, show_menu_button: bool) -> Self {
        self.config.show_menu_button = show_menu_button;
        self
    }

    /// Sets whether the reload button inside the top panel menu should be visible.
    ///
    /// Has no effect when `FileDialog::show_top_panel` or
    /// `FileDialog::show_menu_button` is disabled.
    pub fn show_reload_button(mut self, show_reload_button: bool) -> Self {
        self.config.show_reload_button = show_reload_button;
        self
    }

    /// Sets whether the show hidden files and folders option inside the top panel
    /// menu should be visible.
    ///
    /// Has no effect when `FileDialog::show_top_panel` or
    /// `FileDialog::show_menu_button` is disabled.
    pub fn show_hidden_option(mut self, show_hidden_option: bool) -> Self {
        self.config.show_hidden_option = show_hidden_option;
        self
    }

    /// Sets whether the search input should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub fn show_search(mut self, show_search: bool) -> Self {
        self.config.show_search = show_search;
        self
    }

    /// Sets if the sidebar with the shortcut directories such as
    /// “Home”, “Documents” etc. should be visible.
    pub fn show_left_panel(mut self, show_left_panel: bool) -> Self {
        self.config.show_left_panel = show_left_panel;
        self
    }

    /// Sets if pinned folders should be listed in the left sidebar.
    /// Disabling this will also disable the functionality to pin a folder.
    pub fn show_pinned_folders(mut self, show_pinned_folders: bool) -> Self {
        self.config.show_pinned_folders = show_pinned_folders;
        self
    }

    /// Sets if the "Places" section should be visible in the left sidebar.
    /// The Places section contains the user directories such as Home or Documents.
    ///
    /// Has no effect when `FileDialog::show_left_panel` is disabled.
    pub fn show_places(mut self, show_places: bool) -> Self {
        self.config.show_places = show_places;
        self
    }

    /// Sets if the "Devices" section should be visible in the left sidebar.
    /// The Devices section contains the non removable system disks.
    ///
    /// Has no effect when `FileDialog::show_left_panel` is disabled.
    pub fn show_devices(mut self, show_devices: bool) -> Self {
        self.config.show_devices = show_devices;
        self
    }

    /// Sets if the "Removable Devices" section should be visible in the left sidebar.
    /// The Removable Devices section contains the removable disks like USB disks.
    ///
    /// Has no effect when `FileDialog::show_left_panel` is disabled.
    pub fn show_removable_devices(mut self, show_removable_devices: bool) -> Self {
        self.config.show_removable_devices = show_removable_devices;
        self
    }

    // -------------------------------------------------
    // Getter:

    /// Returns the directory or file that the user selected, or the target file
    /// if the dialog is in `DialogMode::SaveFile` mode.
    ///
    /// None is returned when the user has not yet selected an item.
    pub fn selected(&self) -> Option<&Path> {
        match &self.state {
            DialogState::Selected(path) => Some(path),
            _ => None,
        }
    }

    /// Returns the directory or file that the user selected, or the target file
    /// if the dialog is in `DialogMode::SaveFile` mode.
    /// Unlike `FileDialog::selected`, this method returns the selected path only once and
    /// sets the dialog's state to `DialogState::Closed`.
    ///
    /// None is returned when the user has not yet selected an item.
    pub fn take_selected(&mut self) -> Option<PathBuf> {
        match &mut self.state {
            DialogState::Selected(path) => {
                let path = std::mem::take(path);
                self.state = DialogState::Closed;
                Some(path)
            }
            _ => None,
        }
    }

    /// Returns a list of the files and folders the user selected, when the dialog is in
    /// `DialogMode::SelectMultiple` mode.
    ///
    /// None is returned when the user has not yet selected an item.
    pub fn selected_multiple(&self) -> Option<Vec<&Path>> {
        match &self.state {
            DialogState::SelectedMultiple(items) => {
                Some(items.iter().map(|f| f.as_path()).collect())
            }
            _ => None,
        }
    }

    /// Returns a list of the files and folders the user selected, when the dialog is in
    /// `DialogMode::SelectMultiple` mode.
    /// Unlike `FileDialog::selected_multiple`, this method returns the selected paths only once
    /// and sets the dialog's state to `DialogState::Closed`.
    ///
    /// None is returned when the user has not yet selected an item.
    pub fn take_selected_multiple(&mut self) -> Option<Vec<PathBuf>> {
        match &mut self.state {
            DialogState::SelectedMultiple(items) => {
                let items = std::mem::take(items);
                self.state = DialogState::Closed;
                Some(items)
            }
            _ => None,
        }
    }

    /// Returns the ID of the operation for which the dialog is currently being used.
    ///
    /// See `FileDialog::open` for more information.
    pub fn operation_id(&self) -> Option<&str> {
        self.operation_id.as_deref()
    }

    /// Returns the mode the dialog is currently in.
    pub fn mode(&self) -> DialogMode {
        self.mode
    }

    /// Returns the state the dialog is currently in.
    pub fn state(&self) -> DialogState {
        self.state.clone()
    }
}

/// UI methods
impl FileDialog {
    /// Main update method of the UI
    fn update_ui(&mut self, ctx: &egui::Context) {
        let mut is_open = true;

        if self.config.as_modal {
            let re = self.ui_update_modal_background(ctx);
            ctx.move_to_top(re.response.layer_id);
        }

        let re = self.create_window(&mut is_open).show(ctx, |ui| {
            if !self.modals.is_empty() {
                self.ui_update_modals(ui);
                return;
            }

            if self.config.show_top_panel {
                egui::TopBottomPanel::top("fe_top_panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        self.ui_update_top_panel(ui);
                    });
            }

            if self.config.show_left_panel {
                egui::SidePanel::left("fe_left_panel")
                    .resizable(true)
                    .default_width(150.0)
                    .width_range(90.0..=250.0)
                    .show_inside(ui, |ui| {
                        self.ui_update_left_panel(ui);
                    });
            }

            egui::TopBottomPanel::bottom("fe_bottom_panel")
                .resizable(false)
                .show_inside(ui, |ui| {
                    self.ui_update_bottom_panel(ui);
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.ui_update_central_panel(ui);
            });
        });

        if self.config.as_modal {
            if let Some(inner_response) = re {
                ctx.move_to_top(inner_response.response.layer_id);
            }
        }

        self.any_focused_last_frame = ctx.memory(|r| r.focused()).is_some();

        // User closed the window without finishing the dialog
        if !is_open {
            self.cancel();
        }
    }

    /// Updates the main modal background of the file dialog window.
    fn ui_update_modal_background(&self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::Area::new(egui::Id::from("fe_modal_overlay"))
            .interactable(true)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                let screen_rect = ctx.input(|i| i.screen_rect);

                ui.allocate_response(screen_rect.size(), egui::Sense::click());

                ui.painter().rect_filled(
                    screen_rect,
                    egui::Rounding::ZERO,
                    self.config.modal_overlay_color,
                );
            })
    }

    fn ui_update_modals(&mut self, ui: &mut egui::Ui) {
        // Currently, a rendering error occurs when only a single central panel is rendered
        // inside a window. Therefore, when rendering a modal, we render an invisible bottom panel,
        // which prevents the error.
        // This is currently a bit hacky and should be adjusted again in the future.
        egui::TopBottomPanel::bottom("fe_modal_bottom_panel")
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |_| {});

        // We need to use a central panel for the modals so that the
        // window doesn't resize to the size of the modal.
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(modal) = self.modals.last_mut() {
                #[allow(clippy::single_match)]
                match modal.update(&self.config, ui) {
                    ModalState::Close(action) => {
                        self.exec_modal_action(action);
                        self.modals.pop();
                    }
                    _ => {}
                }
            }
        });
    }

    /// Creates a new egui window with the configured options.
    fn create_window<'a>(&self, is_open: &'a mut bool) -> egui::Window<'a> {
        let window_title = match &self.config.title {
            Some(title) => title,
            None => match &self.mode {
                DialogMode::SelectDirectory => &self.config.labels.title_select_directory,
                DialogMode::SelectFile => &self.config.labels.title_select_file,
                DialogMode::SelectMultiple => &self.config.labels.title_select_multiple,
                DialogMode::SaveFile => &self.config.labels.title_save_file,
            },
        };

        let mut window = egui::Window::new(window_title)
            .open(is_open)
            .default_size(self.config.default_size)
            .min_size(self.config.min_size)
            .resizable(self.config.resizable)
            .movable(self.config.movable)
            .title_bar(self.config.title_bar)
            .collapsible(false);

        if let Some(id) = self.config.id {
            window = window.id(id);
        }

        if let Some(pos) = self.config.default_pos {
            window = window.default_pos(pos);
        }

        if let Some(pos) = self.config.fixed_pos {
            window = window.fixed_pos(pos);
        }

        if let Some((anchor, offset)) = self.config.anchor {
            window = window.anchor(anchor, offset);
        }

        if let Some(size) = self.config.max_size {
            window = window.max_size(size);
        }

        window
    }

    /// Updates the top panel of the dialog. Including the navigation buttons,
    /// the current path display, the reload button and the search field.
    fn ui_update_top_panel(&mut self, ui: &mut egui::Ui) {
        const BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(25.0, 25.0);

        ui.horizontal(|ui| {
            self.ui_update_nav_buttons(ui, &BUTTON_SIZE);

            let mut path_display_width = ui.available_width();

            // Leave some area for the menu button and search input
            if self.config.show_reload_button {
                path_display_width -= BUTTON_SIZE.x + ui.style().spacing.item_spacing.x * 2.5;
            }

            if self.config.show_search {
                path_display_width -= 140.0;
            }

            if self.config.show_current_path {
                self.ui_update_current_path(ui, path_display_width);
            }

            // Menu button containing reload button and different options
            if self.config.show_menu_button
                && (self.config.show_reload_button || self.config.show_hidden_option)
            {
                ui.allocate_ui_with_layout(
                    BUTTON_SIZE,
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.menu_button("☰", |ui| {
                            if self.config.show_reload_button
                                && ui.button(&self.config.labels.reload).clicked()
                            {
                                self.refresh();
                                ui.close_menu();
                            }

                            if self.config.show_hidden_option
                                && ui
                                    .checkbox(
                                        &mut self.config.storage.show_hidden,
                                        &self.config.labels.show_hidden,
                                    )
                                    .clicked()
                            {
                                ui.close_menu();
                            }
                        });
                    },
                );
            }

            if self.config.show_search {
                self.ui_update_search(ui);
            }
        });

        ui.add_space(ui.ctx().style().spacing.item_spacing.y);
    }

    /// Updates the navigation buttons like parent or previous directory
    fn ui_update_nav_buttons(&mut self, ui: &mut egui::Ui, button_size: &egui::Vec2) {
        if self.config.show_parent_button {
            if let Some(x) = self.current_directory() {
                if self.ui_button_sized(ui, x.parent().is_some(), *button_size, "⏶", None) {
                    let _ = self.load_parent_directory();
                }
            } else {
                let _ = self.ui_button_sized(ui, false, *button_size, "⏶", None);
            }
        }

        if self.config.show_back_button
            && self.ui_button_sized(
                ui,
                self.directory_offset + 1 < self.directory_stack.len(),
                *button_size,
                "⏴",
                None,
            )
        {
            let _ = self.load_previous_directory();
        }

        if self.config.show_forward_button
            && self.ui_button_sized(ui, self.directory_offset != 0, *button_size, "⏵", None)
        {
            let _ = self.load_next_directory();
        }

        if self.config.show_new_folder_button
            && self.ui_button_sized(
                ui,
                !self.create_directory_dialog.is_open(),
                *button_size,
                "+",
                None,
            )
        {
            self.open_new_folder_dialog();
        }
    }

    /// Updates the view to display the current path.
    /// This could be the view for displaying the current path and the individual sections,
    /// as well as the view for text editing of the current path.
    fn ui_update_current_path(&mut self, ui: &mut egui::Ui, width: f32) {
        egui::Frame::default()
            .stroke(egui::Stroke::new(
                1.0,
                ui.ctx().style().visuals.window_stroke.color,
            ))
            .inner_margin(egui::Margin::from(4.0))
            .rounding(egui::Rounding::from(4.0))
            .show(ui, |ui| {
                const EDIT_BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(22.0, 20.0);

                match self.path_edit_visible {
                    true => self.ui_update_path_edit(ui, width, EDIT_BUTTON_SIZE),
                    false => self.ui_update_path_display(ui, width, EDIT_BUTTON_SIZE),
                }
            });
    }

    /// Updates the view when the currently open path with the individual sections is displayed.
    fn ui_update_path_display(
        &mut self,
        ui: &mut egui::Ui,
        width: f32,
        edit_button_size: egui::Vec2,
    ) {
        ui.style_mut().always_scroll_the_only_direction = true;
        ui.style_mut().spacing.scroll.bar_width = 8.0;

        let mut max_width: f32 = width;

        if self.config.show_path_edit_button {
            max_width = width - edit_button_size.x - ui.style().spacing.item_spacing.x * 2.0;
        }

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .stick_to_right(true)
            .max_width(max_width)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing.x /= 2.5;

                    let mut path = PathBuf::new();

                    if let Some(data) = self.current_directory() {
                        #[cfg(windows)]
                        let mut drive_letter = String::from("\\");

                        for (i, segment) in data.iter().enumerate() {
                            path.push(segment);

                            #[cfg(windows)]
                            let mut file_name = segment.to_str().unwrap_or("<ERR>");

                            #[cfg(windows)]
                            {
                                // Skip the path namespace prefix generated by
                                // fs::canonicalize() on Windows
                                if i == 0 {
                                    drive_letter = file_name.replace(r"\\?\", "");
                                    continue;
                                }

                                // Replace the root segment with the disk letter
                                if i == 1 && segment == "\\" {
                                    file_name = drive_letter.as_str();
                                } else if i != 0 {
                                    ui.label(self.config.directory_separator.as_str());
                                }
                            }

                            #[cfg(not(windows))]
                            let file_name = segment.to_str().unwrap_or("<ERR>");

                            #[cfg(not(windows))]
                            if i != 0 {
                                ui.label(self.config.directory_separator.as_str());
                            }

                            if ui.button(file_name).clicked() {
                                let _ = self.load_directory(path.as_path());
                                return;
                            }
                        }
                    }
                });
            });

        if !self.config.show_path_edit_button {
            return;
        }

        if ui
            .add_sized(
                edit_button_size,
                egui::Button::new("🖊").fill(egui::Color32::TRANSPARENT),
            )
            .clicked()
        {
            self.open_path_edit();
        }
    }

    /// Updates the view when the user currently wants to text edit the current path.
    fn ui_update_path_edit(&mut self, ui: &mut egui::Ui, width: f32, edit_button_size: egui::Vec2) {
        let desired_width: f32 =
            width - edit_button_size.x - ui.style().spacing.item_spacing.x * 3.0;

        let response = egui::TextEdit::singleline(&mut self.path_edit_value)
            .desired_width(desired_width)
            .show(ui)
            .response;

        if self.path_edit_activate {
            response.request_focus();
            Self::set_cursor_to_end(&response, &self.path_edit_value);
            self.path_edit_activate = false;
        }

        if self.path_edit_request_focus {
            response.request_focus();
            self.path_edit_request_focus = false;
        }

        let btn_response = ui.add_sized(edit_button_size, egui::Button::new("✔"));

        if btn_response.clicked() {
            self.submit_path_edit();
        }

        if !response.has_focus() && !btn_response.contains_pointer() {
            self.path_edit_visible = false;
        }
    }

    /// Updates the search input
    fn ui_update_search(&mut self, ui: &mut egui::Ui) {
        egui::Frame::default()
            .stroke(egui::Stroke::new(
                1.0,
                ui.ctx().style().visuals.window_stroke.color,
            ))
            .inner_margin(egui::Margin::symmetric(4.0, 4.0))
            .rounding(egui::Rounding::from(4.0))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    ui.add_space(ui.ctx().style().spacing.item_spacing.y);
                    ui.label("🔍");
                    let re = ui.add_sized(
                        egui::Vec2::new(ui.available_width(), 0.0),
                        egui::TextEdit::singleline(&mut self.search_value),
                    );

                    self.edit_search_on_text_input(ui);

                    if re.changed() || self.init_search {
                        self.selected_item = None;
                        self.select_first_visible_item();
                    }

                    if self.init_search {
                        re.request_focus();
                        Self::set_cursor_to_end(&re, &self.search_value);
                        self.directory_content.reset_multi_selection();

                        self.init_search = false;
                    }
                });
            });
    }

    /// Focuses and types into the search input, if text input without
    /// shortcut modifiers is detected, and no other inputs are focused.
    ///
    /// # Arguments
    ///
    /// - `re`: The [`egui::Response`] returned by the filter text edit widget
    fn edit_search_on_text_input(&mut self, ui: &mut egui::Ui) {
        if ui.memory(|mem| mem.focused().is_some()) {
            return;
        }

        ui.input(|inp| {
            // We stop if any modifier is active besides only shift
            if inp.modifiers.any() && !inp.modifiers.shift_only() {
                return;
            }

            // If we find any text input event, we append it to the filter string
            // and allow proceeding to activating the filter input widget.
            for text in inp.events.iter().filter_map(|ev| match ev {
                egui::Event::Text(t) => Some(t),
                _ => None,
            }) {
                self.search_value.push_str(text);
                self.init_search = true;
            }
        });
    }

    /// Updates the left panel of the dialog. Including the list of the user directories (Places)
    /// and system disks (Devices, Removable Devices).
    fn ui_update_left_panel(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::containers::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Spacing for the first section in the left sidebar
                    let mut spacing = ui.ctx().style().spacing.item_spacing.y * 2.0;

                    // Spacing multiplier used between sections in the left sidebar
                    const SPACING_MULTIPLIER: f32 = 4.0;

                    // Update paths pinned to the left sidebar by the user
                    if self.config.show_pinned_folders && self.ui_update_pinned_paths(ui, spacing) {
                        spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
                    }

                    // Update custom quick access sections
                    let quick_accesses = std::mem::take(&mut self.config.quick_accesses);

                    for quick_access in &quick_accesses {
                        ui.add_space(spacing);
                        self.ui_update_quick_access(ui, quick_access);
                        spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
                    }

                    self.config.quick_accesses = quick_accesses;

                    // Update native quick access sections
                    if self.config.show_places && self.ui_update_user_directories(ui, spacing) {
                        spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
                    }

                    let disks = std::mem::take(&mut self.system_disks);

                    if self.config.show_devices && self.ui_update_devices(ui, spacing, &disks) {
                        spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
                    }

                    if self.config.show_removable_devices
                        && self.ui_update_removable_devices(ui, spacing, &disks)
                    {
                        // Add this when we add a new section after removable devices
                        // spacing = ui.ctx().style().spacing.item_spacing.y * SPACING_MULTIPLIER;
                    }

                    self.system_disks = disks;
                });
        });
    }

    /// Updates a path entry in the left panel.
    ///
    /// Returns the response of the selectable label.
    fn ui_update_left_panel_entry(
        &mut self,
        ui: &mut egui::Ui,
        display_name: &str,
        path: &Path,
    ) -> egui::Response {
        let response = ui.selectable_label(self.current_directory() == Some(path), display_name);

        if response.clicked() {
            let _ = self.load_directory(path);
        }

        response
    }

    /// Updates a custom quick access section added to the left panel.
    fn ui_update_quick_access(&mut self, ui: &mut egui::Ui, quick_access: &QuickAccess) {
        ui.label(&quick_access.heading);

        for entry in &quick_access.paths {
            self.ui_update_left_panel_entry(ui, &entry.display_name, &entry.path);
        }
    }

    /// Updates the list of pinned folders.
    ///
    /// Returns true if at least one directory item was included in the list and the
    /// heading is visible. If no item was listed, false is returned.
    fn ui_update_pinned_paths(&mut self, ui: &mut egui::Ui, spacing: f32) -> bool {
        let mut visible = false;

        for (i, path) in self
            .config
            .storage
            .pinned_folders
            .clone()
            .iter()
            .enumerate()
        {
            if i == 0 {
                ui.add_space(spacing);
                ui.label(self.config.labels.heading_pinned.as_str());

                visible = true;
            }

            let response = self.ui_update_left_panel_entry(
                ui,
                &format!("{}  {}", self.config.pinned_icon, path.file_name()),
                path.as_path(),
            );

            self.ui_update_path_context_menu(&response, path);
        }

        visible
    }

    /// Updates the list of user directories (Places).
    ///
    /// Returns true if at least one directory was included in the list and the
    /// heading is visible. If no directory was listed, false is returned.
    fn ui_update_user_directories(&mut self, ui: &mut egui::Ui, spacing: f32) -> bool {
        // Take temporary ownership of the user directories and configuration.
        // This is done so that we don't have to clone the user directories and
        // configured display names.
        let user_directories = std::mem::take(&mut self.user_directories);
        let config = std::mem::take(&mut self.config);

        let mut visible = false;

        if let Some(dirs) = &user_directories {
            ui.add_space(spacing);
            ui.label(self.config.labels.heading_places.as_str());

            if let Some(path) = dirs.home_dir() {
                self.ui_update_left_panel_entry(ui, &config.labels.home_dir, path);
            }

            if let Some(path) = dirs.desktop_dir() {
                self.ui_update_left_panel_entry(ui, &config.labels.desktop_dir, path);
            }
            if let Some(path) = dirs.document_dir() {
                self.ui_update_left_panel_entry(ui, &config.labels.documents_dir, path);
            }
            if let Some(path) = dirs.download_dir() {
                self.ui_update_left_panel_entry(ui, &config.labels.downloads_dir, path);
            }
            if let Some(path) = dirs.audio_dir() {
                self.ui_update_left_panel_entry(ui, &config.labels.audio_dir, path);
            }
            if let Some(path) = dirs.picture_dir() {
                self.ui_update_left_panel_entry(ui, &config.labels.pictures_dir, path);
            }
            if let Some(path) = dirs.video_dir() {
                self.ui_update_left_panel_entry(ui, &config.labels.videos_dir, path);
            }

            visible = true;
        }

        self.user_directories = user_directories;
        self.config = config;

        visible
    }

    /// Updates the list of devices like system disks.
    ///
    /// Returns true if at least one device was included in the list and the
    /// heading is visible. If no device was listed, false is returned.
    fn ui_update_devices(&mut self, ui: &mut egui::Ui, spacing: f32, disks: &Disks) -> bool {
        let mut visible = false;

        for (i, disk) in disks.iter().filter(|x| !x.is_removable()).enumerate() {
            if i == 0 {
                ui.add_space(spacing);
                ui.label(self.config.labels.heading_devices.as_str());

                visible = true;
            }

            self.ui_update_device_entry(ui, disk);
        }

        visible
    }

    /// Updates the list of removable devices like USB drives.
    ///
    /// Returns true if at least one device was included in the list and the
    /// heading is visible. If no device was listed, false is returned.
    fn ui_update_removable_devices(
        &mut self,
        ui: &mut egui::Ui,
        spacing: f32,
        disks: &Disks,
    ) -> bool {
        let mut visible = false;

        for (i, disk) in disks.iter().filter(|x| x.is_removable()).enumerate() {
            if i == 0 {
                ui.add_space(spacing);
                ui.label(self.config.labels.heading_removable_devices.as_str());

                visible = true;
            }

            self.ui_update_device_entry(ui, disk);
        }

        visible
    }

    /// Updates a device entry of a device list like "Devices" or "Removable Devices".
    fn ui_update_device_entry(&mut self, ui: &mut egui::Ui, device: &Disk) {
        let label = match device.is_removable() {
            true => format!(
                "{}  {}",
                self.config.removable_device_icon,
                device.display_name()
            ),
            false => format!("{}  {}", self.config.device_icon, device.display_name()),
        };

        self.ui_update_left_panel_entry(ui, &label, device.mount_point());
    }

    /// Updates the bottom panel showing the selected item and main action buttons.
    fn ui_update_bottom_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(5.0);

        const BUTTON_HEIGHT: f32 = 20.0;

        // Calculate the width of the action buttons
        let label_submit_width = match self.mode {
            DialogMode::SelectDirectory | DialogMode::SelectFile | DialogMode::SelectMultiple => {
                Self::calc_text_width(ui, &self.config.labels.open_button)
            }
            DialogMode::SaveFile => Self::calc_text_width(ui, &self.config.labels.save_button),
        };

        let mut btn_width = Self::calc_text_width(ui, &self.config.labels.cancel_button);
        if label_submit_width > btn_width {
            btn_width = label_submit_width;
        }

        btn_width += ui.spacing().button_padding.x * 4.0;

        // The size of the action buttons "cancel" and "open"/"save"
        let button_size: egui::Vec2 = egui::Vec2::new(btn_width, BUTTON_HEIGHT);

        self.ui_update_selection_preview(ui, button_size);

        if self.mode == DialogMode::SaveFile {
            ui.add_space(ui.style().spacing.item_spacing.y * 2.0)
        }

        self.ui_update_action_buttons(ui, button_size);
    }

    /// Updates the selection preview like "Selected directory: X"
    fn ui_update_selection_preview(&mut self, ui: &mut egui::Ui, button_size: egui::Vec2) {
        const SELECTION_PREVIEW_MIN_WIDTH: f32 = 50.0;
        let item_spacing = ui.style().spacing.item_spacing;

        let render_filter_selection = !self.config.file_filters.is_empty()
            && (self.mode == DialogMode::SelectFile || self.mode == DialogMode::SelectMultiple);

        let filter_selection_width = button_size.x * 2.0 + item_spacing.x;
        let mut filter_selection_separate_line = false;

        ui.horizontal(|ui| {
            match &self.mode {
                DialogMode::SelectDirectory => ui.label(&self.config.labels.selected_directory),
                DialogMode::SelectFile => ui.label(&self.config.labels.selected_file),
                DialogMode::SelectMultiple => ui.label(&self.config.labels.selected_items),
                DialogMode::SaveFile => ui.label(&self.config.labels.file_name),
            };

            // Make sure there is enough width for the selection preview. If the available
            // width is not enough, render the drop-down menu to select a file filter on
            // a separate line and give the selection preview the entire available width.
            let mut scroll_bar_width: f32 =
                ui.available_width() - filter_selection_width - item_spacing.x;

            if scroll_bar_width < SELECTION_PREVIEW_MIN_WIDTH || !render_filter_selection {
                filter_selection_separate_line = true;
                scroll_bar_width = ui.available_width();
            }

            match &self.mode {
                DialogMode::SelectDirectory
                | DialogMode::SelectFile
                | DialogMode::SelectMultiple => {
                    use egui::containers::scroll_area::ScrollBarVisibility;

                    let text = self.get_selection_preview_text();

                    egui::containers::ScrollArea::horizontal()
                        .auto_shrink([false, false])
                        .max_width(scroll_bar_width)
                        .stick_to_right(true)
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            ui.colored_label(ui.style().visuals.selection.bg_fill, text);
                        });
                }
                DialogMode::SaveFile => {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.file_name_input)
                            .desired_width(f32::INFINITY),
                    );

                    if self.file_name_input_request_focus {
                        response.request_focus();
                        self.file_name_input_request_focus = false;
                    }

                    if response.changed() {
                        self.file_name_input_error = self.validate_file_name_input();
                    }

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.submit();
                    }
                }
            };

            if !filter_selection_separate_line && render_filter_selection {
                self.ui_update_file_filter_selection(ui, filter_selection_width);
            }
        });

        if filter_selection_separate_line && render_filter_selection {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                self.ui_update_file_filter_selection(ui, filter_selection_width);
            });
        }
    }

    fn get_selection_preview_text(&self) -> String {
        if self.is_selection_valid() {
            match &self.mode {
                DialogMode::SelectDirectory | DialogMode::SelectFile => {
                    if let Some(item) = &self.selected_item {
                        item.file_name().to_string()
                    } else {
                        String::new()
                    }
                }
                DialogMode::SelectMultiple => {
                    let mut result = String::new();

                    for (i, item) in self
                        .get_dir_content_filtered_iter()
                        .filter(|p| p.selected)
                        .enumerate()
                    {
                        if i == 0 {
                            result += item.file_name();
                            continue;
                        }

                        result += format!(", {}", item.file_name()).as_str();
                    }

                    result
                }
                _ => String::new(),
            }
        } else {
            String::new()
        }
    }

    fn ui_update_file_filter_selection(&mut self, ui: &mut egui::Ui, width: f32) {
        let selected_filter = self.get_selected_file_filter();
        let selected_text = match selected_filter {
            Some(f) => &f.name,
            None => &self.config.labels.file_filter_all_files,
        };

        // The item that the user selected inside the drop down.
        // If none, no item was selected by the user.
        let mut select_filter: Option<Option<egui::Id>> = None;

        egui::containers::ComboBox::from_id_source("fe_file_filter_selection")
            .width(width)
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for filter in self.config.file_filters.iter() {
                    let selected = match selected_filter {
                        Some(f) => f.id == filter.id,
                        None => false,
                    };

                    if ui.selectable_label(selected, &filter.name).clicked() {
                        select_filter = Some(Some(filter.id));
                    }
                }

                if ui
                    .selectable_label(
                        selected_filter.is_none(),
                        &self.config.labels.file_filter_all_files,
                    )
                    .clicked()
                {
                    select_filter = Some(None);
                }
            });

        if let Some(i) = select_filter {
            self.selected_file_filter = i;
            self.selected_item = None;
            self.directory_content.reset_multi_selection();
        }
    }

    /// Updates the action buttons like save, open and cancel
    fn ui_update_action_buttons(&mut self, ui: &mut egui::Ui, button_size: egui::Vec2) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let label = match &self.mode {
                DialogMode::SelectDirectory
                | DialogMode::SelectFile
                | DialogMode::SelectMultiple => self.config.labels.open_button.as_str(),
                DialogMode::SaveFile => self.config.labels.save_button.as_str(),
            };

            if self.ui_button_sized(
                ui,
                self.is_selection_valid(),
                button_size,
                label,
                self.file_name_input_error.as_deref(),
            ) {
                self.submit();
            }

            if ui
                .add_sized(
                    button_size,
                    egui::Button::new(self.config.labels.cancel_button.as_str()),
                )
                .clicked()
            {
                self.cancel();
            }
        });
    }

    /// Updates the central panel, including the list of items in the currently open directory.
    fn ui_update_central_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(err) = &self.directory_error {
            ui.centered_and_justified(|ui| {
                ui.colored_label(
                    ui.style().visuals.error_fg_color,
                    format!("{} {}", self.config.err_icon, err),
                );
            });
            return;
        }

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::containers::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut data = std::mem::take(&mut self.directory_content);
                    let file_filter = self.get_selected_file_filter().cloned();

                    // If the multi selection should be reset, excluding the currently
                    // selected primary item
                    let mut reset_multi_selection = false;
                    // The item the user wants to make a batch selection from.
                    // The primary selected item is used for item a.
                    let mut batch_select_item_b: Option<DirectoryEntry> = None;

                    for item in data.filtered_iter_mut(
                        self.config.storage.show_hidden,
                        &self.search_value.clone(),
                        file_filter.as_ref(),
                    ) {
                        let file_name = item.file_name();

                        let mut primary_selected = false;
                        if let Some(x) = &self.selected_item {
                            primary_selected = x.path_eq(item);
                        }

                        let pinned = self.is_pinned(item);
                        let label = match pinned {
                            true => {
                                format!("{} {} {}", item.icon(), self.config.pinned_icon, file_name)
                            }
                            false => format!("{} {}", item.icon(), file_name),
                        };

                        let re = ui.selectable_label(primary_selected || item.selected, label);

                        if item.is_dir() {
                            self.ui_update_path_context_menu(&re, item);

                            if re.context_menu_opened() {
                                self.select_item(item);
                            }
                        }

                        if primary_selected && self.scroll_to_selection {
                            re.scroll_to_me(Some(egui::Align::Center));
                            self.scroll_to_selection = false;
                        }

                        // The user wants to select the item as the primary selected item
                        if re.clicked()
                            && !ui.input(|i| i.modifiers.ctrl)
                            && !ui.input(|i| i.modifiers.shift_only())
                        {
                            self.select_item(item);

                            // Mark the item as part of the multi selection
                            if self.mode == DialogMode::SelectMultiple {
                                reset_multi_selection = true;
                            }
                        }

                        // The user wants to select or unselect the item as part of a
                        // multi selection
                        if self.mode == DialogMode::SelectMultiple
                            && re.clicked()
                            && ui.input(|i| i.modifiers.ctrl)
                        {
                            if primary_selected {
                                // If the clicked item is the primary selected item,
                                // deselect it and remove it from the multi selection
                                item.selected = false;
                                self.selected_item = None;
                            } else {
                                item.selected = !item.selected;

                                // If the item was selected, make it the primary selected item
                                if item.selected {
                                    self.select_item(item);
                                }
                            }
                        }

                        // The user wants to select every item between the last selected item
                        // and the current item
                        if self.mode == DialogMode::SelectMultiple
                            && re.clicked()
                            && ui.input(|i| i.modifiers.shift_only())
                        {
                            if let Some(selected_item) = self.selected_item.clone() {
                                // We perform a batch selection from the item that was
                                // primarily selected before the user clicked on this item.
                                batch_select_item_b = Some(selected_item);

                                // And now make this item the primary selected item
                                item.selected = true;
                                self.select_item(item);
                            }
                        }

                        // The user double clicked on the directory entry.
                        // Either open the directory of submit the dialog.
                        if re.double_clicked() && !ui.input(|i| i.modifiers.ctrl) {
                            if item.is_dir() {
                                let _ = self.load_directory(&item.to_path_buf());
                                return;
                            }

                            self.select_item(item);

                            self.submit();
                        }
                    }

                    // Reset the multi selection except the currently selected primary item
                    if reset_multi_selection {
                        for item in data.filtered_iter_mut(
                            self.config.storage.show_hidden,
                            &self.search_value.clone(),
                            file_filter.as_ref(),
                        ) {
                            if let Some(selected_item) = &self.selected_item {
                                if selected_item.path_eq(item) {
                                    continue;
                                }
                            }

                            item.selected = false;
                        }
                    }

                    // Check if we should perform a batch selection
                    if let Some(item_b) = batch_select_item_b {
                        if let Some(item_a) = &self.selected_item {
                            self.batch_select_between(&mut data, item_a, &item_b);
                        }
                    }

                    self.directory_content = data;
                    self.scroll_to_selection = false;

                    if let Some(path) = self
                        .create_directory_dialog
                        .update(ui, &self.config)
                        .directory()
                    {
                        self.process_new_folder(&path);
                    }
                });
        });
    }

    /// Selects every item inside the directory_content between item_a and item_b,
    /// excluding both given items.
    fn batch_select_between(
        &self,
        directory_content: &mut DirectoryContent,
        item_a: &DirectoryEntry,
        item_b: &DirectoryEntry,
    ) {
        // Get the position of item a and item b
        let pos_a = directory_content
            .filtered_iter(
                self.config.storage.show_hidden,
                &self.search_value,
                self.get_selected_file_filter(),
            )
            .position(|p| p.path_eq(item_a));
        let pos_b = directory_content
            .filtered_iter(
                self.config.storage.show_hidden,
                &self.search_value,
                self.get_selected_file_filter(),
            )
            .position(|p| p.path_eq(item_b));

        // If both items where found inside the directory entry, mark every item between
        // them as selected
        if let Some(pos_a) = pos_a {
            if let Some(pos_b) = pos_b {
                if pos_a == pos_b {
                    return;
                }

                // Get the min and max of both positions.
                // We will iterate from min to max.
                let mut min = pos_a;
                let mut max = pos_b;

                if min > max {
                    min = pos_b;
                    max = pos_a;
                }

                for item in directory_content
                    .filtered_iter_mut(
                        self.config.storage.show_hidden,
                        &self.search_value,
                        self.get_selected_file_filter(),
                    )
                    .enumerate()
                    .filter(|(i, _)| i > &min && i < &max)
                    .map(|(_, p)| p)
                {
                    item.selected = true;
                }
            }
        }
    }

    /// Helper function to add a sized button that can be enabled or disabled
    fn ui_button_sized(
        &self,
        ui: &mut egui::Ui,
        enabled: bool,
        size: egui::Vec2,
        label: &str,
        err_tooltip: Option<&str>,
    ) -> bool {
        let mut clicked = false;

        ui.add_enabled_ui(enabled, |ui| {
            let response = ui.add_sized(size, egui::Button::new(label));
            clicked = response.clicked();

            if let Some(err) = err_tooltip {
                response.on_disabled_hover_ui(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;

                        ui.colored_label(
                            ui.ctx().style().visuals.error_fg_color,
                            format!("{} ", self.config.err_icon),
                        );

                        ui.label(err);
                    });
                });
            }
        });

        clicked
    }

    /// Updates the context menu of a path.
    ///
    /// # Arguments
    ///
    /// * `item_response` - The response of the egui item for which the context menu should
    ///                     be opened.
    /// * `path` - The path for which the context menu should be opened.
    fn ui_update_path_context_menu(
        &mut self,
        item_response: &egui::Response,
        path: &DirectoryEntry,
    ) {
        // Path context menus are currently only used for pinned folders.
        if !self.config.show_pinned_folders {
            return;
        }

        item_response.context_menu(|ui| {
            let pinned = self.is_pinned(path);

            if pinned {
                if ui.button(&self.config.labels.unpin_folder).clicked() {
                    self.unpin_path(path);
                    ui.close_menu();
                }
            } else if ui.button(&self.config.labels.pin_folder).clicked() {
                self.pin_path(path.clone());
                ui.close_menu();
            }
        });
    }

    /// Sets the cursor position to the end of a text input field.
    ///
    /// # Arguments
    ///
    /// * `re` - response of the text input widget
    /// * `data` - buffer holding the text of the input widget
    fn set_cursor_to_end(re: &egui::Response, data: &str) {
        // Set the cursor to the end of the filter input string
        if let Some(mut state) = egui::TextEdit::load_state(&re.ctx, re.id) {
            state
                .cursor
                .set_char_range(Some(CCursorRange::one(CCursor::new(data.len()))));
            state.store(&re.ctx, re.id);
        }
    }

    /// Calculate the width of the specified text using the current font configuration.
    fn calc_text_width(ui: &egui::Ui, text: &str) -> f32 {
        let mut width = 0.0;
        for char in text.chars() {
            width += ui.fonts(|f| f.glyph_width(&egui::TextStyle::Body.resolve(ui.style()), char));
        }

        width
    }
}

/// Keybindings
impl FileDialog {
    /// Checks whether certain keybindings have been pressed and executes the corresponding actions.
    fn update_keybindings(&mut self, ctx: &egui::Context) {
        // We don't want to execute keybindings if a modal is currently open.
        // The modals implement the keybindings themselves.
        if let Some(modal) = self.modals.last_mut() {
            modal.update_keybindings(&self.config, ctx);
            return;
        }

        let keybindings = std::mem::take(&mut self.config.keybindings);

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.submit, false) {
            self.exec_keybinding_submit();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.cancel, false) {
            self.exec_keybinding_cancel();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.parent, true) {
            let _ = self.load_parent_directory();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.back, true) {
            let _ = self.load_previous_directory();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.forward, true) {
            let _ = self.load_next_directory();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.reload, true) {
            self.refresh();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.new_folder, true) {
            self.open_new_folder_dialog();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.edit_path, true) {
            self.open_path_edit();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.home_edit_path, true) {
            if let Some(dirs) = &self.user_directories {
                if let Some(home) = dirs.home_dir() {
                    let _ = self.load_directory(home.to_path_buf().as_path());
                    self.open_path_edit();
                }
            }
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.selection_up, false) {
            self.exec_keybinding_selection_up();

            // We want to break out of input fields like search when pressing selection keys
            if let Some(id) = ctx.memory(|r| r.focused()) {
                ctx.memory_mut(|w| w.surrender_focus(id));
            }
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.selection_down, false) {
            self.exec_keybinding_selection_down();

            // We want to break out of input fields like search when pressing selection keys
            if let Some(id) = ctx.memory(|r| r.focused()) {
                ctx.memory_mut(|w| w.surrender_focus(id));
            }
        }

        self.config.keybindings = keybindings;
    }

    /// Executes the action when the keybinding `submit` is pressed.
    fn exec_keybinding_submit(&mut self) {
        if self.path_edit_visible {
            self.submit_path_edit();
            return;
        }

        if self.create_directory_dialog.is_open() {
            if let Some(dir) = self.create_directory_dialog.submit().directory() {
                self.process_new_folder(&dir);
            }
            return;
        }

        // Check if there is a directory selected we can open
        if let Some(item) = &self.selected_item {
            // Make sure the selected item is visible inside the directory view.
            let is_visible = self
                .get_dir_content_filtered_iter()
                .any(|p| p.path_eq(item));

            if is_visible && item.is_dir() {
                let _ = self.load_directory(&item.to_path_buf());
                return;
            }
        }

        self.submit();
    }

    /// Executes the action when the keybinding `cancel` is pressed.
    fn exec_keybinding_cancel(&mut self) {
        // We have to check if the `create_directory_dialog` and `path_edit_visible` is open,
        // because egui does not consume pressing the escape key inside a text input.
        // So when pressing the escape key inside a text input, the text input is closed
        // but the keybindings still register the press on the escape key.
        // (Although the keybindings are updated before the UI and they check whether another
        //  widget is currently in focus!)
        //
        // This is practical for us because we can close the path edit and
        // the create directory dialog.
        // However, this causes problems when the user presses escape in other text
        // inputs for which we have no status saved. This would then close the entire file dialog.
        // To fix this, we check if any item was focused in the last frame.
        //
        // Note that this only happens with the escape key and not when the enter key is
        // used to close a text input. This is why we don't have to check for the
        // dialogs in `exec_keybinding_submit`.

        if self.create_directory_dialog.is_open() {
            self.create_directory_dialog.close();
        } else if self.path_edit_visible {
            self.close_path_edit()
        } else if !self.any_focused_last_frame {
            self.cancel();
            return;
        }
    }

    /// Executes the action when the keybinding `selection_up` is pressed.
    fn exec_keybinding_selection_up(&mut self) {
        if self.directory_content.len() == 0 {
            return;
        }

        self.directory_content.reset_multi_selection();

        if let Some(item) = &self.selected_item {
            if self.select_next_visible_item_before(&item.clone()) {
                return;
            }
        }

        // No item is selected or no more items left.
        // Select the last item from the directory content.
        self.select_last_visible_item();
    }

    /// Executes the action when the keybinding `selection_down` is pressed.
    fn exec_keybinding_selection_down(&mut self) {
        if self.directory_content.len() == 0 {
            return;
        }

        self.directory_content.reset_multi_selection();

        if let Some(item) = &self.selected_item {
            if self.select_next_visible_item_after(&item.clone()) {
                return;
            }
        }

        // No item is selected or no more items left.
        // Select the last item from the directory content.
        self.select_first_visible_item();
    }
}

/// Implementation
impl FileDialog {
    /// Get the file filter the user currently selected.
    fn get_selected_file_filter(&self) -> Option<&FileFilter> {
        match self.selected_file_filter {
            Some(id) => self.config.file_filters.iter().find(|p| p.id == id),
            None => None,
        }
    }

    /// Gets a filtered iterator of the directory content of this object.
    fn get_dir_content_filtered_iter(&self) -> impl Iterator<Item = &DirectoryEntry> {
        self.directory_content.filtered_iter(
            self.config.storage.show_hidden,
            &self.search_value,
            self.get_selected_file_filter(),
        )
    }

    /// Opens the dialog to create a new folder.
    fn open_new_folder_dialog(&mut self) {
        if let Some(x) = self.current_directory() {
            self.create_directory_dialog.open(x.to_path_buf());
        }
    }

    /// Function that processes a newly created folder.
    fn process_new_folder(&mut self, created_dir: &Path) {
        let mut entry = DirectoryEntry::from_path(&self.config, created_dir);

        self.directory_content.push(entry.clone());

        self.select_item(&mut entry);
    }

    /// Opens a new modal window.
    fn open_modal(&mut self, modal: Box<dyn FileDialogModal>) {
        self.modals.push(modal);
    }

    /// Executes the given modal action.
    fn exec_modal_action(&mut self, action: ModalAction) {
        match action {
            ModalAction::None => {}
            ModalAction::SaveFile(path) => self.state = DialogState::Selected(path),
        };
    }

    /// Canonicalizes the specified path if canonicalization is enabled.
    /// Returns the input path if an error occurs or canonicalization is disabled.
    fn canonicalize_path(&self, path: &Path) -> PathBuf {
        match self.config.canonicalize_paths {
            true => fs::canonicalize(path).unwrap_or(path.to_path_buf()),
            false => path.to_path_buf(),
        }
    }

    /// Pins a path to the left sidebar.
    fn pin_path(&mut self, path: DirectoryEntry) {
        self.config.storage.pinned_folders.push(path);
    }

    /// Unpins a path from the left sidebar.
    fn unpin_path(&mut self, path: &DirectoryEntry) {
        self.config
            .storage
            .pinned_folders
            .retain(|p| !p.path_eq(path));
    }

    /// Checks if the path is pinned to the left sidebar.
    fn is_pinned(&self, path: &DirectoryEntry) -> bool {
        self.config
            .storage
            .pinned_folders
            .iter()
            .any(|p| path.path_eq(p))
    }

    /// Resets the dialog to use default values.
    /// Configuration variables are retained.
    fn reset(&mut self) {
        let config = self.config.clone();
        *self = FileDialog::with_config(config);
    }

    /// Refreshes the dialog.
    /// Including the user directories, system disks and currently open directory.
    fn refresh(&mut self) {
        self.user_directories = UserDirectories::new(self.config.canonicalize_paths);
        self.system_disks = Disks::new_with_refreshed_list(self.config.canonicalize_paths);

        let _ = self.reload_directory();
    }

    /// Submits the current selection and tries to finish the dialog, if the selection is valid.
    fn submit(&mut self) {
        // Make sure the selected item or entered file name is valid.
        if !self.is_selection_valid() {
            return;
        }

        match &self.mode {
            DialogMode::SelectDirectory | DialogMode::SelectFile => {
                // Should always contain a value since `is_selection_valid` is used to
                // validate the selection.
                if let Some(item) = self.selected_item.clone() {
                    self.state = DialogState::Selected(item.to_path_buf());
                }
            }
            DialogMode::SelectMultiple => {
                let result: Vec<PathBuf> = self
                    .get_dir_content_filtered_iter()
                    .filter(|p| p.selected)
                    .map(|p| p.to_path_buf())
                    .collect();

                self.state = DialogState::SelectedMultiple(result);
            }
            DialogMode::SaveFile => {
                // Should always contain a value since `is_selection_valid` is used to
                // validate the selection.
                if let Some(path) = self.current_directory() {
                    let mut full_path = path.to_path_buf();
                    full_path.push(&self.file_name_input);

                    if full_path.exists() {
                        self.open_modal(Box::new(OverwriteFileModal::new(full_path)));

                        return;
                    }

                    self.state = DialogState::Selected(full_path);
                }
            }
        }
    }

    /// Cancels the dialog.
    fn cancel(&mut self) {
        self.state = DialogState::Cancelled;
    }

    /// This function generates the initial directory based on the configuration.
    /// The function does the following things:
    ///   - Canonicalize the path if enabled
    ///   - Attempts to use the parent directory if the path is a file
    fn gen_initial_directory(&self, path: &Path) -> PathBuf {
        let mut path = self.canonicalize_path(path);

        if path.is_file() {
            if let Some(parent) = path.parent() {
                path = parent.to_path_buf();
            }
        }

        path
    }

    /// Gets the currently open directory.
    fn current_directory(&self) -> Option<&Path> {
        if let Some(x) = self.directory_stack.iter().nth_back(self.directory_offset) {
            return Some(x.as_path());
        }

        None
    }

    /// Checks whether the selection or the file name entered is valid.
    /// What is checked depends on the mode the dialog is currently in.
    fn is_selection_valid(&self) -> bool {
        match &self.mode {
            DialogMode::SelectDirectory => {
                if let Some(item) = &self.selected_item {
                    item.is_dir()
                } else {
                    false
                }
            }
            DialogMode::SelectFile => {
                if let Some(item) = &self.selected_item {
                    item.is_file()
                } else {
                    false
                }
            }
            DialogMode::SelectMultiple => self.get_dir_content_filtered_iter().any(|p| p.selected),
            DialogMode::SaveFile => self.file_name_input_error.is_none(),
        }
    }

    /// Validates the file name entered by the user.
    ///
    /// Returns None if the file name is valid. Otherwise returns an error message.
    fn validate_file_name_input(&self) -> Option<String> {
        if self.file_name_input.is_empty() {
            return Some(self.config.labels.err_empty_file_name.clone());
        }

        if let Some(x) = self.current_directory() {
            let mut full_path = x.to_path_buf();
            full_path.push(self.file_name_input.as_str());

            if full_path.is_dir() {
                return Some(self.config.labels.err_directory_exists.clone());
            }

            if !self.config.allow_file_overwrite && full_path.is_file() {
                return Some(self.config.labels.err_file_exists.clone());
            }
        } else {
            // There is most likely a bug in the code if we get this error message!
            return Some("Currently not in a directory".to_string());
        }

        None
    }

    /// Marks the given item as the selected directory item.
    /// Also updates the file_name_input to the name of the selected item.
    fn select_item(&mut self, item: &mut DirectoryEntry) {
        if self.mode == DialogMode::SelectMultiple {
            item.selected = true;
        }
        self.selected_item = Some(item.clone());

        if self.mode == DialogMode::SaveFile && item.is_file() {
            self.file_name_input = item.file_name().to_string();
            self.file_name_input_error = self.validate_file_name_input();
        }
    }

    /// Attempts to select the last visible item in `directory_content` before the specified item.
    ///
    /// Returns true if an item is found and selected.
    /// Returns false if no visible item is found before the specified item.
    fn select_next_visible_item_before(&mut self, item: &DirectoryEntry) -> bool {
        let mut return_val = false;

        let mut directory_content = std::mem::take(&mut self.directory_content);
        let search_value = std::mem::take(&mut self.search_value);
        let file_filter = self.get_selected_file_filter().cloned();

        let index = directory_content
            .filtered_iter(
                self.config.storage.show_hidden,
                &search_value,
                file_filter.as_ref(),
            )
            .position(|p| p.path_eq(item));

        if let Some(index) = index {
            if index != 0 {
                if let Some(item) = directory_content
                    .filtered_iter_mut(
                        self.config.storage.show_hidden,
                        &search_value.clone(),
                        file_filter.as_ref(),
                    )
                    .nth(index.saturating_sub(1))
                {
                    self.select_item(item);
                    self.scroll_to_selection = true;
                    return_val = true;
                }
            }
        }

        self.directory_content = directory_content;
        self.search_value = search_value;

        return_val
    }

    /// Attempts to select the last visible item in `directory_content` after the specified item.
    ///
    /// Returns true if an item is found and selected.
    /// Returns false if no visible item is found after the specified item.
    fn select_next_visible_item_after(&mut self, item: &DirectoryEntry) -> bool {
        let mut return_val = false;

        let mut directory_content = std::mem::take(&mut self.directory_content);
        let search_value = std::mem::take(&mut self.search_value);
        let file_filter = self.get_selected_file_filter().cloned();

        let index = directory_content
            .filtered_iter(
                self.config.storage.show_hidden,
                &search_value,
                file_filter.as_ref(),
            )
            .position(|p| p.path_eq(item));

        if let Some(index) = index {
            if let Some(item) = directory_content
                .filtered_iter_mut(
                    self.config.storage.show_hidden,
                    &search_value.clone(),
                    file_filter.as_ref(),
                )
                .nth(index.saturating_add(1))
            {
                self.select_item(item);
                self.scroll_to_selection = true;
                return_val = true;
            }
        }

        self.directory_content = directory_content;
        self.search_value = search_value;

        return_val
    }

    /// Tries to select the first visible item inside `directory_content`.
    fn select_first_visible_item(&mut self) {
        let mut directory_content = std::mem::take(&mut self.directory_content);

        if let Some(item) = directory_content
            .filtered_iter_mut(
                self.config.storage.show_hidden,
                &self.search_value.clone(),
                self.get_selected_file_filter().cloned().as_ref(),
            )
            .next()
        {
            self.select_item(item);
            self.scroll_to_selection = true;
        }

        self.directory_content = directory_content;
    }

    /// Tries to select the last visible item inside `directory_content`.
    fn select_last_visible_item(&mut self) {
        let mut directory_content = std::mem::take(&mut self.directory_content);

        if let Some(item) = directory_content
            .filtered_iter_mut(
                self.config.storage.show_hidden,
                &self.search_value.clone(),
                self.get_selected_file_filter().cloned().as_ref(),
            )
            .last()
        {
            self.select_item(item);
            self.scroll_to_selection = true;
        }

        self.directory_content = directory_content;
    }

    /// Opens the text field in the top panel to text edit the current path.
    fn open_path_edit(&mut self) {
        let path = match self.current_directory() {
            Some(path) => path.to_str().unwrap_or_default().to_string(),
            None => String::new(),
        };

        self.path_edit_value = path;
        self.path_edit_activate = true;
        self.path_edit_visible = true;
    }

    /// Loads the directory from the path text edit.
    fn submit_path_edit(&mut self) {
        self.close_path_edit();
        let _ = self.load_directory(&self.canonicalize_path(&PathBuf::from(&self.path_edit_value)));
    }

    /// Closes the text field at the top to edit the current path without loading
    /// the entered directory.
    fn close_path_edit(&mut self) {
        self.path_edit_visible = false;
    }

    /// Loads the next directory in the directory_stack.
    /// If directory_offset is 0 and there is no other directory to load, Ok() is returned and
    /// nothing changes.
    /// Otherwise, the result of the directory loading operation is returned.
    fn load_next_directory(&mut self) -> io::Result<()> {
        if self.directory_offset == 0 {
            // There is no next directory that can be loaded
            return Ok(());
        }

        self.directory_offset -= 1;

        // Copy path and load directory
        if let Some(path) = self.current_directory() {
            return self.load_directory_content(path.to_path_buf().as_path());
        }

        Ok(())
    }

    /// Loads the previous directory the user opened.
    /// If there is no previous directory left, Ok() is returned and nothing changes.
    /// Otherwise, the result of the directory loading operation is returned.
    fn load_previous_directory(&mut self) -> io::Result<()> {
        if self.directory_offset + 1 >= self.directory_stack.len() {
            // There is no previous directory that can be loaded
            return Ok(());
        }

        self.directory_offset += 1;

        // Copy path and load directory
        if let Some(path) = self.current_directory() {
            return self.load_directory_content(path.to_path_buf().as_path());
        }

        Ok(())
    }

    /// Loads the parent directory of the currently open directory.
    /// If the directory doesn't have a parent, Ok() is returned and nothing changes.
    /// Otherwise, the result of the directory loading operation is returned.
    fn load_parent_directory(&mut self) -> io::Result<()> {
        if let Some(x) = self.current_directory() {
            if let Some(x) = x.to_path_buf().parent() {
                return self.load_directory(x);
            }
        }

        Ok(())
    }

    /// Reloads the currently open directory.
    /// If no directory is currently open, Ok() will be returned.
    /// Otherwise, the result of the directory loading operation is returned.
    ///
    /// In most cases, this function should not be called directly.
    /// Instead, `refresh` should be used to reload all other data like system disks too.
    fn reload_directory(&mut self) -> io::Result<()> {
        if let Some(x) = self.current_directory() {
            return self.load_directory_content(x.to_path_buf().as_path());
        }

        Ok(())
    }

    /// Loads the given directory and updates the `directory_stack`.
    /// The function deletes all directories from the `directory_stack` that are currently
    /// stored in the vector before the `directory_offset`.
    ///
    /// The function also sets the loaded directory as the selected item.
    fn load_directory(&mut self, path: &Path) -> io::Result<()> {
        // Do not load the same directory again.
        // Use reload_directory if the content of the directory should be updated.
        if let Some(x) = self.current_directory() {
            if x == path {
                return Ok(());
            }
        }

        if self.directory_offset != 0 && self.directory_stack.len() > self.directory_offset {
            self.directory_stack
                .drain(self.directory_stack.len() - self.directory_offset..);
        }

        self.directory_stack.push(path.to_path_buf());
        self.directory_offset = 0;

        self.load_directory_content(path)?;

        let mut dir_entry = DirectoryEntry::from_path(&self.config, path);
        self.select_item(&mut dir_entry);

        // Clear the entry filter buffer.
        // It's unlikely the user wants to keep the current filter when entering a new directory.
        self.search_value.clear();

        Ok(())
    }

    /// Loads the directory content of the given path.
    fn load_directory_content(&mut self, path: &Path) -> io::Result<()> {
        self.directory_error = None;

        self.directory_content =
            match DirectoryContent::from_path(&self.config, path, self.show_files) {
                Ok(content) => content,
                Err(err) => {
                    self.directory_content.clear();
                    self.selected_item = None;
                    self.directory_error = Some(err.to_string());
                    return Err(err);
                }
            };

        self.create_directory_dialog.close();
        self.scroll_to_selection = true;

        if self.mode == DialogMode::SaveFile {
            self.file_name_input_error = self.validate_file_name_input();
        }

        Ok(())
    }
}
