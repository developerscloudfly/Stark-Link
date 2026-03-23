//! Device identification and metadata.
//!
//! Every Stark-Link node is represented by a [`DeviceInfo`] struct that carries
//! a stable UUID, human-readable name, operating system, form factor, and
//! optional battery level.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Physical form factor of a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Desktop,
    Laptop,
    Phone,
    Tablet,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "Desktop"),
            Self::Laptop => write!(f, "Laptop"),
            Self::Phone => write!(f, "Phone"),
            Self::Tablet => write!(f, "Tablet"),
        }
    }
}

/// Operating system family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OsType {
    Windows,
    MacOS,
    Linux,
    Android,
    #[serde(rename = "ios")]
    IOS,
}

impl std::fmt::Display for OsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Windows => write!(f, "Windows"),
            Self::MacOS => write!(f, "macOS"),
            Self::Linux => write!(f, "Linux"),
            Self::Android => write!(f, "Android"),
            Self::IOS => write!(f, "iOS"),
        }
    }
}

/// Metadata describing a single Stark-Link device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Persistent unique identifier (UUID v4).
    pub id: Uuid,
    /// Human-readable device name (usually the hostname).
    pub name: String,
    /// Operating system running on the device.
    pub os: OsType,
    /// Form factor.
    pub device_type: DeviceType,
    /// Battery percentage (0..=100), or `None` for desktops without batteries.
    pub battery_level: Option<u8>,
}

impl DeviceInfo {
    /// Create a new [`DeviceInfo`] with explicit values.
    pub fn new(
        id: Uuid,
        name: String,
        os: OsType,
        device_type: DeviceType,
        battery_level: Option<u8>,
    ) -> Self {
        Self {
            id,
            name,
            os,
            device_type,
            battery_level,
        }
    }

    /// Detect information about the device we are currently running on.
    ///
    /// The UUID is freshly generated each time; callers should persist it and
    /// reuse it across restarts via [`crate::config::Config`].
    pub fn local() -> Self {
        let name = hostname::get()
            .map(|h: std::ffi::OsString| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());

        let os = detect_os();
        let device_type = guess_device_type();

        Self {
            id: Uuid::new_v4(),
            name,
            os,
            device_type,
            battery_level: None, // Battery detection is platform-specific.
        }
    }
}

/// Detect the OS family at compile time.
fn detect_os() -> OsType {
    if cfg!(target_os = "windows") {
        OsType::Windows
    } else if cfg!(target_os = "macos") {
        OsType::MacOS
    } else if cfg!(target_os = "linux") {
        // Could be desktop Linux or Android; default to Linux.
        OsType::Linux
    } else if cfg!(target_os = "android") {
        OsType::Android
    } else if cfg!(target_os = "ios") {
        OsType::IOS
    } else {
        OsType::Linux // Fallback
    }
}

/// Best-effort guess of the form factor.
fn guess_device_type() -> DeviceType {
    if cfg!(target_os = "android") {
        DeviceType::Phone
    } else if cfg!(target_os = "ios") {
        DeviceType::Phone
    } else {
        // On desktop OSes we default to Desktop; the user can override.
        DeviceType::Desktop
    }
}
