//! File transfer engine.
//!
//! Handles chunked file transfers with SHA-256 integrity verification, LZ4
//! compression, pause/resume, and concurrent transfer tracking.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use sha2::{Digest, Sha256};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::config::Config;
use crate::error::{Result, StarkLinkError};
use crate::protocol::{Message, Payload};

/// Default chunk size (5 MiB).
pub const DEFAULT_CHUNK_SIZE: usize = 5 * 1024 * 1024;

/// State of a single file transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferState {
    /// Waiting to start.
    Pending,
    /// Actively sending / receiving chunks.
    InProgress,
    /// Paused by either side.
    Paused,
    /// Completed successfully.
    Completed,
    /// Cancelled.
    Cancelled,
    /// Failed with an error.
    Failed,
}

/// Direction of the transfer relative to this device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    /// We are sending the file.
    Outgoing,
    /// We are receiving the file.
    Incoming,
}

/// Tracks the progress of a single file transfer.
#[derive(Debug, Clone)]
pub struct TransferInfo {
    /// Unique transfer identifier.
    pub id: Uuid,
    /// Name of the file being transferred.
    pub file_name: String,
    /// Total file size in bytes.
    pub file_size: u64,
    /// Total number of chunks.
    pub total_chunks: u32,
    /// Number of chunks transferred so far.
    pub chunks_done: u32,
    /// SHA-256 checksum of the entire file.
    pub file_checksum: String,
    /// Current state.
    pub state: TransferState,
    /// Direction.
    pub direction: TransferDirection,
    /// Bytes transferred so far.
    pub bytes_transferred: u64,
    /// When the transfer started.
    pub started_at: Instant,
    /// Current transfer speed in bytes per second.
    pub speed_bps: f64,
    /// Estimated time remaining in seconds.
    pub eta_secs: f64,
    /// Peer we are transferring with.
    pub peer_id: Uuid,
}

impl TransferInfo {
    /// Fraction of the transfer that is complete (0.0 .. 1.0).
    pub fn progress(&self) -> f64 {
        if self.file_size == 0 {
            return 1.0;
        }
        self.bytes_transferred as f64 / self.file_size as f64
    }

    /// Recalculate speed and ETA based on elapsed time.
    fn update_speed(&mut self) {
        let elapsed = self.started_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.speed_bps = self.bytes_transferred as f64 / elapsed;
            let remaining = self.file_size.saturating_sub(self.bytes_transferred) as f64;
            self.eta_secs = if self.speed_bps > 0.0 {
                remaining / self.speed_bps
            } else {
                f64::INFINITY
            };
        }
    }
}

/// Events the transfer subsystem emits.
#[derive(Debug, Clone)]
pub enum TransferEvent {
    /// A new transfer has been initiated.
    Started(TransferInfo),
    /// Transfer progress updated.
    Progress(TransferInfo),
    /// Transfer completed.
    Completed { transfer_id: Uuid },
    /// Transfer failed.
    Failed {
        transfer_id: Uuid,
        error: String,
    },
    /// Transfer cancelled.
    Cancelled { transfer_id: Uuid },
    /// Transfer paused.
    Paused { transfer_id: Uuid },
    /// Transfer resumed.
    Resumed { transfer_id: Uuid },
}

/// Manages file transfers (both outgoing and incoming).
pub struct TransferManager {
    /// All tracked transfers.
    transfers: Arc<RwLock<HashMap<Uuid, TransferInfo>>>,
    /// Configuration.
    config: Config,
    /// Channel for sending protocol messages to the connection layer.
    msg_tx: mpsc::Sender<(Uuid, Message)>,
    /// Event channel.
    event_tx: tokio::sync::broadcast::Sender<TransferEvent>,
}

impl TransferManager {
    /// Create a new transfer manager.
    ///
    /// `msg_tx` is used to hand off framed protocol messages to the connection
    /// layer for delivery.  Each item is `(peer_id, message)`.
    pub fn new(config: Config, msg_tx: mpsc::Sender<(Uuid, Message)>) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(128);
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            config,
            msg_tx,
            event_tx,
        }
    }

    /// Subscribe to transfer events.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<TransferEvent> {
        self.event_tx.subscribe()
    }

    /// How many transfers are currently in progress.
    pub async fn active_count(&self) -> usize {
        let map = self.transfers.read().await;
        map.values()
            .filter(|t| t.state == TransferState::InProgress)
            .count()
    }

    /// Get info for a specific transfer.
    pub async fn get(&self, transfer_id: &Uuid) -> Option<TransferInfo> {
        let map = self.transfers.read().await;
        map.get(transfer_id).cloned()
    }

    /// List all transfers.
    pub async fn list(&self) -> Vec<TransferInfo> {
        let map = self.transfers.read().await;
        map.values().cloned().collect()
    }

    // ── Outgoing ───────────────────────────────────────────────────────

    /// Start sending a file to a peer.
    ///
    /// Reads the file, computes the checksum, chunks it, and begins streaming.
    pub async fn send_file(&self, peer_id: Uuid, path: &Path) -> Result<Uuid> {
        // Check concurrent limit.
        if self.active_count().await >= self.config.max_concurrent_transfers {
            return Err(StarkLinkError::MaxTransfersReached(
                self.config.max_concurrent_transfers,
            ));
        }

        let file_data = tokio::fs::read(path).await.map_err(|e| {
            StarkLinkError::Transfer(format!("failed to read {}: {e}", path.display()))
        })?;

        let file_size = file_data.len() as u64;
        if file_size > self.config.max_file_size {
            return Err(StarkLinkError::Transfer(format!(
                "file too large: {} bytes (max {})",
                file_size, self.config.max_file_size
            )));
        }

        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unnamed".into());

        let file_checksum = sha256_hex(&file_data);
        let chunk_size = self.config.chunk_size;
        let total_chunks = ((file_size as usize + chunk_size - 1) / chunk_size) as u32;
        let transfer_id = Uuid::new_v4();

        let info = TransferInfo {
            id: transfer_id,
            file_name: file_name.clone(),
            file_size,
            total_chunks,
            chunks_done: 0,
            file_checksum: file_checksum.clone(),
            state: TransferState::InProgress,
            direction: TransferDirection::Outgoing,
            bytes_transferred: 0,
            started_at: Instant::now(),
            speed_bps: 0.0,
            eta_secs: 0.0,
            peer_id,
        };

        {
            let mut map = self.transfers.write().await;
            map.insert(transfer_id, info.clone());
        }

        let _ = self.event_tx.send(TransferEvent::Started(info));

        // Send FileTransferStart.
        let start_msg = Message::new(
            peer_id,
            Payload::FileTransferStart {
                transfer_id,
                file_name,
                file_size,
                total_chunks,
                file_checksum,
            },
        );
        self.msg_tx.send((peer_id, start_msg)).await.map_err(|e| {
            StarkLinkError::Transfer(format!("failed to send start message: {e}"))
        })?;

        // Stream chunks.
        let transfers = Arc::clone(&self.transfers);
        let msg_tx = self.msg_tx.clone();
        let event_tx = self.event_tx.clone();
        let compression_enabled = self.config.compression_enabled;

        tokio::spawn(async move {
            for (i, chunk) in file_data.chunks(chunk_size).enumerate() {
                // Check if paused or cancelled.
                {
                    let map = transfers.read().await;
                    if let Some(t) = map.get(&transfer_id) {
                        match t.state {
                            TransferState::Cancelled => return,
                            TransferState::Paused => {
                                // Wait until resumed.
                                drop(map);
                                loop {
                                    tokio::time::sleep(std::time::Duration::from_millis(200))
                                        .await;
                                    let map = transfers.read().await;
                                    match map.get(&transfer_id).map(|t| t.state) {
                                        Some(TransferState::Paused) => continue,
                                        Some(TransferState::Cancelled) => return,
                                        _ => break,
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                let chunk_checksum = sha256_hex(chunk);

                let (data, compressed) = if compression_enabled {
                    (lz4_flex::compress_prepend_size(chunk), true)
                } else {
                    (chunk.to_vec(), false)
                };

                let chunk_msg = Message::new(
                    peer_id,
                    Payload::FileTransferChunk {
                        transfer_id,
                        chunk_index: i as u32,
                        data,
                        checksum: chunk_checksum,
                        compressed,
                    },
                );

                if msg_tx.send((peer_id, chunk_msg)).await.is_err() {
                    tracing::error!("transfer {transfer_id}: send channel closed");
                    let mut map = transfers.write().await;
                    if let Some(t) = map.get_mut(&transfer_id) {
                        t.state = TransferState::Failed;
                    }
                    let _ = event_tx.send(TransferEvent::Failed {
                        transfer_id,
                        error: "send channel closed".into(),
                    });
                    return;
                }

                // Update progress.
                {
                    let mut map = transfers.write().await;
                    if let Some(t) = map.get_mut(&transfer_id) {
                        t.chunks_done = (i + 1) as u32;
                        t.bytes_transferred += chunk.len() as u64;
                        t.update_speed();

                        let _ = event_tx.send(TransferEvent::Progress(t.clone()));
                    }
                }
            }

            // Send completion.
            let complete_msg =
                Message::new(peer_id, Payload::FileTransferComplete { transfer_id });
            let _ = msg_tx.send((peer_id, complete_msg)).await;

            {
                let mut map = transfers.write().await;
                if let Some(t) = map.get_mut(&transfer_id) {
                    t.state = TransferState::Completed;
                }
            }
            let _ = event_tx.send(TransferEvent::Completed { transfer_id });
            tracing::info!("transfer {transfer_id} completed");
        });

        Ok(transfer_id)
    }

    // ── Incoming ───────────────────────────────────────────────────────

    /// Handle a `FileTransferStart` message from a peer.
    pub async fn handle_transfer_start(
        &self,
        peer_id: Uuid,
        transfer_id: Uuid,
        file_name: String,
        file_size: u64,
        total_chunks: u32,
        file_checksum: String,
    ) -> Result<()> {
        let info = TransferInfo {
            id: transfer_id,
            file_name,
            file_size,
            total_chunks,
            chunks_done: 0,
            file_checksum,
            state: TransferState::InProgress,
            direction: TransferDirection::Incoming,
            bytes_transferred: 0,
            started_at: Instant::now(),
            speed_bps: 0.0,
            eta_secs: 0.0,
            peer_id,
        };

        let mut map = self.transfers.write().await;
        map.insert(transfer_id, info.clone());
        let _ = self.event_tx.send(TransferEvent::Started(info));

        tracing::info!("incoming transfer {transfer_id} started");
        Ok(())
    }

    /// Handle a `FileTransferChunk` message.
    ///
    /// Returns the decompressed chunk data for the caller to write to disk.
    pub async fn handle_chunk(
        &self,
        transfer_id: Uuid,
        chunk_index: u32,
        data: Vec<u8>,
        checksum: String,
        compressed: bool,
    ) -> Result<Vec<u8>> {
        let decompressed = if compressed {
            lz4_flex::decompress_size_prepended(&data).map_err(|e| {
                StarkLinkError::Transfer(format!("decompression failed: {e}"))
            })?
        } else {
            data
        };

        // Verify chunk checksum.
        let actual = sha256_hex(&decompressed);
        if actual != checksum {
            return Err(StarkLinkError::ChecksumMismatch {
                expected: checksum,
                actual,
            });
        }

        // Update progress.
        {
            let mut map = self.transfers.write().await;
            if let Some(t) = map.get_mut(&transfer_id) {
                t.chunks_done = chunk_index + 1;
                t.bytes_transferred += decompressed.len() as u64;
                t.update_speed();

                let _ = self.event_tx.send(TransferEvent::Progress(t.clone()));
            }
        }

        Ok(decompressed)
    }

    /// Handle a `FileTransferComplete` message.
    pub async fn handle_complete(&self, transfer_id: Uuid) -> Result<()> {
        let mut map = self.transfers.write().await;
        if let Some(t) = map.get_mut(&transfer_id) {
            t.state = TransferState::Completed;
            let _ = self.event_tx.send(TransferEvent::Completed { transfer_id });
            tracing::info!("transfer {transfer_id} complete");
        }
        Ok(())
    }

    // ── Control ────────────────────────────────────────────────────────

    /// Pause a transfer.
    pub async fn pause(&self, transfer_id: Uuid) -> Result<()> {
        let mut map = self.transfers.write().await;
        let t = map
            .get_mut(&transfer_id)
            .ok_or_else(|| StarkLinkError::Transfer("transfer not found".into()))?;

        if t.state != TransferState::InProgress {
            return Err(StarkLinkError::Transfer("transfer not in progress".into()));
        }

        t.state = TransferState::Paused;
        let _ = self.event_tx.send(TransferEvent::Paused { transfer_id });
        tracing::info!("transfer {transfer_id} paused");
        Ok(())
    }

    /// Resume a paused transfer.
    pub async fn resume(&self, transfer_id: Uuid) -> Result<()> {
        let mut map = self.transfers.write().await;
        let t = map
            .get_mut(&transfer_id)
            .ok_or_else(|| StarkLinkError::Transfer("transfer not found".into()))?;

        if t.state != TransferState::Paused {
            return Err(StarkLinkError::Transfer("transfer not paused".into()));
        }

        t.state = TransferState::InProgress;
        let _ = self.event_tx.send(TransferEvent::Resumed { transfer_id });
        tracing::info!("transfer {transfer_id} resumed");
        Ok(())
    }

    /// Cancel a transfer.
    pub async fn cancel(&self, transfer_id: Uuid, reason: String) -> Result<()> {
        let mut map = self.transfers.write().await;
        let t = map
            .get_mut(&transfer_id)
            .ok_or_else(|| StarkLinkError::Transfer("transfer not found".into()))?;

        t.state = TransferState::Cancelled;
        let _ = self.event_tx.send(TransferEvent::Cancelled { transfer_id });
        tracing::info!("transfer {transfer_id} cancelled: {reason}");
        Ok(())
    }

    /// Get the download path for a received file.
    pub fn download_path(&self, file_name: &str) -> PathBuf {
        self.config.download_dir.join(file_name)
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Compute the hex-encoded SHA-256 digest of `data`.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_deterministic() {
        let a = sha256_hex(b"hello");
        let b = sha256_hex(b"hello");
        assert_eq!(a, b);
        assert_ne!(a, sha256_hex(b"world"));
    }

    #[test]
    fn compress_decompress_round_trip() {
        let data = b"the quick brown fox jumps over the lazy dog".repeat(100);
        let compressed = lz4_flex::compress_prepend_size(&data);
        let decompressed = lz4_flex::decompress_size_prepended(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }
}
