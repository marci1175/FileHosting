use std::{fs, path::PathBuf};
use egui::{vec2, RichText};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
enum PathItem {
    Folder(FolderItem),
    File(PathBuf),
}

impl PathItem {
    fn get_path(&self) -> PathBuf {
        return match self {
            PathItem::Folder(folder) => {
                folder.path.clone()
            },
            PathItem::File(file) => file.clone(),
        }
    } 
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct FolderItem {
    path: PathBuf,
    opened: bool,
    entries: Vec<PathItem>,
}

impl FolderItem {
    fn new(path: PathBuf) -> Self {
        Self { path: path.clone(), opened: false, entries: iter_folder(&path) }
    }
}



#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Server {
    shared_folders: Vec<PathItem>,

}

impl Default for Server {
    fn default() -> Self {
        Self {
            shared_folders: Vec::new(),
        }
    }
}

impl Server {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for Server {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //Image loading
        egui_extras::install_image_loaders(ctx);

        egui::TopBottomPanel::top("settings").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.shared_folders.is_empty() {
                    ui.label("Add a folder to the shared folders");
                } else {
                    ui.label(format!("Added folders: {}", self.shared_folders.len()));
                }

                if ui.button("Add folder").clicked() {
                    //Add folder
                    if let Some(added_folders) = rfd::FileDialog::new().pick_folders() {
                        for folder in added_folders {
                            self.shared_folders.push(PathItem::Folder(FolderItem::new(folder)));
                        }
                    };
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    //iter over all added folders
                    for (index, group) in self.shared_folders.clone().iter().enumerate() {
                        ui.group(|ui| {
                            //Folder name and delete button
                            ui.horizontal(|ui| {
                                //Folder name
                                ui.label(
                                    RichText::from(format!(
                                        "Folder: {}",
                                        group.get_path().file_stem().unwrap().to_string_lossy()
                                    ))
                                    .size(20.),
                                ).on_hover_text(format!("Full path: {:?}", group));

                                //and delete button
                                ui.allocate_ui(vec2(20., 20.), |ui| {
                                    if ui
                                        .add(egui::widgets::ImageButton::new(egui::include_image!(
                                            "../../../../assets/cross.png"
                                        )))
                                        .clicked()
                                    {
                                        self.shared_folders.remove(index);
                                    }
                                });
                            });

                            

                            render_path(&mut self.shared_folders, ui)
                        });
                    }
                });
        });
    }
}

fn render_path(folder_list: &mut Vec<PathItem>, ui: &mut egui::Ui) {
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
                        if ui.add(egui::widgets::ImageButton::new(
                            egui::include_image!(
                                "../../../../assets/folder_small.png"
                            ),
                        )).clicked() {
                            folder.opened = !folder.opened;
                        }
                    });

                    //Display name
                    ui.label(format!("{}", folder.path.file_stem().unwrap().to_string_lossy().to_string()));
                });

                if folder.opened {
                    //Indent
                    ui.group( |ui| {
                        render_path(&mut folder.entries, ui);
                    });
                }
            },
            PathItem::File(file) => {
                ui.horizontal(|ui| {
                    //file button
                    ui.allocate_ui(vec2(30., 30.), |ui| {
                        ui.add(egui::widgets::ImageButton::new(
                            egui::include_image!(
                                "../../../../assets/file_small.png"
                            ),
                        ))
                    });

                    //Display name
                    ui.label(format!("{}", file.file_stem().unwrap().to_string_lossy().to_string()));
                });
            },
        }
    }
}

fn iter_folder(group: &PathBuf) -> Vec<PathItem> {
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
