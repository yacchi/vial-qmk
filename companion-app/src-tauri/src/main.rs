#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod active_app;
mod config;
mod hid;

use config::Config;
use hid::{create_hid_state, DeviceStatus, HidState};
use std::sync::Mutex;
use tauri::State;

struct ConfigState(Mutex<Config>);

#[tauri::command]
fn connect_device(hid_state: State<HidState>) -> Result<(), String> {
    let mut conn = hid_state.lock().map_err(|e| e.to_string())?;
    conn.connect()
}

#[tauri::command]
fn disconnect_device(hid_state: State<HidState>) -> Result<(), String> {
    let mut conn = hid_state.lock().map_err(|e| e.to_string())?;
    conn.disconnect();
    Ok(())
}

#[tauri::command]
fn get_device_status(hid_state: State<HidState>) -> Result<DeviceStatus, String> {
    let conn = hid_state.lock().map_err(|e| e.to_string())?;
    conn.get_status()
}

#[tauri::command]
fn set_layer(hid_state: State<HidState>, layer: u8) -> Result<(), String> {
    let conn = hid_state.lock().map_err(|e| e.to_string())?;
    conn.set_layer(layer)
}

#[tauri::command]
fn set_os_override(hid_state: State<HidState>, override_type: u8) -> Result<(), String> {
    let conn = hid_state.lock().map_err(|e| e.to_string())?;
    conn.set_os_override(override_type)
}

#[tauri::command]
fn get_active_app() -> Option<String> {
    active_app::get_active_app_name()
}

#[tauri::command]
fn open_vial(config_state: State<ConfigState>) -> Result<(), String> {
    let config = config_state.0.lock().map_err(|e| e.to_string())?;
    if let Some(vial_path) = &config.vial_path {
        open::that(vial_path).map_err(|e| format!("Failed to open Vial: {}", e))
    } else {
        // Try common Vial locations or open the web version
        open::that("https://vial.rocks/").map_err(|e| format!("Failed to open Vial web: {}", e))
    }
}

#[tauri::command]
fn get_config(config_state: State<ConfigState>) -> Result<Config, String> {
    let config = config_state.0.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
fn save_config(config_state: State<ConfigState>, config: Config) -> Result<(), String> {
    let mut current = config_state.0.lock().map_err(|e| e.to_string())?;
    *current = config;
    current.save()
}

fn main() {
    let config = Config::load_with_includes();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(create_hid_state())
        .manage(ConfigState(Mutex::new(config)))
        .invoke_handler(tauri::generate_handler![
            connect_device,
            disconnect_device,
            get_device_status,
            set_layer,
            set_os_override,
            get_active_app,
            open_vial,
            get_config,
            save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
