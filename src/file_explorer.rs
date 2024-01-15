use std::{fs, io};
use std::path::{Path, PathBuf};

use directories::UserDirs;

pub struct FileExplorer {
    current_directory: PathBuf,
    directory_content: Vec<fs::DirEntry>,
    user_directories: Option<UserDirs>,
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
            current_directory: PathBuf::from("./"),
            directory_content: vec![],
            user_directories: UserDirs::new(),
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
                    // TODO: Set scroll area width to available width
                    egui::ScrollArea::horizontal()
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // NOTE: These are currently only hardcoded test values!
                                let _ = ui.add_sized(egui::Vec2::new(0.0, ui.available_height()),
                                                    egui::Button::new("home"));
                                ui.label(">");

                                let _ = ui.add_sized(egui::Vec2::new(0.0, ui.available_height()),
                                                    egui::Button::new("user"));
                                ui.label(">");

                                let _ = ui.add_sized(egui::Vec2::new(0.0, ui.available_height()),
                                                    egui::Button::new("documents"));
                                ui.label(">");

                                let _ = ui.add_sized(egui::Vec2::new(0.0, ui.available_height()),
                                                    egui::Button::new("projects"));
                        });
                    });
                });

            egui::Frame::default()
                .stroke(egui::Stroke::new(2.0, ctx.style().visuals.window_stroke.color))
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .rounding(egui::Rounding::from(5.0))
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        ui.add_space(ctx.style().spacing.item_spacing.y);
                        ui.label("🔍");
                        ui.add_sized(egui::Vec2::new(120.0, ui.available_height()),
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
            ui.label("Selected item: Desktop");

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
            for item in self.directory_content.iter() {
                let path = item.path();

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

                let _ = ui.selectable_label(false, format!("{} {}", icon, file_name));
            }
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

        self.current_directory = PathBuf::from(path);
        self.directory_content.clear();

        for path in paths {
            match path {
                Ok(entry) => self.directory_content.push(entry),
                _ => continue
            };
        }

        Ok(())
    }
}
