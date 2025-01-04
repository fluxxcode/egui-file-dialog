use egui_file_dialog::{Disk, Disks, FileDialog, FileDialogConfig, FileSystem};
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use eframe::egui;

struct MyApp {
    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let root = vec![
            ("im_a_file.txt".to_string(), Node::File),
            (
                "folder_a".to_string(),
                Node::Directory(vec![
                    ("hello.txt".to_string(), Node::File),
                    ("we are files.md".to_string(), Node::File),
                    (
                        "nesting".to_string(),
                        Node::Directory(vec![(
                            "Nesting for beginners.pdf".to_string(),
                            Node::File,
                        )]),
                    ),
                ]),
            ),
            (
                "folder_b".to_string(),
                Node::Directory(vec![(
                    "Yeah this is also a directory.tar".to_string(),
                    Node::File,
                )]),
            ),
        ];

        Self {
            file_dialog: FileDialog::with_config(FileDialogConfig::default_from_filesystem(
                Arc::new(MyFileSystem(root)),
            )),
            picked_file: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Picked file").clicked() {
                self.file_dialog.pick_file();
            }

            ui.label(format!("Picked file: {:?}", self.picked_file));

            if let Some(path) = self.file_dialog.update(ctx).picked() {
                self.picked_file = Some(path.to_path_buf());
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "File dialog example",
        eframe::NativeOptions::default(),
        Box::new(|ctx| Ok(Box::new(MyApp::new(ctx)))),
    )
}

type Directory = Vec<(String, Node)>;

#[derive(Clone, PartialEq)]
enum Node {
    Directory(Directory),
    File,
}

struct MyFileSystem(Directory);

impl MyFileSystem {
    fn browse(&self, path: &Path) -> std::io::Result<&Directory> {
        let mut dir = &self.0;
        for component in path.components() {
            let Component::Normal(part) = component else {
                continue;
            };
            let part = part.to_str().unwrap();

            let subdir = dir
                .iter()
                .find_map(|(name, node)| match node {
                    Node::File => None,
                    Node::Directory(subdir) => (name == part).then(|| subdir),
                })
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Directory not found".to_string(),
                    )
                })?;

            dir = subdir;
        }

        Ok(dir)
    }
}

impl FileSystem for MyFileSystem {
    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        let dir = self.browse(path)?;
        Ok(dir.iter().map(|(name, _)| path.join(name)).collect())
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.browse(path).is_ok()
    }

    fn is_file(&self, path: &Path) -> bool {
        let Some(parent) = path.parent() else {
            return false;
        };
        let Ok(dir) = self.browse(parent) else {
            return false;
        };
        dir.iter().any(|(name, node)| {
            node == &Node::File && Some(name.as_str()) == path.file_name().and_then(|s| s.to_str())
        })
    }

    fn metadata(&self, path: &Path) -> std::io::Result<egui_file_dialog::Metadata> {
        Ok(Default::default())
    }

    fn get_disks(&self, canonicalize_paths: bool) -> egui_file_dialog::Disks {
        Disks::new(vec![Disk::new(
            Some("I'm a fake disk"),
            &PathBuf::from("/disk"),
            false,
            true,
        )])
    }

    fn user_dirs(&self, canonicalize_paths: bool) -> Option<egui_file_dialog::UserDirectories> {
        None
    }

    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported".to_string(),
        ))
    }

    fn current_dir(&self) -> std::io::Result<PathBuf> {
        Ok("".into())
    }

    fn is_path_hidden(&self, path: &Path) -> bool {
        false
    }

    fn load_text_file_preview(&self, path: &Path, max_chars: usize) -> std::io::Result<String> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported".to_string(),
        ))
    }
}
