import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Monitor,
  ArrowUpDown,
  ClipboardList,
  Upload,
  RefreshCw,
  Zap,
  Activity,
  Wifi,
} from "lucide-react";
import type { DeviceInfo, TransferInfo, ClipboardEntry } from "../types";

interface ConnectedPeer {
  id: string;
  state: string;
  address: string;
}

function Dashboard() {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [connectedPeers, setConnectedPeers] = useState<ConnectedPeer[]>([]);
  const [transfers, setTransfers] = useState<TransferInfo[]>([]);
  const [clipboard, setClipboard] = useState<ClipboardEntry[]>([]);
  const [localIp, setLocalIp] = useState("");
  const [sendStatus, setSendStatus] = useState("");

  async function loadData() {
    try {
      const info = await invoke<DeviceInfo>("get_device_info");
      setDeviceInfo(info);
    } catch (e) {
      console.error("Failed to get device info:", e);
    }
    try {
      const peers = await invoke<ConnectedPeer[]>("get_connected_peers");
      setConnectedPeers(peers);
    } catch (e) {
      console.error("Failed to get peers:", e);
    }
    try {
      const txs = await invoke<TransferInfo[]>("get_transfers");
      setTransfers(txs);
    } catch (e) {
      console.error("Failed to get transfers:", e);
    }
    try {
      const clips = await invoke<ClipboardEntry[]>("get_clipboard_history");
      setClipboard(clips);
    } catch (e) {
      console.error("Failed to get clipboard:", e);
    }
    try {
      const ip = await invoke<string>("get_local_ip");
      setLocalIp(ip);
    } catch (e) {
      console.error("Failed to get IP:", e);
    }
  }

  async function handleSendFile() {
    const pairedPeers = connectedPeers.filter(
      (p) => p.state === "Paired" || p.state === "Controlling"
    );

    if (pairedPeers.length === 0) {
      setSendStatus("No connected devices. Go to Devices and connect first.");
      setTimeout(() => setSendStatus(""), 4000);
      return;
    }

    const selected = await open({
      multiple: false,
      title: "Select a file to send",
    });

    if (!selected) return;

    const filePath = selected;
    if (!filePath) return;

    setSendStatus("Sending...");
    try {
      await invoke("send_file", {
        peerId: pairedPeers[0].id,
        filePath: filePath,
      });
      setSendStatus("File sent!");
      setTimeout(() => setSendStatus(""), 3000);
    } catch (e) {
      setSendStatus(`Failed: ${e}`);
      setTimeout(() => setSendStatus(""), 5000);
    }
  }

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 3000);
    return () => clearInterval(interval);
  }, []);

  const activeTransfers = transfers.filter(
    (t) => t.state === "InProgress" || t.state === "Paused"
  );
  const pairedPeers = connectedPeers.filter(
    (p) => p.state === "Paired" || p.state === "Controlling"
  );

  return (
    <div className="max-w-5xl mx-auto">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white mb-1">Dashboard</h1>
        <p className="text-dark-text-secondary text-sm">
          Overview of your Stark-Link network
        </p>
      </div>

      {/* Device identity card */}
      {deviceInfo && (
        <div className="bg-gradient-to-r from-accent-blue/8 to-accent-purple/8 border border-accent-blue/15 rounded-2xl p-6 mb-6">
          <div className="flex items-center gap-4">
            <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-accent-gradient-start to-accent-gradient-end flex items-center justify-center shadow-lg shadow-accent-blue/20">
              <Zap size={26} className="text-white" />
            </div>
            <div className="flex-1">
              <h2 className="text-xl font-bold text-white">{deviceInfo.name}</h2>
              <p className="text-xs text-dark-text-secondary mt-0.5">
                {deviceInfo.os} &middot; {deviceInfo.device_type} &middot;{" "}
                <span className="font-mono">{deviceInfo.id.slice(0, 8)}...</span>
              </p>
              {localIp && (
                <p className="text-xs text-dark-text-secondary mt-1">
                  <Wifi size={10} className="inline mr-1" />
                  <span className="font-mono text-accent-blue">{localIp}:42424</span>
                </p>
              )}
            </div>
            <div className="flex items-center gap-2 bg-status-online/10 px-3 py-1.5 rounded-full">
              <Activity size={14} className="text-status-online" />
              <span className="text-sm text-status-online font-medium">Active</span>
            </div>
          </div>
        </div>
      )}

      {/* Stats grid */}
      <div className="grid grid-cols-3 gap-5 mb-8">
        <div className="bg-dark-card border border-dark-border rounded-2xl p-5 hover:border-accent-blue/20 transition-colors">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-accent-blue/10 flex items-center justify-center">
              <Monitor size={20} className="text-accent-blue" />
            </div>
            <span className="text-sm text-dark-text-secondary">Devices</span>
          </div>
          <p className="text-3xl font-bold text-white">{pairedPeers.length}</p>
          <p className="text-xs text-dark-text-secondary mt-1">
            {pairedPeers.length > 0
              ? `${pairedPeers.length} connected`
              : "No devices connected"}
          </p>
        </div>

        <div className="bg-dark-card border border-dark-border rounded-2xl p-5 hover:border-accent-purple/20 transition-colors">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-accent-purple/10 flex items-center justify-center">
              <ArrowUpDown size={20} className="text-accent-purple" />
            </div>
            <span className="text-sm text-dark-text-secondary">Transfers</span>
          </div>
          <p className="text-3xl font-bold text-white">{activeTransfers.length}</p>
          <p className="text-xs text-dark-text-secondary mt-1">
            {transfers.length} total
          </p>
        </div>

        <div className="bg-dark-card border border-dark-border rounded-2xl p-5 hover:border-status-online/20 transition-colors">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-status-online/10 flex items-center justify-center">
              <ClipboardList size={20} className="text-status-online" />
            </div>
            <span className="text-sm text-dark-text-secondary">Clipboard</span>
          </div>
          <p className="text-3xl font-bold text-white">{clipboard.length}</p>
          <p className="text-xs text-dark-text-secondary mt-1">history items</p>
        </div>
      </div>

      {/* Quick actions */}
      <div className="mb-8">
        <h3 className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-3">
          Quick Actions
        </h3>
        <div className="flex gap-3 items-center">
          <button
            onClick={handleSendFile}
            className="flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-accent-blue to-accent-purple rounded-xl text-sm font-semibold text-white hover:opacity-90 transition-opacity shadow-lg shadow-accent-blue/10"
          >
            <Upload size={16} />
            Send File
          </button>
          <button
            onClick={loadData}
            className="flex items-center gap-2 px-5 py-2.5 bg-dark-card border border-dark-border rounded-xl text-sm font-medium text-dark-text hover:bg-dark-hover transition-colors"
          >
            <RefreshCw size={16} />
            Refresh
          </button>
          {sendStatus && (
            <span
              className={`text-sm ${
                sendStatus.startsWith("Failed") || sendStatus.startsWith("No connected")
                  ? "text-status-error"
                  : sendStatus === "Sending..."
                  ? "text-status-warning"
                  : "text-status-online"
              }`}
            >
              {sendStatus}
            </span>
          )}
        </div>
      </div>

      {/* Connected devices */}
      {pairedPeers.length > 0 && (
        <div className="mb-8">
          <h3 className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-3">
            Connected Devices
          </h3>
          <div className="space-y-2">
            {pairedPeers.map((peer) => (
              <div
                key={peer.id}
                className="bg-dark-card border border-dark-border rounded-xl px-5 py-3 flex items-center gap-3"
              >
                <span className="w-2.5 h-2.5 rounded-full bg-status-online shadow-sm shadow-status-online/50"></span>
                <span className="text-sm text-white font-medium font-mono">
                  {peer.id.slice(0, 8)}...
                </span>
                <span className="text-xs text-dark-text-secondary">{peer.address}</span>
                <span className="ml-auto text-xs text-status-online font-medium bg-status-online/10 px-2 py-0.5 rounded-full">
                  {peer.state}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Recent clipboard */}
      {clipboard.length > 0 && (
        <div>
          <h3 className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-3">
            Recent Clipboard
          </h3>
          <div className="space-y-2">
            {clipboard.slice(0, 5).map((entry) => (
              <div
                key={entry.id}
                className="bg-dark-card border border-dark-border rounded-xl px-5 py-3 text-sm text-dark-text truncate hover:border-accent-blue/20 transition-colors cursor-pointer"
                onClick={() =>
                  entry.text && navigator.clipboard.writeText(entry.text)
                }
              >
                {entry.text || `[${entry.content_type}]`}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export default Dashboard;
