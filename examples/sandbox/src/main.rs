use eframe::egui;
use egui_file_explorer::FileExplorer;

struct MyApp {
    file_explorer: FileExplorer
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            file_explorer: FileExplorer::new()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui application");

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
        Box::new(|_| Box::new(MyApp::new()))
    )
}
