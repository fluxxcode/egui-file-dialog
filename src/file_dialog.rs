use crate::config::{
    FileDialogConfig, FileDialogKeyBindings, FileDialogLabels, FileDialogStorage, FileFilter,
    Filter, OpeningMode, QuickAccess, SaveExtension,
};
use crate::create_directory_dialog::CreateDirectoryDialog;
use crate::data::{
    DirectoryContent, DirectoryContentState, DirectoryEntry, Disk, Disks, UserDirectories,
};
use crate::modals::{FileDialogModal, ModalAction, ModalState, OverwriteFileModal};
use crate::{FileSystem, NativeFileSystem};
use egui::text::{CCursor, CCursorRange};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Represents the mode the file dialog is currently in.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DialogMode {
    /// When the dialog is currently used to select a single file.
    PickFile,

    /// When the dialog is currently used to select a single directory.
    PickDirectory,

    /// When the dialog is currently used to select multiple files and directories.
    PickMultiple,

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
    Picked(PathBuf),

    /// The user has finished selecting multiple files and folders.
    PickedMultiple(Vec<PathBuf>),

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
///         if ui.button("Pick a file").clicked() {
///             self.file_dialog.pick_file();
///         }
///
///         if let Some(path) = self.file_dialog.update(ctx).picked() {
///             println!("Picked file: {:?}", path);
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub struct FileDialog {
    /// The configuration of the file dialog
    config: FileDialogConfig,

    /// Stack of modal windows to be displayed.
    /// The top element is what is currently being rendered.
    modals: Vec<Box<dyn FileDialogModal + Send + Sync>>,

    /// The mode the dialog is currently in
    mode: DialogMode,
    /// The state the dialog is currently in
    state: DialogState,
    /// If files are displayed in addition to directories.
    /// This option will be ignored when mode == `DialogMode::SelectFile`.
    show_files: bool,
    /// This is an optional ID that can be set when opening the dialog to determine which
    /// operation the dialog is used for. This is useful if the dialog is used multiple times
    /// for different actions in the same view. The ID then makes it possible to distinguish
    /// for which action the user has selected an item.
    /// This ID is not used internally.
    operation_id: Option<String>,

    /// The currently used window ID.
    window_id: egui::Id,

    /// The user directories like Home or Documents.
    /// These are loaded once when the dialog is created or when the `refresh()` method is called.
    user_directories: Option<UserDirectories>,
    /// The currently mounted system disks.
    /// These are loaded once when the dialog is created or when the `refresh()` method is called.
    system_disks: Disks,

    /// Contains the directories that the user opened. Every newly opened directory
    /// is pushed to the vector.
    /// Used for the navigation buttons to load the previous or next directory.
    directory_stack: Vec<PathBuf>,
    /// An offset from the back of `directory_stack` telling which directory is currently open.
    /// If 0, the user is currently in the latest open directory.
    /// If not 0, the user has used the "Previous directory" button and has
    /// opened previously opened directories.
    directory_offset: usize,
    /// The content of the currently open directory
    directory_content: DirectoryContent,

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
    /// Buffer for the input of the file name when the dialog is in `SaveFile` mode.
    file_name_input: String,
    /// This variables contains the error message if the `file_name_input` is invalid.
    /// This can be the case, for example, if a file or folder with the name already exists.
    file_name_input_error: Option<String>,
    /// If the file name input text field should request focus in the next frame.
    file_name_input_request_focus: bool,
    /// The file filter the user selected.
    selected_file_filter: Option<egui::Id>,
    /// The save extension that the user selected.
    selected_save_extension: Option<egui::Id>,

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

/// This tests if file dialog is send and sync.
#[cfg(test)]
const fn test_prop<T: Send + Sync>() {}

#[test]
const fn test() {
    test_prop::<FileDialog>();
}

impl Default for FileDialog {
    /// Creates a new file dialog instance with default values.
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for dyn FileDialogModal + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<FileDialogModal>")
    }
}

/// Callback type to inject a custom egui ui inside the file dialog's ui.
///
/// Also gives access to the file dialog, since it would otherwise be inaccessible
/// inside the closure.
type FileDialogUiCallback<'a> = dyn FnMut(&mut egui::Ui, &mut FileDialog) + 'a;

impl FileDialog {
    // ------------------------------------------------------------------------
    // Creation:

    /// Creates a new file dialog instance with default values.
    #[must_use]
    pub fn new() -> Self {
        let file_system = Arc::new(NativeFileSystem);
        Self {
            modals: Vec::new(),

            mode: DialogMode::PickDirectory,
            state: DialogState::Closed,
            show_files: true,
            operation_id: None,

            window_id: egui::Id::new("file_dialog"),

            user_directories: None,
            system_disks: Disks::new_empty(),

            directory_stack: Vec::new(),
            directory_offset: 0,
            directory_content: DirectoryContent::default(),

            create_directory_dialog: CreateDirectoryDialog::from_filesystem(file_system.clone()),

            path_edit_visible: false,
            path_edit_value: String::new(),
            path_edit_activate: false,
            path_edit_request_focus: false,

            selected_item: None,
            file_name_input: String::new(),
            file_name_input_error: None,
            file_name_input_request_focus: true,
            selected_file_filter: None,
            selected_save_extension: None,

            scroll_to_selection: false,
            search_value: String::new(),
            init_search: false,

            any_focused_last_frame: false,

            config: FileDialogConfig::default_from_filesystem(file_system),
        }
    }

    /// Creates a new file dialog object and initializes it with the specified configuration.
    pub fn with_config(config: FileDialogConfig) -> Self {
        let mut obj = Self::new();
        *obj.config_mut() = config;
        obj
    }

    /// Uses the given file system instead of the native file system.
    #[must_use]
    pub fn with_file_system(file_system: Arc<dyn FileSystem + Send + Sync>) -> Self {
        let mut obj = Self::new();
        obj.config.initial_directory = file_system.current_dir().unwrap_or_default();
        obj.config.file_system = file_system;
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
    ///     picked_file_a: Option<PathBuf>,
    ///     picked_file_b: Option<PathBuf>,
    /// }
    ///
    /// impl MyApp {
    ///     fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
    ///         if ui.button("Pick file a").clicked() {
    ///             let _ = self.file_dialog.open(DialogMode::PickFile, true, Some("pick_a"));
    ///         }
    ///
    ///         if ui.button("Pick file b").clicked() {
    ///             let _ = self.file_dialog.open(DialogMode::PickFile, true, Some("pick_b"));
    ///         }
    ///
    ///         self.file_dialog.update(ctx);
    ///
    ///         if let Some(path) = self.file_dialog.picked() {
    ///             if self.file_dialog.operation_id() == Some("pick_a") {
    ///                 self.picked_file_a = Some(path.to_path_buf());
    ///             }
    ///             if self.file_dialog.operation_id() == Some("pick_b") {
    ///                 self.picked_file_b = Some(path.to_path_buf());
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn open(&mut self, mode: DialogMode, mut show_files: bool, operation_id: Option<&str>) {
        self.reset();
        self.refresh();

        if mode == DialogMode::PickFile {
            show_files = true;
        }

        if mode == DialogMode::SaveFile {
            self.file_name_input_request_focus = true;
            self.file_name_input
                .clone_from(&self.config.default_file_name);
        }

        self.selected_file_filter = None;
        self.selected_save_extension = None;

        self.set_default_file_filter();
        self.set_default_save_extension();

        self.mode = mode;
        self.state = DialogState::Open;
        self.show_files = show_files;
        self.operation_id = operation_id.map(String::from);

        self.window_id = self
            .config
            .id
            .map_or_else(|| egui::Id::new(self.get_window_title()), |id| id);

        self.load_directory(&self.get_initial_directory());
    }

    /// Shortcut function to open the file dialog to prompt the user to pick a directory.
    /// If used, no files in the directories will be shown to the user.
    /// Use the `open()` method instead, if you still want to display files to the user.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn pick_directory(&mut self) {
        self.open(DialogMode::PickDirectory, false, None);
    }

    /// Shortcut function to open the file dialog to prompt the user to pick a file.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn pick_file(&mut self) {
        self.open(DialogMode::PickFile, true, None);
    }

    /// Shortcut function to open the file dialog to prompt the user to pick multiple
    /// files and folders.
    /// This function resets the file dialog. Configuration variables such as `initial_directory`
    /// are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn pick_multiple(&mut self) {
        self.open(DialogMode::PickMultiple, true, None);
    }

    /// Shortcut function to open the file dialog to prompt the user to save a file.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn save_file(&mut self) {
        self.open(DialogMode::SaveFile, true, None);
    }

    /// The main update method that should be called every frame if the dialog is to be visible.
    ///
    /// This function has no effect if the dialog state is currently not `DialogState::Open`.
    pub fn update(&mut self, ctx: &egui::Context) -> &Self {
        if self.state != DialogState::Open {
            return self;
        }

        self.update_keybindings(ctx);
        self.update_ui(ctx, None);

        self
    }

    /// Sets the width of the right panel.
    pub fn set_right_panel_width(&mut self, width: f32) {
        self.config.right_panel_width = Some(width);
    }

    /// Clears the width of the right panel by setting it to None.
    pub fn clear_right_panel_width(&mut self) {
        self.config.right_panel_width = None;
    }

    /// Do an [update](`Self::update`) with a custom right panel ui.
    ///
    /// Example use cases:
    /// - Show custom information for a file (size, MIME type, etc.)
    /// - Embed a preview, like a thumbnail for an image
    /// - Add controls for custom open options, like open as read-only, etc.
    ///
    /// See [`active_entry`](Self::active_entry) to get the active directory entry
    /// to show the information for.
    ///
    /// This function has no effect if the dialog state is currently not `DialogState::Open`.
    pub fn update_with_right_panel_ui(
        &mut self,
        ctx: &egui::Context,
        f: &mut FileDialogUiCallback,
    ) -> &Self {
        if self.state != DialogState::Open {
            return self;
        }

        self.update_keybindings(ctx);
        self.update_ui(ctx, Some(f));

        self
    }

    // -------------------------------------------------
    // Setter:

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

    /// Sets which directory is loaded when opening the file dialog.
    pub const fn opening_mode(mut self, opening_mode: OpeningMode) -> Self {
        self.config.opening_mode = opening_mode;
        self
    }

    /// If the file dialog window should be displayed as a modal.
    ///
    /// If the window is displayed as modal, the area outside the dialog can no longer be
    /// interacted with and an overlay is displayed.
    pub const fn as_modal(mut self, as_modal: bool) -> Self {
        self.config.as_modal = as_modal;
        self
    }

    /// Sets the color of the overlay when the dialog is displayed as a modal window.
    pub const fn modal_overlay_color(mut self, modal_overlay_color: egui::Color32) -> Self {
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
        self.config.initial_directory = directory;
        self
    }

    /// Sets the default file name when opening the dialog in `DialogMode::SaveFile` mode.
    pub fn default_file_name(mut self, name: &str) -> Self {
        name.clone_into(&mut self.config.default_file_name);
        self
    }

    /// Sets if the user is allowed to select an already existing file when the dialog is in
    /// `DialogMode::SaveFile` mode.
    ///
    /// If this is enabled, the user will receive a modal asking whether the user really
    /// wants to overwrite an existing file.
    pub const fn allow_file_overwrite(mut self, allow_file_overwrite: bool) -> Self {
        self.config.allow_file_overwrite = allow_file_overwrite;
        self
    }

    /// Sets if the path edit is allowed to select the path as the file to save
    /// if it does not have an extension.
    ///
    /// This can lead to confusion if the user wants to open a directory with the path edit,
    /// types it incorrectly and the dialog tries to select the incorrectly typed folder as
    /// the file to be saved.
    ///
    /// This only affects the `DialogMode::SaveFile` mode.
    pub const fn allow_path_edit_to_save_file_without_extension(mut self, allow: bool) -> Self {
        self.config.allow_path_edit_to_save_file_without_extension = allow;
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
    pub const fn canonicalize_paths(mut self, canonicalize: bool) -> Self {
        self.config.canonicalize_paths = canonicalize;
        self
    }

    /// If the directory content should be loaded via a separate thread.
    /// This prevents the application from blocking when loading large directories
    /// or from slow hard drives.
    pub const fn load_via_thread(mut self, load_via_thread: bool) -> Self {
        self.config.load_via_thread = load_via_thread;
        self
    }

    /// Sets if long filenames should be truncated in the middle.
    /// The extension, if available, will be preserved.
    ///
    /// Warning! If this is disabled, the scroll-to-selection might not work correctly and have
    /// an offset for large directories.
    pub const fn truncate_filenames(mut self, truncate_filenames: bool) -> Self {
        self.config.truncate_filenames = truncate_filenames;
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
    /// use egui_file_dialog::FileDialog;
    ///
    /// let config = FileDialog::default()
    ///     .add_save_extension("PNG files", "png")
    ///     .add_save_extension("JPG files", "jpg");
    /// ```
    pub fn add_save_extension(mut self, name: &str, file_extension: &str) -> Self {
        self.config = self.config.add_save_extension(name, file_extension);
        self
    }

    /// Name of the file extension to be selected by default when saving a file.
    ///
    /// No file extension is selected if there is no extension with that name.
    pub fn default_save_extension(mut self, name: &str) -> Self {
        self.config.default_save_extension = Some(name.to_string());
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
    pub const fn resizable(mut self, resizable: bool) -> Self {
        self.config.resizable = resizable;
        self
    }

    /// Sets if the window is movable.
    ///
    /// Has no effect if an anchor is set.
    pub const fn movable(mut self, movable: bool) -> Self {
        self.config.movable = movable;
        self
    }

    /// Sets if the title bar of the window is shown.
    pub const fn title_bar(mut self, title_bar: bool) -> Self {
        self.config.title_bar = title_bar;
        self
    }

    /// Sets if the top panel with the navigation buttons, current path display
    /// and search input should be visible.
    pub const fn show_top_panel(mut self, show_top_panel: bool) -> Self {
        self.config.show_top_panel = show_top_panel;
        self
    }

    /// Sets whether the parent folder button should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub const fn show_parent_button(mut self, show_parent_button: bool) -> Self {
        self.config.show_parent_button = show_parent_button;
        self
    }

    /// Sets whether the back button should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub const fn show_back_button(mut self, show_back_button: bool) -> Self {
        self.config.show_back_button = show_back_button;
        self
    }

    /// Sets whether the forward button should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub const fn show_forward_button(mut self, show_forward_button: bool) -> Self {
        self.config.show_forward_button = show_forward_button;
        self
    }

    /// Sets whether the button to create a new folder should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub const fn show_new_folder_button(mut self, show_new_folder_button: bool) -> Self {
        self.config.show_new_folder_button = show_new_folder_button;
        self
    }

    /// Sets whether the current path should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub const fn show_current_path(mut self, show_current_path: bool) -> Self {
        self.config.show_current_path = show_current_path;
        self
    }

    /// Sets whether the button to text edit the current path should be visible in the top panel.
    ///
    /// has no effect when `FileDialog::show_top_panel` is disabled.
    pub const fn show_path_edit_button(mut self, show_path_edit_button: bool) -> Self {
        self.config.show_path_edit_button = show_path_edit_button;
        self
    }

    /// Sets whether the menu with the reload button and other options should be visible
    /// inside the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub const fn show_menu_button(mut self, show_menu_button: bool) -> Self {
        self.config.show_menu_button = show_menu_button;
        self
    }

    /// Sets whether the reload button inside the top panel menu should be visible.
    ///
    /// Has no effect when `FileDialog::show_top_panel` or
    /// `FileDialog::show_menu_button` is disabled.
    pub const fn show_reload_button(mut self, show_reload_button: bool) -> Self {
        self.config.show_reload_button = show_reload_button;
        self
    }

    /// Sets if the "Open working directory" button should be visible in the hamburger menu.
    /// The working directory button opens to the currently returned working directory
    /// from `std::env::current_dir()`.
    ///
    /// Has no effect when `FileDialog::show_top_panel` or
    /// `FileDialog::show_menu_button` is disabled.
    pub const fn show_working_directory_button(
        mut self,
        show_working_directory_button: bool,
    ) -> Self {
        self.config.show_working_directory_button = show_working_directory_button;
        self
    }

    /// Sets whether the show hidden files and folders option inside the top panel
    /// menu should be visible.
    ///
    /// Has no effect when `FileDialog::show_top_panel` or
    /// `FileDialog::show_menu_button` is disabled.
    pub const fn show_hidden_option(mut self, show_hidden_option: bool) -> Self {
        self.config.show_hidden_option = show_hidden_option;
        self
    }

    /// Sets whether the show system files option inside the top panel
    /// menu should be visible.
    ///
    /// Has no effect when `FileDialog::show_top_panel` or
    /// `FileDialog::show_menu_button` is disabled.
    pub const fn show_system_files_option(mut self, show_system_files_option: bool) -> Self {
        self.config.show_system_files_option = show_system_files_option;
        self
    }

    /// Sets whether the search input should be visible in the top panel.
    ///
    /// Has no effect when `FileDialog::show_top_panel` is disabled.
    pub const fn show_search(mut self, show_search: bool) -> Self {
        self.config.show_search = show_search;
        self
    }

    /// Sets if the sidebar with the shortcut directories such as
    /// “Home”, “Documents” etc. should be visible.
    pub const fn show_left_panel(mut self, show_left_panel: bool) -> Self {
        self.config.show_left_panel = show_left_panel;
        self
    }

    /// Sets if pinned folders should be listed in the left sidebar.
    /// Disabling this will also disable the functionality to pin a folder.
    pub const fn show_pinned_folders(mut self, show_pinned_folders: bool) -> Self {
        self.config.show_pinned_folders = show_pinned_folders;
        self
    }

    /// Sets if the "Places" section should be visible in the left sidebar.
    /// The Places section contains the user directories such as Home or Documents.
    ///
    /// Has no effect when `FileDialog::show_left_panel` is disabled.
    pub const fn show_places(mut self, show_places: bool) -> Self {
        self.config.show_places = show_places;
        self
    }

    /// Sets if the "Devices" section should be visible in the left sidebar.
    /// The Devices section contains the non removable system disks.
    ///
    /// Has no effect when `FileDialog::show_left_panel` is disabled.
    pub const fn show_devices(mut self, show_devices: bool) -> Self {
        self.config.show_devices = show_devices;
        self
    }

    /// Sets if the "Removable Devices" section should be visible in the left sidebar.
    /// The Removable Devices section contains the removable disks like USB disks.
    ///
    /// Has no effect when `FileDialog::show_left_panel` is disabled.
    pub const fn show_removable_devices(mut self, show_removable_devices: bool) -> Self {
        self.config.show_removable_devices = show_removable_devices;
        self
    }

    // -------------------------------------------------
    // Getter:

    /// Returns the directory or file that the user picked, or the target file
    /// if the dialog is in `DialogMode::SaveFile` mode.
    ///
    /// None is returned when the user has not yet selected an item.
    pub fn picked(&self) -> Option<&Path> {
        match &self.state {
            DialogState::Picked(path) => Some(path),
            _ => None,
        }
    }

    /// Returns the directory or file that the user picked, or the target file
    /// if the dialog is in `DialogMode::SaveFile` mode.
    /// Unlike `FileDialog::picked`, this method returns the picked path only once and
    /// sets the dialog's state to `DialogState::Closed`.
    ///
    /// None is returned when the user has not yet picked an item.
    pub fn take_picked(&mut self) -> Option<PathBuf> {
        match &mut self.state {
            DialogState::Picked(path) => {
                let path = std::mem::take(path);
                self.state = DialogState::Closed;
                Some(path)
            }
            _ => None,
        }
    }

    /// Returns a list of the files and folders the user picked, when the dialog is in
    /// `DialogMode::PickMultiple` mode.
    ///
    /// None is returned when the user has not yet picked an item.
    pub fn picked_multiple(&self) -> Option<Vec<&Path>> {
        match &self.state {
            DialogState::PickedMultiple(items) => {
                Some(items.iter().map(std::path::PathBuf::as_path).collect())
            }
            _ => None,
        }
    }

    /// Returns a list of the files and folders the user picked, when the dialog is in
    /// `DialogMode::PickMultiple` mode.
    /// Unlike `FileDialog::picked_multiple`, this method returns the picked paths only once
    /// and sets the dialog's state to `DialogState::Closed`.
    ///
    /// None is returned when the user has not yet picked an item.
    pub fn take_picked_multiple(&mut self) -> Option<Vec<PathBuf>> {
        match &mut self.state {
            DialogState::PickedMultiple(items) => {
                let items = std::mem::take(items);
                self.state = DialogState::Closed;
                Some(items)
            }
            _ => None,
        }
    }

    /// Returns the currently active directory entry.
    ///
    /// This is either the currently highlighted entry, or the currently active directory
    /// if nothing is being highlighted.
    ///
    /// For the [`DialogMode::SelectMultiple`] counterpart,
    /// see [`FileDialog::active_selected_entries`].
    pub const fn selected_entry(&self) -> Option<&DirectoryEntry> {
        self.selected_item.as_ref()
    }

    /// Returns an iterator over the currently selected entries in [`SelectMultiple`] mode.
    ///
    /// For the counterpart in single selection modes, see [`FileDialog::active_entry`].
    ///
    /// [`SelectMultiple`]: DialogMode::SelectMultiple
    pub fn selected_entries(&self) -> impl Iterator<Item = &DirectoryEntry> {
        self.get_dir_content_filtered_iter().filter(|p| p.selected)
    }

    /// Returns the ID of the operation for which the dialog is currently being used.
    ///
    /// See `FileDialog::open` for more information.
    pub fn operation_id(&self) -> Option<&str> {
        self.operation_id.as_deref()
    }

    /// Returns the mode the dialog is currently in.
    pub const fn mode(&self) -> DialogMode {
        self.mode
    }

    /// Returns the state the dialog is currently in.
    pub fn state(&self) -> DialogState {
        self.state.clone()
    }

    /// Get the window Id
    pub const fn get_window_id(&self) -> egui::Id {
        self.window_id
    }
}

/// UI methods
impl FileDialog {
    /// Main update method of the UI
    ///
    /// Takes an optional callback to show a custom right panel.
    fn update_ui(
        &mut self,
        ctx: &egui::Context,
        right_panel_fn: Option<&mut FileDialogUiCallback>,
    ) {
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
                egui::TopBottomPanel::top(self.window_id.with("top_panel"))
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        self.ui_update_top_panel(ui);
                    });
            }

            if self.config.show_left_panel {
                egui::SidePanel::left(self.window_id.with("left_panel"))
                    .resizable(true)
                    .default_width(150.0)
                    .width_range(90.0..=250.0)
                    .show_inside(ui, |ui| {
                        self.ui_update_left_panel(ui);
                    });
            }

            // Optionally, show a custom right panel (see `update_with_custom_right_panel`)
            if let Some(f) = right_panel_fn {
                let mut right_panel = egui::SidePanel::right(self.window_id.with("right_panel"))
                    // Unlike the left panel, we have no control over the contents, so
                    // we don't restrict the width. It's up to the user to make the UI presentable.
                    .resizable(true);
                if let Some(width) = self.config.right_panel_width {
                    right_panel = right_panel.default_width(width);
                }
                right_panel.show_inside(ui, |ui| {
                    f(ui, self);
                });
            }

            egui::TopBottomPanel::bottom(self.window_id.with("bottom_panel"))
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

        self.any_focused_last_frame = ctx.memory(egui::Memory::focused).is_some();

        // User closed the window without finishing the dialog
        if !is_open {
            self.cancel();
        }

        let mut repaint = false;

        // Collect dropped files:
        ctx.input(|i| {
            // Check if files were dropped
            if let Some(dropped_file) = i.raw.dropped_files.last() {
                if let Some(path) = &dropped_file.path {
                    if self.config.file_system.is_dir(path) {
                        // If we dropped a directory, go there
                        self.load_directory(path.as_path());
                        repaint = true;
                    } else if let Some(parent) = path.parent() {
                        // Else, go to the parent directory
                        self.load_directory(parent);
                        self.select_item(&mut DirectoryEntry::from_path(
                            &self.config,
                            path,
                            &*self.config.file_system,
                        ));
                        self.scroll_to_selection = true;
                        repaint = true;
                    }
                }
            }
        });

        // Update GUI if we dropped a file
        if repaint {
            ctx.request_repaint();
        }
    }

    /// Updates the main modal background of the file dialog window.
    fn ui_update_modal_background(&self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::Area::new(self.window_id.with("modal_overlay"))
            .interactable(true)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                let screen_rect = ctx.input(|i| i.screen_rect);

                ui.allocate_response(screen_rect.size(), egui::Sense::click());

                ui.painter().rect_filled(
                    screen_rect,
                    egui::CornerRadius::ZERO,
                    self.config.modal_overlay_color,
                );
            })
    }

    fn ui_update_modals(&mut self, ui: &mut egui::Ui) {
        // Currently, a rendering error occurs when only a single central panel is rendered
        // inside a window. Therefore, when rendering a modal, we render an invisible bottom panel,
        // which prevents the error.
        // This is currently a bit hacky and should be adjusted again in the future.
        egui::TopBottomPanel::bottom(self.window_id.with("modal_bottom_panel"))
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
                    ModalState::Pending => {}
                }
            }
        });
    }

    /// Creates a new egui window with the configured options.
    fn create_window<'a>(&self, is_open: &'a mut bool) -> egui::Window<'a> {
        let mut window = egui::Window::new(self.get_window_title())
            .id(self.window_id)
            .open(is_open)
            .default_size(self.config.default_size)
            .min_size(self.config.min_size)
            .resizable(self.config.resizable)
            .movable(self.config.movable)
            .title_bar(self.config.title_bar)
            .collapsible(false);

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

    /// Gets the window title to use.
    /// This is either one of the default window titles or the configured window title.
    const fn get_window_title(&self) -> &String {
        match &self.config.title {
            Some(title) => title,
            None => match &self.mode {
                DialogMode::PickDirectory => &self.config.labels.title_select_directory,
                DialogMode::PickFile => &self.config.labels.title_select_file,
                DialogMode::PickMultiple => &self.config.labels.title_select_multiple,
                DialogMode::SaveFile => &self.config.labels.title_save_file,
            },
        }
    }

    /// Updates the top panel of the dialog. Including the navigation buttons,
    /// the current path display, the reload button and the search field.
    fn ui_update_top_panel(&mut self, ui: &mut egui::Ui) {
        const BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(25.0, 25.0);

        ui.horizontal(|ui| {
            self.ui_update_nav_buttons(ui, BUTTON_SIZE);

            let mut path_display_width = ui.available_width();

            // Leave some area for the menu button and search input
            if self.config.show_reload_button {
                path_display_width -= ui
                    .style()
                    .spacing
                    .item_spacing
                    .x
                    .mul_add(2.5, BUTTON_SIZE.x);
            }

            if self.config.show_search {
                path_display_width -= 140.0;
            }

            if self.config.show_current_path {
                self.ui_update_current_path(ui, path_display_width);
            }

            // Hamburger menu containing different options
            if self.config.show_menu_button
                && (self.config.show_reload_button
                    || self.config.show_working_directory_button
                    || self.config.show_hidden_option
                    || self.config.show_system_files_option)
            {
                ui.allocate_ui_with_layout(
                    BUTTON_SIZE,
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.menu_button("☰", |ui| {
                            self.ui_update_hamburger_menu(ui);
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
    fn ui_update_nav_buttons(&mut self, ui: &mut egui::Ui, button_size: egui::Vec2) {
        if self.config.show_parent_button {
            if let Some(x) = self.current_directory() {
                if self.ui_button_sized(ui, x.parent().is_some(), button_size, "⏶", None) {
                    self.load_parent_directory();
                }
            } else {
                let _ = self.ui_button_sized(ui, false, button_size, "⏶", None);
            }
        }

        if self.config.show_back_button
            && self.ui_button_sized(
                ui,
                self.directory_offset + 1 < self.directory_stack.len(),
                button_size,
                "⏴",
                None,
            )
        {
            self.load_previous_directory();
        }

        if self.config.show_forward_button
            && self.ui_button_sized(ui, self.directory_offset != 0, button_size, "⏵", None)
        {
            self.load_next_directory();
        }

        if self.config.show_new_folder_button
            && self.ui_button_sized(
                ui,
                !self.create_directory_dialog.is_open(),
                button_size,
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
            .inner_margin(egui::Margin::from(4))
            .corner_radius(egui::CornerRadius::from(4))
            .show(ui, |ui| {
                const EDIT_BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(22.0, 20.0);

                if self.path_edit_visible {
                    self.ui_update_path_edit(ui, width, EDIT_BUTTON_SIZE);
                } else {
                    self.ui_update_path_display(ui, width, EDIT_BUTTON_SIZE);
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

        let max_width = if self.config.show_path_edit_button {
            ui.style()
                .spacing
                .item_spacing
                .x
                .mul_add(-2.0, width - edit_button_size.x)
        } else {
            width
        };

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .stick_to_right(true)
            .max_width(max_width)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing.x /= 2.5;
                    ui.style_mut().spacing.button_padding = egui::Vec2::new(5.0, 3.0);

                    let mut path = PathBuf::new();

                    if let Some(data) = self.current_directory().map(Path::to_path_buf) {
                        for (i, segment) in data.iter().enumerate() {
                            path.push(segment);

                            let mut segment_str = segment.to_str().unwrap_or_default().to_string();

                            if self.is_pinned(&path) {
                                segment_str =
                                    format!("{} {}", &self.config.pinned_icon, segment_str);
                            };

                            if i != 0 {
                                ui.label(self.config.directory_separator.as_str());
                            }

                            let re = ui.button(segment_str);

                            if re.clicked() {
                                self.load_directory(path.as_path());
                                return;
                            }

                            self.ui_update_path_context_menu(&re, &path.clone());
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
        let desired_width: f32 = ui
            .style()
            .spacing
            .item_spacing
            .x
            .mul_add(-3.0, width - edit_button_size.x);

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

    /// Updates the hamburger menu containing different options.
    fn ui_update_hamburger_menu(&mut self, ui: &mut egui::Ui) {
        const SEPARATOR_SPACING: f32 = 2.0;

        if self.config.show_reload_button && ui.button(&self.config.labels.reload).clicked() {
            self.refresh();
            ui.close_menu();
        }

        let working_dir = self.config.file_system.current_dir();

        if self.config.show_working_directory_button
            && working_dir.is_ok()
            && ui.button(&self.config.labels.working_directory).clicked()
        {
            self.load_directory(&working_dir.unwrap_or_default());
            ui.close_menu();
        }

        if (self.config.show_reload_button || self.config.show_working_directory_button)
            && (self.config.show_hidden_option || self.config.show_system_files_option)
        {
            ui.add_space(SEPARATOR_SPACING);
            ui.separator();
            ui.add_space(SEPARATOR_SPACING);
        }

        if self.config.show_hidden_option
            && ui
                .checkbox(
                    &mut self.config.storage.show_hidden,
                    &self.config.labels.show_hidden,
                )
                .clicked()
        {
            self.refresh();
            ui.close_menu();
        }

        if self.config.show_system_files_option
            && ui
                .checkbox(
                    &mut self.config.storage.show_system_files,
                    &self.config.labels.show_system_files,
                )
                .clicked()
        {
            self.refresh();
            ui.close_menu();
        }
    }

    /// Updates the search input
    fn ui_update_search(&mut self, ui: &mut egui::Ui) {
        egui::Frame::default()
            .stroke(egui::Stroke::new(
                1.0,
                ui.ctx().style().visuals.window_stroke.color,
            ))
            .inner_margin(egui::Margin::symmetric(4, 4))
            .corner_radius(egui::CornerRadius::from(4))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    ui.add_space(ui.ctx().style().spacing.item_spacing.y);

                    ui.label(egui::RichText::from("🔍").size(15.0));

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
    fn edit_search_on_text_input(&mut self, ui: &egui::Ui) {
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
            // Spacing multiplier used between sections in the left sidebar
            const SPACING_MULTIPLIER: f32 = 4.0;

            egui::containers::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Spacing for the first section in the left sidebar
                    let mut spacing = ui.ctx().style().spacing.item_spacing.y * 2.0;

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
            self.load_directory(path);
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

            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            let response = self.ui_update_left_panel_entry(
                ui,
                &format!("{}  {}", self.config.pinned_icon, file_name),
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
        let labels = std::mem::take(&mut self.config.labels);

        let mut visible = false;

        if let Some(dirs) = &user_directories {
            ui.add_space(spacing);
            ui.label(labels.heading_places.as_str());

            if let Some(path) = dirs.home_dir() {
                self.ui_update_left_panel_entry(ui, &labels.home_dir, path);
            }
            if let Some(path) = dirs.desktop_dir() {
                self.ui_update_left_panel_entry(ui, &labels.desktop_dir, path);
            }
            if let Some(path) = dirs.document_dir() {
                self.ui_update_left_panel_entry(ui, &labels.documents_dir, path);
            }
            if let Some(path) = dirs.download_dir() {
                self.ui_update_left_panel_entry(ui, &labels.downloads_dir, path);
            }
            if let Some(path) = dirs.audio_dir() {
                self.ui_update_left_panel_entry(ui, &labels.audio_dir, path);
            }
            if let Some(path) = dirs.picture_dir() {
                self.ui_update_left_panel_entry(ui, &labels.pictures_dir, path);
            }
            if let Some(path) = dirs.video_dir() {
                self.ui_update_left_panel_entry(ui, &labels.videos_dir, path);
            }

            visible = true;
        }

        self.user_directories = user_directories;
        self.config.labels = labels;

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
        let label = if device.is_removable() {
            format!(
                "{}  {}",
                self.config.removable_device_icon,
                device.display_name()
            )
        } else {
            format!("{}  {}", self.config.device_icon, device.display_name())
        };

        self.ui_update_left_panel_entry(ui, &label, device.mount_point());
    }

    /// Updates the bottom panel showing the selected item and main action buttons.
    fn ui_update_bottom_panel(&mut self, ui: &mut egui::Ui) {
        const BUTTON_HEIGHT: f32 = 20.0;
        ui.add_space(5.0);

        // Calculate the width of the action buttons
        let label_submit_width = match self.mode {
            DialogMode::PickDirectory | DialogMode::PickFile | DialogMode::PickMultiple => {
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

        if self.mode == DialogMode::SaveFile && self.config.save_extensions.is_empty() {
            ui.add_space(ui.style().spacing.item_spacing.y);
        }

        self.ui_update_action_buttons(ui, button_size);
    }

    /// Updates the selection preview like "Selected directory: X"
    fn ui_update_selection_preview(&mut self, ui: &mut egui::Ui, button_size: egui::Vec2) {
        const SELECTION_PREVIEW_MIN_WIDTH: f32 = 50.0;
        let item_spacing = ui.style().spacing.item_spacing;

        let render_filter_selection = (!self.config.file_filters.is_empty()
            && (self.mode == DialogMode::PickFile || self.mode == DialogMode::PickMultiple))
            || (!self.config.save_extensions.is_empty() && self.mode == DialogMode::SaveFile);

        let filter_selection_width = button_size.x.mul_add(2.0, item_spacing.x);
        let mut filter_selection_separate_line = false;

        ui.horizontal(|ui| {
            match &self.mode {
                DialogMode::PickDirectory => ui.label(&self.config.labels.selected_directory),
                DialogMode::PickFile => ui.label(&self.config.labels.selected_file),
                DialogMode::PickMultiple => ui.label(&self.config.labels.selected_items),
                DialogMode::SaveFile => ui.label(&self.config.labels.file_name),
            };

            // Make sure there is enough width for the selection preview. If the available
            // width is not enough, render the drop-down menu to select a file filter or
            // save extension on a separate line and give the selection preview
            // the entire available width.
            let mut scroll_bar_width: f32 =
                ui.available_width() - filter_selection_width - item_spacing.x;

            if scroll_bar_width < SELECTION_PREVIEW_MIN_WIDTH || !render_filter_selection {
                filter_selection_separate_line = true;
                scroll_bar_width = ui.available_width();
            }

            match &self.mode {
                DialogMode::PickDirectory | DialogMode::PickFile | DialogMode::PickMultiple => {
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
                    let mut output = egui::TextEdit::singleline(&mut self.file_name_input)
                        .cursor_at_end(false)
                        .margin(egui::Margin::symmetric(4, 3))
                        .desired_width(scroll_bar_width - item_spacing.x)
                        .show(ui);

                    if self.file_name_input_request_focus {
                        self.highlight_file_name_input(&mut output);
                        output.state.store(ui.ctx(), output.response.id);

                        output.response.request_focus();
                        self.file_name_input_request_focus = false;
                    }

                    if output.response.changed() {
                        self.file_name_input_error = self.validate_file_name_input();
                    }

                    if output.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.submit();
                    }
                }
            };

            if !filter_selection_separate_line && render_filter_selection {
                if self.mode == DialogMode::SaveFile {
                    self.ui_update_save_extension_selection(ui, filter_selection_width);
                } else {
                    self.ui_update_file_filter_selection(ui, filter_selection_width);
                }
            }
        });

        if filter_selection_separate_line && render_filter_selection {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if self.mode == DialogMode::SaveFile {
                    self.ui_update_save_extension_selection(ui, filter_selection_width);
                } else {
                    self.ui_update_file_filter_selection(ui, filter_selection_width);
                }
            });
        }
    }

    /// Highlights the characters inside the file name input until the file extension.
    /// Do not forget to store these changes after calling this function:
    /// `output.state.store(ui.ctx(), output.response.id);`
    fn highlight_file_name_input(&self, output: &mut egui::text_edit::TextEditOutput) {
        if let Some(pos) = self.file_name_input.rfind('.') {
            let range = if pos == 0 {
                CCursorRange::two(CCursor::new(0), CCursor::new(0))
            } else {
                CCursorRange::two(CCursor::new(0), CCursor::new(pos))
            };

            output.state.cursor.set_char_range(Some(range));
        }
    }

    fn get_selection_preview_text(&self) -> String {
        if self.is_selection_valid() {
            match &self.mode {
                DialogMode::PickDirectory | DialogMode::PickFile => self
                    .selected_item
                    .as_ref()
                    .map_or_else(String::new, |item| item.file_name().to_string()),
                DialogMode::PickMultiple => {
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
                DialogMode::SaveFile => String::new(),
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
        // If none, the user did not change the selected item this frame.
        let mut select_filter: Option<Option<FileFilter>> = None;

        egui::containers::ComboBox::from_id_salt(self.window_id.with("file_filter_selection"))
            .width(width)
            .selected_text(selected_text)
            .wrap_mode(egui::TextWrapMode::Truncate)
            .show_ui(ui, |ui| {
                for filter in &self.config.file_filters {
                    let selected = selected_filter.is_some_and(|f| f.id == filter.id);

                    if ui.selectable_label(selected, &filter.name).clicked() {
                        select_filter = Some(Some(filter.clone()));
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
            self.select_file_filter(i);
        }
    }

    fn ui_update_save_extension_selection(&mut self, ui: &mut egui::Ui, width: f32) {
        let selected_extension = self.get_selected_save_extension();
        let selected_text = match selected_extension {
            Some(e) => &e.to_string(),
            None => &self.config.labels.save_extension_any,
        };

        // The item that the user selected inside the drop down.
        // If none, the user did not change the selected item this frame.
        let mut select_extension: Option<Option<SaveExtension>> = None;

        egui::containers::ComboBox::from_id_salt(self.window_id.with("save_extension_selection"))
            .width(width)
            .selected_text(selected_text)
            .wrap_mode(egui::TextWrapMode::Truncate)
            .show_ui(ui, |ui| {
                for extension in &self.config.save_extensions {
                    let selected = selected_extension.is_some_and(|s| s.id == extension.id);

                    if ui
                        .selectable_label(selected, extension.to_string())
                        .clicked()
                    {
                        select_extension = Some(Some(extension.clone()));
                    }
                }
            });

        if let Some(i) = select_extension {
            self.file_name_input_request_focus = true;
            self.select_save_extension(i);
        }
    }

    /// Updates the action buttons like save, open and cancel
    fn ui_update_action_buttons(&mut self, ui: &mut egui::Ui, button_size: egui::Vec2) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let label = match &self.mode {
                DialogMode::PickDirectory | DialogMode::PickFile | DialogMode::PickMultiple => {
                    self.config.labels.open_button.as_str()
                }
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

    /// Updates the central panel. This is either the contents of the directory
    /// or the error message when there was an error loading the current directory.
    fn ui_update_central_panel(&mut self, ui: &mut egui::Ui) {
        if self.update_directory_content(ui) {
            return;
        }

        self.ui_update_central_panel_content(ui);
    }

    /// Updates the directory content (Not the UI!).
    /// This is required because the contents of the directory might be loaded on a
    /// separate thread. This function checks the status of the directory content
    /// and updates the UI accordingly.
    fn update_directory_content(&mut self, ui: &mut egui::Ui) -> bool {
        const SHOW_SPINNER_AFTER: f32 = 0.2;

        match self.directory_content.update() {
            DirectoryContentState::Pending(timestamp) => {
                let now = std::time::SystemTime::now();

                if now
                    .duration_since(*timestamp)
                    .unwrap_or_default()
                    .as_secs_f32()
                    > SHOW_SPINNER_AFTER
                {
                    ui.centered_and_justified(egui::Ui::spinner);
                }

                // Prevent egui from not updating the UI when there is no user input
                ui.ctx().request_repaint();

                true
            }
            DirectoryContentState::Errored(err) => {
                ui.centered_and_justified(|ui| ui.colored_label(ui.visuals().error_fg_color, err));
                true
            }
            DirectoryContentState::Finished => {
                if self.mode == DialogMode::PickDirectory {
                    if let Some(dir) = self.current_directory() {
                        let mut dir_entry =
                            DirectoryEntry::from_path(&self.config, dir, &*self.config.file_system);
                        self.select_item(&mut dir_entry);
                    }
                }

                false
            }
            DirectoryContentState::Success => false,
        }
    }

    /// Updates the contents of the currently open directory.
    /// TODO: Refactor
    fn ui_update_central_panel_content(&mut self, ui: &mut egui::Ui) {
        // Temporarily take ownership of the directory content.
        let mut data = std::mem::take(&mut self.directory_content);

        // If the multi selection should be reset, excluding the currently
        // selected primary item.
        let mut reset_multi_selection = false;

        // The item the user wants to make a batch selection from.
        // The primary selected item is used for item a.
        let mut batch_select_item_b: Option<DirectoryEntry> = None;

        // If we should return after updating the directory entries.
        let mut should_return = false;

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            let scroll_area = egui::containers::ScrollArea::vertical().auto_shrink([false, false]);

            if self.search_value.is_empty()
                && !self.create_directory_dialog.is_open()
                && !self.scroll_to_selection
            {
                // Only update visible items when the search value is empty,
                // the create directory dialog is closed and we are currently not scrolling
                // to the current item.
                scroll_area.show_rows(ui, ui.spacing().interact_size.y, data.len(), |ui, range| {
                    for item in data.iter_range_mut(range) {
                        if self.ui_update_central_panel_entry(
                            ui,
                            item,
                            &mut reset_multi_selection,
                            &mut batch_select_item_b,
                        ) {
                            should_return = true;
                        }
                    }
                });
            } else {
                // Update each element if the search value is not empty as we apply the
                // search value in every frame. We can't use `egui::ScrollArea::show_rows`
                // because we don't know how many files the search value applies to.
                // We also have to update every item when the create directory dialog is open as
                // it's displayed as the last element.
                scroll_area.show(ui, |ui| {
                    for item in data.filtered_iter_mut(&self.search_value.clone()) {
                        if self.ui_update_central_panel_entry(
                            ui,
                            item,
                            &mut reset_multi_selection,
                            &mut batch_select_item_b,
                        ) {
                            should_return = true;
                        }
                    }

                    if let Some(entry) = self.ui_update_create_directory_dialog(ui) {
                        data.push(entry);
                    }
                });
            }
        });

        if should_return {
            return;
        }

        // Reset the multi selection except the currently selected primary item
        if reset_multi_selection {
            for item in data.filtered_iter_mut(&self.search_value) {
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
    }

    /// Updates a single directory content entry.
    /// TODO: Refactor
    fn ui_update_central_panel_entry(
        &mut self,
        ui: &mut egui::Ui,
        item: &mut DirectoryEntry,
        reset_multi_selection: &mut bool,
        batch_select_item_b: &mut Option<DirectoryEntry>,
    ) -> bool {
        let file_name = item.file_name();
        let primary_selected = self.is_primary_selected(item);
        let pinned = self.is_pinned(item.as_path());

        let icons = if pinned {
            format!("{} {} ", item.icon(), self.config.pinned_icon)
        } else {
            format!("{} ", item.icon())
        };

        let icons_width = Self::calc_text_width(ui, &icons);

        // Calc available width for the file name and include a small margin
        let available_width = ui.available_width() - icons_width - 15.0;

        let truncate = self.config.truncate_filenames
            && available_width < Self::calc_text_width(ui, file_name);

        let text = if truncate {
            Self::truncate_filename(ui, item, available_width)
        } else {
            file_name.to_owned()
        };

        let mut re =
            ui.selectable_label(primary_selected || item.selected, format!("{icons}{text}"));

        if truncate {
            re = re.on_hover_text(file_name);
        }

        if item.is_dir() {
            self.ui_update_path_context_menu(&re, item.as_path());

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
            && !ui.input(|i| i.modifiers.command)
            && !ui.input(|i| i.modifiers.shift_only())
        {
            self.select_item(item);

            // Reset the multi selection except the now primary selected item
            if self.mode == DialogMode::PickMultiple {
                *reset_multi_selection = true;
            }
        }

        // The user wants to select or unselect the item as part of a
        // multi selection
        if self.mode == DialogMode::PickMultiple
            && re.clicked()
            && ui.input(|i| i.modifiers.command)
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
        if self.mode == DialogMode::PickMultiple
            && re.clicked()
            && ui.input(|i| i.modifiers.shift_only())
        {
            if let Some(selected_item) = self.selected_item.clone() {
                // We perform a batch selection from the item that was
                // primarily selected before the user clicked on this item.
                *batch_select_item_b = Some(selected_item);

                // And now make this item the primary selected item
                item.selected = true;
                self.select_item(item);
            }
        }

        // The user double clicked on the directory entry.
        // Either open the directory or submit the dialog.
        if re.double_clicked() && !ui.input(|i| i.modifiers.command) {
            if item.is_dir() {
                self.load_directory(&item.to_path_buf());
                return true;
            }

            self.select_item(item);

            self.submit();
        }

        false
    }

    fn ui_update_create_directory_dialog(&mut self, ui: &mut egui::Ui) -> Option<DirectoryEntry> {
        self.create_directory_dialog
            .update(ui, &self.config)
            .directory()
            .map(|path| self.process_new_folder(&path))
    }

    /// Selects every item inside the `directory_content` between `item_a` and `item_b`,
    /// excluding both given items.
    fn batch_select_between(
        &self,
        directory_content: &mut DirectoryContent,
        item_a: &DirectoryEntry,
        item_b: &DirectoryEntry,
    ) {
        // Get the position of item a and item b
        let pos_a = directory_content
            .filtered_iter(&self.search_value)
            .position(|p| p.path_eq(item_a));
        let pos_b = directory_content
            .filtered_iter(&self.search_value)
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
                    .filtered_iter_mut(&self.search_value)
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
    fn ui_update_path_context_menu(&mut self, item_response: &egui::Response, path: &Path) {
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
                self.pin_path(path.to_path_buf());
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

    /// Calculates the width of a single char.
    fn calc_char_width(ui: &egui::Ui, char: char) -> f32 {
        ui.fonts(|f| f.glyph_width(&egui::TextStyle::Body.resolve(ui.style()), char))
    }

    /// Calculates the width of the specified text using the current font configuration.
    /// Does not take new lines or text breaks into account!
    fn calc_text_width(ui: &egui::Ui, text: &str) -> f32 {
        let mut width = 0.0;

        for char in text.chars() {
            width += Self::calc_char_width(ui, char);
        }

        width
    }

    fn truncate_filename(ui: &egui::Ui, item: &DirectoryEntry, max_length: f32) -> String {
        const TRUNCATE_STR: &str = "...";

        let path = item.as_path();

        let file_stem = if item.is_file() {
            path.file_stem().and_then(|f| f.to_str()).unwrap_or("")
        } else {
            item.file_name()
        };

        let extension = if item.is_file() {
            path.extension().map_or(String::new(), |ext| {
                format!(".{}", ext.to_str().unwrap_or(""))
            })
        } else {
            String::new()
        };

        let extension_width = Self::calc_text_width(ui, &extension);
        let reserved = extension_width + Self::calc_text_width(ui, TRUNCATE_STR);

        if max_length <= reserved {
            return format!("{TRUNCATE_STR}{extension}");
        }

        let mut width = reserved;
        let mut front = String::new();
        let mut back = String::new();

        for (i, char) in file_stem.chars().enumerate() {
            let w = Self::calc_char_width(ui, char);

            if width + w > max_length {
                break;
            }

            front.push(char);
            width += w;

            let back_index = file_stem.len() - i - 1;

            if back_index <= i {
                break;
            }

            if let Some(char) = file_stem.chars().nth(back_index) {
                let w = Self::calc_char_width(ui, char);

                if width + w > max_length {
                    break;
                }

                back.push(char);
                width += w;
            }
        }

        format!(
            "{front}{TRUNCATE_STR}{}{extension}",
            back.chars().rev().collect::<String>()
        )
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
            self.load_parent_directory();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.back, true) {
            self.load_previous_directory();
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.forward, true) {
            self.load_next_directory();
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
                    self.load_directory(home.to_path_buf().as_path());
                    self.open_path_edit();
                }
            }
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.selection_up, false) {
            self.exec_keybinding_selection_up();

            // We want to break out of input fields like search when pressing selection keys
            if let Some(id) = ctx.memory(egui::Memory::focused) {
                ctx.memory_mut(|w| w.surrender_focus(id));
            }
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.selection_down, false) {
            self.exec_keybinding_selection_down();

            // We want to break out of input fields like search when pressing selection keys
            if let Some(id) = ctx.memory(egui::Memory::focused) {
                ctx.memory_mut(|w| w.surrender_focus(id));
            }
        }

        if FileDialogKeyBindings::any_pressed(ctx, &keybindings.select_all, true)
            && self.mode == DialogMode::PickMultiple
        {
            for item in self.directory_content.filtered_iter_mut(&self.search_value) {
                item.selected = true;
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
                self.load_directory(&item.to_path_buf());
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
            self.close_path_edit();
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
        self.selected_file_filter
            .and_then(|id| self.config.file_filters.iter().find(|p| p.id == id))
    }

    /// Sets the default file filter to use.
    fn set_default_file_filter(&mut self) {
        if let Some(name) = &self.config.default_file_filter {
            for filter in &self.config.file_filters {
                if filter.name == name.as_str() {
                    self.selected_file_filter = Some(filter.id);
                }
            }
        }
    }

    /// Selects the given file filter and applies the appropriate filters.
    fn select_file_filter(&mut self, filter: Option<FileFilter>) {
        self.selected_file_filter = filter.map(|f| f.id);
        self.selected_item = None;
        self.refresh();
    }

    /// Get the save extension the user currently selected.
    fn get_selected_save_extension(&self) -> Option<&SaveExtension> {
        self.selected_save_extension
            .and_then(|id| self.config.save_extensions.iter().find(|p| p.id == id))
    }

    /// Sets the save extension to use.
    fn set_default_save_extension(&mut self) {
        let config = std::mem::take(&mut self.config);

        if let Some(name) = &config.default_save_extension {
            for extension in &config.save_extensions {
                if extension.name == name.as_str() {
                    self.selected_save_extension = Some(extension.id);
                    self.set_file_name_extension(&extension.file_extension);
                }
            }
        }

        self.config = config;
    }

    /// Selects the given save extension.
    fn select_save_extension(&mut self, extension: Option<SaveExtension>) {
        if let Some(ex) = extension {
            self.selected_save_extension = Some(ex.id);
            self.set_file_name_extension(&ex.file_extension);
        }

        self.selected_item = None;
        self.refresh();
    }

    /// Updates the extension of `Self::file_name_input`.
    fn set_file_name_extension(&mut self, extension: &str) {
        // Prevent `PathBuf::set_extension` to append the file extension when there is
        // already one without a file name. For example `.png` would be changed to `.png.txt`
        // when using `PathBuf::set_extension`.
        let dot_count = self.file_name_input.chars().filter(|c| *c == '.').count();
        let use_simple = dot_count == 1 && self.file_name_input.chars().nth(0) == Some('.');

        let mut p = PathBuf::from(&self.file_name_input);
        if !use_simple && p.set_extension(extension) {
            self.file_name_input = p.to_string_lossy().into_owned();
        } else {
            self.file_name_input = format!(".{extension}");
        }
    }

    /// Gets a filtered iterator of the directory content of this object.
    fn get_dir_content_filtered_iter(&self) -> impl Iterator<Item = &DirectoryEntry> {
        self.directory_content.filtered_iter(&self.search_value)
    }

    /// Opens the dialog to create a new folder.
    fn open_new_folder_dialog(&mut self) {
        if let Some(x) = self.current_directory() {
            self.create_directory_dialog.open(x.to_path_buf());
        }
    }

    /// Function that processes a newly created folder.
    fn process_new_folder(&mut self, created_dir: &Path) -> DirectoryEntry {
        let mut entry =
            DirectoryEntry::from_path(&self.config, created_dir, &*self.config.file_system);

        self.directory_content.push(entry.clone());

        self.select_item(&mut entry);

        entry
    }

    /// Opens a new modal window.
    fn open_modal(&mut self, modal: Box<dyn FileDialogModal + Send + Sync>) {
        self.modals.push(modal);
    }

    /// Executes the given modal action.
    fn exec_modal_action(&mut self, action: ModalAction) {
        match action {
            ModalAction::None => {}
            ModalAction::SaveFile(path) => self.state = DialogState::Picked(path),
        };
    }

    /// Canonicalizes the specified path if canonicalization is enabled.
    /// Returns the input path if an error occurs or canonicalization is disabled.
    fn canonicalize_path(&self, path: &Path) -> PathBuf {
        if self.config.canonicalize_paths {
            dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
        } else {
            path.to_path_buf()
        }
    }

    /// Pins a path to the left sidebar.
    fn pin_path(&mut self, path: PathBuf) {
        self.config.storage.pinned_folders.push(path);
    }

    /// Unpins a path from the left sidebar.
    fn unpin_path(&mut self, path: &Path) {
        self.config
            .storage
            .pinned_folders
            .retain(|p| p.as_path() != path);
    }

    /// Checks if the path is pinned to the left sidebar.
    fn is_pinned(&self, path: &Path) -> bool {
        self.config
            .storage
            .pinned_folders
            .iter()
            .any(|p| p.as_path() == path)
    }

    fn is_primary_selected(&self, item: &DirectoryEntry) -> bool {
        self.selected_item.as_ref().is_some_and(|x| x.path_eq(item))
    }

    /// Resets the dialog to use default values.
    /// Configuration variables are retained.
    fn reset(&mut self) {
        let config = self.config.clone();
        *self = Self::with_config(config);
    }

    /// Refreshes the dialog.
    /// Including the user directories, system disks and currently open directory.
    fn refresh(&mut self) {
        self.user_directories = self
            .config
            .file_system
            .user_dirs(self.config.canonicalize_paths);
        self.system_disks = self
            .config
            .file_system
            .get_disks(self.config.canonicalize_paths);

        self.reload_directory();
    }

    /// Submits the current selection and tries to finish the dialog, if the selection is valid.
    fn submit(&mut self) {
        // Make sure the selected item or entered file name is valid.
        if !self.is_selection_valid() {
            return;
        }

        self.config.storage.last_picked_dir = self.current_directory().map(PathBuf::from);

        match &self.mode {
            DialogMode::PickDirectory | DialogMode::PickFile => {
                // Should always contain a value since `is_selection_valid` is used to
                // validate the selection.
                if let Some(item) = self.selected_item.clone() {
                    self.state = DialogState::Picked(item.to_path_buf());
                }
            }
            DialogMode::PickMultiple => {
                let result: Vec<PathBuf> = self
                    .selected_entries()
                    .map(crate::DirectoryEntry::to_path_buf)
                    .collect();

                self.state = DialogState::PickedMultiple(result);
            }
            DialogMode::SaveFile => {
                // Should always contain a value since `is_selection_valid` is used to
                // validate the selection.
                if let Some(path) = self.current_directory() {
                    let full_path = path.join(&self.file_name_input);
                    self.submit_save_file(full_path);
                }
            }
        }
    }

    /// Submits the file dialog with the specified path and opens the `OverwriteFileModal`
    /// if the path already exists.
    fn submit_save_file(&mut self, path: PathBuf) {
        if path.exists() {
            self.open_modal(Box::new(OverwriteFileModal::new(path)));

            return;
        }

        self.state = DialogState::Picked(path);
    }

    /// Cancels the dialog.
    fn cancel(&mut self) {
        self.state = DialogState::Cancelled;
    }

    /// This function generates the initial directory based on the configuration.
    /// The function does the following things:
    ///   - Get the path to open based on the opening mode
    ///   - Canonicalize the path if enabled
    ///   - Attempts to use the parent directory if the path is a file
    fn get_initial_directory(&self) -> PathBuf {
        let path = match self.config.opening_mode {
            OpeningMode::AlwaysInitialDir => &self.config.initial_directory,
            OpeningMode::LastVisitedDir => self
                .config
                .storage
                .last_visited_dir
                .as_deref()
                .unwrap_or(&self.config.initial_directory),
            OpeningMode::LastPickedDir => self
                .config
                .storage
                .last_picked_dir
                .as_deref()
                .unwrap_or(&self.config.initial_directory),
        };

        let mut path = self.canonicalize_path(path);

        if self.config.file_system.is_file(&path) {
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
            DialogMode::PickDirectory => self
                .selected_item
                .as_ref()
                .is_some_and(crate::DirectoryEntry::is_dir),
            DialogMode::PickFile => self
                .selected_item
                .as_ref()
                .is_some_and(DirectoryEntry::is_file),
            DialogMode::PickMultiple => self.get_dir_content_filtered_iter().any(|p| p.selected),
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

            if self.config.file_system.is_dir(&full_path) {
                return Some(self.config.labels.err_directory_exists.clone());
            }

            if !self.config.allow_file_overwrite && self.config.file_system.is_file(&full_path) {
                return Some(self.config.labels.err_file_exists.clone());
            }
        } else {
            // There is most likely a bug in the code if we get this error message!
            return Some("Currently not in a directory".to_string());
        }

        None
    }

    /// Marks the given item as the selected directory item.
    /// Also updates the `file_name_input` to the name of the selected item.
    fn select_item(&mut self, item: &mut DirectoryEntry) {
        if self.mode == DialogMode::PickMultiple {
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

        self.directory_content.reset_multi_selection();

        let mut directory_content = std::mem::take(&mut self.directory_content);
        let search_value = std::mem::take(&mut self.search_value);

        let index = directory_content
            .filtered_iter(&search_value)
            .position(|p| p.path_eq(item));

        if let Some(index) = index {
            if index != 0 {
                if let Some(item) = directory_content
                    .filtered_iter_mut(&search_value)
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

        self.directory_content.reset_multi_selection();

        let mut directory_content = std::mem::take(&mut self.directory_content);
        let search_value = std::mem::take(&mut self.search_value);

        let index = directory_content
            .filtered_iter(&search_value)
            .position(|p| p.path_eq(item));

        if let Some(index) = index {
            if let Some(item) = directory_content
                .filtered_iter_mut(&search_value)
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
        self.directory_content.reset_multi_selection();

        let mut directory_content = std::mem::take(&mut self.directory_content);

        if let Some(item) = directory_content
            .filtered_iter_mut(&self.search_value.clone())
            .next()
        {
            self.select_item(item);
            self.scroll_to_selection = true;
        }

        self.directory_content = directory_content;
    }

    /// Tries to select the last visible item inside `directory_content`.
    fn select_last_visible_item(&mut self) {
        self.directory_content.reset_multi_selection();

        let mut directory_content = std::mem::take(&mut self.directory_content);

        if let Some(item) = directory_content
            .filtered_iter_mut(&self.search_value.clone())
            .last()
        {
            self.select_item(item);
            self.scroll_to_selection = true;
        }

        self.directory_content = directory_content;
    }

    /// Opens the text field in the top panel to text edit the current path.
    fn open_path_edit(&mut self) {
        let path = self.current_directory().map_or_else(String::new, |path| {
            path.to_str().unwrap_or_default().to_string()
        });

        self.path_edit_value = path;
        self.path_edit_activate = true;
        self.path_edit_visible = true;
    }

    /// Loads the directory from the path text edit.
    fn submit_path_edit(&mut self) {
        self.close_path_edit();

        let path = self.canonicalize_path(&PathBuf::from(&self.path_edit_value));

        if self.mode == DialogMode::PickFile && self.config.file_system.is_file(&path) {
            self.state = DialogState::Picked(path);
            return;
        }

        // Assume the user wants to save the given path when
        //   - an extension to the file name is given or the path
        //     edit is allowed to save a file without extension,
        //   - the path is not an existing directory,
        //   - and the parent directory exists
        // Otherwise we will assume the user wants to open the path as a directory.
        if self.mode == DialogMode::SaveFile
            && (path.extension().is_some()
                || self.config.allow_path_edit_to_save_file_without_extension)
            && !self.config.file_system.is_dir(&path)
            && path.parent().is_some_and(std::path::Path::exists)
        {
            self.submit_save_file(path);
            return;
        }

        self.load_directory(&path);
    }

    /// Closes the text field at the top to edit the current path without loading
    /// the entered directory.
    fn close_path_edit(&mut self) {
        self.path_edit_visible = false;
    }

    /// Loads the next directory in the `directory_stack`.
    /// If `directory_offset` is 0 and there is no other directory to load, `Ok()` is returned and
    /// nothing changes.
    /// Otherwise, the result of the directory loading operation is returned.
    fn load_next_directory(&mut self) {
        if self.directory_offset == 0 {
            // There is no next directory that can be loaded
            return;
        }

        self.directory_offset -= 1;

        // Copy path and load directory
        if let Some(path) = self.current_directory() {
            self.load_directory_content(path.to_path_buf().as_path());
        }
    }

    /// Loads the previous directory the user opened.
    /// If there is no previous directory left, `Ok()` is returned and nothing changes.
    /// Otherwise, the result of the directory loading operation is returned.
    fn load_previous_directory(&mut self) {
        if self.directory_offset + 1 >= self.directory_stack.len() {
            // There is no previous directory that can be loaded
            return;
        }

        self.directory_offset += 1;

        // Copy path and load directory
        if let Some(path) = self.current_directory() {
            self.load_directory_content(path.to_path_buf().as_path());
        }
    }

    /// Loads the parent directory of the currently open directory.
    /// If the directory doesn't have a parent, `Ok()` is returned and nothing changes.
    /// Otherwise, the result of the directory loading operation is returned.
    fn load_parent_directory(&mut self) {
        if let Some(x) = self.current_directory() {
            if let Some(x) = x.to_path_buf().parent() {
                self.load_directory(x);
            }
        }
    }

    /// Reloads the currently open directory.
    /// If no directory is currently open, `Ok()` will be returned.
    /// Otherwise, the result of the directory loading operation is returned.
    ///
    /// In most cases, this function should not be called directly.
    /// Instead, `refresh` should be used to reload all other data like system disks too.
    fn reload_directory(&mut self) {
        if let Some(x) = self.current_directory() {
            self.load_directory_content(x.to_path_buf().as_path());
        }
    }

    /// Loads the given directory and updates the `directory_stack`.
    /// The function deletes all directories from the `directory_stack` that are currently
    /// stored in the vector before the `directory_offset`.
    ///
    /// The function also sets the loaded directory as the selected item.
    fn load_directory(&mut self, path: &Path) {
        // Do not load the same directory again.
        // Use reload_directory if the content of the directory should be updated.
        if let Some(x) = self.current_directory() {
            if x == path {
                return;
            }
        }

        if self.directory_offset != 0 && self.directory_stack.len() > self.directory_offset {
            self.directory_stack
                .drain(self.directory_stack.len() - self.directory_offset..);
        }

        self.directory_stack.push(path.to_path_buf());
        self.directory_offset = 0;

        self.load_directory_content(path);

        // Clear the entry filter buffer.
        // It's unlikely the user wants to keep the current filter when entering a new directory.
        self.search_value.clear();
    }

    /// Loads the directory content of the given path.
    fn load_directory_content(&mut self, path: &Path) {
        self.config.storage.last_visited_dir = Some(path.to_path_buf());

        let selected_file_filter = match self.mode {
            DialogMode::PickFile | DialogMode::PickMultiple => self.get_selected_file_filter(),
            _ => None,
        };

        let selected_save_extension = if self.mode == DialogMode::SaveFile {
            self.get_selected_save_extension()
                .map(|e| e.file_extension.as_str())
        } else {
            None
        };

        self.directory_content = DirectoryContent::from_path(
            &self.config,
            path,
            self.show_files,
            selected_file_filter,
            selected_save_extension,
            self.config.file_system.clone(),
        );

        self.create_directory_dialog.close();
        self.scroll_to_selection = true;

        if self.mode == DialogMode::SaveFile {
            self.file_name_input_error = self.validate_file_name_input();
        }
    }
}
