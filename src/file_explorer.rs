
pub struct FileExplorer;

impl FileExplorer {
    pub fn new() -> Self {
        FileExplorer { }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        // TODO: Make window title and options configurable
        egui::Window::new("File explorer")
            .default_size([800.0, 500.0])
            .show(&ctx, |ui| {
                ui.style_mut().spacing.window_margin = egui::Margin::symmetric(0.0, 0.0);

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
                        self.update_left_panel(ui);
                    });

                egui::TopBottomPanel::bottom("fe_bottom_panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        self.update_bottom_panel(ui);
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
                .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .rounding(egui::Rounding::from(5.0))
                .show(ui, |ui| {
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
                })
        });

        ui.add_space(ctx.style().spacing.item_spacing.y);
    }

    fn update_left_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("This is the left panel!");
    }

    fn update_bottom_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("This is the bottom panel!");
    }

    fn update_central_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("This is the central panel!");
    }

}
