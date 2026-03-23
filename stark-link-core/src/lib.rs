//! # Stark-Link Core
//!
//! The foundational library for the Stark-Link cross-device communication
//! platform.  This crate provides:
//!
//! - **Crypto** — X25519 key exchange, AES-256-GCM encryption, device
//!   fingerprints.
//! - **Protocol** — MessagePack-serialized messages with length-prefix framing.
//! - **Discovery** — mDNS service registration and browsing on the LAN.
//! - **Connection** — WebSocket server/client, heartbeats, reconnection.
//! - **Transfer** — Chunked file transfers with checksums, compression,
//!   pause/resume.
//! - **Clipboard** — Clipboard history and cross-device sync.
//! - **Config** — JSON-persisted configuration.
//! - **Device** — Device metadata (UUID, OS, form factor).
//! - **Error** — Unified error type.

pub mod clipboard;
pub mod config;
pub mod connection;
pub mod crypto;
pub mod device;
pub mod discovery;
pub mod error;
pub mod protocol;
pub mod transfer;

use std::path::Path;
use std::sync::Arc;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::connection::ConnectionManager;
use crate::crypto::KeyPair;
use crate::device::DeviceInfo;
use crate::discovery::DiscoveryManager;
use crate::error::Result;
use crate::transfer::TransferManager;

/// Top-level coordinator that initializes and wires together every subsystem.
pub struct StarkLink {
    /// Cryptographic identity of this device.
    pub keypair: KeyPair,
    /// Local device metadata.
    pub device: DeviceInfo,
    /// Runtime configuration.
    pub config: Config,
    /// mDNS discovery manager.
    pub discovery: DiscoveryManager,
    /// WebSocket connection manager.
    pub connection: Arc<ConnectionManager>,
    /// File transfer engine.
    pub transfer: TransferManager,
    /// Clipboard manager.
    pub clipboard: ClipboardManager,
}

impl StarkLink {
    /// Bootstrap a new Stark-Link instance.
    ///
    /// 1. Loads (or creates) configuration from `config_path`.
    /// 2. Generates a cryptographic key pair.
    /// 3. Initializes discovery, connection, transfer, and clipboard managers.
    ///
    /// Call [`start`](Self::start) afterwards to begin listening and browsing.
    pub fn new(config_path: Option<&Path>) -> Result<Self> {
        let config = match config_path {
            Some(p) => Config::load(p)?,
            None => Config::load(&Config::default_path())?,
        };

        let mut device = DeviceInfo::local();
        device.id = config.device_id;
        device.name = config.device_name.clone();

        let keypair = KeyPair::generate();

        tracing::info!(
            "StarkLink initialized — device {} ({}), fingerprint {}",
            device.name,
            device.id,
            keypair.fingerprint()
        );

        let discovery = DiscoveryManager::new(&device, config.port)?;
        let connection = Arc::new(ConnectionManager::new(device.clone(), config.clone()));

        // The transfer manager sends messages through a channel; the
        // application layer is responsible for draining it and forwarding
        // via the connection manager.
        let (transfer_tx, _transfer_rx) = mpsc::channel(256);
        let transfer = TransferManager::new(config.clone(), transfer_tx);

        let clipboard =
            ClipboardManager::new(device.id, config.max_clipboard_history);

        Ok(Self {
            keypair,
            device,
            config,
            discovery,
            connection,
            transfer,
            clipboard,
        })
    }

    /// Start the WebSocket server and mDNS browsing.
    ///
    /// Returns join handles for the server and discovery background tasks.
    pub async fn start(
        &self,
    ) -> Result<(
        tokio::task::JoinHandle<()>,
        tokio::task::JoinHandle<()>,
    )> {
        let server_handle = self.connection.start_server().await?;
        let browse_handle = self.discovery.start_browsing()?;

        tracing::info!("StarkLink is running on port {}", self.config.port);

        Ok((server_handle, browse_handle))
    }

    /// Gracefully shut down all subsystems.
    pub async fn shutdown(&self) -> Result<()> {
        self.connection.shutdown().await;
        self.discovery.shutdown()?;

        // Persist config.
        self.config.save(&Config::default_path())?;

        tracing::info!("StarkLink shut down");
        Ok(())
    }

    /// Return the device's public-key fingerprint.
    pub fn fingerprint(&self) -> String {
        self.keypair.fingerprint()
    }

    /// Return the local device's UUID.
    pub fn device_id(&self) -> Uuid {
        self.device.id
    }
}
