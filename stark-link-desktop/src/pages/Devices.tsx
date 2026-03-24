import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Search, Radar, Plug, Wifi } from "lucide-react";
import type { DiscoveredDevice, DeviceInfo } from "../types";
import DeviceCard from "../components/DeviceCard";

function Devices() {
  const [devices, setDevices] = useState<DiscoveredDevice[]>([]);
  const [search, setSearch] = useState("");
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [showManualConnect, setShowManualConnect] = useState(false);
  const [manualIp, setManualIp] = useState("");
  const [manualPort, setManualPort] = useState("42424");
  const [connectStatus, setConnectStatus] = useState("");
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [localIp, setLocalIp] = useState("");

  async function loadDevices() {
    try {
      const devs = await invoke<DiscoveredDevice[]>("get_discovered_devices");
      setDevices(devs);
    } catch (e) {
      console.error("Failed to get devices:", e);
    }
  }

  async function loadDeviceInfo() {
    try {
      const info = await invoke<DeviceInfo>("get_device_info");
      setDeviceInfo(info);
    } catch (e) {
      console.error("Failed to get device info:", e);
    }
    try {
      const ip = await invoke<string>("get_local_ip");
      setLocalIp(ip);
    } catch (e) {
      console.error("Failed to get local IP:", e);
    }
  }

  async function handleStartDiscovery() {
    setIsDiscovering(true);
    try {
      await invoke("start_discovery");
    } catch (e) {
      console.error("Failed to start discovery:", e);
    }
    setTimeout(() => {
      loadDevices();
      setIsDiscovering(false);
    }, 2000);
  }

  async function handleManualConnect() {
    if (!manualIp.trim()) return;
    setConnectStatus("Connecting...");
    try {
      await invoke("connect_to_device", {
        address: manualIp.trim(),
        port: parseInt(manualPort) || 42424,
      });
      setConnectStatus("Connected!");
      setManualIp("");
      loadDevices();
      setTimeout(() => setConnectStatus(""), 3000);
    } catch (e) {
      setConnectStatus(`Failed: ${e}`);
      setTimeout(() => setConnectStatus(""), 5000);
    }
  }

  async function handleConnect(device: DiscoveredDevice) {
    if (device.addresses.length === 0) return;
    try {
      await invoke("connect_to_device", {
        address: device.addresses[0],
        port: device.port,
      });
    } catch (e) {
      console.error("Failed to connect:", e);
    }
  }

  useEffect(() => {
    loadDevices();
    loadDeviceInfo();
    const interval = setInterval(loadDevices, 5000);
    return () => clearInterval(interval);
  }, []);

  const filtered = devices.filter((d) =>
    d.name.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="max-w-5xl">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white mb-1">Devices</h1>
          <p className="text-dark-text-secondary text-sm">
            Discover and connect to nearby devices
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowManualConnect(!showManualConnect)}
            className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg text-sm text-dark-text hover:bg-dark-hover transition-colors"
          >
            <Plug size={16} />
            Connect by IP
          </button>
          <button
            onClick={handleStartDiscovery}
            disabled={isDiscovering}
            className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-accent-blue to-accent-purple rounded-lg text-sm font-medium text-white hover:opacity-90 transition-opacity disabled:opacity-50"
          >
            <Radar size={16} className={isDiscovering ? "animate-pulse" : ""} />
            {isDiscovering ? "Scanning..." : "Scan"}
          </button>
          <button
            onClick={loadDevices}
            className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg text-sm text-dark-text hover:bg-dark-hover transition-colors"
          >
            <RefreshCw size={16} />
          </button>
        </div>
      </div>

      {/* This device's IP info */}
      {localIp && (
        <div className="bg-accent-blue/5 border border-accent-blue/20 rounded-xl p-4 mb-4">
          <div className="flex items-center gap-3">
            <Wifi size={18} className="text-accent-blue" />
            <div>
              <p className="text-sm text-white font-medium">
                This device's IP: <span className="font-mono text-accent-blue">{localIp}:42424</span>
              </p>
              <p className="text-xs text-dark-text-secondary">
                Enter this IP on your other device to connect manually
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Manual connect panel */}
      {showManualConnect && (
        <div className="bg-dark-card border border-dark-border rounded-xl p-5 mb-6">
          <h3 className="text-sm font-semibold text-white mb-3">Connect by IP Address</h3>
          <p className="text-xs text-dark-text-secondary mb-4">
            If auto-discovery doesn't work (e.g., WiFi + Ethernet), enter the other device's IP address.
            You can find it in the Devices page of the other device.
          </p>
          <div className="flex gap-3">
            <input
              type="text"
              placeholder="IP address (e.g., 192.168.1.100)"
              value={manualIp}
              onChange={(e) => setManualIp(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleManualConnect()}
              className="flex-1 bg-dark-bg border border-dark-border rounded-lg px-4 py-2.5 text-sm text-dark-text placeholder-dark-text-secondary focus:outline-none focus:border-accent-blue/50 transition-colors font-mono"
            />
            <input
              type="text"
              placeholder="Port"
              value={manualPort}
              onChange={(e) => setManualPort(e.target.value)}
              className="w-24 bg-dark-bg border border-dark-border rounded-lg px-4 py-2.5 text-sm text-dark-text placeholder-dark-text-secondary focus:outline-none focus:border-accent-blue/50 transition-colors font-mono"
            />
            <button
              onClick={handleManualConnect}
              className="px-6 py-2.5 bg-gradient-to-r from-accent-blue to-accent-purple rounded-lg text-sm font-medium text-white hover:opacity-90 transition-opacity"
            >
              Connect
            </button>
          </div>
          {connectStatus && (
            <p className={`text-sm mt-3 ${connectStatus.startsWith("Failed") ? "text-red-400" : connectStatus === "Connected!" ? "text-green-400" : "text-yellow-400"}`}>
              {connectStatus}
            </p>
          )}
        </div>
      )}

      {/* Search */}
      <div className="relative mb-6">
        <Search
          size={16}
          className="absolute left-3 top-1/2 -translate-y-1/2 text-dark-text-secondary"
        />
        <input
          type="text"
          placeholder="Search devices..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full bg-dark-card border border-dark-border rounded-lg pl-10 pr-4 py-2.5 text-sm text-dark-text placeholder-dark-text-secondary focus:outline-none focus:border-accent-blue/50 transition-colors"
        />
      </div>

      {/* Device grid */}
      {filtered.length > 0 ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {filtered.map((device) => (
            <DeviceCard
              key={device.id}
              device={device}
              onConnect={handleConnect}
            />
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <div className="w-16 h-16 rounded-2xl bg-dark-card border border-dark-border flex items-center justify-center mb-4">
            <Radar size={28} className="text-dark-text-secondary" />
          </div>
          <h3 className="text-lg font-medium text-dark-text mb-1">No devices found</h3>
          <p className="text-sm text-dark-text-secondary max-w-sm mb-4">
            Make sure other Stark-Link devices are running on the same network, then click Scan.
          </p>
          <button
            onClick={() => setShowManualConnect(true)}
            className="text-sm text-accent-blue hover:underline"
          >
            Or connect manually by IP address →
          </button>
        </div>
      )}
    </div>
  );
}

export default Devices;
