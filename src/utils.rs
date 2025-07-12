use crate::DirectoryEntry;
use chrono::{DateTime, Local};
use std::time::SystemTime;

/// Calculates the width of a single char.
fn calc_char_width(ui: &egui::Ui, char: char) -> f32 {
    ui.fonts(|f| f.glyph_width(&egui::TextStyle::Body.resolve(ui.style()), char))
}

/// Calculates the width of the specified text using the current font configuration.
/// Does not take new lines or text breaks into account!
pub fn calc_text_width(ui: &egui::Ui, text: &str) -> f32 {
    let mut width = 0.0;

    for char in text.chars() {
        width += calc_char_width(ui, char);
    }

    width
}

/// Truncates a date to a specified maximum length `max_length`
/// Returns the truncated date as a string
pub fn truncate_date(ui: &egui::Ui, date: SystemTime, max_length: f32) -> String {
    let date: DateTime<Local> = date.into();
    let today = Local::now().date_naive(); // NaiveDate for today
    let yesterday = today.pred_opt().map_or(today, |day| day); // NaiveDate for yesterday

    let text = if date.date_naive() == today {
        date.format("Today, %H:%M").to_string()
    } else if date.date_naive() == yesterday {
        date.format("Yesterday, %H:%M").to_string()
    } else {
        date.format("%d.%m.%Y, %H:%M").to_string()
    };

    let text_width = calc_text_width(ui, &text);

    if max_length <= text_width {
        if date.date_naive() == today {
            date.format("%H:%M").to_string()
        } else if date.date_naive() == yesterday {
            "Yesterday".to_string()
        } else {
            date.format("%d.%m.%y").to_string()
        }
    } else {
        text
    }
}

/// Truncates a date to a specified maximum length `max_length`
/// Returns the truncated filename as a string
pub fn truncate_filename(ui: &egui::Ui, item: &DirectoryEntry, max_length: f32) -> String {
    const TRUNCATE_STR: &str = "...";

    let path = item.as_path();

    let file_stem = if path.is_file() {
        path.file_stem().and_then(|f| f.to_str()).unwrap_or("")
    } else {
        item.file_name()
    };

    let extension = if path.is_file() {
        path.extension().map_or(String::new(), |ext| {
            format!(".{}", ext.to_str().unwrap_or(""))
        })
    } else {
        String::new()
    };

    let extension_width = calc_text_width(ui, &extension);
    let reserved = extension_width + calc_text_width(ui, TRUNCATE_STR);

    if max_length <= reserved {
        return format!("{TRUNCATE_STR}{extension}");
    }

    let mut width = reserved;
    let mut front = String::new();
    let mut back = String::new();

    for (i, char) in file_stem.chars().enumerate() {
        let w = calc_char_width(ui, char);

        if width + w > max_length {
            break;
        }

        front.push(char);
        width += w;

        let back_index = file_stem.len() - i - 1;

        if back_index <= i {
            break;
        }

        if let Some(char) = file_stem.chars().nth(back_index) {
            let w = calc_char_width(ui, char);

            if width + w > max_length {
                break;
            }

            back.push(char);
            width += w;
        }
    }

    format!(
        "{front}{TRUNCATE_STR}{}{extension}",
        back.chars().rev().collect::<String>()
    )
}

/// Formats a file size (in bytes) into a human-readable string (e.g., KB, MB).
///
/// # Arguments
/// - `bytes`: The file size in bytes.
///
/// # Returns
/// A string representing the file size in an appropriate unit.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
