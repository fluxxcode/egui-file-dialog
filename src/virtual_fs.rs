use std::path::{Path, PathBuf};
use std::io;

use crate::data::Metadata;

/// File system abstraction
pub trait FileSystem {
    /// Queries metadata for the given path
    fn metadata(&self, path: &Path) -> io::Result<Metadata>;

    /// Returns true if the path exists and is a directory
    fn is_dir(&self, path: &Path) -> bool;

    /// Returns true if the path exists and is a file
    fn is_file(&self, path: &Path) -> bool;

    /// Reads
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
}

impl std::fmt::Debug for dyn FileSystem + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<FileSystem>")
    }
}

/// Implementation of FileSystem using the standard library
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
            .into_iter()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect())
    }
}
