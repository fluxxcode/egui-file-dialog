use std::{fs, io};
use std::path::{Path, PathBuf};

use directories::UserDirs;
use sysinfo::Disks;

mod create_directory_dialog;
use create_directory_dialog::CreateDirectoryDialog;

use crate::data::{DirectoryContent, DirectoryEntry};
use crate::ui;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DialogMode {
    SelectFile,
    SelectDirectory,
    SaveFile
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DialogState {
    Open,
    Closed,
    Selected(PathBuf),
    Cancelled
}

pub struct FileDialog {
    mode: DialogMode,
    state: DialogState,
    initial_directory: PathBuf,

    user_directories: Option<UserDirs>,
    system_disks: Disks,

    directory_stack: Vec<PathBuf>,
    directory_offset: usize,
    directory_content: DirectoryContent,

    window_title: String,

    create_directory_dialog: CreateDirectoryDialog,

    selected_item: Option<DirectoryEntry>,
    file_name_input: String,  // Only used when mode = DialogMode::SaveFile
    file_name_input_error: Option<String>,

    scroll_to_selection: bool,
    search_value: String
}

impl Default for FileDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDialog {
    pub fn new() -> Self {
        FileDialog {
            mode: DialogMode::SelectDirectory,
            state: DialogState::Closed,
            initial_directory: std::env::current_dir().unwrap_or_default(),

            user_directories: UserDirs::new(),
            system_disks: Disks::new_with_refreshed_list(),

            directory_stack: vec![],
            directory_offset: 0,
            directory_content: DirectoryContent::new(),

            window_title: String::from("Select directory"),

            create_directory_dialog: CreateDirectoryDialog::new(),

            selected_item: None,
            file_name_input: String::new(),
            file_name_input_error: None,

            scroll_to_selection: false,
            search_value: String::new()
        }
    }

    pub fn initial_directory(mut self, directory: PathBuf) -> Self {
        self.initial_directory = directory.clone();
        self
    }

    pub fn open(&mut self, mode: DialogMode) {
        self.reset();

        self.mode = mode;
        self.state = DialogState::Open;

        self.window_title = match mode {
            DialogMode::SelectDirectory => "📁 Select Folder".to_string(),
            DialogMode::SelectFile => "📂 Open File".to_string(),
            DialogMode::SaveFile => "📥 Save File".to_string()
        };

        // TODO: Error handling
        let _ = self.load_directory(&self.initial_directory.clone());
    }

    pub fn select_directory(&mut self) {
        self.open(DialogMode::SelectDirectory);
    }

    pub fn select_file(&mut self) {
        self.open(DialogMode::SelectFile);
    }

    pub fn save_file(&mut self) {
        self.open(DialogMode::SaveFile);
    }

    pub fn mode(&self) -> DialogMode {
        self.mode
    }

    pub fn state(&self) -> DialogState {
        self.state.clone()
    }

    pub fn update(&mut self, ctx: &egui::Context) -> &Self {
        if self.state != DialogState::Open {
            return self;
        }

        let mut is_open = true;

        egui::Window::new(&self.window_title)
            .open(&mut is_open)
            .default_size([800.0, 500.0])
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

    fn ui_update_top_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        const NAV_BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(25.0, 25.0);

        ui.horizontal(|ui| {

            // Navigation buttons
            if let Some(x) = self.current_directory() {
                if ui::button_sized_enabled_disabled(ui, NAV_BUTTON_SIZE, "⏶",
                                                     x.parent().is_some()) {
                    let _ = self.load_parent();
                }
            }
            else {
                let _ = ui::button_sized_enabled_disabled(ui, NAV_BUTTON_SIZE, "⏶", false);
            }

            if ui::button_sized_enabled_disabled(ui, NAV_BUTTON_SIZE, "⏴",
                    self.directory_offset + 1 < self.directory_stack.len()) {
                let _ = self.load_previous_directory();
            }

            if ui::button_sized_enabled_disabled(ui, NAV_BUTTON_SIZE, "⏵",
                                                 self.directory_offset != 0) {
                let _ = self.load_next_directory();
            }

            if ui::button_sized_enabled_disabled(ui, NAV_BUTTON_SIZE, "+",
                                                 !self.create_directory_dialog.is_open()) {
                if let Some(x) = self.current_directory() {
                    self.create_directory_dialog.open(x.to_path_buf());
                }
            }

            // Leave area for the reload button and search window
            let path_display_width = ui.available_width() - 180.0;

            // Current path display
            egui::Frame::default()
                .stroke(egui::Stroke::new(1.0, ctx.style().visuals.window_stroke.color))
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
                                    for (i, segment) in data.iter().enumerate() {
                                        path.push(segment);

                                        if i != 0 {
                                            ui.label(">");
                                        }

                                        // TODO: Maybe use selectable_label instead of button?
                                        // TODO: Write current directory (last item) in bold text
                                        if ui.button(segment.to_str().unwrap_or("<ERR>"))
                                            .clicked() {
                                                let _ = self.load_directory(path.as_path());
                                                return;
                                        }
                                    }
                                }
                            });
                        });
                });

            // Reload button
            if ui.add_sized(NAV_BUTTON_SIZE, egui::Button::new("⟲")).clicked() {
                self.refresh();
            }

            // Search bar
            egui::Frame::default()
                .stroke(egui::Stroke::new(1.0, ctx.style().visuals.window_stroke.color))
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .rounding(egui::Rounding::from(4.0))
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        ui.add_space(ctx.style().spacing.item_spacing.y);
                        ui.label("🔍");
                        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0),
                                    egui::TextEdit::singleline(&mut self.search_value));
                    });
                });
        });

        ui.add_space(ctx.style().spacing.item_spacing.y);
    }

    fn ui_update_left_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::containers::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.ui_update_user_directories(ui);

                    ui.add_space(ctx.style().spacing.item_spacing.y * 4.0);

                    self.ui_update_devices(ui);
                });
        });
    }

    fn ui_update_bottom_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        const BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(78.0, 20.0);

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            match &self.mode {
                DialogMode::SelectDirectory => ui.label("Selected directory:"),
                DialogMode::SelectFile => ui.label("Selected file:"),
                DialogMode::SaveFile => ui.label("File name:")
            };

            match &self.mode {
                DialogMode::SelectDirectory | DialogMode::SelectFile => {
                    if self.is_selection_valid() {
                        if let Some(x) = &self.selected_item {
                            ui.colored_label(ui.style().visuals.selection.bg_fill, x.file_name());
                        }
                    }
                },
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
                DialogMode::SelectDirectory | DialogMode::SelectFile => "Open",
                DialogMode::SaveFile => "Save"
            };

            if ui::button_sized_enabled_disabled(ui, BUTTON_SIZE, label,
                                                 self.is_selection_valid()) {
                match &self.mode {
                    DialogMode::SelectDirectory | DialogMode::SelectFile => {
                        // self.selected_item should always contain a value,
                        // since self.is_selection_valid() validates the selection and
                        // returns false if the selection is none.
                        if let Some(selection) = self.selected_item.clone() {
                            self.finish(selection.to_path_buf());
                        }
                    },
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

            if ui.add_sized(BUTTON_SIZE, egui::Button::new("Abort")).clicked() {
                self.cancel();
            }
        });
    }

    fn ui_update_central_panel(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            egui::containers::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                // Temporarily take ownership of the directory contents to be able to
                // update it in the for loop using load_directory.
                // Otherwise we would get an error that `*self` cannot be borrowed as mutable
                // more than once at a time.
                // Make sure to return the function after updating the directory_content,
                // otherwise the change will be overwritten with the last statement of the function.
                let data = std::mem::take(&mut self.directory_content);

                for path in data.iter() {
                    let file_name = path.file_name();

                    if !self.search_value.is_empty() &&
                       !file_name.to_lowercase().contains(&self.search_value.to_lowercase()) {
                        continue;
                    }

                    let icon = match path.is_dir() {
                        true => "🗀",
                        _ => "🖹"
                    };

                    let mut selected = false;
                    if let Some(x) = &self.selected_item {
                        selected = x == path;
                    }

                    let response = ui.selectable_label(selected, format!("{} {}", icon, file_name));

                    if selected && self.scroll_to_selection {
                        response.scroll_to_me(None);
                        self.scroll_to_selection = false;
                    }

                    if response.clicked() {
                        self.select_item(&path);
                    }

                    if response.double_clicked() {
                        if path.is_dir() {
                            let _ = self.load_directory(&path.to_path_buf());
                            return;
                        }

                        self.select_item(&path);

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

                self.directory_content = data;

                if let Some(path) = self.create_directory_dialog.update(ui).directory() {
                    let entry = DirectoryEntry::from_path(&path);

                    self.directory_content.push(entry.clone());
                    self.select_item(&entry);
                }
            });
        });
    }

    fn ui_update_user_directories(&mut self, ui: &mut egui::Ui) {
        if let Some(dirs) = self.user_directories.clone() {
            ui.label("Places");

            if ui.selectable_label(self.current_directory() == Some(dirs.home_dir()),
                                   "🏠  Home").clicked() {
                let _ = self.load_directory(dirs.home_dir());
            }

            if let Some(path) = dirs.desktop_dir() {
                if ui.selectable_label(self.current_directory() == Some(path),
                                       "🖵  Desktop").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.document_dir() {
                if ui.selectable_label(self.current_directory() == Some(path),
                                       "🗐  Documents").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.download_dir() {
                if ui.selectable_label(self.current_directory() == Some(path),
                                       "📥  Downloads").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.audio_dir() {
                if ui.selectable_label(self.current_directory() == Some(path),
                                       "🎵  Audio").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.picture_dir() {
                if ui.selectable_label(self.current_directory() == Some(path),
                                       "🖼  Pictures").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.video_dir() {
                if ui.selectable_label(self.current_directory() == Some(path),
                                       "🎞  Videos").clicked() {
                    let _ = self.load_directory(path);
                }
            }
        }
    }

    fn ui_update_devices(&mut self, ui: &mut egui::Ui) {
        ui.label("Devices");

        let disks = std::mem::take(&mut self.system_disks);

        for disk in &disks {
            // TODO: Get display name of the devices.
            // Currently on linux "/dev/sda1" is returned.
            let name = match disk.name().to_str() {
                Some(x) => x,
                None => continue
            };

            if ui.selectable_label(false, format!("🖴  {}", name)).clicked() {
                let _ = self.load_directory(disk.mount_point());
            }
        }

        self.system_disks = disks;
    }

    fn reset(&mut self) {
        self.state = DialogState::Closed;

        self.user_directories = UserDirs::new();
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

    fn refresh(&mut self) {
        self.user_directories = UserDirs::new();
        self.system_disks = Disks::new_with_refreshed_list();

        let _ = self.reload_directory();
    }

    fn finish(&mut self, selected_item: PathBuf) {
        self.state = DialogState::Selected(selected_item);
    }

    fn cancel(&mut self) {
        self.state = DialogState::Cancelled;
    }

    fn current_directory(&self) -> Option<&Path> {
        if let Some(x) = self.directory_stack.iter().nth_back(self.directory_offset) {
            return Some(x.as_path())
        }

        None
    }

    fn is_selection_valid(&self) -> bool {
        if let Some(selection) = &self.selected_item {
            return match &self.mode {
                DialogMode::SelectDirectory => selection.is_dir(),
                DialogMode::SelectFile => selection.is_file(),
                DialogMode::SaveFile => self.file_name_input_error.is_none()
            };
        }

        if self.mode == DialogMode::SaveFile && self.file_name_input_error.is_none() {
            return true;
        }

        false
    }

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
        }
        else {
            // There is most likely a bug in the code if we get this error message!
            return Some("Currently not in a directory".to_string())
        }

        None
    }

    fn select_item(&mut self, dir_entry: &DirectoryEntry) {
        self.selected_item = Some(dir_entry.clone());

        if self.mode == DialogMode::SaveFile && dir_entry.is_file() {
            self.file_name_input = dir_entry.file_name().to_string();
            self.file_name_input_error = self.validate_file_name_input();
        }
    }

    fn load_next_directory(&mut self) -> io::Result<()> {
        if self.directory_offset == 0 {
            // There is no next directory that can be loaded
            return Ok(());
        }

        self.directory_offset -= 1;

        // Copy path and load directory
        let path = self.current_directory().unwrap().to_path_buf();
        self.load_directory_content(path.as_path())
    }

    fn load_previous_directory(&mut self) -> io::Result<()> {
        if self.directory_offset + 1 >= self.directory_stack.len() {
            // There is no previous directory that can be loaded
            return Ok(())
        }

        self.directory_offset += 1;
    
        // Copy path and load directory
        let path = self.current_directory().unwrap().to_path_buf();
        self.load_directory_content(path.as_path())
    }

    fn load_parent(&mut self) -> io::Result<()> {
        if let Some(x) = self.current_directory() {
            if let Some(x) = x.to_path_buf().parent() {
                return self.load_directory(x);
            }
        }

        Ok(())
    }

    fn reload_directory(&mut self) -> io::Result<()> {
        if let Some(x) = self.current_directory() {
            return self.load_directory_content(x.to_path_buf().as_path());
        }

        Ok(())
    }

    fn load_directory(&mut self, path: &Path) -> io::Result<()> {
        // Do not load the same directory again.
        // Use reload_directory if the content of the directory should be updated.
        if let Some(x) = self.current_directory() {
            if x == path {
                return Ok(());
            }
        }

        if self.directory_offset != 0 && self.directory_stack.len() > self.directory_offset {
            self.directory_stack.drain(self.directory_stack.len() - self.directory_offset..);
        }

        self.directory_stack.push(fs::canonicalize(path)?);
        self.directory_offset = 0;

        self.load_directory_content(path)
    }

    fn load_directory_content(&mut self, path: &Path) -> io::Result<()> {
        self.directory_content = DirectoryContent::from_path(path)?;

        self.create_directory_dialog.close();
        self.scroll_to_selection = true;

        if self.mode == DialogMode::SaveFile {
            self.file_name_input_error = self.validate_file_name_input();
        }

        Ok(())
    }
}
