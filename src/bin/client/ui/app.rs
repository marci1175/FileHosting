use egui::{vec2, Color32, RichText};
use tokio::sync::mpsc;

use crate::ui::backend::client;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Client {
    connecting_to: String,
    password: String,

    #[serde(skip)]
    rx: mpsc::Receiver<String>,
}

impl Default for Client {
    fn default() -> Self {
        let (sx, rx) = mpsc::channel(100);
        Self {
            connecting_to: String::new(),
            password: String::new(),

            rx,
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
        egui::TopBottomPanel::bottom("settings").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Connect", |ui| {
                    ui.allocate_ui(vec2(200., 100.), |ui| {
                        ui.label("Establish connection with host");
                        ui.label("Address (IpV6)");
                        ui.text_edit_singleline(&mut self.connecting_to);

                        ui.label("Password");
                        ui.text_edit_singleline(&mut self.password);

                        ui.separator();

                        if ui.button("Connect").clicked() {
                            let ip = self.connecting_to.clone();
                            let password = self.password.clone();

                            //Connect
                            tokio::spawn(async move {
                                match client::connect(ip, password).await {
                                    Ok(_) => todo!(),
                                    Err(_) => todo!(),
                                };
                            });
                        };
                    });
                });

                //Display status
                // if self.connection.is_none() {
                //     ui.label(RichText::from("Offline").color(Color32::RED));
                // } else {
                //     ui.label(RichText::from("Online").color(Color32::GREEN));
                // }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {});
    }
}
