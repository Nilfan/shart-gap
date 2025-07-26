#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod networking;
mod room;
mod user;
mod ping;
mod invite;
mod protocol;
mod commands;

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub rooms: Arc<Mutex<Vec<room::Room>>>,
    pub current_user: Arc<Mutex<Option<user::User>>>,
    pub networking: Arc<Mutex<networking::NetworkManager>>,
}

#[tokio::main]
async fn main() {
    let app_state = AppState {
        rooms: Arc::new(Mutex::new(Vec::new())),
        current_user: Arc::new(Mutex::new(None)),
        networking: Arc::new(Mutex::new(networking::NetworkManager::new())),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::create_room,
            commands::join_room,
            commands::leave_room,
            commands::send_message,
            commands::get_rooms,
            commands::set_user_settings,
            commands::get_ping_stats,
            commands::change_protocol,
            commands::generate_invite,
            commands::parse_invite,
            commands::sync_messages,
            commands::get_room_messages,
            commands::check_room_health,
            commands::mark_user_offline_cmd,
            commands::join_call,
            commands::leave_call
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}