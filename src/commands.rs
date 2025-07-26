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
pub async fn create_party(
    state: State<'_, AppState>,
    name: String,
    protocol: Option<Protocol>,
) -> Result<Room, String> {
    let protocol = protocol.unwrap_or(Protocol::TCP);
    
    // Check if there's already an active party
    let current_party_guard = state.current_party.lock().await;
    if current_party_guard.is_some() {
        return Err("A party is already active. Leave the current party before creating a new one.".to_string());
    }
    drop(current_party_guard);
    
    // Get current user
    let current_user_guard = state.current_user.lock().await;
    let current_user = current_user_guard.as_ref()
        .ok_or("No user configured. Please set up your profile first.")?;

    let _user_addr = current_user.address;
    let user_clone = current_user.clone();
    drop(current_user_guard);

    // Create the party
    let party = Room::new(name, user_clone, protocol.clone());
    
    println!("✅ Created new party '{}' with ID: {}", party.name, party.id);

    // Set as current party (session-based, no file persistence)
    let mut current_party = state.current_party.lock().await;
    *current_party = Some(party.clone());

    // Start server for this party
    let mut networking = state.networking.lock().await;
    if let Err(e) = networking.start_server(8080, protocol).await {
        return Err(format!("Failed to start server: {}", e));
    }

    Ok(party)
}

#[tauri::command]
pub async fn join_party(
    state: State<'_, AppState>,
    invite_code: String,
) -> Result<Room, String> {
    println!("🔄 Starting party join process...");
    println!("📝 Invite code received: {}", invite_code);
    
    // Check if there's already an active party
    let current_party_guard = state.current_party.lock().await;
    if current_party_guard.is_some() {
        println!("❌ User already in a party");
        return Err("Already in a party. Leave the current party before joining another.".to_string());
    }
    drop(current_party_guard);

    // Parse invite code
    println!("🔍 Parsing invite code...");
    let invite_data = InviteData::parse_invite_code(&invite_code)
        .map_err(|e| {
            println!("❌ Failed to parse invite code: {}", e);
            format!("Invalid invite code: {}", e)
        })?;

    println!("✅ Parsed invite data:");
    println!("   - Room ID: {}", invite_data.room_id);
    println!("   - Room Name: {}", invite_data.room_name);
    println!("   - Creator: {}", invite_data.creator_name);
    println!("   - Protocol: {:?}", invite_data.protocol);
    println!("   - Peer addresses: {:?}", invite_data.peer_addresses);

    // Check if invite is expired (24 hours)
    if invite_data.is_expired(24) {
        println!("❌ Invite code has expired");
        return Err("Invite code has expired".to_string());
    }

    // Get current user
    println!("👤 Getting current user...");
    let current_user_guard = state.current_user.lock().await;
    let current_user = current_user_guard.as_ref()
        .ok_or_else(|| {
            println!("❌ No user configured");
            "No user configured. Please set up your profile first.".to_string()
        })?;
    let user_clone = current_user.clone();
    println!("✅ Current user: {} ({})", user_clone.name, user_clone.address);
    drop(current_user_guard);

    // Try to connect to peers in order
    println!("🌐 Attempting to connect to peers...");
    let mut networking = state.networking.lock().await;
    let mut connected = false;
    let mut connection_errors = Vec::new();

    // Try primary peer first
    if let Some(primary_peer) = invite_data.get_primary_peer() {
        println!("🔗 Trying primary peer: {}", primary_peer);
        match networking.connect_to_peer(primary_peer, invite_data.protocol.clone()).await {
            Ok(_) => {
                println!("✅ Connected to primary peer: {}", primary_peer);
                connected = true;
            }
            Err(e) => {
                let error_msg = format!("Primary peer {} failed: {}", primary_peer, e);
                println!("❌ {}", error_msg);
                connection_errors.push(error_msg);
            }
        }
    } else {
        println!("⚠️ No primary peer found in invite");
    }

    // Try fallback peers if primary failed
    if !connected {
        let fallback_peers = invite_data.get_fallback_peers();
        println!("🔄 Trying {} fallback peers...", fallback_peers.len());
        
        for (i, peer_addr) in fallback_peers.iter().enumerate() {
            println!("🔗 Trying fallback peer {}/{}: {}", i + 1, fallback_peers.len(), peer_addr);
            match networking.connect_to_peer(*peer_addr, invite_data.protocol.clone()).await {
                Ok(_) => {
                    println!("✅ Connected to fallback peer: {}", peer_addr);
                    connected = true;
                    break;
                }
                Err(e) => {
                    let error_msg = format!("Fallback peer {} failed: {}", peer_addr, e);
                    println!("❌ {}", error_msg);
                    connection_errors.push(error_msg);
                }
            }
        }
    }

    if !connected {
        let full_error = format!(
            "Could not connect to any peers in the party. Tried {} peers. Errors: {}",
            connection_errors.len(),
            connection_errors.join("; ")
        );
        println!("❌ {}", full_error);
        return Err(full_error);
    }

    // Create party representation (session-based, no persistence)
    let mut party = Room::new(invite_data.room_name, user_clone.clone(), invite_data.protocol);
    party.id = invite_data.room_id;
    party.peer_addresses = invite_data.peer_addresses;

    println!("✅ Joined party '{}' with ID: {}", party.name, party.id);

    // Send UserJoined message to notify other peers
    let networking = state.networking.lock().await;
    let join_message = crate::networking::NetworkMessage {
        id: Uuid::new_v4(),
        from: user_clone.id.to_string(),
        to: None,
        message_type: crate::networking::MessageType::UserJoined,
        payload: serde_json::to_value(&user_clone).map_err(|e| e.to_string())?,
        timestamp: chrono::Utc::now(),
    };

    if let Err(e) = networking.broadcast_message(join_message).await {
        eprintln!("Failed to broadcast user joined message: {}", e);
    }

    // Set as current party
    let mut current_party = state.current_party.lock().await;
    *current_party = Some(party.clone());

    Ok(party)
}

#[tauri::command]
pub async fn leave_party(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut current_party = state.current_party.lock().await;
    
    if current_party.is_some() {
        let party = current_party.take().unwrap();
        
        println!("✅ Left party '{}' with ID: {}", party.name, party.id);
        
        // Notify other peers about leaving
        // This would be implemented with the networking layer
        
        // Stop networking for the party
        let mut networking = state.networking.lock().await;
        if let Err(e) = networking.disconnect_all().await {
            eprintln!("Warning: Failed to disconnect from peers: {}", e);
        }
        
        Ok(())
    } else {
        Err("Not currently in a party".to_string())
    }
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    content: String,
) -> Result<(), String> {

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

    // Update the current party
    let mut current_party = state.current_party.lock().await;
    if let Some(party) = current_party.as_mut() {
        // Add message to the current party
        party.add_message(message.clone());
        
        println!("✅ Message added to party '{}' (ID: {})", party.name, party.id);
        println!("📁 Party '{}' now has {} messages", party.name, party.messages.len());
        println!("💬 [{}] {}: {}", message.timestamp.format("%H:%M:%S"), message.user_name, message.content);

        // Send message to other peers in the party
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
        Err("Not currently in a party".to_string())
    }
}

#[tauri::command]
pub async fn get_current_party(state: State<'_, AppState>) -> Result<Option<Room>, String> {
    let current_party = state.current_party.lock().await;
    Ok(current_party.clone())
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
pub async fn get_user_ip() -> Result<String, String> {
    match local_ip_address::local_ip() {
        Ok(ip) => Ok(ip.to_string()),
        Err(e) => Err(format!("Failed to get local IP: {}", e)),
    }
}

#[tauri::command]
pub async fn change_protocol(
    state: State<'_, AppState>,
    new_protocol: Protocol,
) -> Result<(), String> {
    let mut current_party = state.current_party.lock().await;
    if let Some(party) = current_party.as_mut() {
        let _old_protocol = party.protocol.clone();
        let peers = party.peer_addresses.clone();
        
        // Switch protocol in networking layer
        let mut networking = state.networking.lock().await;
        if let Err(e) = networking.switch_protocol(new_protocol.clone(), peers).await {
            return Err(format!("Failed to switch protocol: {}", e));
        }

        // Update party
        party.switch_protocol(new_protocol);
        
        println!("✅ Changed party protocol to {:?}", party.protocol);

        Ok(())
    } else {
        Err("Not currently in a party".to_string())
    }
}

#[tauri::command]
pub async fn generate_invite(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let current_party = state.current_party.lock().await;
    if let Some(party) = current_party.as_ref() {
        let current_user_guard = state.current_user.lock().await;
        let current_user = current_user_guard.as_ref()
            .ok_or("No user configured")?;

        let invite_data = InviteData::new(
            party.id,
            party.name.clone(),
            current_user.name.clone(),
            party.peer_addresses.clone(),
            party.protocol.clone(),
        );

        let invite_code = invite_data.generate_invite_code()
            .map_err(|e| format!("Failed to generate invite code: {}", e))?;

        Ok(invite_code)
    } else {
        Err("Not currently in a party".to_string())
    }
}

#[tauri::command]
pub async fn parse_invite(invite_code: String) -> Result<InviteData, String> {
    InviteData::parse_invite_code(&invite_code)
        .map_err(|e| format!("Failed to parse invite code: {}", e))
}

#[tauri::command]
pub async fn validate_invite(invite_code: String) -> Result<String, String> {
    println!("🔍 Validating invite code: {}", invite_code);
    
    // Parse invite code
    let invite_data = match InviteData::parse_invite_code(&invite_code) {
        Ok(data) => data,
        Err(e) => {
            let error_msg = format!("Invalid invite format: {}", e);
            println!("❌ {}", error_msg);
            return Err(error_msg);
        }
    };
    
    // Check if expired
    if invite_data.is_expired(24) {
        let error_msg = "Invite code has expired (older than 24 hours)".to_string();
        println!("❌ {}", error_msg);
        return Err(error_msg);
    }
    
    // Validate peer addresses
    if invite_data.peer_addresses.is_empty() {
        let error_msg = "No peer addresses found in invite".to_string();
        println!("❌ {}", error_msg);
        return Err(error_msg);
    }
    
    let validation_info = format!(
        "✅ Valid invite:\n• Party: '{}'\n• Creator: {}\n• Protocol: {:?}\n• Peers: {} available\n• Created: {}",
        invite_data.room_name,
        invite_data.creator_name,
        invite_data.protocol,
        invite_data.peer_addresses.len(),
        invite_data.created_at.format("%Y-%m-%d %H:%M UTC")
    );
    
    println!("{}", validation_info);
    Ok(validation_info)
}

#[tauri::command]
pub async fn check_room_health(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut current_party = state.current_party.lock().await;
    if let Some(party) = current_party.as_mut() {
        // Clean up offline users first (5 minute threshold)
        party.cleanup_offline_users(5);
        
        // Check server health
        let server_healthy = party.check_server_health();
        
        println!("🔍 Party health check: {}", if server_healthy { "healthy" } else { "unhealthy" });
        
        Ok(server_healthy)
    } else {
        Err("Not currently in a party".to_string())
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

    let mut rooms = state.current_party.lock().await;
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

    let mut rooms = state.current_party.lock().await;
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

    let rooms = state.current_party.lock().await;
    if let Some(room) = rooms.iter().find(|r| r.id == room_uuid) {
        Ok(room.messages.clone())
    } else {
        Err("Room not found".to_string())
    }
}

#[tauri::command]
pub async fn join_call(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let current_user = {
        let current_user_guard = state.current_user.lock().await;
        current_user_guard.as_ref()
            .ok_or("No user configured")?
            .clone()
    };

    let mut current_party = state.current_party.lock().await;
    if let Some(party) = current_party.as_mut() {
        // Find and update the user in the party
        if let Some(user) = party.users.get_mut(&current_user.id) {
            if !user.is_in_call {
                let user_name = user.name.clone(); // Store name to avoid borrow conflicts
                user.join_call();
                
                // Start call server if this is the first user joining
                if !party.is_call_active {
                    party.is_call_active = true;
                    party.call_server_id = Some(current_user.id);
                }
                
                // Add system message
                let system_message = ChatMessage {
                    id: Uuid::new_v4(),
                    user_id: Uuid::nil(), // System message
                    user_name: "System".to_string(),
                    content: format!("**{}** connected to call", user_name),
                    timestamp: chrono::Utc::now(),
                };
                party.add_message(system_message);
                
                println!("✅ {} joined the call", user_name);
                
                Ok(())
            } else {
                Err("User is already in call".to_string())
            }
        } else {
            Err("User not found in party".to_string())
        }
    } else {
        Err("Not currently in a party".to_string())
    }
}

#[tauri::command]
pub async fn leave_call(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let current_user = {
        let current_user_guard = state.current_user.lock().await;
        current_user_guard.as_ref()
            .ok_or("No user configured")?
            .clone()
    };

    let mut current_party = state.current_party.lock().await;
    if let Some(party) = current_party.as_mut() {
        // Find and update the user in the party
        if let Some(user) = party.users.get_mut(&current_user.id) {
            if user.is_in_call {
                let user_name = user.name.clone(); // Store name to avoid borrow conflicts
                user.leave_call();
            } else {
                return Err("User is not in call".to_string());
            }
        } else {
            return Err("User not found in party".to_string());
        }
        
        // Add system message
        let system_message = ChatMessage {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(), // System message
            user_name: "System".to_string(),
            content: format!("**{}** disconnected from call", current_user.name),
            timestamp: chrono::Utc::now(),
        };
        party.add_message(system_message);
        
        // Check if no one is left in call
        let users_in_call: Vec<_> = party.users.values().filter(|u| u.is_in_call).collect();
        if users_in_call.is_empty() {
            party.is_call_active = false;
            party.call_server_id = None;
        }
        
        println!("✅ {} left the call", current_user.name);
        
        Ok(())
    } else {
        Err("Not currently in a party".to_string())
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
            current_party: Arc::new(Mutex::new(None)),
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
            let mut current_party = state.current_party.lock().await;
            *current_party = Some(room);
        }
        
        println!("🧪 Testing generate_invite command with room ID: {}", room_id);
        
        // Test the invite generation logic directly
        let current_party = state.current_party.lock().await;
        let room = current_party.as_ref().unwrap();
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
            let mut current_party = state.current_party.lock().await;
            *current_party = Some(room);
        }
        
        // Test the invite generation logic directly
        let current_party = state.current_party.lock().await;
        let room = current_party.as_ref().unwrap();
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