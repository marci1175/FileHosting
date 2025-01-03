use egui::vec2;
use std::{
    fmt::Debug,
    fs::{self},
    path::PathBuf,
    time::SystemTime,
};

///Master packet, when asking for the file
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ServerFile {
    pub bytes: Option<std::vec::Vec<u8>>,
    pub path: PathBuf,
    pub error: Option<String>,
}

impl Debug for ServerFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("File: {:?}\nError: {:?}", self.bytes, self.error))
    }
}

impl ServerFile {
    pub fn new(path: PathBuf) -> Self {
        match fs::read(path.clone()) {
            Ok(bytes) => Self {
                bytes: Some(bytes),
                path,
                error: None,
            },
            Err(err) => Self {
                bytes: None,
                path,
                error: Some(err.to_string()),
            },
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

///Master packet, when asking for the file tree
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerList {
    pub list: Vec<PathItem>,
}

impl Debug for ServerList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("List: {:?}", self.list))
    }
}

impl ServerList {
    pub fn new(file_list: Vec<PathItem>) -> Self {
        Self { list: file_list }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

///This is what the server replies with
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

///Used for tree structure of the sent files
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum PathItem {
    Folder(FolderItem),
    File(FileStruct),
}

///This struct contains the data which is being sent to the client containing the path and the mtadata
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct FileStruct {
    path: PathBuf,
    metadata: Option<FileMetadata>,
}

///This is a newtype for fs::Metadata, which allows us to serialize and deserialize
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct FileMetadata {
    file_size: u64,
    file_modified: SystemTime,
    file_accessed: SystemTime,
    file_created: SystemTime,
}

impl FileMetadata {
    pub fn from_fs_metadata(metadata: fs::Metadata) -> anyhow::Result<Self> {
        Ok(Self {
            file_size: metadata.len(),
            file_modified: metadata.modified()?,
            file_accessed: metadata.accessed()?,
            file_created: metadata.created()?,
        })
    }
}

///This is what the server gets when the client is asking something (MASTER PACKET)
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum ClientRequest {
    ///Client asked for a list
    ListRequest,
    ///Client asked for a file
    FileRequest(PathBuf),
}

impl ClientRequest {
    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl PathItem {
    pub fn get_path(&self) -> PathBuf {
        match self {
            PathItem::Folder(folder) => folder.path.clone(),
            PathItem::File(file) => file.clone().path,
        }
    }
}

///Used in the ui, to make the file tree visualization
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct FolderItem {
    ///Path to the folder (wtf who put this in here)
    pub path: PathBuf,
    ///Should the tree branch be opened
    pub opened: bool,
    ///The folder's entries
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

//It returns which file button it has been clicked on
pub fn render_path(folder_list: &mut Vec<PathItem>, ui: &mut egui::Ui) -> Option<PathBuf> {
    //check if folder is empty
    if folder_list.is_empty() {
        ui.label("Empty");
        return None;
    }

    let mut clicked_button: Option<PathBuf> = None;

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
                    ));
                });

                if folder.opened {
                    //Indent
                    ui.group(|ui| {
                        clicked_button = render_path(&mut folder.entries, ui);
                    });
                }
            }
            PathItem::File(file) => {
                ui.horizontal(|ui| {
                    //file button
                    ui.allocate_ui(vec2(30., 30.), |ui| {
                        if ui
                            .add(egui::widgets::ImageButton::new(egui::include_image!(
                                "../assets/file_small.png"
                            )))
                            .clicked()
                        {
                            clicked_button = Some(file.clone().path);
                        }
                    });

                    ui.label(format!(
                        "{}",
                        file.path.file_name().unwrap().to_string_lossy()
                    ));

                    //Separator
                    ui.separator();

                    //Display metadata
                    if let Some(metadata) = &file.metadata {
                        ui.label(format!("File size: {} KB", metadata.file_size / 1024_u64));

                        if let Ok(dur) = metadata.file_accessed.elapsed() {
                            ui.label(format!(
                                "{}",
                                humantime::format_rfc3339_seconds(
                                    SystemTime::now().checked_sub(dur).unwrap()
                                )
                            ));
                        }
                    }
                });
            }
        }
    }

    clicked_button
}

pub fn iter_folder(group: &PathBuf) -> Vec<PathItem> {
    let mut paths: Vec<PathItem> = Vec::new();
    for dir_entry in fs::read_dir(group).unwrap() {
        let dir_entry = dir_entry.unwrap();
        let path = dir_entry.path();

        if path.is_file() {
            paths.push(PathItem::File(FileStruct {
                path: path.clone(),
                metadata: {
                    match fs::metadata(path) {
                        Ok(metadata) => FileMetadata::from_fs_metadata(metadata).ok(),
                        Err(_) => None,
                    }
                },
            }));
        } else if path.is_dir() {
            paths.push(PathItem::Folder(FolderItem::new(path)));
        }
    }
    paths
}
