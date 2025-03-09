use crate::config::{FileDialogConfig, FileFilter};
use crate::FileSystem;
use egui::mutex::Mutex;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use std::time::SystemTime;
use std::{io, thread};

/// Contains the metadata of a directory item.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Metadata {
    pub(crate) size: Option<u64>,
    pub(crate) last_modified: Option<SystemTime>,
    pub(crate) created: Option<SystemTime>,
    pub(crate) file_type: Option<String>,
}

impl Metadata {
    /// Create a new custom metadata
    pub const fn new(
        size: Option<u64>,
        last_modified: Option<SystemTime>,
        created: Option<SystemTime>,
        file_type: Option<String>,
    ) -> Self {
        Self {
            size,
            last_modified,
            created,
            file_type,
        }
    }
}

/// Contains the information of a directory item.
///
/// This struct is mainly there so that the information and metadata can be loaded once and not that
/// a request has to be sent to the OS every frame using, for example, `path.is_file()`.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DirectoryEntry {
    path: PathBuf,
    metadata: Metadata,
    is_directory: bool,
    is_system_file: bool,
    is_hidden: bool,
    icon: String,
    /// If the item is marked as selected as part of a multi selection.
    pub selected: bool,
}

impl DirectoryEntry {
    /// Creates a new directory entry from a path
    pub fn from_path(config: &FileDialogConfig, path: &Path, file_system: &dyn FileSystem) -> Self {
        Self {
            path: path.to_path_buf(),
            metadata: file_system.metadata(path).unwrap_or_default(),
            is_directory: file_system.is_dir(path),
            is_system_file: !file_system.is_dir(path) && !file_system.is_file(path),
            icon: gen_path_icon(config, path, file_system),
            is_hidden: file_system.is_path_hidden(path),
            selected: false,
        }
    }

    /// Returns the metadata of the directory entry.
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
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
    pub const fn is_hidden(&self) -> bool {
        self.is_hidden
    }
}

/// Contains the state of the directory content.
#[derive(Debug, PartialEq, Eq)]
pub enum DirectoryContentState {
    /// If we are currently waiting for the loading process on another thread.
    /// The value is the timestamp when the loading process started.
    Pending(SystemTime),
    /// If loading the directory content finished since the last update call.
    /// This is only returned once.
    Finished,
    /// If loading the directory content was successful.
    Success,
    /// If there was an error loading the directory content.
    /// The value contains the error message.
    Errored(String),
}

type DirectoryContentReceiver =
    Option<Arc<Mutex<mpsc::Receiver<Result<Vec<DirectoryEntry>, std::io::Error>>>>>;

/// Contains the content of a directory.
pub struct DirectoryContent {
    /// Current state of the directory content.
    state: DirectoryContentState,
    /// The loaded directory contents.
    content: Vec<DirectoryEntry>,
    /// Receiver when the content is loaded on a different thread.
    content_recv: DirectoryContentReceiver,
}

impl Default for DirectoryContent {
    fn default() -> Self {
        Self {
            state: DirectoryContentState::Success,
            content: Vec::new(),
            content_recv: None,
        }
    }
}

impl std::fmt::Debug for DirectoryContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectoryContent")
            .field("state", &self.state)
            .field("content", &self.content)
            .field(
                "content_recv",
                if self.content_recv.is_some() {
                    &"<Receiver>"
                } else {
                    &"None"
                },
            )
            .finish()
    }
}

impl DirectoryContent {
    /// Create a new `DirectoryContent` object and loads the contents of the given path.
    /// Use `include_files` to include or exclude files in the content list.
    pub fn from_path(
        config: &FileDialogConfig,
        path: &Path,
        include_files: bool,
        file_filter: Option<&FileFilter>,
        filter_extension: Option<&str>,
        file_system: Arc<dyn FileSystem + Sync + Send + 'static>,
    ) -> Self {
        if config.load_via_thread {
            Self::with_thread(
                config,
                path,
                include_files,
                file_filter,
                filter_extension,
                file_system,
            )
        } else {
            Self::without_thread(
                config,
                path,
                include_files,
                file_filter,
                filter_extension,
                &*file_system,
            )
        }
    }

    fn with_thread(
        config: &FileDialogConfig,
        path: &Path,
        include_files: bool,
        file_filter: Option<&FileFilter>,
        filter_extension: Option<&str>,
        file_system: Arc<dyn FileSystem + Send + Sync + 'static>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();

        let c = config.clone();
        let p = path.to_path_buf();
        let f = file_filter.cloned();
        let fe = filter_extension.map(str::to_string);
        thread::spawn(move || {
            let _ = tx.send(load_directory(
                &c,
                &p,
                include_files,
                f.as_ref(),
                fe.as_deref(),
                &*file_system,
            ));
        });

        Self {
            state: DirectoryContentState::Pending(SystemTime::now()),
            content: Vec::new(),
            content_recv: Some(Arc::new(Mutex::new(rx))),
        }
    }

    fn without_thread(
        config: &FileDialogConfig,
        path: &Path,
        include_files: bool,
        file_filter: Option<&FileFilter>,
        filter_extension: Option<&str>,
        file_system: &dyn FileSystem,
    ) -> Self {
        match load_directory(
            config,
            path,
            include_files,
            file_filter,
            filter_extension,
            file_system,
        ) {
            Ok(c) => Self {
                state: DirectoryContentState::Success,
                content: c,
                content_recv: None,
            },
            Err(err) => Self {
                state: DirectoryContentState::Errored(err.to_string()),
                content: Vec::new(),
                content_recv: None,
            },
        }
    }

    pub fn update(&mut self) -> &DirectoryContentState {
        if self.state == DirectoryContentState::Finished {
            self.state = DirectoryContentState::Success;
        }

        if !matches!(self.state, DirectoryContentState::Pending(_)) {
            return &self.state;
        }

        self.update_pending_state()
    }

    fn update_pending_state(&mut self) -> &DirectoryContentState {
        let rx = std::mem::take(&mut self.content_recv);
        let mut update_content_recv = true;

        if let Some(recv) = &rx {
            let value = recv.lock().try_recv();
            match value {
                Ok(result) => match result {
                    Ok(content) => {
                        self.state = DirectoryContentState::Finished;
                        self.content = content;
                        update_content_recv = false;
                    }
                    Err(err) => {
                        self.state = DirectoryContentState::Errored(err.to_string());
                        update_content_recv = false;
                    }
                },
                Err(err) => {
                    if mpsc::TryRecvError::Disconnected == err {
                        self.state =
                            DirectoryContentState::Errored("thread ended unexpectedly".to_owned());
                        update_content_recv = false;
                    }
                }
            }
        }

        if update_content_recv {
            self.content_recv = rx;
        }

        &self.state
    }

    /// Returns an iterator in the given range of the directory cotnents.
    /// No filters are applied using this iterator.
    pub fn iter_range_mut(
        &mut self,
        range: std::ops::Range<usize>,
    ) -> impl Iterator<Item = &mut DirectoryEntry> {
        self.content[range].iter_mut()
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

    /// Marks each element in the content as unselected.
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
    file_filter: Option<&FileFilter>,
    filter_extension: Option<&str>,
    file_system: &dyn FileSystem,
) -> io::Result<Vec<DirectoryEntry>> {
    let mut result: Vec<DirectoryEntry> = Vec::new();
    for path in file_system.read_dir(path)? {
        let entry = DirectoryEntry::from_path(config, &path, file_system);

        if !config.storage.show_system_files && entry.is_system_file() {
            continue;
        }

        if !include_files && entry.is_file() {
            continue;
        }

        if !config.storage.show_hidden && entry.is_hidden() {
            continue;
        }

        if let Some(file_filter) = file_filter {
            if entry.is_file() && !(file_filter.filter)(entry.as_path()) {
                continue;
            }
        }

        if let Some(ex) = filter_extension {
            if entry.is_file()
                && path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    != ex
            {
                continue;
            }
        }

        result.push(entry);
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

/// Generates the icon for the specific path.
/// The default icon configuration is taken into account, as well as any configured
/// file icon filters.
fn gen_path_icon(config: &FileDialogConfig, path: &Path, file_system: &dyn FileSystem) -> String {
    for def in &config.file_icon_filters {
        if (def.filter)(path) {
            return def.icon.clone();
        }
    }

    if file_system.is_dir(path) {
        config.default_folder_icon.clone()
    } else {
        config.default_file_icon.clone()
    }
}
