//! Error types for Stark-Link.
//!
//! All errors throughout the library are funneled into [`StarkLinkError`] so
//! callers only need to handle a single error type.

use thiserror::Error;

/// Top-level error type for every fallible operation in Stark-Link.
#[derive(Debug, Error)]
pub enum StarkLinkError {
    // ── Crypto ──────────────────────────────────────────────────────────
    /// Encryption or decryption failed.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// Key exchange or derivation failed.
    #[error("key exchange error: {0}")]
    KeyExchange(String),

    // ── Network ────────────────────────────────────────────────────────
    /// Generic network / I/O error.
    #[error("network error: {0}")]
    Network(String),

    /// WebSocket-level error.
    #[error("websocket error: {0}")]
    WebSocket(String),

    /// Connection timed out.
    #[error("connection timed out")]
    ConnectionTimeout,

    /// Connection was refused by the remote peer.
    #[error("connection refused: {0}")]
    ConnectionRefused(String),

    // ── Transfer ───────────────────────────────────────────────────────
    /// File transfer error.
    #[error("transfer error: {0}")]
    Transfer(String),

    /// Checksum mismatch while verifying a chunk or file.
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    /// The maximum number of concurrent transfers has been reached.
    #[error("max concurrent transfers reached (limit: {0})")]
    MaxTransfersReached(usize),

    /// A transfer was cancelled.
    #[error("transfer cancelled: {0}")]
    TransferCancelled(String),

    // ── Discovery ──────────────────────────────────────────────────────
    /// mDNS discovery error.
    #[error("discovery error: {0}")]
    Discovery(String),

    // ── Protocol ───────────────────────────────────────────────────────
    /// Failed to serialize or deserialize a protocol message.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Received an unknown or unexpected message type.
    #[error("unexpected message type: {0}")]
    UnexpectedMessage(String),

    // ── Clipboard ──────────────────────────────────────────────────────
    /// Clipboard access error.
    #[error("clipboard error: {0}")]
    Clipboard(String),

    // ── Connection ─────────────────────────────────────────────────────
    /// The connection is not in the expected state for this operation.
    #[error("invalid connection state: expected {expected}, got {actual}")]
    InvalidConnectionState { expected: String, actual: String },

    /// The peer is not paired.
    #[error("peer not paired: {0}")]
    NotPaired(String),

    // ── Config ──────────────────────────────────────────────────────────
    /// Configuration error (load / save / parse).
    #[error("config error: {0}")]
    Config(String),

    // ── Wrapping external errors ───────────────────────────────────────
    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// MessagePack encode error.
    #[error("msgpack encode error: {0}")]
    MsgPackEncode(#[from] rmp_serde::encode::Error),

    /// MessagePack decode error.
    #[error("msgpack decode error: {0}")]
    MsgPackDecode(#[from] rmp_serde::decode::Error),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, StarkLinkError>;
