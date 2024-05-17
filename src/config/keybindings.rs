/// Defines a keybinding used for a specific action inside the file dialog.
#[derive(Debug, Clone)]
pub enum KeyBinding {
    /// If a single key should be used as a keybinding
    Key(egui::Key),
    /// If a keyboard shortcut should be used as a keybinding
    KeyboardShortcut(egui::KeyboardShortcut),
    /// If a pointer button should be used as the keybinding
    PointerButton(egui::PointerButton),
}

impl KeyBinding {
    /// Creates a new keybinding where a single key is used.
    pub fn key(key: egui::Key) -> Self {
        Self::Key(key)
    }

    /// Creates a new keybinding where a keyboard shortcut is used.
    pub fn keyboard_shortcut(modifiers: egui::Modifiers, logical_key: egui::Key) -> Self {
        Self::KeyboardShortcut(egui::KeyboardShortcut {
            modifiers,
            logical_key,
        })
    }

    /// Creates a new keybinding where a pointer button is used.
    pub fn pointer_button(pointer_button: egui::PointerButton) -> Self {
        Self::PointerButton(pointer_button)
    }

    /// Checks if the keybinding was pressed by the user.
    pub fn pressed(&self, ctx: &egui::Context) -> bool {
        match self {
            KeyBinding::Key(k) => ctx.input(|i| i.key_pressed(*k)),
            KeyBinding::KeyboardShortcut(s) => ctx.input_mut(|i| i.consume_shortcut(s)),
            KeyBinding::PointerButton(b) => ctx.input(|i| i.pointer.button_clicked(*b)),
        }
    }
}

/// Stores the keybindings used for the file dialog.
#[derive(Debug, Clone)]
pub struct FileDialogKeyBindings {
    /// Shortcut to submit the current action
    pub submit: Vec<KeyBinding>,
    /// Shortcut to cancel the current action
    pub cancel: Vec<KeyBinding>,
    /// Shortcut to open the parent directory
    pub parent: Vec<KeyBinding>,
    /// Shortcut to go back
    pub back: Vec<KeyBinding>,
    /// Shortcut to go forward
    pub forward: Vec<KeyBinding>,
    /// Shortcut to reload the file dialog
    pub reload: Vec<KeyBinding>,
    /// Shortcut to open the dialog to create a new folder
    pub create_new_folder: Vec<KeyBinding>,
    /// Shortcut to text edit the current path
    pub edit_path: Vec<KeyBinding>,
}

impl FileDialogKeyBindings {
    /// Checks wether any of the given keybindings is pressed.
    pub fn any_pressed(ctx: &egui::Context, keybindings: &Vec<KeyBinding>) -> bool {
        for keybinding in keybindings {
            if keybinding.pressed(ctx) {
                return true;
            }
        }

        false
    }
}

impl Default for FileDialogKeyBindings {
    fn default() -> Self {
        use egui::{Key, Modifiers, PointerButton};

        Self {
            submit: vec![KeyBinding::key(Key::Enter)],
            cancel: vec![KeyBinding::key(Key::Escape)],
            parent: vec![KeyBinding::keyboard_shortcut(Modifiers::ALT, Key::ArrowUp)],
            back: vec![
                KeyBinding::pointer_button(PointerButton::Extra1),
                KeyBinding::keyboard_shortcut(Modifiers::ALT, Key::ArrowLeft),
            ],
            forward: vec![
                KeyBinding::pointer_button(PointerButton::Extra2),
                KeyBinding::keyboard_shortcut(Modifiers::ALT, Key::ArrowRight),
            ],
            reload: vec![KeyBinding::key(egui::Key::F5)],
            create_new_folder: vec![KeyBinding::keyboard_shortcut(Modifiers::CTRL, Key::N)],
            edit_path: vec![
                KeyBinding::key(Key::Slash),
                KeyBinding::key(Key::Backslash),
            ]
        }
    }
}
