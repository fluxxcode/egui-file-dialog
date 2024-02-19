use std::path::{Path, PathBuf};
use std::{fs, io};

/// Contains the metadata of a directory item.
/// This struct is mainly there so that the metadata can be loaded once and not that
/// a request has to be sent to the OS every frame using, for example, `path.is_file()`.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    path: PathBuf,
    is_directory: bool,
    is_system_file: bool,
}

impl DirectoryEntry {
    /// Creates a new directory entry from a path
    pub fn from_path(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            is_directory: path.is_dir(),
            is_system_file: !path.is_dir() && !path.is_file(),
        }
    }

    /// Returns true if the item is a directory.
    /// False is returned if the item is a file or the path did not exist when the
    /// DirectoryEntry object was created.
    pub fn is_dir(&self) -> bool {
        self.is_directory
    }

    /// Returns true if the item is a file.
    /// False is returned if the item is a directory or the path did not exist when the
    /// DirectoryEntry object was created.
    pub fn is_file(&self) -> bool {
        !self.is_directory
    }

    /// Returns true if the item is a system file.
    pub fn is_system_file(&self) -> bool {
        self.is_system_file
    }

    /// Returns the path of the directory item.
    pub fn to_path_buf(&self) -> PathBuf {
        self.path.clone()
    }

    /// Returns the file name of the directory item.
    pub fn file_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| {
                // Make sure the root directories like "/" or "C:\\" are displayed correctly
                if self.path.iter().count() == 1 {
                    return self.path.to_str().unwrap_or_default();
                }

                ""
            })
    }
}

/// Contains the content of a directory.
#[derive(Default)]
pub struct DirectoryContent {
    content: Vec<DirectoryEntry>,
}

impl DirectoryContent {
    /// Create a new object with empty content
    pub fn new() -> Self {
        Self { content: vec![] }
    }

    /// Create a new DirectoryContent object and loads the contents of the given path.
    /// Use include_files to include or exclude files in the content list.
    pub fn from_path(path: &Path, include_files: bool) -> io::Result<Self> {
        Ok(Self {
            content: load_directory(path, include_files)?,
        })
    }

    /// Very simple wrapper methods of the contents .iter() method.
    /// No trait is implemented since this is currently only used internal.
    pub fn iter(&self) -> std::slice::Iter<'_, DirectoryEntry> {
        self.content.iter()
    }

    /// Pushes a new item to the content.
    pub fn push(&mut self, item: DirectoryEntry) {
        self.content.push(item);
    }
}

/// Loads the contents of the given directory.
fn load_directory(path: &Path, include_files: bool) -> io::Result<Vec<DirectoryEntry>> {
    let paths = fs::read_dir(path)?;

    let mut result: Vec<DirectoryEntry> = Vec::new();
    for path in paths {
        match path {
            Ok(entry) => {
                let entry = DirectoryEntry::from_path(entry.path().as_path());

                if entry.is_system_file() {
                    continue;
                }

                if !include_files && entry.is_file() {
                    continue;
                }

                result.push(entry);
            }
            Err(_) => continue,
        };
    }

    result.sort_by(|a, b| match a.is_dir() == b.is_dir() {
        true => a.file_name().cmp(b.file_name()),
        false => match a.is_dir() {
            true => std::cmp::Ordering::Less,
            false => std::cmp::Ordering::Greater,
        },
    });

    // TODO: Implement "Show hidden files and folders" option

    Ok(result)
}
