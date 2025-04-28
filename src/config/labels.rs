/// Contains the text labels that the file dialog uses.
///
/// This is used to enable multiple language support.
///
/// # Example
///
/// The following example shows how the default title of the dialog can be displayed
/// in German instead of English.
///
/// ```
/// use egui_file_dialog::{FileDialog, FileDialogLabels};
///
/// let labels_german = FileDialogLabels {
///     title_select_directory: "📁 Ordner Öffnen".to_string(),
///     title_select_file: "📂 Datei Öffnen".to_string(),
///     title_save_file: "📥 Datei Speichern".to_string(),
///     ..Default::default()
/// };
///
/// let file_dialog = FileDialog::new().labels(labels_german);
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileDialogLabels {
    // ------------------------------------------------------------------------
    // General:
    /// The default window title used when the dialog is in `DialogMode::SelectDirectory` mode.
    pub title_select_directory: String,
    /// The default window title used when the dialog is in `DialogMode::SelectFile` mode.
    pub title_select_file: String,
    /// The default window title used when the dialog is in `DialogMode::SelectMultiple` mode.
    pub title_select_multiple: String,
    /// The default window title used when the dialog is in `DialogMode::SaveFile` mode.
    pub title_save_file: String,

    /// Text displayed in the buttons to cancel the current action.
    pub cancel: String,
    /// Text displayed in the buttons to overwrite something, such as a file.
    pub overwrite: String,

    // ------------------------------------------------------------------------
    // Top panel:
    /// Text used for the option to reload the file dialog.
    pub reload: String,
    /// Text used for the option to open the working directory.
    pub working_directory: String,
    /// Text used for the option to show or hide hidden files and folders.
    pub show_hidden: String,
    /// Text used for the option to show or hide system files.
    pub show_system_files: String,

    // ------------------------------------------------------------------------
    // Left panel:
    /// Heading of the "Pinned" sections in the left panel
    pub heading_pinned: String,
    /// Heading of the "Places" section in the left panel
    pub heading_places: String,
    /// Heading of the "Devices" section in the left panel
    pub heading_devices: String,
    /// Heading of the "Removable Devices" section in the left panel
    pub heading_removable_devices: String,

    /// Name of the home directory
    pub home_dir: String,
    /// Name of the desktop directory
    pub desktop_dir: String,
    /// Name of the documents directory
    pub documents_dir: String,
    /// Name of the downloads directory
    pub downloads_dir: String,
    /// Name of the audio directory
    pub audio_dir: String,
    /// Name of the pictures directory
    pub pictures_dir: String,
    /// Name of the videos directory
    pub videos_dir: String,

    // ------------------------------------------------------------------------
    // Central panel:
    /// Text used for the option to pin a folder.
    pub pin_folder: String,
    /// Text used for the option to unpin a folder.
    pub unpin_folder: String,
    /// Text used for the option to rename a pinned folder.
    pub rename_pinned_folder: String,
    /// Text used for the file name column.
    pub file_name_header: String,
    /// Text used for the file size column.
    pub file_size_header: String,
    /// Text used for the created date column.
    pub created_date_header: String,
    /// Text used for the modified date column.
    pub modified_date_header: String,

    // ------------------------------------------------------------------------
    // Bottom panel:
    /// Text that appears in front of the selected folder preview in the bottom panel.
    pub selected_directory: String,
    /// Text that appears in front of the selected file preview in the bottom panel.
    pub selected_file: String,
    /// Text that appears in front of the selected items preview in the bottom panel.
    pub selected_items: String,
    /// Text that appears in front of the file name input in the bottom panel.
    pub file_name: String,
    /// Text displayed in the file filter dropdown for the "All Files" option.
    pub file_filter_all_files: String,
    /// Text displayed in the save extension dropdown for the "Any" option.
    pub save_extension_any: String,

    /// Button text to open the selected item.
    pub open_button: String,
    /// Button text to save the file.
    pub save_button: String,
    /// Button text to cancel the dialog.
    pub cancel_button: String,

    // ------------------------------------------------------------------------
    // Modal windows:
    /// Text displayed after the path within the modal to overwrite the selected file.
    pub overwrite_file_modal_text: String,

    // ------------------------------------------------------------------------
    // Error message:
    /// Error if no folder name was specified.
    pub err_empty_folder_name: String,
    /// Error if no file name was specified.
    pub err_empty_file_name: String,
    /// Error if the directory already exists.
    pub err_directory_exists: String,
    /// Error if the file already exists.
    pub err_file_exists: String,
}

impl Default for FileDialogLabels {
    /// Creates a new object with the default english labels.
    fn default() -> Self {
        Self {
            title_select_directory: "📁 Select Folder".to_string(),
            title_select_file: "📂 Open File".to_string(),
            title_select_multiple: "🗐 Select Multiple".to_string(),
            title_save_file: "📥 Save File".to_string(),

            cancel: "Cancel".to_string(),
            overwrite: "Overwrite".to_string(),

            reload: "⟲  Reload".to_string(),
            working_directory: "↗  Go to working directory".to_string(),
            show_hidden: " Show hidden".to_string(),
            show_system_files: " Show system files".to_string(),

            heading_pinned: "Pinned".to_string(),
            heading_places: "Places".to_string(),
            heading_devices: "Devices".to_string(),
            heading_removable_devices: "Removable Devices".to_string(),

            home_dir: "🏠  Home".to_string(),
            desktop_dir: "🖵  Desktop".to_string(),
            documents_dir: "🗐  Documents".to_string(),
            downloads_dir: "📥  Downloads".to_string(),
            audio_dir: "🎵  Audio".to_string(),
            pictures_dir: "🖼  Pictures".to_string(),
            videos_dir: "🎞  Videos".to_string(),

            pin_folder: "📌 Pin".to_string(),
            unpin_folder: "✖ Unpin".to_string(),
            rename_pinned_folder: "✏ Rename".to_string(),

            file_name_header: "Name".to_string(),
            file_size_header: "File Size".to_string(),
            created_date_header: "Created".to_string(),
            modified_date_header: "Modified".to_string(),
            selected_directory: "Selected directory:".to_string(),
            selected_file: "Selected file:".to_string(),
            selected_items: "Selected items:".to_string(),
            file_name: "File name:".to_string(),
            file_filter_all_files: "All Files".to_string(),
            save_extension_any: "Any".to_string(),

            open_button: "🗀  Open".to_string(),
            save_button: "📥  Save".to_string(),
            cancel_button: "🚫 Cancel".to_string(),

            overwrite_file_modal_text: "already exists. Do you want to overwrite it?".to_string(),

            err_empty_folder_name: "Name of the folder cannot be empty".to_string(),
            err_empty_file_name: "The file name cannot be empty".to_string(),
            err_directory_exists: "A directory with the name already exists".to_string(),
            err_file_exists: "A file with the name already exists".to_string(),
        }
    }
}
