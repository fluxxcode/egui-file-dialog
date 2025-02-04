use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::data::{Disks, Metadata, UserDirectories};

/// An abstraction over the host system, allowing the file dialog to be used to browse e.g. in
/// memory filesystems.
///
/// # Examples
///
/// ```
/// use egui_file_dialog::{FileDialog, FileSystem, UserDirectories, Metadata, Disks};
/// use std::{io, path::{Path, PathBuf}};
///
/// struct MyFileSystem;
///
/// impl FileSystem for MyFileSystem {
///     fn metadata(&self, path: &Path) -> io::Result<Metadata> { todo!() }
///     fn is_dir(&self, path: &Path) -> bool { todo!() }
///     fn is_file(&self, path: &Path) -> bool { todo!() }
///     fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> { todo!() }
///     fn get_disks(&self, canonicalize_paths: bool) -> Disks { todo!() }
///     fn is_path_hidden(&self, path: &Path) -> bool { todo!() }
///     fn create_dir(&self, path: &Path) -> io::Result<()> { todo!() }
///     fn user_dirs(&self, canonicalize_paths: bool) -> Option<UserDirectories> { todo!() }
///     fn current_dir(&self) -> io::Result<PathBuf> { Ok("/".into()) }
/// }
///
/// let dialog = FileDialog::with_file_system(std::sync::Arc::new(MyFileSystem));
///
/// /* Use the file dialog as usual */
/// ```
pub trait FileSystem {
    /// Queries metadata for the given path
    fn metadata(&self, path: &Path) -> io::Result<Metadata>;

    /// Returns true if the path exists and is a directory
    fn is_dir(&self, path: &Path) -> bool;

    /// Returns true if the path exists and is a file
    fn is_file(&self, path: &Path) -> bool;

    /// Gets the children of a directory
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;

    /// List out the disks in the system
    fn get_disks(&self, canonicalize_paths: bool) -> Disks;

    /// Determine if a path is hidden
    fn is_path_hidden(&self, path: &Path) -> bool;

    /// Creates a new directory
    fn create_dir(&self, path: &Path) -> io::Result<()>;

    /// Returns the user directories
    fn user_dirs(&self, canonicalize_paths: bool) -> Option<UserDirectories>;

    /// Get the current working directory
    fn current_dir(&self) -> io::Result<PathBuf>;

    /// Read a short preview of a text file
    fn load_text_file_preview(&self, _path: &Path, _max_chars: usize) -> io::Result<String> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "load_text_file_preview not implemented.".to_string(),
        ))
    }
}

impl std::fmt::Debug for dyn FileSystem + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<FileSystem>")
    }
}

/// Implementation of `FileSystem` using the standard library
pub struct NativeFileSystem;

impl FileSystem for NativeFileSystem {
    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let mut metadata = Metadata::default();

        let md = std::fs::metadata(path)?;
        metadata.size = Some(md.len());
        metadata.last_modified = md.modified().ok();
        metadata.created = md.created().ok();
        metadata.file_type = Some(format!("{:?}", md.file_type()));

        Ok(metadata)
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect())
    }

    fn load_text_file_preview(&self, path: &Path, max_chars: usize) -> io::Result<String> {
        let mut file = std::fs::File::open(path)?;
        let mut chunk = [0; 96]; // Temporary buffer
        let mut buffer = String::new();

        // Add the first chunk to the buffer as text
        let mut total_read = 0;

        // Continue reading if needed
        while total_read < max_chars {
            let bytes_read = file.read(&mut chunk)?;
            if bytes_read == 0 {
                break; // End of file
            }
            let chars_read: String = String::from_utf8(chunk[..bytes_read].to_vec())
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
            total_read += chars_read.len();
            buffer.push_str(&chars_read);
        }

        Ok(buffer.to_string())
    }

    fn get_disks(&self, canonicalize_paths: bool) -> Disks {
        Disks::new_native_disks(canonicalize_paths)
    }

    fn is_path_hidden(&self, path: &Path) -> bool {
        is_path_hidden(path)
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir(path)
    }

    fn user_dirs(&self, canonicalize_paths: bool) -> Option<UserDirectories> {
        if let Some(dirs) = directories::UserDirs::new() {
            return Some(UserDirectories::new(
                UserDirectories::canonicalize(Some(dirs.home_dir()), canonicalize_paths),
                UserDirectories::canonicalize(dirs.audio_dir(), canonicalize_paths),
                UserDirectories::canonicalize(dirs.desktop_dir(), canonicalize_paths),
                UserDirectories::canonicalize(dirs.document_dir(), canonicalize_paths),
                UserDirectories::canonicalize(dirs.download_dir(), canonicalize_paths),
                UserDirectories::canonicalize(dirs.picture_dir(), canonicalize_paths),
                UserDirectories::canonicalize(dirs.video_dir(), canonicalize_paths),
            ));
        }

        None
    }

    fn current_dir(&self) -> io::Result<PathBuf> {
        std::env::current_dir()
    }
}

#[cfg(windows)]
fn is_path_hidden(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;

    std::fs::metadata(path).is_ok_and(|metadata| metadata.file_attributes() & 0x2 > 0)
}

#[cfg(not(windows))]
fn is_path_hidden(path: &Path) -> bool {
    let Some(file_name) = path.file_name() else {
        return false;
    };
    let Some(s) = file_name.to_str() else {
        return false;
    };

    if s.starts_with('.') {
        return true;
    }

    false
}
