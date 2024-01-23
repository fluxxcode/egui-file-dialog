use std::{fs, io};
use std::path::{Path, PathBuf};

use directories::UserDirs;
use sysinfo::Disks;

// NOTE: Currently not implemented, just an idea!
pub enum FileExplorerMode {
    OpenFile,
    OpenDirectory,
    SaveFile
}

pub struct FileExplorer {
    user_directories: Option<UserDirs>,
    system_disks: Disks,

    directory_stack: Vec<PathBuf>,
    directory_offset: usize,
    directory_content: Vec<PathBuf>,

    create_directory_dialog: CreateDirectoryDialog,

    selected_item: Option<PathBuf>,
    search_value: String
}

impl Default for FileExplorer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileExplorer {
    pub fn new() -> Self {
        FileExplorer {
            user_directories: UserDirs::new(),
            system_disks: Disks::new_with_refreshed_list(),

            directory_stack: vec![],
            directory_offset: 0,
            directory_content: vec![],

            create_directory_dialog: CreateDirectoryDialog::new(),

            selected_item: None,
            search_value: String::new(),
        }
    }

    // TODO: Enable option to set initial directory
    pub fn open(&mut self) {
        // TODO: Error handling
        let _ = self.load_directory(Path::new("./"));
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        // TODO: Make window title and options configurable
        egui::Window::new("File explorer")
            .default_size([800.0, 500.0])
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("fe_top_panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        self.update_top_panel(ctx, ui);
                    });

                egui::SidePanel::left("fe_left_panel")
                    .resizable(true)
                    .default_width(150.0)
                    .width_range(100.0..=400.0)
                    .show_inside(ui, |ui| {
                        self.update_left_panel(ctx, ui);
                    });

                egui::TopBottomPanel::bottom("fe_bottom_panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        self.update_bottom_panel(ctx, ui);
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    self.update_central_panel(ui);
                });
            });
    }

    fn update_top_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        const NAV_BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(25.0, 25.0);
        const SEARCH_INPUT_WIDTH: f32 = 120.0;

        ui.horizontal(|ui| {

            // Navigation buttons
            if let Some(x) = self.current_directory() {
                if ui_button_sized(ui, NAV_BUTTON_SIZE, "⏶", x.parent().is_some()) {
                    let _ = self.load_parent();
                }
            }
            else {
                let _ = ui_button_sized(ui, NAV_BUTTON_SIZE, "⏶", false);
            }

            if ui_button_sized(ui, NAV_BUTTON_SIZE, "⏴",
                               self.directory_offset + 1 < self.directory_stack.len()) {
                let _ = self.load_previous_directory();
            }

            if ui_button_sized(ui, NAV_BUTTON_SIZE, "⏵", self.directory_offset != 0) {
                let _ = self.load_next_directory();
            }

            if ui_button_sized(ui, NAV_BUTTON_SIZE, "+", !self.create_directory_dialog.is_open()) {
                if let Some(x) = self.current_directory() {
                    self.create_directory_dialog.open(x.to_path_buf());
                }
            }

            // Current path display
            egui::Frame::default()
                .stroke(egui::Stroke::new(2.0, ctx.style().visuals.window_stroke.color))
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .rounding(egui::Rounding::from(5.0))
                .show(ui, |ui| {
                    // TODO: Enable scrolling with mouse wheel
                    egui::ScrollArea::horizontal()
                        .auto_shrink([false, false])
                        .stick_to_right(true)
                        // TODO: Dynamically size scroll area to available width
                        .max_width(500.0)
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
                let _ = self.reload_directory();
            }

            // Search bar
            egui::Frame::default()
                .stroke(egui::Stroke::new(2.0, ctx.style().visuals.window_stroke.color))
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .rounding(egui::Rounding::from(5.0))
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        ui.add_space(ctx.style().spacing.item_spacing.y);
                        ui.label("🔍");
                        ui.add_sized(egui::Vec2::new(SEARCH_INPUT_WIDTH, 0.0),
                                    egui::TextEdit::singleline(&mut self.search_value));
                    });
                });
        });

        ui.add_space(ctx.style().spacing.item_spacing.y);
    }

    fn update_left_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.update_user_directories(ui);

        ui.add_space(ctx.style().spacing.item_spacing.y * 4.0);

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

    fn update_bottom_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        const BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(78.0, 20.0);

        ui.add_space(5.0);

        ui.horizontal(|ui|{
            ui.label("Selected item:");

            if let Some(x) = &self.selected_item {
                if let Some(x) = x.file_name() {
                    if let Some(x) = x.to_str() {
                        ui.colored_label(ui.style().visuals.selection.bg_fill, x);
                    }
                }
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let _ = ui.add_sized(BUTTON_SIZE, egui::Button::new("Open"));
            ui.add_space(ctx.style().spacing.item_spacing.y);
            let _ = ui.add_sized(BUTTON_SIZE, egui::Button::new("Abort"));
        });
    }

    fn update_central_panel(&mut self, ui: &mut egui::Ui) {
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
                // Is there a way to write this better?
                let file_name = match path.file_name() {
                    Some(x) => {
                        match x.to_str() {
                            Some(v) => v,
                            _ => continue
                        }
                    },
                    _ => continue
                };

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

                if response.clicked() {
                    self.selected_item = Some(path.clone());
                }

                if response.double_clicked() {
                    if path.is_dir() {
                        let _ = self.load_directory(path);
                        return;
                    }

                    self.selected_item = Some(path.clone());
                    // TODO: Close file explorer
                }
            }

            self.directory_content = data;

            if let Some(dir) = self.create_directory_dialog.update(ui).directory() {
                self.directory_content.push(dir);
            }
        });
    }

    fn update_user_directories(&mut self, ui: &mut egui::Ui) {
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

    fn current_directory(&self) -> Option<&Path> {
        if let Some(x) = self.directory_stack.iter().nth_back(self.directory_offset) {
            return Some(x.as_path())
        }

        None
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
        let paths = fs::read_dir(path)?;

        self.create_directory_dialog.close();

        self.directory_content.clear();

        for path in paths {
            match path {
                Ok(entry) => self.directory_content.push(entry.path()),
                _ => continue
            };
        }

        // TODO: Sort content to display folders first
        // TODO: Implement "Show hidden files and folders" option

        Ok(())
    }
}

struct CreateDirectoryResponse {
    directory: Option<PathBuf>
}

impl CreateDirectoryResponse {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory: Some(directory)
        }
    }

    pub fn new_empty() -> Self {
        Self {
            directory: None
        }
    }

    pub fn directory(&self) -> Option<PathBuf> {
        self.directory.clone()
    }
}

struct CreateDirectoryDialog {
    open: bool,
    init: bool,
    directory: Option<PathBuf>,

    input: String,
    error: Option<String>
}

impl CreateDirectoryDialog {
    pub fn new() -> Self {
        Self {
            open: false,
            init: false,
            directory: None,

            input: String::new(),
            error: None
        }
    }

    pub fn open(&mut self, directory: PathBuf) {
        self.reset();

        self.open = true;
        self.init = true;
        self.directory = Some(directory);
    }

    pub fn close(&mut self) {
        self.reset();
    }

    pub fn update(&mut self, ui: &mut egui::Ui) -> CreateDirectoryResponse {
        if !self.open {
            return CreateDirectoryResponse::new_empty();
        }

        ui.horizontal(|ui| {
            ui.label("🗀");

            let response = ui.text_edit_singleline(&mut self.input);

            if self.init {
                response.scroll_to_me(None);
                response.request_focus();

                self.error = self.validate_input();
                self.init = false;
            }

            if response.changed() {
                self.error = self.validate_input();
            }

            if ui_button(ui, "✔", self.error.is_none()) {
                self.close()
            }

            if ui.button("✖").clicked() {
                self.close();
            }

            if let Some(err) = &self.error {
                ui.label(err);
            }
        });

        CreateDirectoryResponse::new_empty()
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    fn validate_input(&mut self) -> Option<String> {
        if self.input.is_empty() {
            return Some("Name of the folder can not be empty".to_string());
        }

        if let Some(mut x) = self.directory.clone() {
            x.push(self.input.as_str());

            if x.is_dir() {
                return Some("A directory with the name already exists".to_string())
            }
        }
        else {
            // This error should not occur because the validate_input function is only
            // called when the dialog is open and the directory is set.
            // If this error occurs, there is most likely a bug in the code.
            return Some("No directory given".to_string())
        }

        None
    }

    fn reset(&mut self) {
        self.open = false;
        self.init = false;
        self.directory = None;
        self.input.clear();
    }
}

fn ui_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> bool {
    if !enabled {
        let c = ui.style().visuals.widgets.noninteractive.bg_fill;
        let bg_color = egui::Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 100);
        let _ = ui.add(egui::Button::new(text).fill(bg_color));
        return false;
    }

    ui.add(egui::Button::new(text)).clicked()
}

fn ui_button_sized(ui: &mut egui::Ui, size: egui::Vec2, text: &str, enabled: bool) -> bool {
    if !enabled {
        let c = ui.style().visuals.widgets.noninteractive.bg_fill;
        let bg_color = egui::Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 100);
        let _ = ui.add_sized(size, egui::Button::new(text).fill(bg_color));
        return false;
    }

    ui.add_sized(size, egui::Button::new(text)).clicked()
}
