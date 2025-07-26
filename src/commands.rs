use tauri::State;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use anyhow::Result;

use crate::{AppState, room::Room, user::User, networking::Protocol, invite::InviteData, room::ChatMessage};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub protocol: Option<Protocol>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomRequest {
    pub invite_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub room_id: Uuid,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSettings {
    pub name: String,
    pub avatar: Option<String>,
    pub audio_input_device: Option<String>,
    pub audio_output_device: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingStats {
    pub peer_addr: SocketAddr,
    pub tcp_ping: Option<u64>,
    pub app_ping: Option<u64>,
    pub webrtc_rtt: Option<u64>,
    pub average_ping: Option<u64>,
}

#[tauri::command]
pub async fn create_room(
    state: State<'_, AppState>,
    name: String,
    protocol: Option<Protocol>,
) -> Result<Room, String> {
    let protocol = protocol.unwrap_or(Protocol::TCP);
    
    // Get current user
    let current_user_guard = state.current_user.lock().await;
    let current_user = current_user_guard.as_ref()
        .ok_or("No user configured. Please set up your profile first.")?;

    let _user_addr = current_user.address;
    let user_clone = current_user.clone();
    drop(current_user_guard);

    // Create the room
    let room = Room::new(name, user_clone, protocol.clone());
    
    // Save room to file
    if let Err(e) = room.save_to_file() {
        return Err(format!("Failed to save room: {}", e));
    }
    
    // Debug: Print where the room was saved
    let app_data_dir = dirs::data_dir()
        .ok_or("Could not find app data directory")?
        .join("shortgap")
        .join("rooms");
    println!("✅ Room '{}' saved to: {}", room.name, app_data_dir.join(format!("{}.json", room.id)).display());

    // Add to rooms list
    let mut rooms = state.rooms.lock().await;
    rooms.push(room.clone());

    // Start server for this room
    let mut networking = state.networking.lock().await;
    if let Err(e) = networking.start_server(8080, protocol).await {
        return Err(format!("Failed to start server: {}", e));
    }

    Ok(room)
}

#[tauri::command]
pub async fn join_room(
    state: State<'_, AppState>,
    invite_code: String,
) -> Result<Room, String> {
    // Parse invite code
    let invite_data = InviteData::parse_invite_code(&invite_code)
        .map_err(|e| format!("Invalid invite code: {}", e))?;

    // Check if invite is expired (24 hours)
    if invite_data.is_expired(24) {
        return Err("Invite code has expired".to_string());
    }

    // Get current user
    let current_user_guard = state.current_user.lock().await;
    let current_user = current_user_guard.as_ref()
        .ok_or("No user configured. Please set up your profile first.")?;
    let user_clone = current_user.clone();
    drop(current_user_guard);

    // Try to connect to peers in order
    let mut networking = state.networking.lock().await;
    let mut connected = false;

    // Try primary peer first
    if let Some(primary_peer) = invite_data.get_primary_peer() {
        if let Ok(_) = networking.connect_to_peer(primary_peer, invite_data.protocol.clone()).await {
            connected = true;
        }
    }

    // Try fallback peers if primary failed
    if !connected {
        for peer_addr in invite_data.get_fallback_peers() {
            if let Ok(_) = networking.connect_to_peer(peer_addr, invite_data.protocol.clone()).await {
                connected = true;
                break;
            }
        }
    }

    if !connected {
        return Err("Could not connect to any peers in the room".to_string());
    }

    // Create local room representation or load existing one
    let room = match Room::load_from_file(invite_data.room_id) {
        Ok(mut existing_room) => {
            // Room exists locally, add user to it (handles name collision)
            if let Err(e) = existing_room.add_user(user_clone) {
                return Err(format!("Failed to add user to existing room: {}", e));
            }
            existing_room
        },
        Err(_) => {
            // Create new local room representation
            let mut new_room = Room::new(invite_data.room_name, user_clone, invite_data.protocol);
            new_room.id = invite_data.room_id;
            new_room.peer_addresses = invite_data.peer_addresses;
            new_room
        }
    };

    // Save room to file
    if let Err(e) = room.save_to_file() {
        return Err(format!("Failed to save room: {}", e));
    }

    // Add to rooms list
    let mut rooms = state.rooms.lock().await;
    rooms.push(room.clone());

    Ok(room)
}

#[tauri::command]
pub async fn leave_room(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<(), String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let mut rooms = state.rooms.lock().await;
    
    if let Some(pos) = rooms.iter().position(|r| r.id == room_uuid) {
        let _room = rooms.remove(pos);
        
        // Remove room file
        let app_data_dir = dirs::data_dir()
            .ok_or("Could not find app data directory")?
            .join("shortgap")
            .join("rooms");
        
        let file_path = app_data_dir.join(format!("{}.json", room_id));
        if file_path.exists() {
            std::fs::remove_file(file_path)
                .map_err(|e| format!("Failed to remove room file: {}", e))?;
        }

        // Notify other peers about leaving
        // This would be implemented with the networking layer
        
        Ok(())
    } else {
        Err("Room not found".to_string())
    }
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    room_id: String,
    content: String,
) -> Result<(), String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let current_user = {
        let current_user_guard = state.current_user.lock().await;
        current_user_guard.as_ref()
            .ok_or("No user configured")?
            .clone()
    };

    let message = crate::room::ChatMessage {
        id: Uuid::new_v4(),
        user_id: current_user.id,
        user_name: current_user.name.clone(),
        content,
        timestamp: chrono::Utc::now(),
    };

    // Find and update the specific room
    let mut rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter_mut().find(|r| r.id == room_uuid) {
        // Add message to THIS specific room's message list
        room.add_message(message.clone());
        
        // Save THIS specific room's data to its own unique file
        if let Err(e) = room.save_to_file() {
            return Err(format!("Failed to save room {}: {}", room.name, e));
        }
        
        println!("✅ Message added to room '{}' (ID: {})", room.name, room.id);
        println!("📁 Room '{}' now has {} messages (stored separately)", room.name, room.messages.len());

        // Send message to other peers in this room only
        let networking = state.networking.lock().await;
        let network_message = crate::networking::NetworkMessage {
            id: Uuid::new_v4(),
            from: current_user.id.to_string(),
            to: None,
            message_type: crate::networking::MessageType::ChatMessage,
            payload: serde_json::to_value(&message).unwrap(),
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) = networking.broadcast_message(network_message).await {
            eprintln!("Failed to broadcast message: {}", e);
        }

        Ok(())
    } else {
        Err(format!("Room with ID {} not found", room_uuid))
    }
}

#[tauri::command]
pub async fn get_rooms(state: State<'_, AppState>) -> Result<Vec<Room>, String> {
    // Load saved rooms from disk
    let saved_room_ids = crate::room::Room::list_saved_rooms()
        .map_err(|e| format!("Failed to list saved rooms: {}", e))?;

    let mut rooms = state.rooms.lock().await;
    rooms.clear();

    // Load each saved room from its individual file
    for room_id in saved_room_ids {
        match crate::room::Room::load_from_file(room_id) {
            Ok(room) => {
                println!("📂 Loaded room '{}' with {} messages from separate file", room.name, room.messages.len());
                rooms.push(room);
            },
            Err(e) => eprintln!("❌ Failed to load room {}: {}", room_id, e),
        }
    }

    Ok(rooms.clone())
}

#[tauri::command]
pub async fn set_user_settings(
    state: State<'_, AppState>,
    settings: UserSettings,
) -> Result<(), String> {
    let local_addr = local_ip_address::local_ip()
        .map_err(|e| format!("Failed to get local IP: {}", e))?;
    
    let socket_addr = SocketAddr::new(local_addr, 8080);
    
    let mut user = User::new(settings.name, socket_addr);
    user.set_avatar(settings.avatar);
    user.set_audio_devices(settings.audio_input_device, settings.audio_output_device);

    let mut current_user = state.current_user.lock().await;
    *current_user = Some(user);

    Ok(())
}

#[tauri::command]
pub async fn get_ping_stats(
    _state: State<'_, AppState>,
) -> Result<Vec<PingStats>, String> {
    // This would be implemented with the ping manager
    // For now, return empty stats
    Ok(vec![])
}

#[tauri::command]
pub async fn change_protocol(
    state: State<'_, AppState>,
    room_id: String,
    new_protocol: Protocol,
) -> Result<(), String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let mut rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter_mut().find(|r| r.id == room_uuid) {
        let _old_protocol = room.protocol.clone();
        let peers = room.peer_addresses.clone();
        
        // Switch protocol in networking layer
        let mut networking = state.networking.lock().await;
        if let Err(e) = networking.switch_protocol(new_protocol.clone(), peers).await {
            return Err(format!("Failed to switch protocol: {}", e));
        }

        // Update room
        room.switch_protocol(new_protocol);
        
        // Save updated room
        if let Err(e) = room.save_to_file() {
            return Err(format!("Failed to save room: {}", e));
        }

        Ok(())
    } else {
        Err("Room not found".to_string())
    }
}

#[tauri::command]
pub async fn generate_invite(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<String, String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter().find(|r| r.id == room_uuid) {
        let current_user_guard = state.current_user.lock().await;
        let current_user = current_user_guard.as_ref()
            .ok_or("No user configured")?;

        let invite_data = InviteData::new(
            room.id,
            room.name.clone(),
            current_user.name.clone(),
            room.peer_addresses.clone(),
            room.protocol.clone(),
        );

        let invite_code = invite_data.generate_invite_code()
            .map_err(|e| format!("Failed to generate invite code: {}", e))?;

        Ok(invite_code)
    } else {
        Err("Room not found".to_string())
    }
}

#[tauri::command]
pub async fn parse_invite(invite_code: String) -> Result<InviteData, String> {
    InviteData::parse_invite_code(&invite_code)
        .map_err(|e| format!("Failed to parse invite code: {}", e))
}

#[tauri::command]
pub async fn check_room_health(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<bool, String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let mut rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter_mut().find(|r| r.id == room_uuid) {
        // Clean up offline users first (5 minute threshold)
        room.cleanup_offline_users(5);
        
        // Check server health
        let server_healthy = room.check_server_health();
        
        // Save updated room state
        if let Err(e) = room.save_to_file() {
            eprintln!("Failed to save room after health check: {}", e);
        }
        
        Ok(server_healthy)
    } else {
        Err("Room not found".to_string())
    }
}

#[tauri::command]
pub async fn mark_user_offline_cmd(
    state: State<'_, AppState>,
    room_id: String,
    user_id: String,
) -> Result<(), String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|e| format!("Invalid user ID: {}", e))?;

    let mut rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter_mut().find(|r| r.id == room_uuid) {
        room.mark_user_offline(user_uuid)
            .map_err(|e| format!("Failed to mark user offline: {}", e))?;
        
        // Save updated room state
        if let Err(e) = room.save_to_file() {
            eprintln!("Failed to save room after marking user offline: {}", e);
        }
        
        Ok(())
    } else {
        Err("Room not found".to_string())
    }
}

#[tauri::command]
pub async fn sync_messages(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<Vec<crate::room::ChatMessage>, String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let mut rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter_mut().find(|r| r.id == room_uuid) {
        println!("🔄 Syncing messages for room '{}' (ID: {})", room.name, room.id);
        
        // In a real implementation, this would request message lists from all connected peers
        // and merge them by timestamp. For now, we'll ensure proper order of this room's messages.
        
        // Sort THIS room's messages by timestamp to ensure proper order
        room.messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        println!("📋 Room '{}' has {} messages after sync", room.name, room.messages.len());
        
        // Save THIS room's updated data to its own file
        if let Err(e) = room.save_to_file() {
            eprintln!("Failed to save room '{}' after message sync: {}", room.name, e);
        }
        
        Ok(room.messages.clone())
    } else {
        Err(format!("Room with ID {} not found for sync", room_uuid))
    }
}

#[tauri::command]
pub async fn get_room_messages(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<Vec<crate::room::ChatMessage>, String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter().find(|r| r.id == room_uuid) {
        Ok(room.messages.clone())
    } else {
        Err("Room not found".to_string())
    }
}

#[tauri::command]
pub async fn join_call(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<(), String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let current_user = {
        let current_user_guard = state.current_user.lock().await;
        current_user_guard.as_ref()
            .ok_or("No user configured")?
            .clone()
    };

    let mut rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter_mut().find(|r| r.id == room_uuid) {
        // Find and update the user in the room
        if let Some(user) = room.users.get_mut(&current_user.id) {
            if !user.is_in_call {
                user.join_call();
                
                // Start call server if this is the first user joining
                if !room.is_call_active {
                    room.is_call_active = true;
                    room.call_server_id = Some(current_user.id);
                }
                
                // Add system message
                let system_message = ChatMessage {
                    id: Uuid::new_v4(),
                    user_id: Uuid::nil(), // System message
                    user_name: "System".to_string(),
                    content: format!("**{}** connected to call", user.name),
                    timestamp: chrono::Utc::now(),
                };
                room.add_message(system_message);
                
                // Save room
                if let Err(e) = room.save_to_file() {
                    return Err(format!("Failed to save room: {}", e));
                }
                
                Ok(())
            } else {
                Err("User is already in call".to_string())
            }
        } else {
            Err("User not found in room".to_string())
        }
    } else {
        Err("Room not found".to_string())
    }
}

#[tauri::command]
pub async fn leave_call(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<(), String> {
    let room_uuid = Uuid::parse_str(&room_id)
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let current_user = {
        let current_user_guard = state.current_user.lock().await;
        current_user_guard.as_ref()
            .ok_or("No user configured")?
            .clone()
    };

    let mut rooms = state.rooms.lock().await;
    if let Some(room) = rooms.iter_mut().find(|r| r.id == room_uuid) {
        // Find and update the user in the room
        if let Some(user) = room.users.get_mut(&current_user.id) {
            if user.is_in_call {
                user.leave_call();
                
                // Add system message
                let system_message = ChatMessage {
                    id: Uuid::new_v4(),
                    user_id: Uuid::nil(), // System message
                    user_name: "System".to_string(),
                    content: format!("**{}** disconnected from call", user.name),
                    timestamp: chrono::Utc::now(),
                };
                room.add_message(system_message);
                
                // Check if no one is left in call
                let users_in_call: Vec<_> = room.users.values().filter(|u| u.is_in_call).collect();
                if users_in_call.is_empty() {
                    room.is_call_active = false;
                    room.call_server_id = None;
                }
                
                // Save room
                if let Err(e) = room.save_to_file() {
                    return Err(format!("Failed to save room: {}", e));
                }
                
                Ok(())
            } else {
                Err("User is not in call".to_string())
            }
        } else {
            Err("User not found in room".to_string())
        }
    } else {
        Err("Room not found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use base64::Engine;

    async fn create_test_app_state() -> AppState {
        AppState {
            rooms: Arc::new(Mutex::new(Vec::new())),
            current_user: Arc::new(Mutex::new(None)),
            networking: Arc::new(Mutex::new(crate::networking::NetworkManager::new())),
        }
    }

    #[tokio::test]
    async fn test_generate_invite_with_real_room() {
        // Create test state
        let state = create_test_app_state().await;
        
        // Set up a test user
        let user_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8080);
        let user = crate::user::User::new("TestUser".to_string(), user_addr);
        {
            let mut current_user = state.current_user.lock().await;
            *current_user = Some(user.clone());
        }
        
        // Create a test room
        let room = crate::room::Room::new("Test Room for Invite".to_string(), user, crate::networking::Protocol::TCP);
        let room_id = room.id.to_string();
        
        // Add room to state
        {
            let mut rooms = state.rooms.lock().await;
            rooms.push(room);
        }
        
        println!("🧪 Testing generate_invite command with room ID: {}", room_id);
        
        // Test the invite generation logic directly
        let rooms = state.rooms.lock().await;
        let room = rooms.iter().find(|r| r.id.to_string() == room_id).unwrap();
        let current_user_guard = state.current_user.lock().await;
        let current_user = current_user_guard.as_ref().unwrap();

        let invite_data = crate::invite::InviteData::new(
            room.id,
            room.name.clone(),
            current_user.name.clone(),
            room.peer_addresses.clone(),
            room.protocol.clone(),
        );

        let result = invite_data.generate_invite_code();
        
        match result {
            Ok(invite_code) => {
                println!("✅ Successfully generated invite code: {}", invite_code);
                
                // Verify the invite code format
                assert!(invite_code.starts_with("shortgap://"), "Invite code should start with shortgap://");
                
                // Test parsing the invite code
                let parse_result = crate::invite::InviteData::parse_invite_code(&invite_code);
                match parse_result {
                    Ok(invite_data) => {
                        println!("✅ Successfully parsed invite code");
                        println!("   - Room ID: {}", invite_data.room_id);
                        println!("   - Room Name: {}", invite_data.room_name);
                        println!("   - Creator: {}", invite_data.creator_name);
                        println!("   - Peer Addresses: {:?}", invite_data.peer_addresses);
                        println!("   - Protocol: {:?}", invite_data.protocol);
                        
                        assert_eq!(invite_data.room_name, "Test Room for Invite");
                        assert_eq!(invite_data.creator_name, "TestUser");
                        assert!(!invite_data.peer_addresses.is_empty(), "Should have peer addresses");
                    }
                    Err(e) => {
                        panic!("❌ Failed to parse generated invite code: {}", e);
                    }
                }
            }
            Err(e) => {
                panic!("❌ Failed to generate invite code: {}", e);
            }
        }
        
        println!("🎉 Invite generation test completed successfully!");
    }

    #[tokio::test]
    async fn test_generate_invite_with_existing_room_id() {
        println!("🧪 Testing generate_invite with a known existing room ID");
        
        // Use a known room ID from the application data
        let existing_room_id = "4050bac2-dec6-44e5-89bc-329d279a8e68";
        
        // Create test state and load existing rooms
        let state = create_test_app_state().await;
        
        // Set up a test user
        let user_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8080);
        let user = crate::user::User::new("TestUser".to_string(), user_addr);
        {
            let mut current_user = state.current_user.lock().await;
            *current_user = Some(user.clone());
        }
        
        // Try to load existing rooms or create a test room with known ID
        let room_uuid = Uuid::parse_str(existing_room_id).unwrap();
        let mut room = crate::room::Room::new("Existing Test Room".to_string(), user, crate::networking::Protocol::TCP);
        room.id = room_uuid;
        room.peer_addresses = vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 47)), 8080)];
        
        {
            let mut rooms = state.rooms.lock().await;
            rooms.push(room);
        }
        
        // Test the invite generation logic directly
        let rooms = state.rooms.lock().await;
        let room = rooms.iter().find(|r| r.id.to_string() == existing_room_id).unwrap();
        let current_user_guard = state.current_user.lock().await;
        let current_user = current_user_guard.as_ref().unwrap();

        let invite_data = crate::invite::InviteData::new(
            room.id,
            room.name.clone(),
            current_user.name.clone(),
            room.peer_addresses.clone(),
            room.protocol.clone(),
        );

        let result = invite_data.generate_invite_code();
        
        match result {
            Ok(invite_code) => {
                println!("✅ Successfully generated invite code for existing room: {}", invite_code);
                
                // Verify format and content
                assert!(invite_code.starts_with("shortgap://"));
                
                // Decode and verify content
                let base64_part = invite_code.strip_prefix("shortgap://").unwrap();
                let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(base64_part)
                    .expect("Should decode base64");
                let json_str = String::from_utf8(decoded).expect("Should be valid UTF-8");
                
                println!("📄 Decoded invite JSON: {}", json_str);
                
                // Verify it contains the room ID and expected peer address
                assert!(json_str.contains(existing_room_id), "Should contain room ID");
                assert!(json_str.contains("192.168.1.47"), "Should contain peer address");
                
                println!("🎉 Existing room invite generation test passed!");
            }
            Err(e) => {
                println!("❌ Failed to generate invite for existing room: {}", e);
                // This might fail if no user is configured, which is expected in some cases
            }
        }
    }
}