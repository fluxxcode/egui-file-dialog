use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::FileDialogConfig;

/// Contains the metadata of a directory item.
/// This struct is mainly there so that the metadata can be loaded once and not that
/// a request has to be sent to the OS every frame using, for example, `path.is_file()`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct DirectoryEntry {
    path: PathBuf,
    is_directory: bool,
    is_system_file: bool,
    icon: String,
}

impl DirectoryEntry {
    /// Creates a new directory entry from a path
    pub fn from_path(config: &FileDialogConfig, path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            is_directory: path.is_dir(),
            is_system_file: !path.is_dir() && !path.is_file(),
            icon: gen_path_icon(config, path),
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

    /// Returns the icon of the directory item.
    pub fn icon(&self) -> &str {
        &self.icon
    }

    /// Returns the path of the directory item.
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// Clones the path of the directory item.
    pub fn to_path_buf(&self) -> PathBuf {
        self.path.clone()
    }

    /// Returns the file name of the directory item.
    pub fn file_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| {
                // Make sure the root directories like ["C:", "\"] and ["\\?\C:", "\"] are
                // displayed correctly
                #[cfg(windows)]
                if self.path.components().count() == 2 {
                    let path = self
                        .path
                        .iter()
                        .nth(0)
                        .and_then(|seg| seg.to_str())
                        .unwrap_or_default();

                    // Skip path namespace prefix if present, for example: "\\?\C:"
                    if path.contains(r"\\?\") {
                        return path.get(4..).unwrap_or(path);
                    }

                    return path;
                }

                // Make sure the root directory "/" is displayed correctly
                #[cfg(not(windows))]
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
    pub fn from_path(
        config: &FileDialogConfig,
        path: &Path,
        include_files: bool,
    ) -> io::Result<Self> {
        Ok(Self {
            content: load_directory(config, path, include_files)?,
        })
    }

    /// Checks if the given directory entry is visible with the applied filters.
    fn is_entry_visible(dir_entry: &DirectoryEntry, search_value: &str) -> bool {
        if !search_value.is_empty()
            && !dir_entry
                .file_name()
                .to_lowercase()
                .contains(&search_value.to_lowercase())
        {
            return false;
        }

        true
    }

    pub fn filtered_iter<'s>(
        &'s self,
        search_value: &'s str,
    ) -> impl Iterator<Item = &DirectoryEntry> + 's {
        self.content
            .iter()
            .filter(|p| Self::is_entry_visible(p, search_value))
    }

    /// Returns the number of elements inside the directory.
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Pushes a new item to the content.
    pub fn push(&mut self, item: DirectoryEntry) {
        self.content.push(item);
    }

    /// Clears the items inside the directory.
    pub fn clear(&mut self) {
        self.content.clear();
    }
}

/// Loads the contents of the given directory.
fn load_directory(
    config: &FileDialogConfig,
    path: &Path,
    include_files: bool,
) -> io::Result<Vec<DirectoryEntry>> {
    let paths = fs::read_dir(path)?;

    let mut result: Vec<DirectoryEntry> = Vec::new();
    for path in paths {
        match path {
            Ok(entry) => {
                let entry = DirectoryEntry::from_path(config, entry.path().as_path());

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

/// Generates the icon for the specific path.
/// The default icon configuration is taken into account, as well as any configured file icon filters.
fn gen_path_icon(config: &FileDialogConfig, path: &Path) -> String {
    for def in &config.file_icon_filters {
        if (def.filter)(path) {
            return def.icon.clone();
        }
    }

    match path.is_dir() {
        true => config.default_folder_icon.clone(),
        false => config.default_file_icon.clone(),
    }
}
