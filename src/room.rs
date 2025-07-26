use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::user::User;
use crate::networking::Protocol;
use anyhow::Result;
// use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub users: HashMap<Uuid, User>,
    pub messages: Vec<ChatMessage>,
    pub server_user_id: Option<Uuid>,
    pub protocol: Protocol,
    pub peer_addresses: Vec<SocketAddr>,
    pub ping_measurements: HashMap<Uuid, u64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_voice_enabled: bool,
    pub call_server_id: Option<Uuid>,
    pub is_call_active: bool,
}

impl Room {
    pub fn new(name: String, creator: User, protocol: Protocol) -> Self {
        let room_id = Uuid::new_v4();
        let creator_id = creator.id;
        
        println!("🆔 Creating new room '{}' with unique ID: {}", name, room_id);
        let mut users = HashMap::new();
        let peer_addresses = vec![creator.address];
        
        users.insert(creator_id, creator);

        Self {
            id: room_id,
            name,
            creator_id,
            users,
            messages: Vec::new(),
            server_user_id: Some(creator_id), // Creator starts as server
            protocol,
            peer_addresses,
            ping_measurements: HashMap::new(),
            created_at: chrono::Utc::now(),
            is_voice_enabled: false,
            call_server_id: Some(creator_id), // Creator starts as call server
            is_call_active: false,
        }
    }

    pub fn add_user(&mut self, mut user: User) -> Result<()> {
        // Check if user ID already exists in this room
        if self.users.contains_key(&user.id) {
            println!("⚠️ User ID {} already exists in room '{}', user not added", user.id, self.name);
            return Err(anyhow::anyhow!("User ID already exists in this room"));
        }
        
        // Handle name collision by adding connection order suffix
        let original_name = user.name.clone();
        user.name = self.resolve_name_collision(original_name);
        
        if !self.peer_addresses.contains(&user.address) {
            self.peer_addresses.push(user.address);
        }
        
        println!("✅ Added user '{}' (ID: {}) to room '{}'", user.name, user.id, self.name);
        self.users.insert(user.id, user);
        Ok(())
    }

    pub fn resolve_name_collision(&self, desired_name: String) -> String {
        let existing_names: Vec<&String> = self.users.values().map(|u| &u.name).collect();
        
        // If name doesn't exist, use it as-is
        if !existing_names.iter().any(|&name| name == &desired_name) {
            return desired_name;
        }
        
        // Find the highest suffix number for this base name
        let mut highest_suffix = 1;
        for existing_name in existing_names {
            if existing_name.starts_with(&desired_name) {
                if let Some(suffix_str) = existing_name.strip_prefix(&format!("{}-", desired_name)) {
                    if let Ok(suffix_num) = suffix_str.parse::<u32>() {
                        highest_suffix = highest_suffix.max(suffix_num + 1);
                    }
                } else if existing_name == &desired_name {
                    // The exact name exists, so we need at least suffix 2
                    highest_suffix = highest_suffix.max(2);
                }
            }
        }
        
        format!("{}-{}", desired_name, highest_suffix)
    }

    pub fn remove_user(&mut self, user_id: Uuid) -> Result<()> {
        if let Some(user) = self.users.remove(&user_id) {
            // Remove from peer addresses
            self.peer_addresses.retain(|&addr| addr != user.address);
            
            // If this was the server, elect a new one
            if self.server_user_id == Some(user_id) {
                self.elect_new_server();
            }
        }
        Ok(())
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        
        // Keep only last 1000 messages
        if self.messages.len() > 1000 {
            self.messages.drain(0..self.messages.len() - 1000);
        }
    }

    pub fn update_ping(&mut self, user_id: Uuid, ping_ms: u64) {
        self.ping_measurements.insert(user_id, ping_ms);
        
        // Update user's last_seen timestamp and online status
        if let Some(user) = self.users.get_mut(&user_id) {
            user.update_last_seen();
            user.is_online = true;
        }
    }

    fn is_ping_recent(&self, user_id: Uuid) -> bool {
        if let Some(user) = self.users.get(&user_id) {
            let now = chrono::Utc::now();
            let time_diff = now.signed_duration_since(user.last_seen);
            time_diff.num_minutes() < 5 // Consider ping recent if within 5 minutes
        } else {
            false
        }
    }

    pub fn elect_new_server(&mut self) {
        // Find online users with valid ping measurements
        let best_candidate = self.ping_measurements
            .iter()
            .filter(|(user_id, ping)| {
                // Check if user exists, is online, and has recent ping data
                if let Some(user) = self.users.get(user_id) {
                    user.is_online && **ping < u64::MAX && self.is_ping_recent(**user_id)
                } else {
                    false
                }
            })
            .min_by_key(|(_, ping)| *ping)
            .map(|(user_id, _)| *user_id);

        // If no candidate with ping data, fall back to any online user
        let fallback_candidate = if best_candidate.is_none() {
            self.users
                .iter()
                .find(|(_, user)| user.is_online)
                .map(|(user_id, _)| *user_id)
        } else {
            None
        };

        self.server_user_id = best_candidate.or(fallback_candidate);
        
        if let Some(new_server_id) = self.server_user_id {
            if let Some(new_server) = self.users.get(&new_server_id) {
                println!("🏆 Elected new server: '{}' (ID: {})", new_server.name, new_server_id);
            }
        } else {
            println!("⚠️ No online users available to elect as server");
        }
    }

    pub fn get_ordered_peer_list(&self) -> Vec<SocketAddr> {
        let mut peers: Vec<(SocketAddr, u64)> = self.users
            .values()
            .filter_map(|user| {
                let ping = self.ping_measurements.get(&user.id).copied().unwrap_or(u64::MAX);
                Some((user.address, ping))
            })
            .collect();

        // Sort by ping (lowest first)
        peers.sort_by_key(|(_, ping)| *ping);
        peers.into_iter().map(|(addr, _)| addr).collect()
    }

    pub fn switch_protocol(&mut self, new_protocol: Protocol) {
        self.protocol = new_protocol;
    }

    pub fn save_to_file(&self) -> Result<()> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find app data directory"))?
            .join("shortgap")
            .join("rooms");

        std::fs::create_dir_all(&app_data_dir)?;
        
        let file_path = app_data_dir.join(format!("{}.json", self.id));
        let json_data = serde_json::to_string_pretty(self)?;
        std::fs::write(file_path, json_data)?;
        
        Ok(())
    }

    pub fn load_from_file(room_id: Uuid) -> Result<Self> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find app data directory"))?
            .join("shortgap")
            .join("rooms");

        let file_path = app_data_dir.join(format!("{}.json", room_id));
        
        if !file_path.exists() {
            return Err(anyhow::anyhow!("Room file does not exist"));
        }

        let json_data = std::fs::read_to_string(file_path)?;
        let room: Room = serde_json::from_str(&json_data)?;
        
        Ok(room)
    }

    pub fn list_saved_rooms() -> Result<Vec<Uuid>> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find app data directory"))?
            .join("shortgap")
            .join("rooms");

        if !app_data_dir.exists() {
            return Ok(Vec::new());
        }

        let mut room_ids = Vec::new();
        
        for entry in std::fs::read_dir(app_data_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(file_name) = path.file_stem() {
                if let Some(file_name_str) = file_name.to_str() {
                    if let Ok(room_id) = Uuid::parse_str(file_name_str) {
                        room_ids.push(room_id);
                    }
                }
            }
        }
        
        Ok(room_ids)
    }

    pub fn get_server_user(&self) -> Option<&User> {
        self.server_user_id
            .and_then(|id| self.users.get(&id))
    }

    pub fn is_user_server(&self, user_id: Uuid) -> bool {
        self.server_user_id == Some(user_id)
    }

    pub fn mark_user_offline(&mut self, user_id: Uuid) -> Result<()> {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.is_online = false;
            println!("📴 Marked user '{}' as offline", user.name);
            
            // If this was the server, elect a new one
            if self.server_user_id == Some(user_id) {
                println!("🔄 Current server '{}' went offline, electing new server", user.name);
                self.elect_new_server();
            }
        }
        Ok(())
    }

    pub fn mark_user_online(&mut self, user_id: Uuid) -> Result<()> {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.is_online = true;
            user.update_last_seen();
            println!("📶 Marked user '{}' as online", user.name);
        }
        Ok(())
    }

    pub fn cleanup_offline_users(&mut self, offline_threshold_minutes: i64) {
        let now = chrono::Utc::now();
        let mut users_to_mark_offline = Vec::new();
        
        for (user_id, user) in &self.users {
            if user.is_online {
                let time_diff = now.signed_duration_since(user.last_seen);
                if time_diff.num_minutes() > offline_threshold_minutes {
                    users_to_mark_offline.push(*user_id);
                }
            }
        }
        
        for user_id in users_to_mark_offline {
            let _ = self.mark_user_offline(user_id);
        }
        
        // Remove stale ping measurements for offline users
        let now = chrono::Utc::now();
        self.ping_measurements.retain(|user_id, _| {
            if let Some(user) = self.users.get(user_id) {
                if user.is_online {
                    true
                } else {
                    // Check if ping is recent without calling self.is_ping_recent to avoid borrow issues
                    let time_diff = now.signed_duration_since(user.last_seen);
                    time_diff.num_minutes() < 5
                }
            } else {
                false
            }
        });
    }

    pub fn check_server_health(&mut self) -> bool {
        if let Some(server_id) = self.server_user_id {
            if let Some(server_user) = self.users.get(&server_id) {
                if !server_user.is_online {
                    println!("💔 Current server is offline, triggering re-election");
                    self.elect_new_server();
                    return false;
                }
                
                // Check if server ping is too old
                if !self.is_ping_recent(server_id) {
                    println!("⏰ Current server ping is stale, triggering re-election");
                    self.elect_new_server();
                    return false;
                }
                
                return true;
            } else {
                println!("👻 Current server user not found, triggering re-election");
                self.elect_new_server();
                return false;
            }
        } else {
            println!("🚫 No server assigned, triggering election");
            self.elect_new_server();
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use crate::networking::Protocol;

    #[test]
    fn test_server_election_with_offline_users() {
        // Create test users
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082);
        
        let user1 = User::new("User1".to_string(), addr1);
        let user2 = User::new("User2".to_string(), addr2);
        let user3 = User::new("User3".to_string(), addr3);
        
        let creator_id = user1.id;
        let user2_id = user2.id;
        let user3_id = user3.id;
        
        // Create room with user1 as creator/server
        let mut room = Room::new("Test Room".to_string(), user1, Protocol::TCP);
        room.add_user(user2).unwrap();
        room.add_user(user3).unwrap();
        
        // Set ping measurements
        room.update_ping(creator_id, 50);
        room.update_ping(user2_id, 30); // Best ping
        room.update_ping(user3_id, 100);
        
        // Verify initial server is creator
        assert_eq!(room.server_user_id, Some(creator_id));
        
        // Mark creator (current server) as offline
        room.mark_user_offline(creator_id).unwrap();
        
        // Server should now be user2 (best ping among online users)
        assert_eq!(room.server_user_id, Some(user2_id));
        
        // Mark user2 as offline too
        room.mark_user_offline(user2_id).unwrap();
        
        // Server should now be user3 (only online user left)
        assert_eq!(room.server_user_id, Some(user3_id));
        
        // Mark all users as offline
        room.mark_user_offline(user3_id).unwrap();
        
        // No server should be assigned
        assert_eq!(room.server_user_id, None);
    }

    #[test]
    fn test_server_health_check() {
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        
        let user1 = User::new("User1".to_string(), addr1);
        let user2 = User::new("User2".to_string(), addr2);
        
        let creator_id = user1.id;
        let user2_id = user2.id;
        
        let mut room = Room::new("Test Room".to_string(), user1, Protocol::TCP);
        room.add_user(user2).unwrap();
        
        // Set ping measurements
        room.update_ping(creator_id, 50);
        room.update_ping(user2_id, 30);
        
        // Server health should be good initially
        assert!(room.check_server_health());
        assert_eq!(room.server_user_id, Some(creator_id));
        
        // Mark current server as offline
        room.mark_user_offline(creator_id).unwrap();
        
        // Health check should detect offline server and elect new one
        assert!(room.check_server_health());
        assert_eq!(room.server_user_id, Some(user2_id));
    }

    #[test]
    fn test_cleanup_offline_users() {
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        
        let mut user1 = User::new("User1".to_string(), addr1);
        let user2 = User::new("User2".to_string(), addr2);
        
        // Make user1's last_seen old
        user1.last_seen = chrono::Utc::now() - chrono::Duration::minutes(10);
        
        let user1_id = user1.id;
        let user2_id = user2.id;
        
        let mut room = Room::new("Test Room".to_string(), user1, Protocol::TCP);
        room.add_user(user2).unwrap();
        
        // Both users should be online initially
        assert!(room.users.get(&user1_id).unwrap().is_online);
        assert!(room.users.get(&user2_id).unwrap().is_online);
        
        // Cleanup with 5 minute threshold
        room.cleanup_offline_users(5);
        
        // User1 should be marked offline due to old last_seen
        assert!(!room.users.get(&user1_id).unwrap().is_online);
        assert!(room.users.get(&user2_id).unwrap().is_online);
        
        // Server should have been re-elected to user2
        assert_eq!(room.server_user_id, Some(user2_id));
    }
}