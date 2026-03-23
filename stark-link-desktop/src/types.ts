export interface DeviceInfo {
  id: string;
  name: string;
  os: string;
  device_type: string;
  battery_level: number | null;
  fingerprint: string;
}

export interface DiscoveredDevice {
  id: string;
  name: string;
  os: string;
  device_type: string;
  battery_level: number | null;
  addresses: string[];
  port: number;
  online: boolean;
}

export interface ClipboardEntry {
  id: string;
  content_type: string;
  text: string | null;
  timestamp: string;
  source_device: string;
}

export interface TransferInfo {
  id: string;
  file_name: string;
  file_size: number;
  total_chunks: number;
  chunks_done: number;
  state: string;
  direction: string;
  bytes_transferred: number;
  progress: number;
  speed_bps: number;
  eta_secs: number;
  peer_id: string;
}
