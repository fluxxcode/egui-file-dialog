use image::RgbaImage;

#[derive(Debug, Default)]
pub struct MetaData {
    pub file_name: String,
    pub dimensions: Option<(usize, usize)>,
    pub scaled_dimensions: Option<(usize, usize)>,
    pub preview_image_bytes: Option<RgbaImage>,
    pub preview_text: Option<String>,
    pub file_size: Option<String>,
    pub file_type: Option<String>,
    pub file_modified: Option<String>,
    pub file_created: Option<String>,
}

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
        format!("{} B", bytes)
    }
}

pub fn format_pixels(pixels: usize) -> String {
    const K: usize = 1_000;
    const M: usize = K * 1_000;
    if pixels >= K {
        format!("{:.2} MPx", pixels as f64 / M as f64)
    } else {
        format!("{} Px", pixels)
    }
}
