
pub fn ui_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> bool {
    if !enabled {
        let button = egui::Button::new(text)
            .stroke(egui::Stroke::NONE)
            .fill(get_disabled_fill_color(ui));

        let _ = ui.add(button);

        return false;
    }

    ui.add(egui::Button::new(text)).clicked()
}

pub fn ui_button_sized(ui: &mut egui::Ui, size: egui::Vec2, text: &str, enabled: bool) -> bool {
    if !enabled {
        let button = egui::Button::new(text)
            .stroke(egui::Stroke::NONE)
            .fill(get_disabled_fill_color(ui));

        let _ = ui.add_sized(size, button);

        return false;
    }

    ui.add_sized(size, egui::Button::new(text)).clicked()
}

#[inline]
fn get_disabled_fill_color(ui: &egui::Ui) -> egui::Color32 {
    let c = ui.style().visuals.widgets.noninteractive.bg_fill;
    egui::Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 100)
}