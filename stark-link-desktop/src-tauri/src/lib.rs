use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::RwLock;

use stark_link_core::StarkLink;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub stark_link: Arc<RwLock<StarkLink>>,
}

// ── Serializable types for the frontend ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfoResponse {
    pub id: String,
    pub name: String,
    pub os: String,
    pub device_type: String,
    pub battery_level: Option<u8>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDeviceResponse {
    pub id: String,
    pub name: String,
    pub os: String,
    pub device_type: String,
    pub battery_level: Option<u8>,
    pub addresses: Vec<String>,
    pub port: u16,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntryResponse {
    pub id: String,
    pub content_type: String,
    pub text: Option<String>,
    pub timestamp: String,
    pub source_device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInfoResponse {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub total_chunks: u32,
    pub chunks_done: u32,
    pub state: String,
    pub direction: String,
    pub bytes_transferred: u64,
    pub progress: f64,
    pub speed_bps: f64,
    pub eta_secs: f64,
    pub peer_id: String,
}

// ── Tauri Commands ──────────────────────────────────────────────────────────

#[tauri::command]
async fn get_device_info(state: State<'_, AppState>) -> Result<DeviceInfoResponse, String> {
    let sl = state.stark_link.read().await;
    Ok(DeviceInfoResponse {
        id: sl.device.id.to_string(),
        name: sl.device.name.clone(),
        os: format!("{}", sl.device.os),
        device_type: format!("{}", sl.device.device_type),
        battery_level: sl.device.battery_level,
        fingerprint: sl.fingerprint(),
    })
}

#[tauri::command]
async fn get_discovered_devices(
    state: State<'_, AppState>,
) -> Result<Vec<DiscoveredDeviceResponse>, String> {
    let sl = state.stark_link.read().await;
    let devices = sl.discovery.devices().await;
    Ok(devices
        .into_iter()
        .map(|d| DiscoveredDeviceResponse {
            id: d.info.id.to_string(),
            name: d.info.name.clone(),
            os: format!("{}", d.info.os),
            device_type: format!("{}", d.info.device_type),
            battery_level: d.info.battery_level,
            addresses: d.addresses.iter().map(|a| a.to_string()).collect(),
            port: d.port,
            online: d.online,
        })
        .collect())
}

#[tauri::command]
async fn start_discovery(state: State<'_, AppState>) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    sl.discovery
        .start_browsing()
        .map_err(|e| format!("Failed to start discovery: {}", e))?;
    Ok("Discovery started".to_string())
}

#[tauri::command]
async fn send_file(
    state: State<'_, AppState>,
    peer_id: String,
    file_path: String,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let peer_uuid =
        uuid::Uuid::parse_str(&peer_id).map_err(|e| format!("Invalid peer ID: {}", e))?;
    let path = PathBuf::from(file_path);

    let transfer_id = sl
        .transfer
        .send_file(peer_uuid, &path)
        .await
        .map_err(|e| format!("Failed to send file: {}", e))?;

    Ok(transfer_id.to_string())
}

#[tauri::command]
async fn get_clipboard_history(
    state: State<'_, AppState>,
) -> Result<Vec<ClipboardEntryResponse>, String> {
    let sl = state.stark_link.read().await;
    let history = sl.clipboard.history().await;
    Ok(history
        .into_iter()
        .map(|e| ClipboardEntryResponse {
            id: e.id.to_string(),
            content_type: format!("{:?}", e.content_type),
            text: e.as_text().map(|s| s.to_string()),
            timestamp: e.timestamp.to_rfc3339(),
            source_device: e.source_device.to_string(),
        })
        .collect())
}

#[tauri::command]
async fn add_clipboard_entry(
    state: State<'_, AppState>,
    text: String,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    sl.clipboard
        .set_local(
            stark_link_core::protocol::ClipboardContentType::Text,
            text.into_bytes(),
        )
        .await
        .map_err(|e| format!("Failed to add: {}", e))?;
    Ok("Added".to_string())
}

#[tauri::command]
async fn connect_to_device(
    state: State<'_, AppState>,
    address: String,
    port: u16,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let addr: std::net::SocketAddr = format!("{}:{}", address, port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;

    sl.connection
        .connect(addr)
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    Ok("Connected".to_string())
}

#[tauri::command]
async fn get_transfers(
    state: State<'_, AppState>,
) -> Result<Vec<TransferInfoResponse>, String> {
    let sl = state.stark_link.read().await;
    let transfers = sl.transfer.list().await;
    Ok(transfers
        .into_iter()
        .map(|t| TransferInfoResponse {
            id: t.id.to_string(),
            file_name: t.file_name.clone(),
            file_size: t.file_size,
            total_chunks: t.total_chunks,
            chunks_done: t.chunks_done,
            state: format!("{:?}", t.state),
            direction: format!("{:?}", t.direction),
            bytes_transferred: t.bytes_transferred,
            progress: t.progress(),
            speed_bps: t.speed_bps,
            eta_secs: t.eta_secs,
            peer_id: t.peer_id.to_string(),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedPeerResponse {
    pub id: String,
    pub state: String,
    pub address: String,
}

#[tauri::command]
async fn get_connected_peers(
    state: State<'_, AppState>,
) -> Result<Vec<ConnectedPeerResponse>, String> {
    let sl = state.stark_link.read().await;
    let peers = sl.connection.connected_peers_info().await;
    Ok(peers
        .into_iter()
        .map(|p| ConnectedPeerResponse {
            id: p.peer_id.to_string(),
            state: format!("{}", p.state),
            address: p.address.map(|a| a.to_string()).unwrap_or_default(),
        })
        .collect())
}

#[tauri::command]
async fn pause_transfer(
    state: State<'_, AppState>,
    transfer_id: String,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let id = uuid::Uuid::parse_str(&transfer_id)
        .map_err(|e| format!("Invalid transfer ID: {}", e))?;
    sl.transfer
        .pause(id)
        .await
        .map_err(|e| format!("Failed to pause: {}", e))?;
    Ok("Paused".to_string())
}

#[tauri::command]
async fn resume_transfer(
    state: State<'_, AppState>,
    transfer_id: String,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let id = uuid::Uuid::parse_str(&transfer_id)
        .map_err(|e| format!("Invalid transfer ID: {}", e))?;
    sl.transfer
        .resume(id)
        .await
        .map_err(|e| format!("Failed to resume: {}", e))?;
    Ok("Resumed".to_string())
}

#[tauri::command]
async fn cancel_transfer(
    state: State<'_, AppState>,
    transfer_id: String,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let id = uuid::Uuid::parse_str(&transfer_id)
        .map_err(|e| format!("Invalid transfer ID: {}", e))?;
    sl.transfer
        .cancel(id, "Cancelled by user".to_string())
        .await
        .map_err(|e| format!("Failed to cancel: {}", e))?;
    Ok("Cancelled".to_string())
}

#[tauri::command]
async fn get_local_ip() -> Result<String, String> {
    // Find the local network IP by connecting to a remote address (no data sent)
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket.connect("8.8.8.8:80").map_err(|e| e.to_string())?;
    let addr = socket.local_addr().map_err(|e| e.to_string())?;
    Ok(addr.ip().to_string())
}

// ── Remote Control Commands ──────────────────────────────────────────────────

#[tauri::command]
async fn send_mouse_event(
    state: State<'_, AppState>,
    peer_id: String,
    x: f64,
    y: f64,
    button: String,
    action: String,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let peer_uuid =
        uuid::Uuid::parse_str(&peer_id).map_err(|e| format!("Invalid peer ID: {}", e))?;

    let mouse_button = match button.as_str() {
        "left" => stark_link_core::protocol::MouseButton::Left,
        "right" => stark_link_core::protocol::MouseButton::Right,
        "middle" => stark_link_core::protocol::MouseButton::Middle,
        _ => stark_link_core::protocol::MouseButton::None,
    };

    let mouse_action = match action.as_str() {
        "move" => stark_link_core::protocol::MouseAction::Move,
        "down" => stark_link_core::protocol::MouseAction::Down,
        "up" => stark_link_core::protocol::MouseAction::Up,
        "click" => stark_link_core::protocol::MouseAction::Click,
        "doubleclick" => stark_link_core::protocol::MouseAction::DoubleClick,
        "scroll" => stark_link_core::protocol::MouseAction::Scroll,
        _ => stark_link_core::protocol::MouseAction::Move,
    };

    let msg = stark_link_core::protocol::Message::new(
        sl.device.id,
        stark_link_core::protocol::Payload::MouseEvent {
            session_id: uuid::Uuid::new_v4(),
            event: stark_link_core::protocol::MouseEventData {
                x,
                y,
                button: mouse_button,
                action: mouse_action,
            },
        },
    );

    sl.connection
        .send(&peer_uuid, msg)
        .await
        .map_err(|e| format!("Failed to send mouse event: {}", e))?;

    Ok("Sent".to_string())
}

#[tauri::command]
async fn send_keyboard_event(
    state: State<'_, AppState>,
    peer_id: String,
    key: String,
    action: String,
    modifiers: Vec<String>,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let peer_uuid =
        uuid::Uuid::parse_str(&peer_id).map_err(|e| format!("Invalid peer ID: {}", e))?;

    let key_action = match action.as_str() {
        "down" => stark_link_core::protocol::KeyAction::Down,
        "up" => stark_link_core::protocol::KeyAction::Up,
        _ => stark_link_core::protocol::KeyAction::Down,
    };

    let key_modifiers: Vec<stark_link_core::protocol::KeyModifier> = modifiers
        .iter()
        .filter_map(|m| match m.as_str() {
            "ctrl" => Some(stark_link_core::protocol::KeyModifier::Ctrl),
            "alt" => Some(stark_link_core::protocol::KeyModifier::Alt),
            "shift" => Some(stark_link_core::protocol::KeyModifier::Shift),
            "meta" => Some(stark_link_core::protocol::KeyModifier::Meta),
            _ => None,
        })
        .collect();

    let msg = stark_link_core::protocol::Message::new(
        sl.device.id,
        stark_link_core::protocol::Payload::KeyboardEvent {
            session_id: uuid::Uuid::new_v4(),
            event: stark_link_core::protocol::KeyboardEventData {
                key,
                action: key_action,
                modifiers: key_modifiers,
            },
        },
    );

    sl.connection
        .send(&peer_uuid, msg)
        .await
        .map_err(|e| format!("Failed to send keyboard event: {}", e))?;

    Ok("Sent".to_string())
}

#[tauri::command]
async fn send_remote_command(
    state: State<'_, AppState>,
    peer_id: String,
    command: String,
    timeout_secs: u32,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let peer_uuid =
        uuid::Uuid::parse_str(&peer_id).map_err(|e| format!("Invalid peer ID: {}", e))?;

    let msg = stark_link_core::protocol::Message::new(
        sl.device.id,
        stark_link_core::protocol::Payload::CommandExecute {
            command,
            timeout_secs,
        },
    );

    sl.connection
        .send(&peer_uuid, msg)
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    Ok("Command sent".to_string())
}

#[tauri::command]
async fn send_media_control(
    state: State<'_, AppState>,
    peer_id: String,
    action: String,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let peer_uuid =
        uuid::Uuid::parse_str(&peer_id).map_err(|e| format!("Invalid peer ID: {}", e))?;

    let media_action = match action.as_str() {
        "play" => stark_link_core::protocol::MediaAction::Play,
        "pause" => stark_link_core::protocol::MediaAction::Pause,
        "next" => stark_link_core::protocol::MediaAction::Next,
        "previous" => stark_link_core::protocol::MediaAction::Previous,
        "volume_up" => stark_link_core::protocol::MediaAction::VolumeUp,
        "volume_down" => stark_link_core::protocol::MediaAction::VolumeDown,
        "mute" => stark_link_core::protocol::MediaAction::Mute,
        _ => return Err("Unknown media action".to_string()),
    };

    let msg = stark_link_core::protocol::Message::new(
        sl.device.id,
        stark_link_core::protocol::Payload::MediaControl {
            action: media_action,
        },
    );

    sl.connection
        .send(&peer_uuid, msg)
        .await
        .map_err(|e| format!("Failed to send media control: {}", e))?;

    Ok("Sent".to_string())
}

#[tauri::command]
async fn lock_remote_device(
    state: State<'_, AppState>,
    peer_id: String,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let peer_uuid =
        uuid::Uuid::parse_str(&peer_id).map_err(|e| format!("Invalid peer ID: {}", e))?;

    let msg = stark_link_core::protocol::Message::new(
        sl.device.id,
        stark_link_core::protocol::Payload::RemoteLock,
    );

    sl.connection
        .send(&peer_uuid, msg)
        .await
        .map_err(|e| format!("Failed to lock device: {}", e))?;

    Ok("Lock command sent".to_string())
}

#[tauri::command]
async fn launch_remote_app(
    state: State<'_, AppState>,
    peer_id: String,
    app_id: String,
    args: Vec<String>,
) -> Result<String, String> {
    let sl = state.stark_link.read().await;
    let peer_uuid =
        uuid::Uuid::parse_str(&peer_id).map_err(|e| format!("Invalid peer ID: {}", e))?;

    let msg = stark_link_core::protocol::Message::new(
        sl.device.id,
        stark_link_core::protocol::Payload::AppLaunch { app_id, args },
    );

    sl.connection
        .send(&peer_uuid, msg)
        .await
        .map_err(|e| format!("Failed to launch app: {}", e))?;

    Ok("Launch command sent".to_string())
}

// ── App setup ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let stark_link = StarkLink::new(None).expect("Failed to initialize StarkLink");

    let app_state = AppState {
        stark_link: Arc::new(RwLock::new(stark_link)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(|app| {
            // Auto-start discovery and WebSocket server on launch
            let state = app.state::<AppState>();
            let sl = state.stark_link.clone();
            tauri::async_runtime::spawn(async move {
                let stark_link: tokio::sync::RwLockReadGuard<'_, StarkLink> = sl.read().await;
                match stark_link.start().await {
                    Ok(_) => {
                        eprintln!("[StarkLink] Discovery and server started successfully");
                    }
                    Err(e) => {
                        eprintln!("[StarkLink] Failed to start: {}", e);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_device_info,
            get_discovered_devices,
            start_discovery,
            send_file,
            get_clipboard_history,
            connect_to_device,
            get_transfers,
            get_connected_peers,
            pause_transfer,
            resume_transfer,
            cancel_transfer,
            get_local_ip,
            add_clipboard_entry,
            send_mouse_event,
            send_keyboard_event,
            send_remote_command,
            send_media_control,
            lock_remote_device,
            launch_remote_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Stark-Link");
}
