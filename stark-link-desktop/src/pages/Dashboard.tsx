import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Monitor,
  ArrowUpDown,
  ClipboardList,
  Upload,
  RefreshCw,
  Zap,
  Activity,
} from "lucide-react";
import type { DeviceInfo, DiscoveredDevice, TransferInfo, ClipboardEntry } from "../types";

function Dashboard() {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [devices, setDevices] = useState<DiscoveredDevice[]>([]);
  const [transfers, setTransfers] = useState<TransferInfo[]>([]);
  const [clipboard, setClipboard] = useState<ClipboardEntry[]>([]);

  async function loadData() {
    try {
      const info = await invoke<DeviceInfo>("get_device_info");
      setDeviceInfo(info);
    } catch (e) {
      console.error("Failed to get device info:", e);
    }
    try {
      const devs = await invoke<DiscoveredDevice[]>("get_discovered_devices");
      setDevices(devs);
    } catch (e) {
      console.error("Failed to get devices:", e);
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
  }

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, []);

  const activeTransfers = transfers.filter(
    (t) => t.state === "InProgress" || t.state === "Paused"
  );
  const onlineDevices = devices.filter((d) => d.online);

  return (
    <div className="max-w-5xl">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-white mb-1">Dashboard</h1>
        <p className="text-dark-text-secondary text-sm">
          Overview of your Stark-Link network
        </p>
      </div>

      {/* Device identity card */}
      {deviceInfo && (
        <div className="bg-gradient-to-r from-accent-blue/10 to-accent-purple/10 border border-accent-blue/20 rounded-xl p-5 mb-6">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-accent-gradient-start to-accent-gradient-end flex items-center justify-center">
              <Zap size={24} className="text-white" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">{deviceInfo.name}</h2>
              <p className="text-xs text-dark-text-secondary">
                {deviceInfo.os} &middot; {deviceInfo.device_type} &middot;{" "}
                <span className="font-mono">{deviceInfo.id.slice(0, 8)}...</span>
              </p>
            </div>
            <div className="ml-auto flex items-center gap-2">
              <Activity size={14} className="text-status-online" />
              <span className="text-sm text-status-online font-medium">Active</span>
            </div>
          </div>
        </div>
      )}

      {/* Stats grid */}
      <div className="grid grid-cols-3 gap-4 mb-8">
        <div className="bg-dark-card border border-dark-border rounded-xl p-5">
          <div className="flex items-center gap-3 mb-3">
            <div className="w-9 h-9 rounded-lg bg-accent-blue/10 flex items-center justify-center">
              <Monitor size={18} className="text-accent-blue" />
            </div>
            <span className="text-sm text-dark-text-secondary">Devices</span>
          </div>
          <p className="text-2xl font-bold text-white">{onlineDevices.length}</p>
          <p className="text-xs text-dark-text-secondary mt-1">
            {devices.length} discovered
          </p>
        </div>

        <div className="bg-dark-card border border-dark-border rounded-xl p-5">
          <div className="flex items-center gap-3 mb-3">
            <div className="w-9 h-9 rounded-lg bg-accent-purple/10 flex items-center justify-center">
              <ArrowUpDown size={18} className="text-accent-purple" />
            </div>
            <span className="text-sm text-dark-text-secondary">Transfers</span>
          </div>
          <p className="text-2xl font-bold text-white">{activeTransfers.length}</p>
          <p className="text-xs text-dark-text-secondary mt-1">
            {transfers.length} total
          </p>
        </div>

        <div className="bg-dark-card border border-dark-border rounded-xl p-5">
          <div className="flex items-center gap-3 mb-3">
            <div className="w-9 h-9 rounded-lg bg-status-online/10 flex items-center justify-center">
              <ClipboardList size={18} className="text-status-online" />
            </div>
            <span className="text-sm text-dark-text-secondary">Clipboard</span>
          </div>
          <p className="text-2xl font-bold text-white">{clipboard.length}</p>
          <p className="text-xs text-dark-text-secondary mt-1">history items</p>
        </div>
      </div>

      {/* Quick actions */}
      <div className="mb-8">
        <h3 className="text-sm font-semibold text-dark-text-secondary uppercase tracking-wider mb-3">
          Quick Actions
        </h3>
        <div className="flex gap-3">
          <button className="flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-accent-blue to-accent-purple rounded-lg text-sm font-medium text-white hover:opacity-90 transition-opacity">
            <Upload size={16} />
            Send File
          </button>
          <button
            onClick={loadData}
            className="flex items-center gap-2 px-4 py-2.5 bg-dark-card border border-dark-border rounded-lg text-sm font-medium text-dark-text hover:bg-dark-hover transition-colors"
          >
            <RefreshCw size={16} />
            Refresh
          </button>
        </div>
      </div>

      {/* Recent clipboard items */}
      {clipboard.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-dark-text-secondary uppercase tracking-wider mb-3">
            Recent Clipboard
          </h3>
          <div className="space-y-2">
            {clipboard.slice(0, 5).map((entry) => (
              <div
                key={entry.id}
                className="bg-dark-card border border-dark-border rounded-lg px-4 py-3 text-sm text-dark-text truncate hover:border-dark-hover transition-colors"
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
