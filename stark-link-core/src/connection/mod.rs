//! WebSocket-based peer connections.
//!
//! Provides a WebSocket server, client, per-peer connection state machine,
//! heartbeat keep-alive, and a [`ConnectionManager`] that orchestrates
//! multiple simultaneous peer connections.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::{accept_async, connect_async};
use uuid::Uuid;

use crate::config::Config;
use crate::device::DeviceInfo;
use crate::error::{Result, StarkLinkError};
use crate::protocol::{Message, Payload};

// ── Connection state machine ───────────────────────────────────────────────

/// States a peer connection can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// No connection to this peer.
    Disconnected,
    /// TCP / WebSocket handshake in progress.
    Connecting,
    /// Stark-Link protocol handshake (Hello exchange).
    Handshake,
    /// Paired and ready for data exchange.
    Paired,
    /// Remote-control session is active.
    Controlling,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Handshake => write!(f, "Handshake"),
            Self::Paired => write!(f, "Paired"),
            Self::Controlling => write!(f, "Controlling"),
        }
    }
}

// ── Peer connection handle ─────────────────────────────────────────────────

/// Represents a single peer connection.
#[derive(Debug)]
pub struct PeerConnection {
    /// The peer's device UUID.
    pub peer_id: Uuid,
    /// Current state.
    pub state: ConnectionState,
    /// When we last received *any* message from the peer.
    pub last_seen: Instant,
    /// Outgoing message channel — the writer task drains this.
    pub tx: mpsc::Sender<Message>,
    /// The address of the peer (for reconnect).
    pub address: Option<SocketAddr>,
}

// ── Events emitted by the connection layer ─────────────────────────────────

/// Events that the connection layer pushes to the application.
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// A new peer connected (inbound or outbound).
    PeerConnected { peer_id: Uuid },
    /// A peer disconnected.
    PeerDisconnected { peer_id: Uuid },
    /// Received a protocol message from a peer.
    MessageReceived { peer_id: Uuid, message: Message },
    /// The connection state changed.
    StateChanged {
        peer_id: Uuid,
        old: ConnectionState,
        new: ConnectionState,
    },
}

// ── Connection manager ─────────────────────────────────────────────────────

/// Manages all peer connections, the WebSocket server, and heartbeats.
pub struct ConnectionManager {
    /// Active peer connections.
    peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
    /// Broadcast channel for connection events.
    event_tx: broadcast::Sender<ConnectionEvent>,
    /// Local device info (used during handshake).
    local_device: DeviceInfo,
    /// Configuration.
    config: Config,
}

impl ConnectionManager {
    /// Create a new connection manager.
    pub fn new(local_device: DeviceInfo, config: Config) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            local_device,
            config,
        }
    }

    /// Subscribe to connection events.
    pub fn subscribe(&self) -> broadcast::Receiver<ConnectionEvent> {
        self.event_tx.subscribe()
    }

    /// Return a list of currently connected peer UUIDs.
    pub async fn connected_peers(&self) -> Vec<Uuid> {
        let map = self.peers.read().await;
        map.keys().copied().collect()
    }

    /// Get the current state of a peer connection.
    pub async fn peer_state(&self, peer_id: &Uuid) -> Option<ConnectionState> {
        let map = self.peers.read().await;
        map.get(peer_id).map(|p| p.state)
    }

    /// Send a protocol message to a specific peer.
    pub async fn send(&self, peer_id: &Uuid, msg: Message) -> Result<()> {
        let map = self.peers.read().await;
        let peer = map
            .get(peer_id)
            .ok_or_else(|| StarkLinkError::Network(format!("peer {peer_id} not connected")))?;

        peer.tx
            .send(msg)
            .await
            .map_err(|e| StarkLinkError::Network(format!("send to {peer_id} failed: {e}")))?;

        Ok(())
    }

    /// Send a message to all connected, paired peers.
    pub async fn broadcast(&self, payload: Payload) -> Result<()> {
        let map = self.peers.read().await;
        for (id, peer) in map.iter() {
            if peer.state == ConnectionState::Paired
                || peer.state == ConnectionState::Controlling
            {
                let msg = Message::new(self.local_device.id, payload.clone());
                if let Err(e) = peer.tx.send(msg).await {
                    tracing::warn!("broadcast to {id} failed: {e}");
                }
            }
        }
        Ok(())
    }

    // ── Server ─────────────────────────────────────────────────────────

    /// Start the WebSocket server on the configured port.
    ///
    /// Spawns a background task that accepts incoming connections.
    pub async fn start_server(&self) -> Result<tokio::task::JoinHandle<()>> {
        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            StarkLinkError::Network(format!("failed to bind {addr}: {e}"))
        })?;

        tracing::info!("WebSocket server listening on {addr}");

        let peers = Arc::clone(&self.peers);
        let event_tx = self.event_tx.clone();
        let local = self.local_device.clone();
        let heartbeat_secs = self.config.heartbeat_interval_secs;

        let handle = tokio::spawn(async move {
            loop {
                let (stream, remote_addr) = match listener.accept().await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("accept failed: {e}");
                        continue;
                    }
                };

                tracing::info!("incoming connection from {remote_addr}");

                let peers = Arc::clone(&peers);
                let event_tx = event_tx.clone();
                let local = local.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_incoming(
                        stream,
                        remote_addr,
                        peers,
                        event_tx,
                        local,
                        heartbeat_secs,
                    )
                    .await
                    {
                        tracing::error!("error handling connection from {remote_addr}: {e}");
                    }
                });
            }
        });

        Ok(handle)
    }

    // ── Client ─────────────────────────────────────────────────────────

    /// Connect to a remote peer at the given address.
    pub async fn connect(&self, addr: SocketAddr) -> Result<()> {
        let url = format!("ws://{addr}");
        tracing::info!("connecting to {url}");

        let (ws_stream, _) = connect_async(&url).await.map_err(|e| {
            StarkLinkError::WebSocket(format!("failed to connect to {url}: {e}"))
        })?;

        let (write, read) = ws_stream.split();

        let (msg_tx, msg_rx) = mpsc::channel::<Message>(128);

        // Send a Hello immediately.
        let hello = Message::new(
            self.local_device.id,
            Payload::Hello {
                device_info: self.local_device.clone(),
                public_key: Vec::new(), // Caller sets the real key via crypto module.
            },
        );
        msg_tx.send(hello).await.map_err(|e| {
            StarkLinkError::Network(format!("failed to queue hello: {e}"))
        })?;

        // We don't know the peer's UUID yet — use a temporary one until
        // we receive their Hello. The reader task will update the map.
        let temp_id = Uuid::new_v4();

        {
            let mut map = self.peers.write().await;
            map.insert(
                temp_id,
                PeerConnection {
                    peer_id: temp_id,
                    state: ConnectionState::Connecting,
                    last_seen: Instant::now(),
                    tx: msg_tx,
                    address: Some(addr),
                },
            );
        }

        let peers = Arc::clone(&self.peers);
        let event_tx = self.event_tx.clone();
        let heartbeat_secs = self.config.heartbeat_interval_secs;

        // Spawn writer task.
        tokio::spawn(writer_task(msg_rx, write));

        // Spawn reader task.
        tokio::spawn(reader_task(
            temp_id,
            read,
            peers,
            event_tx,
            heartbeat_secs,
        ));

        Ok(())
    }

    // ── Disconnect ─────────────────────────────────────────────────────

    /// Disconnect from a specific peer.
    pub async fn disconnect(&self, peer_id: &Uuid) -> Result<()> {
        let mut map = self.peers.write().await;
        if map.remove(peer_id).is_some() {
            tracing::info!("disconnected from {peer_id}");
            let _ = self.event_tx.send(ConnectionEvent::PeerDisconnected {
                peer_id: *peer_id,
            });
            Ok(())
        } else {
            Err(StarkLinkError::Network(format!(
                "peer {peer_id} not found"
            )))
        }
    }

    /// Disconnect all peers and shut down the connection manager.
    pub async fn shutdown(&self) {
        let mut map = self.peers.write().await;
        let ids: Vec<Uuid> = map.keys().copied().collect();
        map.clear();

        for id in ids {
            let _ = self
                .event_tx
                .send(ConnectionEvent::PeerDisconnected { peer_id: id });
        }

        tracing::info!("connection manager shut down");
    }
}

// ── Internal tasks ─────────────────────────────────────────────────────────

/// Handle an incoming (server-side) WebSocket connection.
async fn handle_incoming(
    stream: TcpStream,
    remote_addr: SocketAddr,
    peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
    event_tx: broadcast::Sender<ConnectionEvent>,
    local: DeviceInfo,
    heartbeat_secs: u64,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.map_err(|e| {
        StarkLinkError::WebSocket(format!("websocket accept failed for {remote_addr}: {e}"))
    })?;

    let (write, read) = ws_stream.split();
    let (msg_tx, msg_rx) = mpsc::channel::<Message>(128);

    // Send our Hello.
    let hello = Message::new(
        local.id,
        Payload::Hello {
            device_info: local.clone(),
            public_key: Vec::new(),
        },
    );
    msg_tx
        .send(hello)
        .await
        .map_err(|e| StarkLinkError::Network(format!("failed to queue hello: {e}")))?;

    let temp_id = Uuid::new_v4();

    {
        let mut map = peers.write().await;
        map.insert(
            temp_id,
            PeerConnection {
                peer_id: temp_id,
                state: ConnectionState::Handshake,
                last_seen: Instant::now(),
                tx: msg_tx,
                address: Some(remote_addr),
            },
        );
    }

    tokio::spawn(writer_task(msg_rx, write));
    tokio::spawn(reader_task(
        temp_id,
        read,
        peers,
        event_tx,
        heartbeat_secs,
    ));

    Ok(())
}

/// Background task that drains the outgoing message queue and writes to the
/// WebSocket.
async fn writer_task<S>(mut rx: mpsc::Receiver<Message>, mut sink: S)
where
    S: futures_util::Sink<WsMessage, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin
        + Send,
{
    while let Some(msg) = rx.recv().await {
        let bytes = match msg.to_bytes() {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("failed to serialize message: {e}");
                continue;
            }
        };

        if let Err(e) = sink.send(WsMessage::Binary(bytes)).await {
            tracing::error!("write error: {e}");
            break;
        }
    }
}

/// Background task that reads from the WebSocket, updates peer state, and
/// emits connection events.
async fn reader_task<S>(
    initial_id: Uuid,
    mut stream: S,
    peers: Arc<RwLock<HashMap<Uuid, PeerConnection>>>,
    event_tx: broadcast::Sender<ConnectionEvent>,
    heartbeat_secs: u64,
) where
    S: futures_util::Stream<Item = std::result::Result<WsMessage, tokio_tungstenite::tungstenite::Error>>
        + Unpin
        + Send,
{
    let mut current_id = initial_id;
    let heartbeat_interval = Duration::from_secs(heartbeat_secs);
    let mut last_heartbeat = Instant::now();

    loop {
        // Use a timeout so we can send periodic pings.
        let read_result =
            tokio::time::timeout(heartbeat_interval, stream.next()).await;

        match read_result {
            Ok(Some(Ok(ws_msg))) => {
                let data = match ws_msg {
                    WsMessage::Binary(b) => b,
                    WsMessage::Close(_) => {
                        tracing::info!("peer {current_id} sent close frame");
                        break;
                    }
                    _ => continue,
                };

                let msg = match Message::from_bytes(&data) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!("invalid message from {current_id}: {e}");
                        continue;
                    }
                };

                // If this is a Hello, update the peer map to use the real UUID.
                if let Payload::Hello { ref device_info, .. } = msg.payload {
                    let real_id = device_info.id;

                    if real_id != current_id {
                        let mut map = peers.write().await;
                        if let Some(mut peer) = map.remove(&current_id) {
                            peer.peer_id = real_id;
                            peer.state = ConnectionState::Paired;
                            peer.last_seen = Instant::now();
                            map.insert(real_id, peer);
                        }
                        current_id = real_id;
                    } else {
                        let mut map = peers.write().await;
                        if let Some(peer) = map.get_mut(&current_id) {
                            peer.state = ConnectionState::Paired;
                            peer.last_seen = Instant::now();
                        }
                    }

                    let _ = event_tx.send(ConnectionEvent::PeerConnected {
                        peer_id: current_id,
                    });
                    let _ = event_tx.send(ConnectionEvent::StateChanged {
                        peer_id: current_id,
                        old: ConnectionState::Handshake,
                        new: ConnectionState::Paired,
                    });

                    tracing::info!("paired with {current_id}");
                }

                // Handle Ping / Pong at connection level.
                match &msg.payload {
                    Payload::Ping => {
                        let pong = Message::new(msg.sender, Payload::Pong);
                        let map = peers.read().await;
                        if let Some(peer) = map.get(&current_id) {
                            let _ = peer.tx.send(pong).await;
                        }
                    }
                    Payload::Pong => {
                        // Just update last_seen (done below).
                    }
                    _ => {}
                }

                // Update last_seen.
                {
                    let mut map = peers.write().await;
                    if let Some(peer) = map.get_mut(&current_id) {
                        peer.last_seen = Instant::now();
                    }
                }

                let _ = event_tx.send(ConnectionEvent::MessageReceived {
                    peer_id: current_id,
                    message: msg,
                });
            }

            Ok(Some(Err(e))) => {
                tracing::error!("read error from {current_id}: {e}");
                break;
            }

            Ok(None) => {
                // Stream ended.
                tracing::info!("connection to {current_id} closed");
                break;
            }

            Err(_) => {
                // Timeout — send heartbeat ping if needed.
                if last_heartbeat.elapsed() >= heartbeat_interval {
                    let map = peers.read().await;
                    if let Some(peer) = map.get(&current_id) {
                        let ping = Message::new(current_id, Payload::Ping);
                        if peer.tx.send(ping).await.is_err() {
                            tracing::warn!("failed to send heartbeat to {current_id}");
                            break;
                        }
                    }
                    last_heartbeat = Instant::now();
                }
            }
        }
    }

    // Clean up.
    {
        let mut map = peers.write().await;
        map.remove(&current_id);
    }
    let _ = event_tx.send(ConnectionEvent::PeerDisconnected {
        peer_id: current_id,
    });
    tracing::info!("reader task for {current_id} exiting");
}
