use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use anyhow::Result;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolSwitchState {
    Idle,
    Preparing,
    Switching,
    Reconnecting,
    Complete,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSwitchEvent {
    pub room_id: Uuid,
    pub from_protocol: crate::networking::Protocol,
    pub to_protocol: crate::networking::Protocol,
    pub state: ProtocolSwitchState,
    pub affected_peers: Vec<SocketAddr>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct ProtocolManager {
    current_switches: std::collections::HashMap<Uuid, ProtocolSwitchEvent>,
    event_sender: mpsc::UnboundedSender<ProtocolSwitchEvent>,
    event_receiver: Option<mpsc::UnboundedReceiver<ProtocolSwitchEvent>>,
}

impl ProtocolManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            current_switches: std::collections::HashMap::new(),
            event_sender: tx,
            event_receiver: Some(rx),
        }
    }

    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<ProtocolSwitchEvent>> {
        self.event_receiver.take()
    }

    pub async fn initiate_protocol_switch(
        &mut self,
        room_id: Uuid,
        from_protocol: crate::networking::Protocol,
        to_protocol: crate::networking::Protocol,
        peers: Vec<SocketAddr>,
    ) -> Result<()> {
        // Check if a switch is already in progress for this room
        if let Some(existing_switch) = self.current_switches.get(&room_id) {
            if !matches!(existing_switch.state, ProtocolSwitchState::Complete | ProtocolSwitchState::Failed(_)) {
                return Err(anyhow::anyhow!("Protocol switch already in progress for room {}", room_id));
            }
        }

        let switch_event = ProtocolSwitchEvent {
            room_id,
            from_protocol: from_protocol.clone(),
            to_protocol: to_protocol.clone(),
            state: ProtocolSwitchState::Preparing,
            affected_peers: peers.clone(),
            timestamp: chrono::Utc::now(),
        };

        self.current_switches.insert(room_id, switch_event.clone());
        let _ = self.event_sender.send(switch_event);

        // Execute the protocol switch
        self.execute_protocol_switch(room_id, from_protocol, to_protocol, peers).await
    }

    async fn execute_protocol_switch(
        &mut self,
        room_id: Uuid,
        _from_protocol: crate::networking::Protocol,
        to_protocol: crate::networking::Protocol,
        peers: Vec<SocketAddr>,
    ) -> Result<()> {
        // Phase 1: Prepare all peers for the switch
        self.update_switch_state(room_id, ProtocolSwitchState::Preparing).await?;
        
        if let Err(e) = self.prepare_peers_for_switch(&peers, &to_protocol).await {
            self.update_switch_state(room_id, ProtocolSwitchState::Failed(e.to_string())).await?;
            return Err(e);
        }

        // Phase 2: Coordinate the actual switch
        self.update_switch_state(room_id, ProtocolSwitchState::Switching).await?;
        
        // Wait a moment for all peers to be ready
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Phase 3: Reconnect with new protocol
        self.update_switch_state(room_id, ProtocolSwitchState::Reconnecting).await?;
        
        if let Err(e) = self.reconnect_peers(&peers, &to_protocol).await {
            self.update_switch_state(room_id, ProtocolSwitchState::Failed(e.to_string())).await?;
            return Err(e);
        }

        // Phase 4: Complete
        self.update_switch_state(room_id, ProtocolSwitchState::Complete).await?;
        
        Ok(())
    }

    async fn prepare_peers_for_switch(
        &self,
        peers: &[SocketAddr],
        new_protocol: &crate::networking::Protocol,
    ) -> Result<()> {
        // Send preparation messages to all peers
        for peer in peers {
            if let Err(e) = self.send_preparation_message(*peer, new_protocol).await {
                return Err(anyhow::anyhow!("Failed to prepare peer {}: {}", peer, e));
            }
        }

        // Wait for acknowledgments from all peers
        self.wait_for_peer_acknowledgments(peers).await
    }

    async fn send_preparation_message(
        &self,
        peer: SocketAddr,
        new_protocol: &crate::networking::Protocol,
    ) -> Result<()> {
        // Implementation would depend on the current networking layer
        // This is a placeholder for sending the preparation message
        
        let _message = crate::networking::NetworkMessage {
            id: Uuid::new_v4(),
            from: "self".to_string(),
            to: Some(peer.to_string()),
            message_type: crate::networking::MessageType::ProtocolChange,
            payload: serde_json::to_value(new_protocol)?,
            timestamp: chrono::Utc::now(),
        };

        // Send the message through the current networking layer
        // This would be implemented based on the active protocol
        
        Ok(())
    }

    async fn wait_for_peer_acknowledgments(&self, _peers: &[SocketAddr]) -> Result<()> {
        // Wait for acknowledgments from all peers with timeout
        let timeout_duration = tokio::time::Duration::from_secs(10);
        let start_time = tokio::time::Instant::now();

        while start_time.elapsed() < timeout_duration {
            // Check if all peers have acknowledged
            // This would be implemented based on the networking layer
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn reconnect_peers(
        &self,
        peers: &[SocketAddr],
        new_protocol: &crate::networking::Protocol,
    ) -> Result<()> {
        // Attempt to reconnect to all peers using the new protocol
        for peer in peers {
            if let Err(e) = self.reconnect_to_peer(*peer, new_protocol).await {
                eprintln!("Failed to reconnect to peer {} with new protocol: {}", peer, e);
                // Continue with other peers rather than failing completely
            }
        }

        Ok(())
    }

    async fn reconnect_to_peer(
        &self,
        _peer: SocketAddr,
        protocol: &crate::networking::Protocol,
    ) -> Result<()> {
        // Implementation would use the networking manager to establish
        // a new connection with the specified protocol
        
        match protocol {
            crate::networking::Protocol::TCP => {
                // Reconnect using TCP
            }
            crate::networking::Protocol::WebSocket => {
                // Reconnect using WebSocket
            }
            crate::networking::Protocol::WebRTC => {
                // Reconnect using WebRTC
            }
        }

        Ok(())
    }

    async fn update_switch_state(&mut self, room_id: Uuid, new_state: ProtocolSwitchState) -> Result<()> {
        if let Some(switch_event) = self.current_switches.get_mut(&room_id) {
            switch_event.state = new_state;
            switch_event.timestamp = chrono::Utc::now();
            
            let _ = self.event_sender.send(switch_event.clone());
        }

        Ok(())
    }

    pub fn get_switch_status(&self, room_id: &Uuid) -> Option<&ProtocolSwitchEvent> {
        self.current_switches.get(room_id)
    }

    pub fn cleanup_completed_switches(&mut self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        
        self.current_switches.retain(|_, switch_event| {
            match switch_event.state {
                ProtocolSwitchState::Complete | ProtocolSwitchState::Failed(_) => {
                    switch_event.timestamp > cutoff
                }
                _ => true, // Keep active switches
            }
        });
    }

    pub fn cancel_switch(&mut self, room_id: Uuid) -> Result<()> {
        if let Some(switch_event) = self.current_switches.get_mut(&room_id) {
            switch_event.state = ProtocolSwitchState::Failed("Cancelled by user".to_string());
            switch_event.timestamp = chrono::Utc::now();
            
            let _ = self.event_sender.send(switch_event.clone());
        }

        Ok(())
    }

    pub fn get_active_switches(&self) -> Vec<&ProtocolSwitchEvent> {
        self.current_switches
            .values()
            .filter(|switch| !matches!(switch.state, ProtocolSwitchState::Complete | ProtocolSwitchState::Failed(_)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_protocol_switch_initiation() {
        let mut manager = ProtocolManager::new();
        let room_id = Uuid::new_v4();
        let peer = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        
        let result = manager.initiate_protocol_switch(
            room_id,
            crate::networking::Protocol::TCP,
            crate::networking::Protocol::WebRTC,
            vec![peer],
        ).await;

        assert!(result.is_ok());
        
        let status = manager.get_switch_status(&room_id);
        assert!(status.is_some());
    }

    #[test]
    fn test_switch_cleanup() {
        let mut manager = ProtocolManager::new();
        let room_id = Uuid::new_v4();
        
        let old_switch = ProtocolSwitchEvent {
            room_id,
            from_protocol: crate::networking::Protocol::TCP,
            to_protocol: crate::networking::Protocol::WebRTC,
            state: ProtocolSwitchState::Complete,
            affected_peers: vec![],
            timestamp: chrono::Utc::now() - chrono::Duration::hours(2),
        };

        manager.current_switches.insert(room_id, old_switch);
        
        manager.cleanup_completed_switches();
        
        assert!(manager.current_switches.is_empty());
    }
}