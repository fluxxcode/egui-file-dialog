/// Defines a keybinding used for a specific action inside the file dialog.
#[derive(Debug, Clone)]
pub enum KeyBinding {
    /// If pointer buttons should be used as the keybinding
    PointerButton(egui::PointerButton),
    /// If a keyboard shortcut should be used as a keybinding
    KeyboardShortcut(egui::KeyboardShortcut),
}

impl KeyBinding {
    pub fn pointer_button(pointer_button: egui::PointerButton) -> Self {
        Self::PointerButton(pointer_button)
    }

    pub fn keyboard_shortcut(modifiers: egui::Modifiers, logical_key: egui::Key) -> Self {
        Self::KeyboardShortcut(egui::KeyboardShortcut {
            modifiers,
            logical_key,
        })
    }

    pub fn key(logical_key: egui::Key) -> Self {
        Self::KeyboardShortcut(egui::KeyboardShortcut {
            modifiers: egui::Modifiers::NONE,
            logical_key,
        })
    }
}

/// Stores the keybindings used for the file dialog.
#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub open_previous_directory: Vec<KeyBinding>,
    pub search: Vec<KeyBinding>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        use egui::{Key, PointerButton};

        Self {
            open_previous_directory: vec![KeyBinding::pointer_button(PointerButton::Extra1)],
            search: vec![
                KeyBinding::key(Key::A),
                KeyBinding::key(Key::B),
                KeyBinding::key(Key::C),
                KeyBinding::key(Key::D),
                KeyBinding::key(Key::E),
                KeyBinding::key(Key::F),
                KeyBinding::key(Key::G),
                KeyBinding::key(Key::H),
                KeyBinding::key(Key::I),
                KeyBinding::key(Key::J),
                KeyBinding::key(Key::K),
                KeyBinding::key(Key::L),
                KeyBinding::key(Key::M),
                KeyBinding::key(Key::N),
                KeyBinding::key(Key::O),
                KeyBinding::key(Key::P),
                KeyBinding::key(Key::Q),
                KeyBinding::key(Key::R),
                KeyBinding::key(Key::S),
                KeyBinding::key(Key::T),
                KeyBinding::key(Key::U),
                KeyBinding::key(Key::V),
                KeyBinding::key(Key::W),
                KeyBinding::key(Key::X),
                KeyBinding::key(Key::Y),
                KeyBinding::key(Key::Z),
            ],
        }
    }
}
