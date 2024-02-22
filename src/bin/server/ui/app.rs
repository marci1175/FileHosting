#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Server {
    label: String,
    value: f32,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            label: String::new(),
            value: 0.,
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

        egui::CentralPanel::default().show(ctx, |ui| {
            
        });
        
    }
}