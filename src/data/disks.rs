use std::path::{Path, PathBuf};

/// Wrapper above the `sysinfo::Disk` struct.
/// Used for helper functions and so that more flexibility is guaranteed in the future if
/// the names of the disks are generated dynamically.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Disk {
    mount_point: PathBuf,
    display_name: String,
    is_removable: bool,
}

impl Disk {
    /// Create a new Disk object based on the data of a `sysinfo::Disk`.
    pub fn from_sysinfo_disk(disk: &sysinfo::Disk, canonicalize_paths: bool) -> Self {
        Self {
            mount_point: Self::canonicalize(disk.mount_point(), canonicalize_paths),
            display_name: gen_display_name(
                disk.name().to_str().unwrap_or_default(),
                disk.mount_point().to_str().unwrap_or_default(),
            ),
            is_removable: disk.is_removable(),
        }
    }

    /// Returns the mount point of the disk
    pub fn mount_point(&self) -> &Path {
        &self.mount_point
    }

    /// Returns the display name of the disk
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub const fn is_removable(&self) -> bool {
        self.is_removable
    }

    /// Canonicalizes the given path.
    /// Returns the input path in case of an error.
    fn canonicalize(path: &Path, canonicalize: bool) -> PathBuf {
        if canonicalize {
            dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
        } else {
            path.to_path_buf()
        }
    }
}

/// Wrapper above the `sysinfo::Disks` struct
#[derive(Default, Debug)]
pub struct Disks {
    disks: Vec<Disk>,
}

impl Disks {
    /// Creates a new Disks object with a refreshed list of the system disks.
    pub fn new_with_refreshed_list(canonicalize_paths: bool) -> Self {
        let disks = sysinfo::Disks::new_with_refreshed_list();

        let mut result: Vec<Disk> = Vec::new();
        for disk in &disks {
            result.push(Disk::from_sysinfo_disk(disk, canonicalize_paths));
        }

        Self { disks: result }
    }

    /// Very simple wrapper method of the disks `.iter()` method.
    /// No trait is implemented since this is currently only used internal.
    pub fn iter(&self) -> std::slice::Iter<'_, Disk> {
        self.disks.iter()
    }
}

#[cfg(windows)]
fn gen_display_name(name: &str, mount_point: &str) -> String {
    let mount_point = mount_point.replace('\\', "");

    // Try using the mount point as the display name if the specified name
    // from sysinfo::Disk is empty or contains invalid characters
    if name.is_empty() {
        return mount_point;
    }

    format!("{name} ({mount_point})")
}

#[cfg(not(windows))]
fn gen_display_name(name: &str, mount_point: &str) -> String {
    // Try using the mount point as the display name if the specified name
    // from sysinfo::Disk is empty or contains invalid characters
    if name.is_empty() {
        return mount_point.to_string();
    }

    name.to_string()
}
