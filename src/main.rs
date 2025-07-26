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
    pub current_party: Arc<Mutex<Option<room::Room>>>,
    pub current_user: Arc<Mutex<Option<user::User>>>,
    pub networking: Arc<Mutex<networking::NetworkManager>>,
}

#[tokio::main]
async fn main() {
    let app_state = AppState {
        current_party: Arc::new(Mutex::new(None)),
        current_user: Arc::new(Mutex::new(None)),
        networking: Arc::new(Mutex::new(networking::NetworkManager::new())),
    };

    // Start message handler for incoming network messages
    let app_state_clone = AppState {
        current_party: Arc::clone(&app_state.current_party),
        current_user: Arc::clone(&app_state.current_user),
        networking: Arc::clone(&app_state.networking),
    };
    
    tokio::spawn(async move {
        handle_incoming_messages(app_state_clone).await;
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::create_party,
            commands::join_party,
            commands::leave_party,
            commands::send_message,
            commands::get_current_party,
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
            commands::leave_call,
            commands::get_user_ip,
            commands::validate_invite,
            commands::test_network_connectivity
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn handle_incoming_messages(app_state: AppState) {
    let mut networking = app_state.networking.lock().await;
    let receiver = networking.message_receiver.take();
    drop(networking);
    
    if let Some(mut receiver) = receiver {
        println!("🎧 Started network message handler");
        
        while let Some(message) = receiver.recv().await {
            match message.message_type {
                networking::MessageType::ChatMessage => {
                    if let Ok(chat_message) = serde_json::from_value::<room::ChatMessage>(message.payload) {
                        let mut current_party = app_state.current_party.lock().await;
                        if let Some(party) = current_party.as_mut() {
                            party.add_message(chat_message.clone());
                            println!("📨 RECEIVED MESSAGE from {}", chat_message.user_name);
                            println!("📥 [{}] {}: {}", 
                                chat_message.timestamp.format("%H:%M:%S"), 
                                chat_message.user_name, 
                                chat_message.content);
                        }
                    }
                }
                networking::MessageType::UserJoined => {
                    if let Ok(user) = serde_json::from_value::<user::User>(message.payload) {
                        let mut current_party = app_state.current_party.lock().await;
                        if let Some(party) = current_party.as_mut() {
                            let _ = party.add_user(user.clone());
                            println!("🎉 NEW USER JOINED: {} (ID: {})", user.name, user.id);
                            println!("📍 User address: {}", user.address);
                            
                            // Log full user list
                            println!("👥 FULL PARTY MEMBER LIST:");
                            for (i, (_, member)) in party.users.iter().enumerate() {
                                let status = if member.is_online { "🟢 Online" } else { "🔴 Offline" };
                                println!("   {}. {} {} ({})", i + 1, member.name, status, member.address);
                            }
                            println!("📊 Total members: {}", party.users.len());
                        }
                    }
                }
                networking::MessageType::UserLeft => {
                    if let Ok(user_id) = serde_json::from_value::<String>(message.payload) {
                        if let Ok(user_uuid) = uuid::Uuid::parse_str(&user_id) {
                            let mut current_party = app_state.current_party.lock().await;
                            if let Some(party) = current_party.as_mut() {
                                let _ = party.remove_user(user_uuid);
                                println!("👋 User left the party");
                            }
                        }
                    }
                }
                _ => {
                    println!("📨 Received message type: {:?}", message.message_type);
                }
            }
        }
    }
}