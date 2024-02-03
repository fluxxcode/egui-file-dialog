#![warn(missing_docs)] // Let's keep the public API well documented!

use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::create_directory_dialog::CreateDirectoryDialog;

use crate::data::{DirectoryContent, DirectoryEntry, Disks, UserDirectories};
use crate::ui;

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
    /// The mode the dialog is currently in
    mode: DialogMode,
    /// The state the dialog is currently in
    state: DialogState,
    /// The first directory that will be opened when the dialog opens
    initial_directory: PathBuf,
    /// If files are displayed in addition to directories.
    /// This option will be ignored when mode == DialogMode::SelectFile.
    show_files: bool,

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
    /// The default size of the window.
    default_window_size: egui::Vec2,

    /// The dialog that is shown when the user wants to create a new directory.
    create_directory_dialog: CreateDirectoryDialog,

    /// The item that the user currently selected.
    /// Can be a directory or a folder.
    selected_item: Option<DirectoryEntry>,
    /// The default filename when opening the dialog in DialogMode::SaveFile mode.
    default_file_name: String,
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
    /// Creates a new file dialog instance with default values.
    pub fn new() -> Self {
        FileDialog {
            mode: DialogMode::SelectDirectory,
            state: DialogState::Closed,
            initial_directory: std::env::current_dir().unwrap_or_default(),
            show_files: true,

            user_directories: UserDirectories::new(),
            system_disks: Disks::new_with_refreshed_list(),

            directory_stack: vec![],
            directory_offset: 0,
            directory_content: DirectoryContent::new(),
            directory_error: None,

            window_title: String::from("Select directory"),
            default_window_size: egui::Vec2::new(650.0, 370.0),

            create_directory_dialog: CreateDirectoryDialog::new(),

            selected_item: None,
            default_file_name: String::new(),
            file_name_input: String::new(),
            file_name_input_error: None,

            scroll_to_selection: false,
            search_value: String::new(),
        }
    }

    /// Sets the first loaded directory when the dialog opens.
    /// If the path is a file, the file's parent directory is used. If the path then has no
    /// parent directory or cannot be loaded, the user will receive an error.
    /// However, the user directories and system disk allow the user to still select a file in
    /// the event of an error.
    ///
    /// Relative and absolute paths are allowed, but absolute paths are recommended.
    pub fn initial_directory(mut self, directory: PathBuf) -> Self {
        self.initial_directory = directory.clone();
        self
    }

    /// Sets the default size of the window.
    pub fn default_window_size(mut self, size: egui::Vec2) -> Self {
        self.default_window_size = size;
        self
    }

    /// Sets the default file name when opening the dialog in `DialogMode::SaveFile` mode.
    pub fn default_file_name(mut self, name: &str) -> Self {
        self.default_file_name = name.to_string();
        self
    }

    /// Opens the file dialog in the given mode with the given options.
    /// This function resets the file dialog and takes care for the variables that need to be
    /// set when opening the file dialog.
    ///
    /// Returns the result of the operation to load the initial directory.
    ///
    /// The `show_files` parameter will be ignored when the mode equals `DialogMode::SelectFile`.
    pub fn open(&mut self, mode: DialogMode, mut show_files: bool) -> io::Result<()> {
        self.reset();

        // Try to use the parent directory if the initial directory is a file.
        // If the path then has no parent directory, the user will see an error that the path
        // does not exist. However, using the user directories or disks, the user is still able
        // to select an item or save a file.
        if self.initial_directory.is_file() {
            if let Some(parent) = self.initial_directory.parent() {
                self.initial_directory = parent.to_path_buf();
            }
        }

        if mode == DialogMode::SelectFile {
            show_files = true;
        }

        if mode == DialogMode::SaveFile {
            self.file_name_input = self.default_file_name.clone();
        }

        self.mode = mode;
        self.state = DialogState::Open;
        self.show_files = show_files;

        self.window_title = match mode {
            DialogMode::SelectDirectory => "📁 Select Folder".to_string(),
            DialogMode::SelectFile => "📂 Open File".to_string(),
            DialogMode::SaveFile => "📥 Save File".to_string(),
        };

        self.load_directory(&self.initial_directory.clone())
    }

    /// Shortcut function to open the file dialog to prompt the user to select a directory.
    /// If used, no files in the directories will be shown to the user.
    /// Use the `open()` method instead, if you still want to display files to the user.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn select_directory(&mut self) {
        let _ = self.open(DialogMode::SelectDirectory, false);
    }

    /// Shortcut function to open the file dialog to prompt the user to select a file.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn select_file(&mut self) {
        let _ = self.open(DialogMode::SelectFile, false);
    }

    /// Shortcut function to open the file dialog to prompt the user to save a file.
    /// This function resets the file dialog. Configuration variables such as
    /// `initial_directory` are retained.
    ///
    /// The function ignores the result of the initial directory loading operation.
    pub fn save_file(&mut self) {
        let _ = self.open(DialogMode::SaveFile, true);
    }

    /// Returns the mode the dialog is currently in.
    pub fn mode(&self) -> DialogMode {
        self.mode
    }

    /// Returns the state the dialog is currently in.
    pub fn state(&self) -> DialogState {
        self.state.clone()
    }

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

    /// The main update method that should be called every frame if the dialog is to be visible.
    ///
    /// This function has no effect if the dialog state is currently not `DialogState::Open`.
    pub fn update(&mut self, ctx: &egui::Context) -> &Self {
        if self.state != DialogState::Open {
            return self;
        }

        let mut is_open = true;

        egui::Window::new(&self.window_title)
            .open(&mut is_open)
            .default_size(self.default_window_size)
            .min_width(335.0)
            .min_height(200.0)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("fe_top_panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        self.ui_update_top_panel(ctx, ui);
                    });

                egui::SidePanel::left("fe_left_panel")
                    .resizable(true)
                    .default_width(150.0)
                    .width_range(90.0..=250.0)
                    .show_inside(ui, |ui| {
                        self.ui_update_left_panel(ctx, ui);
                    });

                egui::TopBottomPanel::bottom("fe_bottom_panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        self.ui_update_bottom_panel(ctx, ui);
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    self.ui_update_central_panel(ui);
                });
            });

        // User closed the window without finishing the dialog
        if !is_open {
            self.cancel();
        }

        self
    }

    /// Updates the top panel of the dialog. Including the navigation buttons,
    /// the current path display, the reload button and the search field.
    fn ui_update_top_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        const NAV_BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(25.0, 25.0);

        ui.horizontal(|ui| {
            // Navigation buttons
            if let Some(x) = self.current_directory() {
                if ui::button_sized_enabled_disabled(ui, NAV_BUTTON_SIZE, "⏶", x.parent().is_some())
                {
                    let _ = self.load_parent_directory();
                }
            } else {
                let _ = ui::button_sized_enabled_disabled(ui, NAV_BUTTON_SIZE, "⏶", false);
            }

            if ui::button_sized_enabled_disabled(
                ui,
                NAV_BUTTON_SIZE,
                "⏴",
                self.directory_offset + 1 < self.directory_stack.len(),
            ) {
                let _ = self.load_previous_directory();
            }

            if ui::button_sized_enabled_disabled(
                ui,
                NAV_BUTTON_SIZE,
                "⏵",
                self.directory_offset != 0,
            ) {
                let _ = self.load_next_directory();
            }

            if ui::button_sized_enabled_disabled(
                ui,
                NAV_BUTTON_SIZE,
                "+",
                !self.create_directory_dialog.is_open(),
            ) {
                if let Some(x) = self.current_directory() {
                    self.create_directory_dialog.open(x.to_path_buf());
                }
            }

            // Leave area for the reload button and search window
            let path_display_width = ui.available_width() - 180.0;

            // Current path display
            egui::Frame::default()
                .stroke(egui::Stroke::new(
                    1.0,
                    ctx.style().visuals.window_stroke.color,
                ))
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .rounding(egui::Rounding::from(4.0))
                .show(ui, |ui| {
                    // TODO: Enable scrolling with mouse wheel
                    egui::ScrollArea::horizontal()
                        .auto_shrink([false, false])
                        .stick_to_right(true)
                        .max_width(path_display_width)
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

                                        // TODO: Maybe use selectable_label instead of button?
                                        // TODO: Write current directory (last item) in bold text
                                        if ui.button(file_name).clicked() {
                                            let _ = self.load_directory(path.as_path());
                                            return;
                                        }
                                    }
                                }
                            });
                        });
                });

            // Reload button
            if ui
                .add_sized(NAV_BUTTON_SIZE, egui::Button::new("⟲"))
                .clicked()
            {
                self.refresh();
            }

            // Search bar
            egui::Frame::default()
                .stroke(egui::Stroke::new(
                    1.0,
                    ctx.style().visuals.window_stroke.color,
                ))
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .rounding(egui::Rounding::from(4.0))
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        ui.add_space(ctx.style().spacing.item_spacing.y);
                        ui.label("🔍");
                        ui.add_sized(
                            egui::Vec2::new(ui.available_width(), 0.0),
                            egui::TextEdit::singleline(&mut self.search_value),
                        );
                    });
                });
        });

        ui.add_space(ctx.style().spacing.item_spacing.y);
    }

    /// Updates the left panel of the dialog. Including the list of the user directories (Places)
    /// and system disks.
    fn ui_update_left_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::containers::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(ctx.style().spacing.item_spacing.y * 2.0);

                    self.ui_update_user_directories(ui);

                    ui.add_space(ctx.style().spacing.item_spacing.y * 4.0);

                    self.ui_update_devices(ui);
                });
        });
    }

    /// Updates the bottom panel showing the selected item and main action buttons.
    fn ui_update_bottom_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        const BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(78.0, 20.0);

        ui.add_space(5.0);

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
                            ui.colored_label(ui.style().visuals.selection.bg_fill, x.file_name());
                        }
                    }
                }
                DialogMode::SaveFile => {
                    let response = ui.add(egui::TextEdit::singleline(&mut self.file_name_input));

                    if response.changed() {
                        self.file_name_input_error = self.validate_file_name_input();
                    }

                    if let Some(x) = &self.file_name_input_error {
                        // TODO: Use error icon instead
                        ui.label(x);
                    }
                }
            };
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let label = match &self.mode {
                DialogMode::SelectDirectory | DialogMode::SelectFile => "🗀  Open",
                DialogMode::SaveFile => "📥  Save",
            };

            if ui::button_sized_enabled_disabled(ui, BUTTON_SIZE, label, self.is_selection_valid())
            {
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

            ui.add_space(ctx.style().spacing.item_spacing.y);

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

    /// Updates the list of the user directories (Places).
    fn ui_update_user_directories(&mut self, ui: &mut egui::Ui) {
        if let Some(dirs) = self.user_directories.clone() {
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
        }
    }

    /// Updates the list of the system disks (Disks).
    fn ui_update_devices(&mut self, ui: &mut egui::Ui) {
        ui.label("Devices");

        let disks = std::mem::take(&mut self.system_disks);

        for disk in disks.iter() {
            if ui
                .selectable_label(false, format!("🖴  {}", disk.display_name()))
                .clicked()
            {
                let _ = self.load_directory(disk.mount_point());
            }
        }

        self.system_disks = disks;
    }

    /// Resets the dialog to use default values.
    /// Configuration variables such as `initial_directory` are retained.
    fn reset(&mut self) {
        self.state = DialogState::Closed;
        self.show_files = true;

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
                return Some("A directory the name already exists".to_string());
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

        let full_path = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(err) => {
                self.directory_error = Some(err.to_string());
                return Err(err);
            }
        };

        self.directory_stack.push(full_path);
        self.directory_offset = 0;

        self.load_directory_content(path)
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
