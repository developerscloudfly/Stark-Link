import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Search, Radar } from "lucide-react";
import type { DiscoveredDevice } from "../types";
import DeviceCard from "../components/DeviceCard";

function Devices() {
  const [devices, setDevices] = useState<DiscoveredDevice[]>([]);
  const [search, setSearch] = useState("");
  const [isDiscovering, setIsDiscovering] = useState(false);

  async function loadDevices() {
    try {
      const devs = await invoke<DiscoveredDevice[]>("get_discovered_devices");
      setDevices(devs);
    } catch (e) {
      console.error("Failed to get devices:", e);
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
          <p className="text-sm text-dark-text-secondary max-w-sm">
            Make sure other Stark-Link devices are running on the same network, then click
            Scan to discover them.
          </p>
        </div>
      )}
    </div>
  );
}

export default Devices;
