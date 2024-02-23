use std::ffi::OsStr;

use egui::{vec2, Color32, RichText};
use tokio::sync::mpsc;
use CommonDefinitions::{render_path, ClientRequest, PathItem, ServerReply};

use crate::ui::backend::client;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Client {
    connecting_to: String,
    connecting_port: i64,

    password: String,

    //this_sx gets moved to connection, and you can send instruction to the connection thread byy this channel
    #[serde(skip)]
    connection: Option<mpsc::Sender<Option<ClientRequest>>>,

    #[serde(skip)]
    main_rx: mpsc::Receiver<String>,
    #[serde(skip)]
    main_sx: mpsc::Sender<String>,

    #[serde(skip)]
    this_rx: mpsc::Receiver<Option<ClientRequest>>,
    #[serde(skip)]
    this_sx: mpsc::Sender<Option<ClientRequest>>,

    #[serde(skip)]
    shared_folders: Vec<PathItem>,
}

impl Default for Client {
    fn default() -> Self {
        //this sx is used to send info the the connection thread
        let (this_sx, this_rx) = mpsc::channel(100);

        //Main rx is used to recive data to main, sx is passed to connection thread
        let (main_sx, main_rx) = mpsc::channel(100);
        Self {
            connecting_to: String::new(),
            password: String::new(),
            connecting_port: 0,
            connection: None,

            main_rx,
            main_sx,

            this_rx,
            this_sx,

            shared_folders: Vec::new(),
        }
    }
}

impl Client {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for Client {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        egui::TopBottomPanel::bottom("settings").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Connect", |ui| {
                    ui.allocate_ui(vec2(200., 100.), |ui| {
                        ui.label("Establish connection with host");
                        ui.label("Address (IpV6)");
                        ui.text_edit_singleline(&mut self.connecting_to);

                        ui.label("Port (double click to edit)");
                        ui.add(
                            egui::widgets::DragValue::new(&mut self.connecting_port)
                                .clamp_range(0..=65535),
                        );

                        ui.label("Password");
                        ui.text_edit_singleline(&mut self.password);

                        ui.separator();
                        
                        ui.add_enabled_ui(self.connection.is_none(), |ui| {
                            if ui.button("Connect").clicked() {
                                let ip = format!("[{}]:{}", self.connecting_to, self.connecting_port);
                                let password = self.password.clone();

                                //The info is passed TO the MAIN from the connection
                                let main_sx = self.main_sx.clone();

                                let (this_sx, this_rx) = mpsc::channel(100);

                                //The info is send BY MAIN to the connection thread
                                self.this_sx = this_sx;

                                //Connect
                                tokio::spawn(async move {
                                    match client::connect(ip, password, main_sx, this_rx).await {
                                        Ok(_) => {}
                                        Err(err) => {
                                            println!("{err}");
                                        }
                                    };
                                });
                            };
                        });
                        ui.add_enabled_ui(self.connection.is_some(), |ui| {
                            if ui.button("Disconnect").clicked() {
                                let this_sx = self.this_sx.clone();

                                //Stop the connection thread gracefully
                                tokio::spawn(async move {
                                    this_sx.send(None).await.unwrap();
                                });

                                //reset state
                                self.connection = None;
                            };
                        });
                    });
                });

                // Display status
                if self.connection.is_none() {
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
                    //iter over all added folders
                    for group in self.shared_folders.iter_mut() {
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
                            });

                            if let PathItem::Folder(folder) = group {
                                //Get pathbuf which we have clicked on
                                let file_clicked_on = render_path(&mut folder.entries, ui);

                                if let Some(path) = file_clicked_on {
                                    let this_sx = self.this_sx.clone();

                                    //Send requested path
                                    tokio::spawn(async move {
                                        let _ = this_sx
                                            .send(Some(ClientRequest::FileRequest(dbg!(path))))
                                            .await
                                            .map_err(|err| dbg!(err));
                                    });
                                }
                            }
                        });
                    }
                });
        });

        match self.main_rx.try_recv() {
            Ok(struct_str) => {
                if struct_str == "Invalid Password!" {
                    let ctx = ctx.clone();
                    std::thread::spawn(move || {
                        println!("NEW THREAD");
                        egui::Window::new("Error").show(&ctx, |ui| {
                            ui.label("Nigga");
                        });
                    });

                    self.connection = None;
                } else {
                    self.connection = Some(self.this_sx.clone());

                    match serde_json::from_str::<ServerReply>(&struct_str) {
                        Ok(ok) => {
                            match ok {
                                ServerReply::List(list) => {
                                    self.shared_folders = list.list;
                                }
                                ServerReply::File(file) => {
                                    if let Some(err) = file.error {
                                        dbg!(err);
                                    } else if let Some(file_bytes) = file.bytes {
                                        //Handle download
                                        let files = rfd::FileDialog::new()
                                            .set_title("Save to")
                                            .set_directory("/")
                                            .add_filter("File extension", &[file.path.extension().unwrap_or(file.path.file_stem().unwrap()).to_os_string().to_string_lossy()])
                                            .save_file();

                                        if let Some(file_path) = files {
                                            let _ = std::fs::write(file_path, file_bytes)
                                                .map_err(|err| dbg!(err));
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            dbg!(err);
                        }
                    }
                }
            }
            Err(_) => {}
        };

        ctx.request_repaint();
    }
}
