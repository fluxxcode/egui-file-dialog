/// Defines a keybinding used for a specific action inside the file dialog.
#[derive(Debug, Clone)]
pub enum KeyBinding {
    /// If a single key should be used as a keybinding
    Key(egui::Key),
    /// If a keyboard shortcut should be used as a keybinding
    KeyboardShortcut(egui::KeyboardShortcut),
    /// If a pointer button should be used as the keybinding
    PointerButton(egui::PointerButton),
    /// If a text event should be used as the keybinding.
    Text(String),
}

impl KeyBinding {
    /// Creates a new keybinding where a single key is used.
    pub const fn key(key: egui::Key) -> Self {
        Self::Key(key)
    }

    /// Creates a new keybinding where a keyboard shortcut is used.
    pub const fn keyboard_shortcut(modifiers: egui::Modifiers, logical_key: egui::Key) -> Self {
        Self::KeyboardShortcut(egui::KeyboardShortcut {
            modifiers,
            logical_key,
        })
    }

    /// Creates a new keybinding where a pointer button is used.
    pub const fn pointer_button(pointer_button: egui::PointerButton) -> Self {
        Self::PointerButton(pointer_button)
    }

    /// Creates a new keybinding where a text event is used.
    pub const fn text(text: String) -> Self {
        Self::Text(text)
    }

    /// Checks if the keybinding was pressed by the user.
    ///
    /// # Arguments
    ///
    /// * `ignore_if_any_focused` - Determines whether keyboard shortcuts pressed while another
    ///    widget is currently in focus should be ignored.
    ///    In most cases, this should be enabled so that no shortcuts are executed if,
    ///    for example, the search  text field is currently in focus. With the selection
    ///    keybindings, however, it is desired that when they are pressed, the text fields
    ///    lose focus and the keybinding is executed.
    pub fn pressed(&self, ctx: &egui::Context, ignore_if_any_focused: bool) -> bool {
        let any_focused = ctx.memory(egui::Memory::focused).is_some();

        // We want to suppress keyboard input when any other widget like
        // text fields have focus.
        if ignore_if_any_focused && any_focused {
            return false;
        }

        match self {
            Self::Key(k) => ctx.input(|i| i.key_pressed(*k)),
            Self::KeyboardShortcut(s) => ctx.input_mut(|i| i.consume_shortcut(s)),
            Self::PointerButton(b) => ctx.input(|i| i.pointer.button_clicked(*b)),
            Self::Text(s) => ctx.input_mut(|i| {
                // We force to suppress the text events when any other widget has focus
                if any_focused {
                    return false;
                }

                let mut found_item: Option<usize> = None;

                for (i, text) in i
                    .events
                    .iter()
                    .filter_map(|ev| match ev {
                        egui::Event::Text(t) => Some(t),
                        _ => None,
                    })
                    .enumerate()
                {
                    if text == s {
                        found_item = Some(i);
                        break;
                    }
                }

                if let Some(index) = found_item {
                    i.events.remove(index);
                    return true;
                }

                false
            }),
        }
    }
}

/// Stores the keybindings used for the file dialog.
#[derive(Debug, Clone)]
pub struct FileDialogKeyBindings {
    /// Shortcut to submit the current action or enter the currently selected directory
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
    pub new_folder: Vec<KeyBinding>,
    /// Shortcut to text edit the current path
    pub edit_path: Vec<KeyBinding>,
    /// Shortcut to switch to the home directory and text edit the current path
    pub home_edit_path: Vec<KeyBinding>,
    /// Shortcut to move the selection one item up
    pub selection_up: Vec<KeyBinding>,
    /// Shortcut to move the selection one item down
    pub selection_down: Vec<KeyBinding>,
    /// Shortcut to select every item when the dialog is in `DialogMode::SelectMultiple` mode
    pub select_all: Vec<KeyBinding>,
}

impl FileDialogKeyBindings {
    /// Checks whether any of the given keybindings is pressed.
    pub fn any_pressed(
        ctx: &egui::Context,
        keybindings: &Vec<KeyBinding>,
        suppress_if_any_focused: bool,
    ) -> bool {
        for keybinding in keybindings {
            if keybinding.pressed(ctx, suppress_if_any_focused) {
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
                KeyBinding::key(Key::Backspace),
            ],
            forward: vec![
                KeyBinding::pointer_button(PointerButton::Extra2),
                KeyBinding::keyboard_shortcut(Modifiers::ALT, Key::ArrowRight),
            ],
            reload: vec![KeyBinding::key(egui::Key::F5)],
            new_folder: vec![KeyBinding::keyboard_shortcut(Modifiers::COMMAND, Key::N)],
            edit_path: vec![KeyBinding::key(Key::Slash)],
            home_edit_path: vec![
                KeyBinding::keyboard_shortcut(Modifiers::SHIFT, egui::Key::Backtick),
                KeyBinding::text("~".to_string()),
            ],
            selection_up: vec![KeyBinding::key(Key::ArrowUp)],
            selection_down: vec![KeyBinding::key(Key::ArrowDown)],
            select_all: vec![KeyBinding::keyboard_shortcut(Modifiers::COMMAND, Key::A)],
        }
    }
}
