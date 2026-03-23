//! Wire protocol for Stark-Link.
//!
//! Every message exchanged between peers is a variant of [`Message`].  Messages
//! are serialized with MessagePack for compactness and framed with a 4-byte
//! big-endian length prefix for reliable streaming over WebSocket.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::device::DeviceInfo;
use crate::error::{Result, StarkLinkError};

// ── Top-level message envelope ─────────────────────────────────────────────

/// Every message carries a unique `id`, the `sender` UUID, a `timestamp`, and
/// a typed `payload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier.
    pub id: Uuid,
    /// UUID of the sending device.
    pub sender: Uuid,
    /// When the message was created (UTC).
    pub timestamp: DateTime<Utc>,
    /// The actual message content.
    pub payload: Payload,
}

impl Message {
    /// Build a new message with the given sender and payload.  A fresh UUID and
    /// the current UTC timestamp are generated automatically.
    pub fn new(sender: Uuid, payload: Payload) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            timestamp: Utc::now(),
            payload,
        }
    }

    /// Serialize to MessagePack bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec_named(self).map_err(Into::into)
    }

    /// Deserialize from MessagePack bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data).map_err(Into::into)
    }

    /// Frame the message: 4-byte big-endian length prefix followed by the
    /// MessagePack payload.
    pub fn to_framed(&self) -> Result<Vec<u8>> {
        let body = self.to_bytes()?;
        let len = body.len() as u32;
        let mut frame = Vec::with_capacity(4 + body.len());
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(&body);
        Ok(frame)
    }

    /// Read a single framed message from a byte slice.
    ///
    /// Returns the message and the number of bytes consumed.
    pub fn from_framed(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < 4 {
            return Err(StarkLinkError::Protocol(
                "frame too short for length prefix".into(),
            ));
        }

        let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let total = 4 + len;

        if data.len() < total {
            return Err(StarkLinkError::Protocol(format!(
                "incomplete frame: need {total} bytes, have {}",
                data.len()
            )));
        }

        let msg = Self::from_bytes(&data[4..total])?;
        Ok((msg, total))
    }
}

// ── Payload variants ───────────────────────────────────────────────────────

/// Every distinct message type the protocol supports.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Payload {
    // ── Pairing ────────────────────────────────────────────────────────
    /// Initial handshake: the sender announces itself.
    Hello {
        device_info: DeviceInfo,
        public_key: Vec<u8>,
    },

    /// Request to pair with the receiving device.
    PairRequest { device_info: DeviceInfo },

    /// The peer accepted the pairing.
    PairAccept { device_info: DeviceInfo },

    /// The peer rejected the pairing.
    PairReject { reason: String },

    // ── Clipboard ──────────────────────────────────────────────────────
    /// Synchronize clipboard content.
    ClipboardSync {
        content_type: ClipboardContentType,
        data: Vec<u8>,
    },

    // ── File transfer ──────────────────────────────────────────────────
    /// Announce a new file transfer.
    FileTransferStart {
        transfer_id: Uuid,
        file_name: String,
        file_size: u64,
        total_chunks: u32,
        file_checksum: String,
    },

    /// A single chunk of file data.
    FileTransferChunk {
        transfer_id: Uuid,
        chunk_index: u32,
        data: Vec<u8>,
        checksum: String,
        compressed: bool,
    },

    /// The transfer completed successfully.
    FileTransferComplete { transfer_id: Uuid },

    /// Cancel an in-progress transfer.
    FileTransferCancel {
        transfer_id: Uuid,
        reason: String,
    },

    /// Pause an in-progress transfer.
    FileTransferPause { transfer_id: Uuid },

    /// Resume a paused transfer.
    FileTransferResume { transfer_id: Uuid },

    // ── Screen sharing / remote control ────────────────────────────────
    /// Begin a screen-sharing session.
    ScreenShareStart { session_id: Uuid },

    /// End a screen-sharing session.
    ScreenShareStop { session_id: Uuid },

    /// A mouse event from the controlling device.
    MouseEvent {
        session_id: Uuid,
        event: MouseEventData,
    },

    /// A keyboard event from the controlling device.
    KeyboardEvent {
        session_id: Uuid,
        event: KeyboardEventData,
    },

    /// Request remote-control permission.
    ControlRequest { session_id: Uuid },

    /// Revoke previously granted remote-control permission.
    ControlRevoke { session_id: Uuid },

    // ── Utilities ──────────────────────────────────────────────────────
    /// Control media playback on the remote device.
    MediaControl { action: MediaAction },

    /// Request or deliver system information.
    SystemInfo { info: SystemInfoData },

    /// Lock the remote device.
    RemoteLock,

    /// Launch an application on the remote device.
    AppLaunch { app_id: String, args: Vec<String> },

    /// Execute a shell command on the remote device.
    CommandExecute { command: String, timeout_secs: u32 },

    /// Response to a previously sent [`Payload::CommandExecute`].
    CommandResponse {
        exit_code: i32,
        stdout: String,
        stderr: String,
    },

    // ── Notifications ──────────────────────────────────────────────────
    /// Mirror a notification to the peer.
    NotificationSync {
        notification_id: String,
        app_name: String,
        title: String,
        body: String,
        icon: Option<Vec<u8>>,
    },

    /// Act on a mirrored notification (dismiss, reply, etc.).
    NotificationAction {
        notification_id: String,
        action: String,
        reply: Option<String>,
    },

    // ── Keep-alive & errors ────────────────────────────────────────────
    /// Heartbeat ping.
    Ping,

    /// Heartbeat pong.
    Pong,

    /// An error occurred on the peer.
    Error { code: u32, message: String },
}

// ── Sub-types ──────────────────────────────────────────────────────────────

/// Content types that can be carried on the clipboard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardContentType {
    Text,
    Image,
    Url,
    FilePath,
}

/// Data describing a mouse event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEventData {
    pub x: f64,
    pub y: f64,
    pub button: MouseButton,
    pub action: MouseAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseAction {
    Move,
    Down,
    Up,
    Click,
    DoubleClick,
    Scroll,
}

/// Data describing a keyboard event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEventData {
    pub key: String,
    pub action: KeyAction,
    pub modifiers: Vec<KeyModifier>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyAction {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

/// Media playback commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaAction {
    Play,
    Pause,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    Mute,
}

/// Snapshot of system information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfoData {
    pub cpu_usage: Option<f32>,
    pub memory_total: Option<u64>,
    pub memory_used: Option<u64>,
    pub disk_total: Option<u64>,
    pub disk_used: Option<u64>,
    pub battery_level: Option<u8>,
    pub uptime_secs: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_msgpack() {
        let msg = Message::new(Uuid::new_v4(), Payload::Ping);
        let bytes = msg.to_bytes().unwrap();
        let decoded = Message::from_bytes(&bytes).unwrap();
        assert_eq!(msg.id, decoded.id);
    }

    #[test]
    fn round_trip_framed() {
        let msg = Message::new(Uuid::new_v4(), Payload::Pong);
        let framed = msg.to_framed().unwrap();
        let (decoded, consumed) = Message::from_framed(&framed).unwrap();
        assert_eq!(consumed, framed.len());
        assert_eq!(msg.id, decoded.id);
    }
}
