use serde::{Deserialize, Serialize};

/// Stats for the file server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub total_bytes_sent: u64,
    pub current_speed_bps: u64,
    pub active_connections: u32,
    pub unique_ips: u32,
    pub requests_per_minute: u32,
}

/// A single request log entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestLog {
    pub timestamp: u64,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub size_bytes: u64,
    pub user_agent: String,
    pub ip: String,
}

/// File stats for directory mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    pub name: String,
    pub requests: u64,
    pub bytes_sent: u64,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub file_path: String,
    pub is_directory: bool,
    pub port: u16,
    pub tunnel_url: Option<String>,
}
