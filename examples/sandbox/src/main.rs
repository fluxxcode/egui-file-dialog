use eframe::egui;
use egui_file_dialog::FileDialog;

struct MyApp {
    file_explorer: FileDialog
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let mut obj = Self {
            file_explorer: FileDialog::new()
        };
        obj.file_explorer.open();
        obj
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui application");
            egui::widgets::global_dark_light_mode_buttons(ui);

            self.file_explorer.update(ctx);
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default() .with_inner_size([1080.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui application",
        options,
        Box::new(|ctx| Box::new(MyApp::new(ctx)))
    )
}
