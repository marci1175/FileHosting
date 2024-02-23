use std::{fmt::Debug, fs, path::PathBuf};
use egui::vec2;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerFile {
    pub file: Option<std::vec::Vec<u8>>,
    pub error: Option<String>,
}

impl Debug for ServerFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("File: {:?}\nError: {:?}", self.file, self.error))
    }
}

impl ServerFile {
    pub fn new(path: PathBuf) -> Self {
        match fs::read(path) {
            Ok(bytes) => Self {
                file: Some(bytes),
                error: None,
            },
            Err(err) => Self {
                file: None,
                error: Some(err.to_string()),
            },
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerList {
    pub list: Option<String>,
    pub error: Option<String>,
}

impl Debug for ServerList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("List: {:?}\nError: {:?}", self.list, self.error))
    }
}

impl ServerList {
    pub fn new(file_list: Vec<PathItem>) -> Self {
        match serde_json::to_string(&file_list) {
            Ok(vec_string) => Self {
                list: Some(vec_string),
                error: None,
            },
            Err(err) => Self {
                list: None,
                error: Some(err.to_string()),
            },
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ServerReply {
    List(ServerList),
    File(ServerFile),
}

impl ServerReply {
    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum PathItem {
    Folder(FolderItem),
    File(PathBuf),
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum ClientRequest {
    ListRequest,
    FileRequest(PathBuf),
}

impl PathItem {
    pub fn get_path(&self) -> PathBuf {
        return match self {
            PathItem::Folder(folder) => folder.path.clone(),
            PathItem::File(file) => file.clone(),
        };
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct FolderItem {
    pub path: PathBuf,
    pub opened: bool,
    pub entries: Vec<PathItem>,
}

impl FolderItem {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path.clone(),
            opened: false,
            entries: iter_folder(&path),
        }
    }
}

pub fn render_path(folder_list: &mut Vec<PathItem>, ui: &mut egui::Ui) {
    //check if folder is empty
    if folder_list.is_empty() {
        ui.label("Empty");
        return;
    }

    //Iter over entries of the directory
    for entry in folder_list {
        match entry {
            PathItem::Folder(folder) => {
                ui.horizontal(|ui| {
                    //dir button
                    ui.allocate_ui(vec2(30., 30.), |ui| {
                        if ui
                            .add(egui::widgets::ImageButton::new(egui::include_image!(
                                "../assets/folder_small.png"
                            )))
                            .clicked()
                        {
                            folder.opened = !folder.opened;
                        }
                    });

                    //Display name
                    ui.label(format!(
                        "{}",
                        folder
                            .path
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()
                    ));
                });

                if folder.opened {
                    //Indent
                    ui.group(|ui| {
                        render_path(&mut folder.entries, ui);
                    });
                }
            }
            PathItem::File(file) => {
                ui.horizontal(|ui| {
                    //file button
                    ui.allocate_ui(vec2(30., 30.), |ui| {
                        ui.add(egui::widgets::ImageButton::new(egui::include_image!(
                            "../assets/file_small.png"
                        )))
                    });

                    //Display name
                    ui.label(format!(
                        "{}",
                        file.file_stem().unwrap().to_string_lossy().to_string()
                    ));
                });
            }
        }
    }
}

pub fn iter_folder(group: &PathBuf) -> Vec<PathItem> {
    let mut paths: Vec<PathItem> = Vec::new();
    for dir_entry in fs::read_dir(group).unwrap() {
        let dir_entry = dir_entry.unwrap();
        let path = dir_entry.path();

        if path.is_file() {
            paths.push(PathItem::File(path));
        } else if path.is_dir() {
            paths.push(PathItem::Folder(FolderItem::new(path)));
        }
    }
    paths
}
