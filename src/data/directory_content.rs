use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use std::{fs, io, thread};

use egui::mutex::Mutex;

use crate::config::{FileDialogConfig, FileFilter};

/// Contains the metadata of a directory item.
///
/// This struct is mainly there so that the metadata can be loaded once and not that
/// a request has to be sent to the OS every frame using, for example, `path.is_file()`.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DirectoryEntry {
    path: PathBuf,
    is_directory: bool,
    is_system_file: bool,
    icon: String,
    /// If the item is marked as selected as part of a multi selection.
    pub selected: bool,
}

impl DirectoryEntry {
    /// Creates a new directory entry from a path
    pub fn from_path(config: &FileDialogConfig, path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            is_directory: path.is_dir(),
            is_system_file: !path.is_dir() && !path.is_file(),
            icon: gen_path_icon(config, path),
            selected: false,
        }
    }

    /// Checks if the path of the current directory entry matches the other directory entry.
    pub fn path_eq(&self, other: &Self) -> bool {
        other.as_path() == self.as_path()
    }

    /// Returns true if the item is a directory.
    /// False is returned if the item is a file or the path did not exist when the
    /// `DirectoryEntry` object was created.
    pub const fn is_dir(&self) -> bool {
        self.is_directory
    }

    /// Returns true if the item is a file.
    /// False is returned if the item is a directory or the path did not exist when the
    /// `DirectoryEntry` object was created.
    pub const fn is_file(&self) -> bool {
        !self.is_directory
    }

    /// Returns true if the item is a system file.
    pub const fn is_system_file(&self) -> bool {
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

    /// Returns whether the path this `DirectoryEntry` points to is considered hidden.
    pub fn is_hidden(&self) -> bool {
        is_path_hidden(self)
    }
}

/// Contains the content of a directory.
#[derive(Default)]
pub struct DirectoryContent {
    content: Vec<DirectoryEntry>,
    content_recv: Option<Arc<Mutex<mpsc::Receiver<Result<Vec<DirectoryEntry>, std::io::Error>>>>>,
    /// Contains the error message if there was an error when loading the directory.
    error: Option<String>,
}

impl std::fmt::Debug for DirectoryContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl DirectoryContent {
    /// Create a new object with empty content
    pub const fn new() -> Self {
        Self {
            content: vec![],
            content_recv: None,
            error: None,
        }
    }

    /// Create a new `DirectoryContent` object and loads the contents of the given path.
    /// Use `include_files` to include or exclude files in the content list.
    pub fn from_path(
        config: &FileDialogConfig,
        path: &Path,
        include_files: bool,
        show_hidden: bool,
        show_system_files: bool,
        file_filter: Option<&FileFilter>,
    ) -> Self {
        if config.load_via_thread {
            Self::with_thread(
                config,
                path,
                include_files,
                show_hidden,
                show_system_files,
                file_filter,
            )
        } else {
            Self::without_thread(
                config,
                path,
                include_files,
                show_hidden,
                show_system_files,
                file_filter,
            )
        }
    }

    fn with_thread(
        config: &FileDialogConfig,
        path: &Path,
        include_files: bool,
        show_hidden: bool,
        show_system_files: bool,
        file_filter: Option<&FileFilter>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();

        let c = config.clone();
        let p = path.to_path_buf();
        let f = file_filter.cloned();
        thread::spawn(move || {
            let _ = tx.send(load_directory(
                &c,
                &p,
                include_files,
                show_hidden,
                show_system_files,
                f.as_ref(),
            ));
        });

        Self {
            content: Vec::new(),
            content_recv: Some(Arc::new(Mutex::new(rx))),
            error: None,
        }
    }

    fn without_thread(
        config: &FileDialogConfig,
        path: &Path,
        include_files: bool,
        show_hidden: bool,
        show_system_files: bool,
        file_filter: Option<&FileFilter>,
    ) -> Self {
        match load_directory(
            config,
            path,
            include_files,
            show_hidden,
            show_system_files,
            file_filter,
        ) {
            Ok(c) => Self {
                content: c,
                content_recv: None,
                error: None,
            },
            Err(err) => Self {
                content: Vec::new(),
                content_recv: None,
                error: Some(err.to_string()),
            },
        }
    }

    pub fn filtered_iter<'s>(
        &'s self,
        search_value: &'s str,
    ) -> impl Iterator<Item = &'s DirectoryEntry> + 's {
        self.content
            .iter()
            .filter(|p| apply_search_value(p, search_value))
    }

    pub fn filtered_iter_mut<'s>(
        &'s mut self,
        search_value: &'s str,
    ) -> impl Iterator<Item = &'s mut DirectoryEntry> + 's {
        self.content
            .iter_mut()
            .filter(|p| apply_search_value(p, search_value))
    }

    pub fn is_loading(&mut self) -> bool {
        let mut rm_recv = false;

        if let Some(recv) = &self.content_recv {
            match recv.lock().try_recv() {
                Ok(t) => {
                    match t {
                        Ok(t) => self.content = t,
                        Err(err) => self.error = Some(err.to_string()),
                    }
                    rm_recv = true;
                }
                Err(err) => match err {
                    mpsc::TryRecvError::Empty => return true,
                    mpsc::TryRecvError::Disconnected => {
                        self.error = Some("thread ended unexpectedly".to_owned());
                        rm_recv = true;
                    }
                },
            }
        }

        if rm_recv {
            self.content_recv = None;
        }

        false
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn reset_multi_selection(&mut self) {
        for item in &mut self.content {
            item.selected = false;
        }
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

fn apply_search_value(entry: &DirectoryEntry, value: &str) -> bool {
    value.is_empty()
        || entry
            .file_name()
            .to_lowercase()
            .contains(&value.to_lowercase())
}

/// Loads the contents of the given directory.
fn load_directory(
    config: &FileDialogConfig,
    path: &Path,
    include_files: bool,
    show_hidden: bool,
    show_system_files: bool,
    file_filter: Option<&FileFilter>,
) -> io::Result<Vec<DirectoryEntry>> {
    let paths = fs::read_dir(path)?;

    let mut result: Vec<DirectoryEntry> = Vec::new();
    for path in paths {
        match path {
            Ok(entry) => {
                let entry = DirectoryEntry::from_path(config, entry.path().as_path());

                if !show_system_files && entry.is_system_file() {
                    continue;
                }

                if !include_files && entry.is_file() {
                    continue;
                }

                if !show_hidden && entry.is_hidden() {
                    continue;
                }

                if let Some(file_filter) = file_filter {
                    if entry.is_file() && !(file_filter.filter)(entry.as_path()) {
                        continue;
                    }
                }

                result.push(entry);
            }
            Err(_) => continue,
        };
    }

    result.sort_by(|a, b| {
        if a.is_dir() == b.is_dir() {
            a.file_name().cmp(b.file_name())
        } else if a.is_dir() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(result)
}

#[cfg(windows)]
fn is_path_hidden(item: &DirectoryEntry) -> bool {
    use std::os::windows::fs::MetadataExt;

    match fs::metadata(item.as_path()) {
        Ok(metadata) => metadata.file_attributes() & 0x2 > 0,
        Err(_) => false,
    }
}

#[cfg(not(windows))]
fn is_path_hidden(item: &DirectoryEntry) -> bool {
    if item.file_name().bytes().next() == Some(b'.') {
        return true;
    }

    false
}

/// Generates the icon for the specific path.
/// The default icon configuration is taken into account, as well as any configured
/// file icon filters.
fn gen_path_icon(config: &FileDialogConfig, path: &Path) -> String {
    for def in &config.file_icon_filters {
        if (def.filter)(path) {
            return def.icon.clone();
        }
    }

    if path.is_dir() {
        config.default_folder_icon.clone()
    } else {
        config.default_file_icon.clone()
    }
}
