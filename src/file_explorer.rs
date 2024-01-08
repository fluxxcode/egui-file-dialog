
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
                egui::TopBottomPanel::top("fe_top_panel")
                    .resizable(false)
                    .min_height(32.0)
                    .show_inside(ui, |ui| {
                        ui.label("This is the top panel!")
                    });

                egui::SidePanel::left("fe_left_panel")
                    .resizable(true)
                    .default_width(150.0)
                    .width_range(80.0..=300.0)
                    .show_inside(ui, |ui| {
                        ui.label("This is the left panel!")
                    });

                egui::TopBottomPanel::bottom("fe_bottom_panel")
                    .resizable(false)
                    .min_height(60.0)
                    .show_inside(ui, |ui| {
                        ui.label("This is the bottom panel!")
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.label("This is the central panel!")
                });
            });
    }
}
