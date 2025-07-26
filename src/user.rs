use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub address: SocketAddr,
    pub audio_input_device: Option<String>,
    pub audio_output_device: Option<String>,
    pub is_online: bool,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub is_in_call: bool,
}

impl User {
    pub fn new(name: String, address: SocketAddr) -> Self {
        let user_id = Uuid::new_v4();
        println!("👤 Creating new user '{}' with unique ID: {}", name, user_id);
        
        Self {
            id: user_id,
            name,
            avatar: None,
            address,
            audio_input_device: None,
            audio_output_device: None,
            is_online: true,
            last_seen: chrono::Utc::now(),
            is_in_call: false,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now();
    }

    pub fn set_audio_devices(&mut self, input: Option<String>, output: Option<String>) {
        self.audio_input_device = input;
        self.audio_output_device = output;
    }

    pub fn set_avatar(&mut self, avatar: Option<String>) {
        self.avatar = avatar;
    }

    pub fn join_call(&mut self) {
        self.is_in_call = true;
        self.update_last_seen();
    }

    pub fn leave_call(&mut self) {
        self.is_in_call = false;
        self.update_last_seen();
    }
}