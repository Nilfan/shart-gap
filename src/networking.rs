use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    TCP,
    WebSocket,
    WebRTC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub id: Uuid,
    pub from: String,
    pub to: Option<String>,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    ChatMessage,
    UserJoined,
    UserLeft,
    PingMeasurement,
    ServerTransfer,
    ProtocolChange,
    VoiceData,
    RoomSync,
}

#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub addr: SocketAddr,
    pub protocol: Protocol,
    pub is_server: bool,
    pub ping_ms: Option<u64>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

pub struct NetworkManager {
    pub connections: HashMap<String, PeerConnection>,
    pub message_sender: Option<mpsc::UnboundedSender<NetworkMessage>>,
    pub message_receiver: Option<mpsc::UnboundedReceiver<NetworkMessage>>,
    pub current_protocol: Protocol,
    pub is_server: bool,
    pub server_peer: Option<String>,
}

impl NetworkManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            connections: HashMap::new(),
            message_sender: Some(tx),
            message_receiver: Some(rx),
            current_protocol: Protocol::TCP,
            is_server: false,
            server_peer: None,
        }
    }

    pub async fn start_server(&mut self, port: u16, protocol: Protocol) -> Result<()> {
        self.is_server = true;
        self.current_protocol = protocol.clone();
        
        match protocol {
            Protocol::TCP => self.start_tcp_server(port).await,
            Protocol::WebSocket => self.start_websocket_server(port).await,
            Protocol::WebRTC => self.start_webrtc_server().await,
        }
    }

    pub async fn connect_to_peer(&mut self, addr: SocketAddr, protocol: Protocol) -> Result<()> {
        match protocol {
            Protocol::TCP => self.connect_tcp(addr).await,
            Protocol::WebSocket => self.connect_websocket(addr).await,
            Protocol::WebRTC => self.connect_webrtc(addr).await,
        }
    }

    pub async fn switch_protocol(&mut self, new_protocol: Protocol, peers: Vec<SocketAddr>) -> Result<()> {
        // Notify all peers about protocol change
        let switch_message = NetworkMessage {
            id: Uuid::new_v4(),
            from: "self".to_string(),
            to: None,
            message_type: MessageType::ProtocolChange,
            payload: serde_json::to_value(&new_protocol)?,
            timestamp: chrono::Utc::now(),
        };

        self.broadcast_message(switch_message).await?;

        // Wait a moment for peers to prepare
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Switch to new protocol
        self.current_protocol = new_protocol.clone();
        
        // Reconnect to all peers with new protocol
        for addr in peers {
            if let Err(e) = self.connect_to_peer(addr, new_protocol.clone()).await {
                eprintln!("Failed to reconnect to peer {}: {}", addr, e);
            }
        }

        Ok(())
    }

    async fn start_tcp_server(&mut self, _port: u16) -> Result<()> {
        // TCP server implementation
        Ok(())
    }

    async fn start_websocket_server(&mut self, _port: u16) -> Result<()> {
        // WebSocket server implementation
        Ok(())
    }

    async fn start_webrtc_server(&mut self) -> Result<()> {
        // WebRTC server implementation
        Ok(())
    }

    async fn connect_tcp(&mut self, _addr: SocketAddr) -> Result<()> {
        // TCP client implementation
        Ok(())
    }

    async fn connect_websocket(&mut self, _addr: SocketAddr) -> Result<()> {
        // WebSocket client implementation
        Ok(())
    }

    async fn connect_webrtc(&mut self, _addr: SocketAddr) -> Result<()> {
        // WebRTC client implementation
        Ok(())
    }

    pub async fn broadcast_message(&self, _message: NetworkMessage) -> Result<()> {
        // Broadcast to all connected peers
        Ok(())
    }

    pub async fn send_to_peer(&self, _peer_id: &str, _message: NetworkMessage) -> Result<()> {
        // Send to specific peer
        Ok(())
    }

    pub fn get_peer_list(&self) -> Vec<(String, PeerConnection)> {
        self.connections.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn update_ping(&mut self, peer_id: &str, ping_ms: u64) {
        if let Some(peer) = self.connections.get_mut(peer_id) {
            peer.ping_ms = Some(ping_ms);
            peer.last_seen = chrono::Utc::now();
        }
    }

    pub fn mark_peer_offline(&mut self, peer_id: &str) {
        if let Some(peer) = self.connections.get_mut(peer_id) {
            peer.ping_ms = None;
            println!("📴 Network: Marked peer {} as offline", peer_id);
        }
    }

    pub fn is_peer_healthy(&self, peer_id: &str, timeout_minutes: i64) -> bool {
        if let Some(peer) = self.connections.get(peer_id) {
            let now = chrono::Utc::now();
            let time_diff = now.signed_duration_since(peer.last_seen);
            time_diff.num_minutes() < timeout_minutes && peer.ping_ms.is_some()
        } else {
            false
        }
    }

    pub fn cleanup_stale_connections(&mut self, timeout_minutes: i64) {
        let now = chrono::Utc::now();
        let stale_peers: Vec<String> = self.connections
            .iter()
            .filter(|(_, peer)| {
                let time_diff = now.signed_duration_since(peer.last_seen);
                time_diff.num_minutes() > timeout_minutes
            })
            .map(|(id, _)| id.clone())
            .collect();

        for peer_id in stale_peers {
            self.connections.remove(&peer_id);
            println!("🧹 Removed stale connection: {}", peer_id);
        }
    }

    pub fn get_best_server_candidate(&self) -> Option<String> {
        self.connections
            .iter()
            .filter(|(_, peer)| peer.ping_ms.is_some())
            .min_by_key(|(_, peer)| peer.ping_ms.unwrap_or(u64::MAX))
            .map(|(id, _)| id.clone())
    }

    pub async fn disconnect_all(&mut self) -> Result<()> {
        // Clear all connections
        self.connections.clear();
        
        // Reset state
        self.is_server = false;
        self.server_peer = None;
        
        // TODO: Implement actual connection teardown for each protocol
        println!("🔌 Disconnected from all peers");
        
        Ok(())
    }
}