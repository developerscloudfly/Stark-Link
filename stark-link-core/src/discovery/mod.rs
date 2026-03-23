//! mDNS-based device discovery.
//!
//! Registers this device as `_starklink._tcp.local.` and continuously browses
//! for other Stark-Link devices on the local network.  When a peer appears or
//! disappears an [`DiscoveryEvent`] is emitted through a broadcast channel.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::device::DeviceInfo;
use crate::error::{Result, StarkLinkError};

/// mDNS service type registered by every Stark-Link node.
const SERVICE_TYPE: &str = "_starklink._tcp.local.";

/// A discovered peer on the local network.
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    /// Device metadata advertised via mDNS TXT records.
    pub info: DeviceInfo,
    /// Addresses where the peer can be reached.
    pub addresses: Vec<std::net::IpAddr>,
    /// WebSocket port.
    pub port: u16,
    /// Whether the device is currently reachable.
    pub online: bool,
}

/// Events emitted by the discovery subsystem.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// A new device appeared on the network.
    DeviceFound(DiscoveredDevice),
    /// A previously discovered device went offline.
    DeviceLost(Uuid),
    /// An already-known device updated its metadata.
    DeviceUpdated(DiscoveredDevice),
}

/// Manages mDNS registration and browsing.
pub struct DiscoveryManager {
    /// The mDNS daemon handle.
    daemon: ServiceDaemon,
    /// Currently known devices keyed by UUID.
    devices: Arc<RwLock<HashMap<Uuid, DiscoveredDevice>>>,
    /// Broadcast channel for discovery events.
    event_tx: broadcast::Sender<DiscoveryEvent>,
    /// The instance name we registered.
    instance_name: String,
}

impl DiscoveryManager {
    /// Create a new discovery manager and register the local device.
    pub fn new(local: &DeviceInfo, port: u16) -> Result<Self> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| StarkLinkError::Discovery(format!("failed to create mDNS daemon: {e}")))?;

        let (event_tx, _) = broadcast::channel(64);

        let instance_name = format!("starklink-{}", local.id);

        let manager = Self {
            daemon,
            devices: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            instance_name,
        };

        manager.register(local, port)?;

        Ok(manager)
    }

    /// Register this device on the local network via mDNS.
    fn register(&self, local: &DeviceInfo, port: u16) -> Result<()> {
        let properties = [
            ("id", local.id.to_string()),
            ("name", local.name.clone()),
            ("os", format!("{}", local.os)),
            ("device_type", format!("{}", local.device_type)),
        ];

        let txt_properties: Vec<(&str, &str)> = properties
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        let hostname = format!("{}.local.", self.instance_name);

        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &self.instance_name,
            &hostname,
            "",
            port,
            &txt_properties[..],
        )
        .map_err(|e| StarkLinkError::Discovery(format!("failed to create service info: {e}")))?;

        self.daemon
            .register(service_info)
            .map_err(|e| StarkLinkError::Discovery(format!("failed to register service: {e}")))?;

        tracing::info!(
            "registered mDNS service: {} on port {}",
            self.instance_name,
            port
        );

        Ok(())
    }

    /// Subscribe to discovery events.
    pub fn subscribe(&self) -> broadcast::Receiver<DiscoveryEvent> {
        self.event_tx.subscribe()
    }

    /// Return a snapshot of all currently known (online) devices.
    pub async fn devices(&self) -> Vec<DiscoveredDevice> {
        let map = self.devices.read().await;
        map.values().filter(|d| d.online).cloned().collect()
    }

    /// Look up a specific device by UUID.
    pub async fn get_device(&self, id: &Uuid) -> Option<DiscoveredDevice> {
        let map = self.devices.read().await;
        map.get(id).cloned()
    }

    /// Start the background browse loop.  This spawns a tokio task that
    /// processes mDNS events and updates the internal device list.
    ///
    /// The returned [`tokio::task::JoinHandle`] can be used to wait for (or
    /// abort) the background task.
    pub fn start_browsing(&self) -> Result<tokio::task::JoinHandle<()>> {
        let browse_receiver = self
            .daemon
            .browse(SERVICE_TYPE)
            .map_err(|e| StarkLinkError::Discovery(format!("failed to browse: {e}")))?;

        let devices = Arc::clone(&self.devices);
        let event_tx = self.event_tx.clone();
        let own_instance = self.instance_name.clone();

        let handle = tokio::spawn(async move {
            loop {
                // mdns-sd's receiver is a std channel; poll it on a background
                // thread via `tokio::task::spawn_blocking` batched reads.
                let recv = browse_receiver.clone();
                let event = tokio::task::spawn_blocking(move || {
                    recv.recv_timeout(Duration::from_secs(2))
                })
                .await;

                let event = match event {
                    Ok(Ok(e)) => e,
                    Ok(Err(_)) => continue, // timeout
                    Err(_) => break,         // task cancelled
                };

                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        // Skip our own service.
                        if info.get_fullname().contains(&own_instance) {
                            continue;
                        }

                        let device = match parse_service_info(&info) {
                            Some(d) => d,
                            None => {
                                tracing::warn!(
                                    "could not parse device info from mDNS: {}",
                                    info.get_fullname()
                                );
                                continue;
                            }
                        };

                        let id = device.info.id;
                        let mut map = devices.write().await;
                        let is_new = !map.contains_key(&id);
                        map.insert(id, device.clone());

                        let evt = if is_new {
                            tracing::info!("discovered device: {} ({})", device.info.name, id);
                            DiscoveryEvent::DeviceFound(device)
                        } else {
                            tracing::debug!("updated device: {} ({})", device.info.name, id);
                            DiscoveryEvent::DeviceUpdated(device)
                        };
                        let _ = event_tx.send(evt);
                    }

                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        let mut map = devices.write().await;
                        let removed_id = map
                            .iter()
                            .find(|(_, d)| {
                                fullname.contains(&d.info.id.to_string())
                            })
                            .map(|(id, _)| *id);

                        if let Some(id) = removed_id {
                            if let Some(dev) = map.get_mut(&id) {
                                dev.online = false;
                            }
                            tracing::info!("device went offline: {id}");
                            let _ = event_tx.send(DiscoveryEvent::DeviceLost(id));
                        }
                    }

                    _ => {} // SearchStarted, SearchStopped, etc.
                }
            }
        });

        tracing::info!("started mDNS browsing for {SERVICE_TYPE}");
        Ok(handle)
    }

    /// Unregister the local mDNS service and stop the daemon.
    pub fn shutdown(&self) -> Result<()> {
        let fullname = format!("{}.{}", self.instance_name, SERVICE_TYPE);
        self.daemon
            .unregister(&fullname)
            .map_err(|e| StarkLinkError::Discovery(format!("failed to unregister: {e}")))?;

        self.daemon
            .shutdown()
            .map_err(|e| StarkLinkError::Discovery(format!("failed to shutdown daemon: {e}")))?;

        tracing::info!("mDNS discovery shut down");
        Ok(())
    }
}

// ── helpers ────────────────────────────────────────────────────────────────

/// Parse mDNS TXT records into a [`DiscoveredDevice`].
fn parse_service_info(info: &ServiceInfo) -> Option<DiscoveredDevice> {
    let properties = info.get_properties();

    let id_str = properties.get_property_val_str("id")?;
    let id = Uuid::parse_str(id_str).ok()?;
    let name = properties
        .get_property_val_str("name")
        .unwrap_or("Unknown")
        .to_string();
    let os_str = properties.get_property_val_str("os").unwrap_or("Linux");
    let dt_str = properties
        .get_property_val_str("device_type")
        .unwrap_or("Desktop");

    let os = match os_str {
        "Windows" => crate::device::OsType::Windows,
        "macOS" => crate::device::OsType::MacOS,
        "Linux" => crate::device::OsType::Linux,
        "Android" => crate::device::OsType::Android,
        "iOS" => crate::device::OsType::IOS,
        _ => crate::device::OsType::Linux,
    };

    let device_type = match dt_str {
        "Desktop" => crate::device::DeviceType::Desktop,
        "Laptop" => crate::device::DeviceType::Laptop,
        "Phone" => crate::device::DeviceType::Phone,
        "Tablet" => crate::device::DeviceType::Tablet,
        _ => crate::device::DeviceType::Desktop,
    };

    let addresses: Vec<std::net::IpAddr> = info.get_addresses().iter().copied().collect();
    let port = info.get_port();

    Some(DiscoveredDevice {
        info: DeviceInfo::new(id, name, os, device_type, None),
        addresses,
        port,
        online: true,
    })
}
