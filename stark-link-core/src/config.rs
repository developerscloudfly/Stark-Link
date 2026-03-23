//! Application configuration.
//!
//! [`Config`] holds every tunable parameter for a Stark-Link node.  It can be
//! serialized to / deserialized from a JSON file on disk.

use crate::error::{Result, StarkLinkError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// All configurable parameters for the Stark-Link runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Persistent device UUID (generated once, then reused).
    pub device_id: Uuid,

    /// Human-readable device name shown to peers.
    pub device_name: String,

    /// TCP port used by the WebSocket server.
    pub port: u16,

    /// Size of each file-transfer chunk in bytes (default 5 MiB).
    pub chunk_size: usize,

    /// Maximum individual file size in bytes (default 10 GiB).
    pub max_file_size: u64,

    /// Heartbeat interval in seconds (default 30).
    pub heartbeat_interval_secs: u64,

    /// Maximum clipboard history entries to keep.
    pub max_clipboard_history: usize,

    /// Maximum concurrent file transfers.
    pub max_concurrent_transfers: usize,

    /// Whether to compress chunks with LZ4 before sending.
    pub compression_enabled: bool,

    /// Directory where received files are saved.
    pub download_dir: PathBuf,

    /// Whether to automatically accept pair requests from known devices.
    pub auto_accept_known: bool,
}

impl Default for Config {
    fn default() -> Self {
        let download_dir = dirs_next_download_dir();
        Self {
            device_id: Uuid::new_v4(),
            device_name: hostname::get()
                .map(|h: std::ffi::OsString| h.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "StarkLink Device".into()),
            port: 42424,
            chunk_size: 5 * 1024 * 1024, // 5 MiB
            max_file_size: 10 * 1024 * 1024 * 1024, // 10 GiB
            heartbeat_interval_secs: 30,
            max_clipboard_history: 50,
            max_concurrent_transfers: 5,
            compression_enabled: true,
            download_dir,
            auto_accept_known: false,
        }
    }
}

impl Config {
    /// Load configuration from a JSON file.  If the file does not exist a
    /// default configuration is returned (but *not* written to disk).
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::info!("config file not found at {}, using defaults", path.display());
            return Ok(Self::default());
        }

        let data = std::fs::read_to_string(path).map_err(|e| {
            StarkLinkError::Config(format!("failed to read {}: {e}", path.display()))
        })?;

        let config: Self = serde_json::from_str(&data).map_err(|e| {
            StarkLinkError::Config(format!("failed to parse {}: {e}", path.display()))
        })?;

        tracing::info!("loaded config from {}", path.display());
        Ok(config)
    }

    /// Persist the current configuration to a JSON file, creating parent
    /// directories if necessary.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StarkLinkError::Config(format!(
                    "failed to create config directory {}: {e}",
                    parent.display()
                ))
            })?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json).map_err(|e| {
            StarkLinkError::Config(format!("failed to write {}: {e}", path.display()))
        })?;

        tracing::info!("saved config to {}", path.display());
        Ok(())
    }

    /// Return the default config-file path for this platform.
    ///
    /// - **Windows:** `%APPDATA%\StarkLink\config.json`
    /// - **macOS:** `~/Library/Application Support/StarkLink/config.json`
    /// - **Linux:** `~/.config/starklink/config.json`
    pub fn default_path() -> PathBuf {
        let base = if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."))
        } else if cfg!(target_os = "macos") {
            dirs_home()
                .map(|h| h.join("Library/Application Support"))
                .unwrap_or_else(|| PathBuf::from("."))
        } else {
            std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    dirs_home()
                        .map(|h| h.join(".config"))
                        .unwrap_or_else(|| PathBuf::from("."))
                })
        };

        base.join("StarkLink").join("config.json")
    }
}

// ── helpers ────────────────────────────────────────────────────────────────

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .ok()
}

fn dirs_next_download_dir() -> PathBuf {
    dirs_home()
        .map(|h| h.join("Downloads").join("StarkLink"))
        .unwrap_or_else(|| PathBuf::from("StarkLink-Downloads"))
}
