use std::path::{Path, PathBuf};

/// Wrapper above `directories::UserDirs`.
/// Currently only used to canonicalize the paths.
#[derive(Default, Clone, Debug)]
pub struct UserDirectories {
    pub home_dir: Option<PathBuf>,

    pub audio_dir: Option<PathBuf>,
    pub desktop_dir: Option<PathBuf>,
    pub document_dir: Option<PathBuf>,
    pub download_dir: Option<PathBuf>,
    pub picture_dir: Option<PathBuf>,
    pub video_dir: Option<PathBuf>,
}

impl UserDirectories {
    pub fn home_dir(&self) -> Option<&Path> {
        self.home_dir.as_deref()
    }

    pub fn audio_dir(&self) -> Option<&Path> {
        self.audio_dir.as_deref()
    }

    pub fn desktop_dir(&self) -> Option<&Path> {
        self.desktop_dir.as_deref()
    }

    pub fn document_dir(&self) -> Option<&Path> {
        self.document_dir.as_deref()
    }

    pub fn download_dir(&self) -> Option<&Path> {
        self.download_dir.as_deref()
    }

    pub fn picture_dir(&self) -> Option<&Path> {
        self.picture_dir.as_deref()
    }

    pub fn video_dir(&self) -> Option<&Path> {
        self.video_dir.as_deref()
    }

    /// Canonicalizes the given paths. Returns None if an error occurred.
    pub fn canonicalize(path: Option<&Path>, canonicalize: bool) -> Option<PathBuf> {
        if !canonicalize {
            return path.map(PathBuf::from);
        }

        if let Some(path) = path {
            return dunce::canonicalize(path).ok();
        }

        None
    }
}
