#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::NativeOptions;

mod ui;

use ui::Client;

/// Include proto
pub mod file_hosting {
    tonic::include_proto!("file_hosting");
}

#[tokio::main]
async fn main() -> anyhow::Result<(), Box<dyn std::error::Error>> {
    eframe::run_native(
        "File Hosting Client",
        NativeOptions {
            ..Default::default()
        },
        Box::new(|cc| Box::new(Client::new(cc))),
    )?;

    Ok(())
}
