//! Clipboard synchronization.
//!
//! Maintains a local clipboard history and provides methods for syncing
//! clipboard content across paired devices.

use std::collections::VecDeque;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::error::{Result, StarkLinkError};
use crate::protocol::ClipboardContentType;

/// A single clipboard entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    /// Unique entry identifier.
    pub id: Uuid,
    /// Type of content.
    pub content_type: ClipboardContentType,
    /// Raw content bytes (UTF-8 text, PNG image data, etc.).
    pub data: Vec<u8>,
    /// When this entry was captured.
    pub timestamp: DateTime<Utc>,
    /// The device that originated this entry.
    pub source_device: Uuid,
}

impl ClipboardEntry {
    /// Create a new entry originating from the local device.
    pub fn new(
        content_type: ClipboardContentType,
        data: Vec<u8>,
        source_device: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            content_type,
            data,
            timestamp: Utc::now(),
            source_device,
        }
    }

    /// If the content is text, return it as a `&str`.
    pub fn as_text(&self) -> Option<&str> {
        if self.content_type == ClipboardContentType::Text
            || self.content_type == ClipboardContentType::Url
        {
            std::str::from_utf8(&self.data).ok()
        } else {
            None
        }
    }
}

/// Events emitted by the clipboard subsystem.
#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    /// The local clipboard changed (captured by the platform layer).
    LocalChanged(ClipboardEntry),
    /// A remote device pushed new clipboard content.
    RemoteReceived(ClipboardEntry),
}

/// Manages clipboard content and history.
pub struct ClipboardManager {
    /// Rolling history of clipboard entries (newest first).
    history: Arc<RwLock<VecDeque<ClipboardEntry>>>,
    /// Maximum number of history entries to keep.
    max_history: usize,
    /// The local device UUID (used as source for locally captured entries).
    local_device_id: Uuid,
    /// Broadcast channel for clipboard events.
    event_tx: broadcast::Sender<ClipboardEvent>,
}

impl ClipboardManager {
    /// Create a new clipboard manager.
    pub fn new(local_device_id: Uuid, max_history: usize) -> Self {
        let (event_tx, _) = broadcast::channel(64);
        Self {
            history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history))),
            max_history,
            local_device_id,
            event_tx,
        }
    }

    /// Subscribe to clipboard events.
    pub fn subscribe(&self) -> broadcast::Receiver<ClipboardEvent> {
        self.event_tx.subscribe()
    }

    /// Capture new content from the local system clipboard.
    ///
    /// This is a **placeholder** for OS-specific clipboard integration.  The
    /// platform layer (Tauri, Android, etc.) should call this method whenever
    /// the clipboard changes.
    pub async fn set_local(
        &self,
        content_type: ClipboardContentType,
        data: Vec<u8>,
    ) -> Result<ClipboardEntry> {
        let entry = ClipboardEntry::new(content_type, data, self.local_device_id);
        self.push_entry(entry.clone()).await;
        let _ = self
            .event_tx
            .send(ClipboardEvent::LocalChanged(entry.clone()));
        tracing::debug!("local clipboard updated: {:?}", entry.content_type);
        Ok(entry)
    }

    /// Handle clipboard content received from a remote peer.
    pub async fn set_remote(&self, entry: ClipboardEntry) -> Result<()> {
        self.push_entry(entry.clone()).await;
        let _ = self
            .event_tx
            .send(ClipboardEvent::RemoteReceived(entry.clone()));
        tracing::debug!(
            "remote clipboard received from {}: {:?}",
            entry.source_device,
            entry.content_type
        );
        Ok(())
    }

    /// Get the latest clipboard entry (if any).
    pub async fn latest(&self) -> Option<ClipboardEntry> {
        let history = self.history.read().await;
        history.front().cloned()
    }

    /// Return the full clipboard history (newest first).
    pub async fn history(&self) -> Vec<ClipboardEntry> {
        let history = self.history.read().await;
        history.iter().cloned().collect()
    }

    /// Clear the clipboard history.
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
        tracing::debug!("clipboard history cleared");
    }

    /// Read the current system clipboard as text.
    ///
    /// **Placeholder**: returns `Err` until a platform implementation is wired
    /// in.
    pub fn get_system_clipboard_text(&self) -> Result<String> {
        Err(StarkLinkError::Clipboard(
            "system clipboard access not implemented for this platform".into(),
        ))
    }

    /// Write text to the system clipboard.
    ///
    /// **Placeholder**: returns `Err` until a platform implementation is wired
    /// in.
    pub fn set_system_clipboard_text(&self, _text: &str) -> Result<()> {
        Err(StarkLinkError::Clipboard(
            "system clipboard access not implemented for this platform".into(),
        ))
    }

    // ── internal ───────────────────────────────────────────────────────

    /// Push an entry to the front of the history, evicting old entries if
    /// needed.
    async fn push_entry(&self, entry: ClipboardEntry) {
        let mut history = self.history.write().await;
        history.push_front(entry);
        while history.len() > self.max_history {
            history.pop_back();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn history_cap() {
        let mgr = ClipboardManager::new(Uuid::new_v4(), 3);
        for i in 0..5 {
            mgr.set_local(
                ClipboardContentType::Text,
                format!("item {i}").into_bytes(),
            )
            .await
            .unwrap();
        }
        let h = mgr.history().await;
        assert_eq!(h.len(), 3);
        assert_eq!(h[0].as_text().unwrap(), "item 4");
    }
}
