use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub tcp_streams: Arc<Mutex<HashMap<String, TcpStream>>>,
    pub server_handle: Option<tokio::task::JoinHandle<()>>,
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
            tcp_streams: Arc::new(Mutex::new(HashMap::new())),
            server_handle: None,
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

    async fn start_tcp_server(&mut self, port: u16) -> Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        let tcp_streams = Arc::clone(&self.tcp_streams);
        let message_sender = self.message_sender.as_ref().unwrap().clone();
        
        println!("🌐 TCP server started on port {}", port);
        
        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        println!("📞 NEW USER CONNECTION from {}", addr);
                        println!("🔗 Connection established with peer: {}", addr);
                        
                        let peer_id = addr.to_string();
                        {
                            let mut streams = tcp_streams.lock().await;
                            streams.insert(peer_id.clone(), stream);
                            println!("📝 Total active connections: {}", streams.len());
                        }
                        
                        // Handle incoming messages from this peer
                        let tcp_streams_clone = Arc::clone(&tcp_streams);
                        let sender_clone = message_sender.clone();
                        let peer_id_clone = peer_id.clone();
                        
                        tokio::spawn(async move {
                            Self::handle_tcp_peer(tcp_streams_clone, sender_clone, peer_id_clone).await;
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to accept TCP connection: {}", e);
                    }
                }
            }
        });
        
        self.server_handle = Some(handle);
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

    async fn connect_tcp(&mut self, addr: SocketAddr) -> Result<()> {
        println!("🔗 Connecting to TCP peer at {}", addr);
        
        let stream = TcpStream::connect(addr).await?;
        let peer_id = addr.to_string();
        
        // Store the stream
        {
            let mut streams = self.tcp_streams.lock().await;
            streams.insert(peer_id.clone(), stream);
        }
        
        // Update connection info
        let connection = PeerConnection {
            addr,
            protocol: Protocol::TCP,
            is_server: false,
            ping_ms: None,
            last_seen: chrono::Utc::now(),
        };
        self.connections.insert(peer_id.clone(), connection);
        
        // Start handling messages from this peer
        let tcp_streams = Arc::clone(&self.tcp_streams);
        let message_sender = self.message_sender.as_ref().unwrap().clone();
        
        tokio::spawn(async move {
            Self::handle_tcp_peer(tcp_streams, message_sender, peer_id).await;
        });
        
        println!("✅ Connected to TCP peer {}", addr);
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

    pub async fn broadcast_message(&self, message: NetworkMessage) -> Result<()> {
        let serialized = serde_json::to_string(&message)?;
        let message_bytes = serialized.as_bytes();
        let length = message_bytes.len() as u32;
        
        println!("📡 Broadcasting message type {:?} to {} peers", message.message_type, self.connections.len());
        
        // Log the message content if it's a chat message
        if matches!(message.message_type, MessageType::ChatMessage) {
            if let Ok(chat_message) = serde_json::from_value::<crate::room::ChatMessage>(message.payload.clone()) {
                println!("📤 Sending chat message: [{}] {}: {}", 
                    chat_message.timestamp.format("%H:%M:%S"), 
                    chat_message.user_name, 
                    chat_message.content);
            }
        }
        
        // Collect peer IDs first to avoid borrow conflicts
        let peer_ids: Vec<String> = {
            let streams = self.tcp_streams.lock().await;
            streams.keys().cloned().collect()
        };
        
        for peer_id in peer_ids {
            // Send message to each peer individually
            if let Err(e) = self.send_to_peer(&peer_id, message.clone()).await {
                eprintln!("❌ Failed to send message to peer {}: {}", peer_id, e);
            }
        }
        
        Ok(())
    }
    
    async fn handle_tcp_peer(
        tcp_streams: Arc<Mutex<HashMap<String, TcpStream>>>,
        message_sender: mpsc::UnboundedSender<NetworkMessage>,
        peer_id: String,
    ) {
        let mut stream = {
            let mut streams = tcp_streams.lock().await;
            if let Some(stream) = streams.remove(&peer_id) {
                stream
            } else {
                return;
            }
        };
        
        println!("🎧 Started listening for messages from peer {}", peer_id);
        
        loop {
            // Read message length
            match stream.read_u32().await {
                Ok(length) => {
                    // Read message data
                    let mut buffer = vec![0u8; length as usize];
                    match stream.read_exact(&mut buffer).await {
                        Ok(_) => {
                            match String::from_utf8(buffer) {
                                Ok(json_str) => {
                                    match serde_json::from_str::<NetworkMessage>(&json_str) {
                                        Ok(message) => {
                                            println!("📨 Received message from {}: {:?}", peer_id, message.message_type);
                                            if let Err(_) = message_sender.send(message) {
                                                eprintln!("❌ Failed to forward message from peer {}", peer_id);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("❌ Failed to parse message from peer {}: {}", peer_id, e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Invalid UTF-8 from peer {}: {}", peer_id, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to read message from peer {}: {}", peer_id, e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("📴 Peer {} disconnected: {}", peer_id, e);
                    break;
                }
            }
        }
        
        println!("👋 Stopped listening to peer {}", peer_id);
    }

    pub async fn send_to_peer(&self, peer_id: &str, message: NetworkMessage) -> Result<()> {
        let serialized = serde_json::to_string(&message)?;
        let message_bytes = serialized.as_bytes();
        let length = message_bytes.len() as u32;
        
        let mut streams = self.tcp_streams.lock().await;
        if let Some(stream) = streams.get_mut(peer_id) {
            stream.write_u32(length).await?;
            stream.write_all(message_bytes).await?;
            println!("✅ Sent message to peer {}", peer_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Peer {} not found", peer_id))
        }
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