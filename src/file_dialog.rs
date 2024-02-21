#![warn(missing_docs)] // Let's keep the public API well documented!

use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::create_directory_dialog::CreateDirectoryDialog;

use crate::data::{DirectoryContent, DirectoryEntry, Disk, Disks, UserDirectories};

/// Represents the mode the file dialog is currently in.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DialogMode {
    /// When the dialog is currently used to select a file
    SelectFile,

    /// When the dialog is currently used to select a directory
    SelectDirectory,

    /// When the dialog is currently used to save a file
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

    /// The user cancelled the dialog and didn't select anything.
    Cancelled,
}

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
    pub overwrite_title: Option<String>,
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

            overwrite_title: None,
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

            show_left_panel: true,
            show_places: true,
            show_devices: true,
            show_removable_devices: true,
        }
    }
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

    /// The currently used window title.
    /// This changes depending on the mode the dialog is in.
    window_title: String,

    /// The dialog that is shown when the user wants to create a new directory.
    create_directory_dialog: CreateDirectoryDialog,

    /// The item that the user currently selected.
    /// Can be a directory or a folder.
    selected_item: Option<DirectoryEntry>,
    /// Buffer for the input of the file name when the dialog is in "SaveFile" mode.
    file_name_input: String,
    /// This variables contains the error message if the file_name_input is invalid.
    /// This can be the case, for example, if a file or folder with the name already exists.
    file_name_input_error: Option<String>,

    /// If we should scroll to the item selected by the user in the next frame.
    scroll_to_selection: bool,
    /// Buffer containing the value of the search input.
    search_value: String,
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

            mode: DialogMode::SelectDirectory,
            state: DialogState::Closed,
            show_files: true,
            operation_id: None,

            user_directories: UserDirectories::new(),
            system_disks: Disks::new_with_refreshed_list(),

            directory_stack: vec![],
            directory_offset: 0,
            directory_content: DirectoryContent::new(),
            directory_error: None,

            window_title: String::from("Select directory"),

            create_directory_dialog: CreateDirectoryDialog::new(),

            selected_item: None,
            file_name_input: String::new(),
            file_name_input_error: None,

            scroll_to_selection: false,
            search_value: String::new(),
        }
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

        // Try to use the parent directory if the initial directory is a file.
        // If the path then has no parent directory, the user will see an error that the path
        // does not exist. However, using the user directories or disks, the user is still able
        // to select an item or save a file.
        if self.config.initial_directory.is_file() {
            if let Some(parent) = self.config.initial_directory.parent() {
                self.config.initial_directory = parent.to_path_buf();
            }
        }

        if mode == DialogMode::SelectFile {
            show_files = true;
        }

        if mode == DialogMode::SaveFile {
            self.file_name_input = self.config.default_file_name.clone();
        }

        self.mode = mode;
        self.state = DialogState::Open;
        self.show_files = show_files;
        self.operation_id = operation_id.map(String::from);

        if let Some(title) = &self.config.overwrite_title {
            self.window_title = title.clone();
        } else {
            self.window_title = match mode {
                DialogMode::SelectDirectory => "📁 Select Folder".to_string(),
                DialogMode::SelectFile => "📂 Open File".to_string(),
                DialogMode::SaveFile => "📥 Save File".to_string(),
            };
        }

        self.load_directory(&self.config.initial_directory.clone())
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
        let _ = self.open(DialogMode::SelectFile, false, None);
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

        self.update_ui(ctx);

        self
    }

    // -------------------------------------------------
    // Setter:

    /// Sets the first loaded directory when the dialog opens.
    /// If the path is a file, the file's parent directory is used. If the path then has no
    /// parent directory or cannot be loaded, the user will receive an error.
    /// However, the user directories and system disk allow the user to still select a file in
    /// the event of an error.
    ///
    /// Relative and absolute paths are allowed, but absolute paths are recommended.
    pub fn initial_directory(mut self, directory: PathBuf) -> Self {
        self.config.initial_directory = directory.clone();
        self
    }

    /// Sets the default file name when opening the dialog in `DialogMode::SaveFile` mode.
    pub fn default_file_name(mut self, name: &str) -> Self {
        self.config.default_file_name = name.to_string();
        self
    }

    /// Overwrites the window title.
    ///
    /// By default, the title is set dynamically, based on the `DialogMode`
    /// the dialog is currently in.
    pub fn title(mut self, title: &str) -> Self {
        self.config.overwrite_title = Some(title.to_string());
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

    /// Sets if the sidebar with the shortcut directories such as
    /// “Home”, “Documents” etc. should be visible.
    pub fn show_left_panel(mut self, show_left_panel: bool) -> Self {
        self.config.show_left_panel = show_left_panel;
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

        self.create_window(&mut is_open).show(ctx, |ui| {
            egui::TopBottomPanel::top("fe_top_panel")
                .resizable(false)
                .show_inside(ui, |ui| {
                    self.ui_update_top_panel(ui);
                });

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

        // User closed the window without finishing the dialog
        if !is_open {
            self.cancel();
        }
    }

    /// Creates a new egui window with the configured options.
    fn create_window<'a>(&self, is_open: &'a mut bool) -> egui::Window<'a> {
        let mut window = egui::Window::new(&self.window_title)
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

            // Leave some area for the reload button and search input
            let path_display_width = ui.available_width() - 180.0;

            self.ui_update_current_path_display(ui, path_display_width);

            // Reload button
            if ui.add_sized(BUTTON_SIZE, egui::Button::new("⟲")).clicked() {
                self.refresh();
            }

            self.ui_update_search(ui);
        });

        ui.add_space(ui.ctx().style().spacing.item_spacing.y);
    }

    /// Updates the navigation buttons like parent or previous directory
    fn ui_update_nav_buttons(&mut self, ui: &mut egui::Ui, button_size: &egui::Vec2) {
        if let Some(x) = self.current_directory() {
            if self.ui_button_sized(ui, x.parent().is_some(), *button_size, "⏶", None) {
                let _ = self.load_parent_directory();
            }
        } else {
            let _ = self.ui_button_sized(ui, false, *button_size, "⏶", None);
        }

        if self.ui_button_sized(
            ui,
            self.directory_offset + 1 < self.directory_stack.len(),
            *button_size,
            "⏴",
            None,
        ) {
            let _ = self.load_previous_directory();
        }

        if self.ui_button_sized(ui, self.directory_offset != 0, *button_size, "⏵", None) {
            let _ = self.load_next_directory();
        }

        if self.ui_button_sized(
            ui,
            !self.create_directory_dialog.is_open(),
            *button_size,
            "+",
            None,
        ) {
            if let Some(x) = self.current_directory() {
                self.create_directory_dialog.open(x.to_path_buf());
            }
        }
    }

    /// Updates the view to display the currently open path
    fn ui_update_current_path_display(&mut self, ui: &mut egui::Ui, width: f32) {
        egui::Frame::default()
            .stroke(egui::Stroke::new(
                1.0,
                ui.ctx().style().visuals.window_stroke.color,
            ))
            .inner_margin(egui::Margin {
                left: 4.0,
                right: 8.0,
                top: 4.0,
                bottom: 4.0,
            })
            .rounding(egui::Rounding::from(4.0))
            .show(ui, |ui| {
                ui.style_mut().always_scroll_the_only_direction = true;
                ui.style_mut().spacing.scroll.bar_width = 8.0;

                egui::ScrollArea::horizontal()
                    .auto_shrink([false, false])
                    .stick_to_right(true)
                    .max_width(width)
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
                                        if i == 0 && file_name.contains(r"\\?\") {
                                            drive_letter = file_name.replace(r"\\?\", "");
                                            continue;
                                        }

                                        // Replace the root segment with the disk letter
                                        if i == 1 && segment == "\\" {
                                            file_name = drive_letter.as_str();
                                        } else if i != 0 {
                                            ui.label(">");
                                        }
                                    }

                                    #[cfg(not(windows))]
                                    let file_name = segment.to_str().unwrap_or("<ERR>");

                                    #[cfg(not(windows))]
                                    if i != 0 {
                                        ui.label(">");
                                    }

                                    if ui.button(file_name).clicked() {
                                        let _ = self.load_directory(path.as_path());
                                        return;
                                    }
                                }
                            }
                        });
                    });
            });
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
                    ui.add_sized(
                        egui::Vec2::new(ui.available_width(), 0.0),
                        egui::TextEdit::singleline(&mut self.search_value),
                    );
                });
            });
    }

    /// Updates the left panel of the dialog. Including the list of the user directories (Places)
    /// and system disks (Devices, Removable Devices).
    fn ui_update_left_panel(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::containers::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut spacing = ui.ctx().style().spacing.item_spacing.y * 2.0;

                    if self.config.show_places && self.ui_update_user_directories(ui, spacing) {
                        spacing = ui.ctx().style().spacing.item_spacing.y * 4.0;
                    }

                    let disks = std::mem::take(&mut self.system_disks);

                    if self.config.show_devices && self.ui_update_devices(ui, spacing, &disks) {
                        spacing = ui.ctx().style().spacing.item_spacing.y * 4.0;
                    }

                    if self.config.show_removable_devices
                        && self.ui_update_removable_devices(ui, spacing, &disks)
                    {
                        // Add this when we add a new section after removable devices
                        // spacing = ui.ctx().style().spacing.item_spacing.y * 4.0;
                    }

                    self.system_disks = disks;
                });
        });
    }

    /// Updates the list of the user directories (Places).
    ///
    /// Returns true if at least one directory was included in the list and the
    /// heading is visible. If no directory was listed, false is returned.
    fn ui_update_user_directories(&mut self, ui: &mut egui::Ui, spacing: f32) -> bool {
        if let Some(dirs) = self.user_directories.clone() {
            ui.add_space(spacing);
            ui.label("Places");

            if let Some(path) = dirs.home_dir() {
                if ui
                    .selectable_label(self.current_directory() == Some(path), "🏠  Home")
                    .clicked()
                {
                    let _ = self.load_directory(path);
                }
            }

            if let Some(path) = dirs.desktop_dir() {
                if ui
                    .selectable_label(self.current_directory() == Some(path), "🖵  Desktop")
                    .clicked()
                {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.document_dir() {
                if ui
                    .selectable_label(self.current_directory() == Some(path), "🗐  Documents")
                    .clicked()
                {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.download_dir() {
                if ui
                    .selectable_label(self.current_directory() == Some(path), "📥  Downloads")
                    .clicked()
                {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.audio_dir() {
                if ui
                    .selectable_label(self.current_directory() == Some(path), "🎵  Audio")
                    .clicked()
                {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.picture_dir() {
                if ui
                    .selectable_label(self.current_directory() == Some(path), "🖼  Pictures")
                    .clicked()
                {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.video_dir() {
                if ui
                    .selectable_label(self.current_directory() == Some(path), "🎞  Videos")
                    .clicked()
                {
                    let _ = self.load_directory(path);
                }
            }

            return true;
        }

        false
    }

    /// Updates the list of devices like system disks
    ///
    /// Returns true if at least one device was included in the list and the
    /// heading is visible. If no device was listed, false is returned.
    fn ui_update_devices(&mut self, ui: &mut egui::Ui, spacing: f32, disks: &Disks) -> bool {
        let mut visible = false;

        for (i, disk) in disks.iter().filter(|x| !x.is_removable()).enumerate() {
            if i == 0 {
                ui.add_space(spacing);
                ui.label("Devices");

                visible = true;
            }

            self.ui_update_device_entry(ui, disk);
        }

        visible
    }

    /// Updates the list of removable devices like USB drives
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
                ui.label("Removable Devices");

                visible = true;
            }

            self.ui_update_device_entry(ui, disk);
        }

        visible
    }

    /// Updates a device entry of a device list like "Devices" or "Removable Devices".
    fn ui_update_device_entry(&mut self, ui: &mut egui::Ui, device: &Disk) {
        let label = match device.is_removable() {
            true => format!("💾  {}", device.display_name()),
            false => format!("🖴  {}", device.display_name()),
        };

        if ui.selectable_label(false, label).clicked() {
            let _ = self.load_directory(device.mount_point());
        }
    }

    /// Updates the bottom panel showing the selected item and main action buttons.
    fn ui_update_bottom_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(5.0);

        self.ui_update_selection_preview(ui);

        if self.mode == DialogMode::SaveFile {
            ui.add_space(ui.style().spacing.item_spacing.y * 2.0)
        }

        self.ui_update_action_buttons(ui);
    }

    /// Updates the selection preview like "Selected directory: X"
    fn ui_update_selection_preview(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            match &self.mode {
                DialogMode::SelectDirectory => ui.label("Selected directory:"),
                DialogMode::SelectFile => ui.label("Selected file:"),
                DialogMode::SaveFile => ui.label("File name:"),
            };

            match &self.mode {
                DialogMode::SelectDirectory | DialogMode::SelectFile => {
                    if self.is_selection_valid() {
                        if let Some(x) = &self.selected_item {
                            use egui::containers::scroll_area::ScrollBarVisibility;

                            egui::containers::ScrollArea::horizontal()
                                .auto_shrink([false, false])
                                .stick_to_right(true)
                                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                                .show(ui, |ui| {
                                    ui.colored_label(
                                        ui.style().visuals.selection.bg_fill,
                                        x.file_name(),
                                    );
                                });
                        }
                    }
                }
                DialogMode::SaveFile => {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.file_name_input)
                            .desired_width(f32::INFINITY),
                    );

                    if response.changed() {
                        self.file_name_input_error = self.validate_file_name_input();
                    }
                }
            };
        });
    }

    /// Updates the action buttons like save, open and cancel
    fn ui_update_action_buttons(&mut self, ui: &mut egui::Ui) {
        const BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(78.0, 20.0);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let label = match &self.mode {
                DialogMode::SelectDirectory | DialogMode::SelectFile => "🗀  Open",
                DialogMode::SaveFile => "📥  Save",
            };

            if self.ui_button_sized(
                ui,
                self.is_selection_valid(),
                BUTTON_SIZE,
                label,
                self.file_name_input_error.as_deref(),
            ) {
                match &self.mode {
                    DialogMode::SelectDirectory | DialogMode::SelectFile => {
                        // self.selected_item should always contain a value,
                        // since self.is_selection_valid() validates the selection and
                        // returns false if the selection is none.
                        if let Some(selection) = self.selected_item.clone() {
                            self.finish(selection.to_path_buf());
                        }
                    }
                    DialogMode::SaveFile => {
                        // self.current_directory should always contain a value,
                        // since self.is_selection_valid() makes sure there is no
                        // file_name_input_error. The file_name_input_error
                        // gets validated every time something changes
                        // by the validate_file_name_input, which sets an error
                        // if we are currently not in a directory.
                        if let Some(path) = self.current_directory() {
                            let mut full_path = path.to_path_buf();
                            full_path.push(&self.file_name_input);

                            self.finish(full_path);
                        }
                    }
                }
            }

            ui.add_space(ui.ctx().style().spacing.item_spacing.y);

            if ui
                .add_sized(BUTTON_SIZE, egui::Button::new("🚫 Cancel"))
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
                ui.colored_label(egui::Color32::RED, format!("⚠ {}", err));
            });
            return;
        }

        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::containers::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Temporarily take ownership of the directory contents to be able to
                    // update it in the for loop using load_directory.
                    // Otherwise we would get an error that `*self` cannot be borrowed as mutable
                    // more than once at a time.
                    // Make sure to return the function after updating the directory_content,
                    // otherwise the change will be overwritten with the last statement
                    // of the function.
                    let data = std::mem::take(&mut self.directory_content);

                    for path in data.iter() {
                        let file_name = path.file_name();

                        if !self.search_value.is_empty()
                            && !file_name
                                .to_lowercase()
                                .contains(&self.search_value.to_lowercase())
                        {
                            continue;
                        }

                        let icon = match path.is_dir() {
                            true => "🗀",
                            _ => "🖹",
                        };

                        let mut selected = false;
                        if let Some(x) = &self.selected_item {
                            selected = x == path;
                        }

                        let response =
                            ui.selectable_label(selected, format!("{} {}", icon, file_name));

                        if selected && self.scroll_to_selection {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }

                        if response.clicked() {
                            self.select_item(path);
                        }

                        if response.double_clicked() {
                            if path.is_dir() {
                                let _ = self.load_directory(&path.to_path_buf());
                                return;
                            }

                            self.select_item(path);

                            if self.is_selection_valid() {
                                // self.selected_item should always contain a value
                                // since self.is_selection_valid() validates the selection
                                // and returns false if the selection is none.
                                if let Some(selection) = self.selected_item.clone() {
                                    self.finish(selection.to_path_buf());
                                }
                            }
                        }
                    }

                    self.scroll_to_selection = false;
                    self.directory_content = data;

                    if let Some(path) = self.create_directory_dialog.update(ui).directory() {
                        let entry = DirectoryEntry::from_path(&path);

                        self.directory_content.push(entry.clone());
                        self.select_item(&entry);
                    }
                });
        });
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

                        ui.colored_label(ui.ctx().style().visuals.error_fg_color, "⚠ ");
                        ui.label(err);
                    });
                });
            }
        });

        clicked
    }
}

/// Implementation
impl FileDialog {
    /// Resets the dialog to use default values.
    /// Configuration variables such as `initial_directory` are retained.
    fn reset(&mut self) {
        self.state = DialogState::Closed;
        self.show_files = true;
        self.operation_id = None;

        self.system_disks = Disks::new_with_refreshed_list();

        self.directory_stack = vec![];
        self.directory_offset = 0;
        self.directory_content = DirectoryContent::new();

        self.create_directory_dialog = CreateDirectoryDialog::new();

        self.selected_item = None;
        self.file_name_input = String::new();
        self.scroll_to_selection = false;
        self.search_value = String::new();
    }

    /// Refreshes the dialog.
    /// Including the user directories, system disks and currently open directory.
    fn refresh(&mut self) {
        self.user_directories = UserDirectories::new();
        self.system_disks = Disks::new_with_refreshed_list();

        let _ = self.reload_directory();
    }

    /// Finishes the dialog.
    /// `selected_item`` is the item that was selected by the user.
    fn finish(&mut self, selected_item: PathBuf) {
        self.state = DialogState::Selected(selected_item);
    }

    /// Cancels the dialog.
    fn cancel(&mut self) {
        self.state = DialogState::Cancelled;
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
        if let Some(selection) = &self.selected_item {
            return match &self.mode {
                DialogMode::SelectDirectory => selection.is_dir(),
                DialogMode::SelectFile => selection.is_file(),
                DialogMode::SaveFile => self.file_name_input_error.is_none(),
            };
        }

        if self.mode == DialogMode::SaveFile && self.file_name_input_error.is_none() {
            return true;
        }

        false
    }

    /// Validates the file name entered by the user.
    ///
    /// Returns None if the file name is valid. Otherwise returns an error message.
    fn validate_file_name_input(&self) -> Option<String> {
        if self.file_name_input.is_empty() {
            return Some("The file name cannot be empty".to_string());
        }

        if let Some(x) = self.current_directory() {
            let mut full_path = x.to_path_buf();
            full_path.push(self.file_name_input.as_str());

            if full_path.is_dir() {
                return Some("A directory with the name already exists".to_string());
            }
            if full_path.is_file() {
                return Some("A file with the name already exists".to_string());
            }
        } else {
            // There is most likely a bug in the code if we get this error message!
            return Some("Currently not in a directory".to_string());
        }

        None
    }

    /// Marks the given item as the selected directory item.
    /// Also updates the file_name_input to the name of the selected item.
    fn select_item(&mut self, dir_entry: &DirectoryEntry) {
        self.selected_item = Some(dir_entry.clone());

        if self.mode == DialogMode::SaveFile && dir_entry.is_file() {
            self.file_name_input = dir_entry.file_name().to_string();
            self.file_name_input_error = self.validate_file_name_input();
        }
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
        let full_path = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(err) => {
                self.directory_error = Some(err.to_string());
                return Err(err);
            }
        };

        // Do not load the same directory again.
        // Use reload_directory if the content of the directory should be updated.
        if let Some(x) = self.current_directory() {
            if x == full_path {
                return Ok(());
            }
        }

        if self.directory_offset != 0 && self.directory_stack.len() > self.directory_offset {
            self.directory_stack
                .drain(self.directory_stack.len() - self.directory_offset..);
        }

        self.directory_stack.push(full_path);
        self.directory_offset = 0;

        self.load_directory_content(path)?;

        let dir_entry = DirectoryEntry::from_path(path);
        self.select_item(&dir_entry);

        Ok(())
    }

    /// Loads the directory content of the given path.
    fn load_directory_content(&mut self, path: &Path) -> io::Result<()> {
        self.directory_error = None;

        self.directory_content = match DirectoryContent::from_path(path, self.show_files) {
            Ok(content) => content,
            Err(err) => {
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
