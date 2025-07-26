use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingMeasurement {
    pub peer_addr: SocketAddr,
    pub tcp_ping: Option<u64>,
    pub app_ping: Option<u64>,
    pub webrtc_rtt: Option<u64>,
    pub average_ping: Option<u64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PingMeasurement {
    pub fn new(peer_addr: SocketAddr) -> Self {
        Self {
            peer_addr,
            tcp_ping: None,
            app_ping: None,
            webrtc_rtt: None,
            average_ping: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn calculate_average(&mut self) {
        let mut values = Vec::new();
        
        if let Some(tcp) = self.tcp_ping {
            values.push(tcp);
        }
        if let Some(app) = self.app_ping {
            values.push(app);
        }
        if let Some(webrtc) = self.webrtc_rtt {
            values.push(webrtc);
        }

        if !values.is_empty() {
            self.average_ping = Some(values.iter().sum::<u64>() / values.len() as u64);
        }
    }

    pub fn is_complete(&self) -> bool {
        self.tcp_ping.is_some() && self.app_ping.is_some() && self.webrtc_rtt.is_some()
    }
}

pub struct PingManager {
    measurements: std::collections::HashMap<SocketAddr, PingMeasurement>,
}

impl PingManager {
    pub fn new() -> Self {
        Self {
            measurements: std::collections::HashMap::new(),
        }
    }

    pub async fn measure_tcp_ping(&mut self, addr: SocketAddr) -> Result<u64> {
        let start = Instant::now();
        
        match timeout(Duration::from_secs(5), TcpStream::connect(addr)).await {
            Ok(Ok(_)) => {
                let ping_ms = start.elapsed().as_millis() as u64;
                
                let measurement = self.measurements.entry(addr)
                    .or_insert_with(|| PingMeasurement::new(addr));
                measurement.tcp_ping = Some(ping_ms);
                measurement.timestamp = chrono::Utc::now();
                measurement.calculate_average();
                
                Ok(ping_ms)
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("TCP connection failed: {}", e)),
            Err(_) => Err(anyhow::anyhow!("TCP connection timed out")),
        }
    }

    pub async fn measure_application_ping(&mut self, addr: SocketAddr) -> Result<u64> {
        // Send a custom ping message and measure round-trip time
        let start = Instant::now();
        
        // This would be implemented with the actual networking layer
        // For now, we'll simulate with a basic TCP connection
        match timeout(Duration::from_secs(3), self.send_ping_message(addr)).await {
            Ok(Ok(_)) => {
                let ping_ms = start.elapsed().as_millis() as u64;
                
                let measurement = self.measurements.entry(addr)
                    .or_insert_with(|| PingMeasurement::new(addr));
                measurement.app_ping = Some(ping_ms);
                measurement.timestamp = chrono::Utc::now();
                measurement.calculate_average();
                
                Ok(ping_ms)
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Application ping failed: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Application ping timed out")),
        }
    }

    async fn send_ping_message(&self, addr: SocketAddr) -> Result<()> {
        // Implementation would depend on the networking layer
        // This is a placeholder that simulates sending a ping message
        let _stream = TcpStream::connect(addr).await?;
        Ok(())
    }

    pub fn update_webrtc_rtt(&mut self, addr: SocketAddr, rtt_ms: u64) {
        let measurement = self.measurements.entry(addr)
            .or_insert_with(|| PingMeasurement::new(addr));
        measurement.webrtc_rtt = Some(rtt_ms);
        measurement.timestamp = chrono::Utc::now();
        measurement.calculate_average();
    }

    pub async fn measure_all_pings(&mut self, addrs: Vec<SocketAddr>) -> Result<()> {
        let mut handles = Vec::new();
        
        for addr in addrs {
            let tcp_handle = {
                let addr = addr.clone();
                tokio::spawn(async move {
                    match timeout(Duration::from_secs(5), TcpStream::connect(addr)).await {
                        Ok(Ok(_)) => Some((addr, "tcp", 0)), // Placeholder
                        _ => None,
                    }
                })
            };
            handles.push(tcp_handle);
        }

        // Wait for all measurements to complete
        for handle in handles {
            if let Ok(Some((addr, ping_type, ping_ms))) = handle.await {
                let measurement = self.measurements.entry(addr)
                    .or_insert_with(|| PingMeasurement::new(addr));
                
                match ping_type {
                    "tcp" => measurement.tcp_ping = Some(ping_ms),
                    "app" => measurement.app_ping = Some(ping_ms),
                    _ => {}
                }
                
                measurement.calculate_average();
            }
        }

        Ok(())
    }

    pub fn get_measurement(&self, addr: &SocketAddr) -> Option<&PingMeasurement> {
        self.measurements.get(addr)
    }

    pub fn get_all_measurements(&self) -> Vec<&PingMeasurement> {
        self.measurements.values().collect()
    }

    pub fn get_sorted_by_ping(&self) -> Vec<&PingMeasurement> {
        let mut measurements: Vec<&PingMeasurement> = self.measurements.values().collect();
        measurements.sort_by_key(|m| m.average_ping.unwrap_or(u64::MAX));
        measurements
    }

    pub fn get_best_peer(&self) -> Option<SocketAddr> {
        self.get_sorted_by_ping()
            .first()
            .map(|m| m.peer_addr)
    }

    pub fn cleanup_old_measurements(&mut self, max_age_minutes: u64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(max_age_minutes as i64);
        self.measurements.retain(|_, measurement| measurement.timestamp > cutoff);
    }

    pub fn export_measurements(&self) -> Result<String> {
        let measurements: Vec<&PingMeasurement> = self.measurements.values().collect();
        Ok(serde_json::to_string_pretty(&measurements)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_ping_measurement_average() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut measurement = PingMeasurement::new(addr);
        
        measurement.tcp_ping = Some(10);
        measurement.app_ping = Some(20);
        measurement.webrtc_rtt = Some(30);
        measurement.calculate_average();
        
        assert_eq!(measurement.average_ping, Some(20));
    }

    #[test]
    fn test_ping_manager_sorting() {
        let mut manager = PingManager::new();
        
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082);
        
        let mut measurement1 = PingMeasurement::new(addr1);
        measurement1.average_ping = Some(100);
        
        let mut measurement2 = PingMeasurement::new(addr2);
        measurement2.average_ping = Some(50);
        
        manager.measurements.insert(addr1, measurement1);
        manager.measurements.insert(addr2, measurement2);
        
        let best_peer = manager.get_best_peer();
        assert_eq!(best_peer, Some(addr2));
    }
}