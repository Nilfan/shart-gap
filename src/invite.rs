use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::net::SocketAddr;
use base64::{Engine as _, engine::general_purpose};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteData {
    pub room_id: Uuid,
    pub room_name: String,
    pub creator_name: String,
    pub peer_addresses: Vec<SocketAddr>,
    pub protocol: crate::networking::Protocol,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl InviteData {
    pub fn new(
        room_id: Uuid,
        room_name: String,
        creator_name: String,
        peer_addresses: Vec<SocketAddr>,
        protocol: crate::networking::Protocol,
    ) -> Self {
        Self {
            room_id,
            room_name,
            creator_name,
            peer_addresses,
            protocol,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn generate_invite_code(&self) -> Result<String> {
        let json_data = serde_json::to_string(self)?;
        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(json_data.as_bytes());
        Ok(format!("shortgap://{}", encoded))
    }

    pub fn parse_invite_code(invite_code: &str) -> Result<Self> {
        // Remove the protocol prefix if present
        let code = invite_code
            .strip_prefix("shortgap://")
            .unwrap_or(invite_code);

        let decoded_bytes = general_purpose::URL_SAFE_NO_PAD.decode(code)?;
        let json_data = String::from_utf8(decoded_bytes)?;
        let invite_data: InviteData = serde_json::from_str(&json_data)?;
        
        Ok(invite_data)
    }

    pub fn is_expired(&self, max_age_hours: u64) -> bool {
        let max_age = chrono::Duration::hours(max_age_hours as i64);
        chrono::Utc::now() - self.created_at > max_age
    }

    pub fn get_primary_peer(&self) -> Option<SocketAddr> {
        self.peer_addresses.first().copied()
    }

    pub fn get_fallback_peers(&self) -> Vec<SocketAddr> {
        if self.peer_addresses.len() > 1 {
            self.peer_addresses[1..].to_vec()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_invite_code_generation_and_parsing() {
        let room_id = Uuid::new_v4();
        let peer_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8080);
        
        let invite_data = InviteData::new(
            room_id,
            "Test Room".to_string(),
            "Alice".to_string(),
            vec![peer_addr],
            crate::networking::Protocol::TCP,
        );

        let invite_code = invite_data.generate_invite_code().unwrap();
        assert!(invite_code.starts_with("shortgap://"));

        let parsed_data = InviteData::parse_invite_code(&invite_code).unwrap();
        assert_eq!(parsed_data.room_id, room_id);
        assert_eq!(parsed_data.room_name, "Test Room");
        assert_eq!(parsed_data.creator_name, "Alice");
        assert_eq!(parsed_data.peer_addresses, vec![peer_addr]);
    }

    #[test]
    fn test_invite_expiration() {
        let invite_data = InviteData {
            room_id: Uuid::new_v4(),
            room_name: "Test".to_string(),
            creator_name: "Test".to_string(),
            peer_addresses: vec![],
            protocol: crate::networking::Protocol::TCP,
            created_at: chrono::Utc::now() - chrono::Duration::hours(25),
        };

        assert!(invite_data.is_expired(24));
        assert!(!invite_data.is_expired(48));
    }

    #[test]
    fn test_comprehensive_invite_functionality() {
        println!("🧪 Testing comprehensive invite code functionality");
        
        // Create test data with multiple peer addresses
        let room_id = Uuid::new_v4();
        let peer_addresses = vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8080),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101)), 8080),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)), 8080),
        ];
        
        let invite_data = InviteData::new(
            room_id,
            "Comprehensive Test Room".to_string(),
            "TestUser".to_string(),
            peer_addresses.clone(),
            crate::networking::Protocol::TCP,
        );

        // Test invite code generation
        let invite_code = invite_data.generate_invite_code().unwrap();
        println!("✅ Generated invite code: {}", invite_code);
        
        // Verify format
        assert!(invite_code.starts_with("shortgap://"), "Invite code should start with 'shortgap://'");
        
        // Extract and verify base64 part
        let base64_part = invite_code.strip_prefix("shortgap://").unwrap();
        println!("📦 Base64 part: {}", base64_part);
        
        // Test base64 decoding
        let decoded_bytes = general_purpose::URL_SAFE_NO_PAD.decode(base64_part).unwrap();
        let json_str = String::from_utf8(decoded_bytes).unwrap();
        println!("📄 Decoded JSON: {}", json_str);
        
        // Verify JSON contains expected fields
        assert!(json_str.contains(&room_id.to_string()), "JSON should contain room ID");
        assert!(json_str.contains("Comprehensive Test Room"), "JSON should contain room name");
        assert!(json_str.contains("TestUser"), "JSON should contain creator name");
        assert!(json_str.contains("192.168.1.100"), "JSON should contain peer addresses");
        
        // Test parsing
        let parsed_data = InviteData::parse_invite_code(&invite_code).unwrap();
        assert_eq!(parsed_data.room_id, room_id, "Room ID should match");
        assert_eq!(parsed_data.room_name, "Comprehensive Test Room", "Room name should match");
        assert_eq!(parsed_data.creator_name, "TestUser", "Creator name should match");
        assert_eq!(parsed_data.peer_addresses.len(), 3, "Should have 3 peer addresses");
        assert_eq!(parsed_data.peer_addresses, peer_addresses, "Peer addresses should match");
        
        // Test peer address methods
        let primary_peer = parsed_data.get_primary_peer();
        assert!(primary_peer.is_some(), "Should have a primary peer");
        assert_eq!(primary_peer.unwrap(), peer_addresses[0], "Primary peer should be first address");
        
        let fallback_peers = parsed_data.get_fallback_peers();
        assert_eq!(fallback_peers.len(), 2, "Should have 2 fallback peers");
        assert_eq!(fallback_peers, &peer_addresses[1..], "Fallback peers should match");
        
        // Test expiration
        assert!(!parsed_data.is_expired(24), "Fresh invite should not be expired");
        
        println!("✅ All comprehensive tests passed!");
    }

    #[test]
    fn test_base64_encoding_format() {
        let room_id = Uuid::new_v4();
        let invite_data = InviteData::new(
            room_id,
            "Base64 Test Room".to_string(),
            "Base64User".to_string(),
            vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000)],
            crate::networking::Protocol::WebRTC,
        );

        let invite_code = invite_data.generate_invite_code().unwrap();
        
        // Extract base64 part
        let base64_part = invite_code.strip_prefix("shortgap://").unwrap();
        
        // Verify it uses URL_SAFE_NO_PAD encoding (no padding characters)
        assert!(!base64_part.contains('='), "Should use URL_SAFE_NO_PAD encoding (no padding)");
        assert!(!base64_part.contains('+'), "Should use URL_SAFE encoding (no + characters)");
        assert!(!base64_part.contains('/'), "Should use URL_SAFE encoding (no / characters)");
        
        // Verify it decodes properly
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(base64_part).unwrap();
        let json_str = String::from_utf8(decoded).unwrap();
        
        // Verify JSON is valid
        let _parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        
        println!("✅ Base64 encoding format is correct");
    }
}