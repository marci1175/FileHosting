use egui::{vec2, Color32, Response, RichText};
use std::{fs, path::PathBuf, sync::Arc};
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum PathItem {
    Folder(FolderItem),
    File(PathBuf),
}

impl PathItem {
    fn get_path(&self) -> PathBuf {
        return match self {
            PathItem::Folder(folder) => folder.path.clone(),
            PathItem::File(file) => file.clone(),
        };
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
        Self {
            path: path.clone(),
            opened: false,
            entries: iter_folder(&path),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Server {
    shared_folders: Vec<PathItem>,

    //Server doe not persist
    #[serde(skip)]
    server: Option<JoinHandle<()>>,

    server_password: String,
    server_port: i64,

    #[serde(skip)]
    rx: mpsc::Receiver<()>,
    #[serde(skip)]
    sx: mpsc::Sender<()>,
}

impl Default for Server {
    fn default() -> Self {
        //Default channel, this is not going to be used
        let (sx, rx) = mpsc::channel::<()>(1);

        Self {
            shared_folders: Vec::new(),
            server: None,

            server_password: String::new(),
            server_port: 0,

            rx,
            sx,
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
                //Display hint
                if self.shared_folders.is_empty() {
                    ui.label("Add a folder to the shared folders");
                } else {
                    ui.label(format!("Added folders: {}", self.shared_folders.len()));
                }

                //Add folder
                ui.add_enabled_ui(self.server.is_none(), |ui| {
                    if ui.button("Add folder").clicked() {
                        //Add folder
                        if let Some(added_folders) = rfd::FileDialog::new().pick_folders() {
                            for folder in added_folders {
                                self.shared_folders
                                    .push(PathItem::Folder(FolderItem::new(folder)));
                            }
                        };
                    }
                })
                .response
                .on_hover_text(
                    //Display warning message
                    if self.server.is_some() {
                        "You cannot add folders while the server is running"
                    } else {
                        "Add folder to share"
                    },
                );

                //Display status
                if self.server.is_none() {
                    ui.label(RichText::from("Offline").color(Color32::RED));
                } else {
                    ui.label(RichText::from("Online").color(Color32::GREEN));
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    //Kind of cheat the rust compiler
                    let mut should_remove: Option<usize> = None;

                    //iter over all added folders
                    for (index, group) in self.shared_folders.iter_mut().enumerate() {
                        ui.group(|ui| {
                            //Folder name and delete button
                            ui.horizontal(|ui| {
                                //Folder name
                                ui.label(
                                    RichText::from(format!(
                                        "Folder: {}",
                                        group.get_path().file_name().unwrap().to_string_lossy()
                                    ))
                                    .size(20.),
                                )
                                .on_hover_text(format!("Full path: {:?}", group.get_path()));

                                //Check if server has started
                                ui.add_enabled_ui(self.server.is_none(), |ui| {
                                    //and delete button
                                    ui.allocate_ui(vec2(20., 20.), |ui| {
                                        if ui
                                            .add(egui::widgets::ImageButton::new(egui::include_image!(
                                                "../../../../assets/cross.png"
                                            )))
                                            .clicked()
                                        {
                                            should_remove = Some(index);
                                        }
                                    });
                                });
                            });

                            if let PathItem::Folder(folder) = group {
                                render_path(&mut folder.entries, ui)
                            }
                        });
                    }

                    //Check if we need any deletion
                    if let Some(remove_index) = should_remove {
                        self.shared_folders.remove(remove_index);
                    }

                    //Debug panel
                    #[cfg(debug_assertions)]
                    {
                        ui.label("DEBUG PANEL");
                        if ui.button("Serialize shared_folders").clicked() {
                            dbg!(self.shared_folders.clone());
                        }
                    }
                });
        });

        egui::TopBottomPanel::bottom("server_manager").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.menu_button("Server", |ui| {
                    ui.label("Start file-hosting service");

                    ui.add_enabled_ui(self.server.is_none(), |ui| {
                        ui.label("Password");
                        ui.add(egui::widgets::TextEdit::singleline(
                            &mut self.server_password,
                        ));

                        ui.label("Port (double click to edit)");
                        ui.add(
                            egui::widgets::DragValue::new(&mut self.server_port)
                                .clamp_range(0..=65535),
                        );
                    });

                    ui.separator();

                    if ui
                        .add_enabled(self.server.is_none(), |ui: &mut egui::Ui| {
                            ui.button("Start")
                        })
                        .clicked()
                    {
                        //Spawn channels
                        let (sx, rx) = mpsc::channel::<()>(1);

                        //Sender clone
                        self.sx = sx;

                        //force ownership
                        let password = self.server_password.clone();
                        let port = self.server_port.clone();
                        let folder = self.shared_folders.clone();
                        //Server
                        self.server = Some(tokio::spawn(async move {
                            crate::ui::backend::server::server_spawner(password, port, rx, folder)
                                .await
                                .unwrap();
                        }));
                    };

                    if ui
                        .add_enabled(self.server.is_some(), |ui: &mut egui::Ui| ui.button("Stop"))
                        .clicked()
                    {
                        let sx = self.sx.clone();

                        //Shut down server
                        tokio::spawn(async move {
                            let _ = sx.send(()).await;
                        });

                        //Reset state
                        self.server = None;
                    }
                });
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
                        if ui
                            .add(egui::widgets::ImageButton::new(egui::include_image!(
                                "../../../../assets/folder_small.png"
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
                            "../../../../assets/file_small.png"
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
