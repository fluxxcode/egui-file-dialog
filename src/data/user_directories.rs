use std::path::{Path, PathBuf};

/// Wrapper above `directories::UserDirs`.
/// Currently only used to canonicalize the paths.
#[derive(Default, Clone, Debug)]
pub struct UserDirectories {
    home_dir: Option<PathBuf>,

    audio_dir: Option<PathBuf>,
    desktop_dir: Option<PathBuf>,
    document_dir: Option<PathBuf>,
    download_dir: Option<PathBuf>,
    picture_dir: Option<PathBuf>,
    video_dir: Option<PathBuf>,
}

impl UserDirectories {
    /// Creates a new custom `UserDirectories` object
    pub const fn new(
        home_dir: Option<PathBuf>,
        audio_dir: Option<PathBuf>,
        desktop_dir: Option<PathBuf>,
        document_dir: Option<PathBuf>,
        download_dir: Option<PathBuf>,
        picture_dir: Option<PathBuf>,
        video_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            home_dir,
            audio_dir,
            desktop_dir,
            document_dir,
            download_dir,
            picture_dir,
            video_dir,
        }
    }

    pub(crate) fn home_dir(&self) -> Option<&Path> {
        self.home_dir.as_deref()
    }

    pub(crate) fn audio_dir(&self) -> Option<&Path> {
        self.audio_dir.as_deref()
    }

    pub(crate) fn desktop_dir(&self) -> Option<&Path> {
        self.desktop_dir.as_deref()
    }

    pub(crate) fn document_dir(&self) -> Option<&Path> {
        self.document_dir.as_deref()
    }

    pub(crate) fn download_dir(&self) -> Option<&Path> {
        self.download_dir.as_deref()
    }

    pub(crate) fn picture_dir(&self) -> Option<&Path> {
        self.picture_dir.as_deref()
    }

    pub(crate) fn video_dir(&self) -> Option<&Path> {
        self.video_dir.as_deref()
    }

    /// Canonicalizes the given paths. Returns None if an error occurred.
    pub(crate) fn canonicalize(path: Option<&Path>, canonicalize: bool) -> Option<PathBuf> {
        if !canonicalize {
            return path.map(PathBuf::from);
        }

        if let Some(path) = path {
            return dunce::canonicalize(path).ok();
        }

        None
    }
}
