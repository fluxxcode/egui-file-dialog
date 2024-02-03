/// Adds a dynamically sized button to the UI that can be enabled or disabled.
/// Returns true if the button is clicked. Otherwise None is returned.
pub fn button_enabled_disabled(ui: &mut egui::Ui, text: &str, enabled: bool) -> bool {
    if !enabled {
        let button = egui::Button::new(text)
            .stroke(egui::Stroke::NONE)
            .fill(get_disabled_fill_color(ui));

        let _ = ui.add(button);

        return false;
    }

    ui.add(egui::Button::new(text)).clicked()
}

/// Adds a fixed sized button to the UI that can be enabled or disabled.
/// Returns true if the button is clicked. Otherwise None is returned.
pub fn button_sized_enabled_disabled(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    text: &str,
    enabled: bool,
) -> bool {
    if !enabled {
        let button = egui::Button::new(text)
            .stroke(egui::Stroke::NONE)
            .fill(get_disabled_fill_color(ui));

        let _ = ui.add_sized(size, button);

        return false;
    }

    ui.add_sized(size, egui::Button::new(text)).clicked()
}

/// Returns the fill color of disabled buttons
#[inline]
fn get_disabled_fill_color(ui: &egui::Ui) -> egui::Color32 {
    let c = ui.style().visuals.widgets.noninteractive.bg_fill;
    egui::Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 100)
}
