use std::{fs, io};
use std::path::{Path, PathBuf};

use directories::UserDirs;

pub struct FileExplorer {
    user_directories: Option<UserDirs>,
    current_directory: PathBuf,
    directory_content: Vec<PathBuf>,
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
            current_directory: PathBuf::from("./"),
            directory_content: vec![],
            selected_item: None,
            search_value: String::new() }
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
                    .width_range(80.0..=300.0)
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
            let _ = ui.add_sized(NAV_BUTTON_SIZE, egui::Button::new("<-"));
            let _ = ui.add_sized(NAV_BUTTON_SIZE, egui::Button::new("<"));
            let _ = ui.add_sized(NAV_BUTTON_SIZE, egui::Button::new(">"));
            let _ = ui.add_sized(NAV_BUTTON_SIZE, egui::Button::new("+"));

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

                                let data = std::mem::take(&mut self.current_directory);
                                let mut path = PathBuf::new();

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

                                self.current_directory = data;
                            });
                        });
                });

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

        let _ = ui.selectable_label(false, "🖴  (C:)");
        let _ = ui.selectable_label(false, "🖴  Toshiba(D:)");
        let _ = ui.selectable_label(false, "🖴  Samsung 980..(E:)");
        let _ = ui.selectable_label(false, "🖴  (F:)");
    }

    fn update_bottom_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        const BUTTON_SIZE: egui::Vec2 = egui::Vec2::new(78.0, 20.0);

        ui.add_space(5.0);

        ui.horizontal(|ui|{
            let mut selected = String::from("Selected item:");

            if let Some(x) = &self.selected_item {
                if let Some(x) = x.file_name() {
                    if let Some(x) = x.to_str() {
                        selected = format!("Selected item: {}", x);
                    }
                }
            }

            ui.label(selected);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let _ = ui.add_sized(BUTTON_SIZE, egui::Button::new("Open"));
                ui.add_space(ctx.style().spacing.item_spacing.y);
                let _ = ui.add_sized(BUTTON_SIZE, egui::Button::new("Abort"));
            });
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
                let icon = match path.is_dir() {
                    true => "🗀",
                    _ => "🖹"
                };

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
        });
    }

    fn update_user_directories(&mut self, ui: &mut egui::Ui) {
        if let Some(dirs) = self.user_directories.clone() {
            ui.label("Places");

            if ui.selectable_label(self.current_directory == dirs.home_dir(),
                                   "🏠  Home").clicked() {
                let _ = self.load_directory(dirs.home_dir());
            }

            if let Some(path) = dirs.desktop_dir() {
                if ui.selectable_label(self.current_directory == path, "🖵  Desktop").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.document_dir() {
                if ui.selectable_label(self.current_directory == path, "🗐  Documents").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.download_dir() {
                if ui.selectable_label(self.current_directory == path, "📥  Downloads").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.audio_dir() {
                if ui.selectable_label(self.current_directory == path, "🎵  Audio").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.picture_dir() {
                if ui.selectable_label(self.current_directory == path, "🖼  Pictures").clicked() {
                    let _ = self.load_directory(path);
                }
            }
            if let Some(path) = dirs.video_dir() {
                if ui.selectable_label(self.current_directory == path, "🎞  Videos").clicked() {
                    let _ = self.load_directory(path);
                }
            }
        }
    }

    fn load_directory(&mut self, path: &Path) -> io::Result<()> {
        let paths = fs::read_dir(path)?;

        self.current_directory = fs::canonicalize(path)?;
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
