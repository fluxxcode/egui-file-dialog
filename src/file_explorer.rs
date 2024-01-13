
pub struct FileExplorer {
    search_value: String
}

impl FileExplorer {
    pub fn new() -> Self {
        FileExplorer { search_value: String::new() }
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
                        self.update_left_panel(ctx, ui);
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
                .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
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
        ui.label("Drives:");

        // NOTE: These are currently only hardcoded test values!
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0), egui::Button::new("(C:)"));
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0), egui::Button::new("Toshiba(D:)"));
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0),
                     egui::Button::new("Samsung 980..(E:)"));
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0), egui::Button::new("(F:)"));

        ui.add_space(ctx.style().spacing.item_spacing.y * 4.0);

        ui.label("User:");

        // NOTE: These are currently only hardcoded test values!
        // TODO: Align button text to the left!
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0),
                     egui::Button::new("🗀  Desktop"));
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0),
                     egui::Button::new("🗀  Documents"));
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0),
                     egui::Button::new("🗀  Downloads"));
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0),
                     egui::Button::new("🗀  Music"));
        ui.add_sized(egui::Vec2::new(ui.available_width(), 0.0),
                     egui::Button::new("🗀  Pictures"));
    }

    fn update_bottom_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("This is the bottom panel!");
    }

    fn update_central_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("This is the central panel!");
    }

}
