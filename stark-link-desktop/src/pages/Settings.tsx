import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Save, Shield } from "lucide-react";
import type { DeviceInfo } from "../types";

function Settings() {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [deviceName, setDeviceName] = useState("");
  const [port, setPort] = useState("42424");
  const [autoDiscovery, setAutoDiscovery] = useState(true);
  const [clipboardSync, setClipboardSync] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const info = await invoke<DeviceInfo>("get_device_info");
        setDeviceInfo(info);
        setDeviceName(info.name);
      } catch (e) {
        console.error("Failed to get device info:", e);
      }
    }
    load();
  }, []);

  return (
    <div className="max-w-2xl">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-white mb-1">Settings</h1>
        <p className="text-dark-text-secondary text-sm">
          Configure your Stark-Link device
        </p>
      </div>

      {/* Device Settings */}
      <section className="mb-8">
        <h2 className="text-sm font-semibold text-dark-text-secondary uppercase tracking-wider mb-4">
          Device
        </h2>
        <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-5">
          <div>
            <label className="block text-sm font-medium text-dark-text mb-1.5">
              Device Name
            </label>
            <input
              type="text"
              value={deviceName}
              onChange={(e) => setDeviceName(e.target.value)}
              className="w-full bg-dark-surface border border-dark-border rounded-lg px-3 py-2.5 text-sm text-dark-text focus:outline-none focus:border-accent-blue/50 transition-colors"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-dark-text mb-1.5">
              Port
            </label>
            <input
              type="number"
              value={port}
              onChange={(e) => setPort(e.target.value)}
              className="w-full bg-dark-surface border border-dark-border rounded-lg px-3 py-2.5 text-sm text-dark-text focus:outline-none focus:border-accent-blue/50 transition-colors"
            />
            <p className="text-xs text-dark-text-secondary mt-1">
              WebSocket server port for incoming connections
            </p>
          </div>
          {deviceInfo && (
            <div>
              <label className="block text-sm font-medium text-dark-text mb-1.5">
                Device ID
              </label>
              <p className="text-sm text-dark-text-secondary font-mono bg-dark-surface border border-dark-border rounded-lg px-3 py-2.5 select-all">
                {deviceInfo.id}
              </p>
            </div>
          )}
          {deviceInfo && (
            <div>
              <label className="block text-sm font-medium text-dark-text mb-1.5">
                Fingerprint
              </label>
              <div className="flex items-center gap-2">
                <Shield size={14} className="text-accent-blue shrink-0" />
                <p className="text-sm text-dark-text-secondary font-mono bg-dark-surface border border-dark-border rounded-lg px-3 py-2.5 flex-1 select-all truncate">
                  {deviceInfo.fingerprint}
                </p>
              </div>
            </div>
          )}
        </div>
      </section>

      {/* Network Settings */}
      <section className="mb-8">
        <h2 className="text-sm font-semibold text-dark-text-secondary uppercase tracking-wider mb-4">
          Network
        </h2>
        <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-dark-text">Auto-Discovery</p>
              <p className="text-xs text-dark-text-secondary mt-0.5">
                Automatically discover devices on the local network via mDNS
              </p>
            </div>
            <button
              onClick={() => setAutoDiscovery(!autoDiscovery)}
              className={`relative w-11 h-6 rounded-full transition-colors ${
                autoDiscovery ? "bg-accent-blue" : "bg-dark-border"
              }`}
            >
              <span
                className={`absolute top-0.5 w-5 h-5 rounded-full bg-white transition-transform ${
                  autoDiscovery ? "left-[22px]" : "left-0.5"
                }`}
              />
            </button>
          </div>
        </div>
      </section>

      {/* Sync Settings */}
      <section className="mb-8">
        <h2 className="text-sm font-semibold text-dark-text-secondary uppercase tracking-wider mb-4">
          Sync
        </h2>
        <div className="bg-dark-card border border-dark-border rounded-xl p-5 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-dark-text">Clipboard Sync</p>
              <p className="text-xs text-dark-text-secondary mt-0.5">
                Synchronize clipboard content with paired devices
              </p>
            </div>
            <button
              onClick={() => setClipboardSync(!clipboardSync)}
              className={`relative w-11 h-6 rounded-full transition-colors ${
                clipboardSync ? "bg-accent-blue" : "bg-dark-border"
              }`}
            >
              <span
                className={`absolute top-0.5 w-5 h-5 rounded-full bg-white transition-transform ${
                  clipboardSync ? "left-[22px]" : "left-0.5"
                }`}
              />
            </button>
          </div>
        </div>
      </section>

      {/* Theme Settings */}
      <section className="mb-8">
        <h2 className="text-sm font-semibold text-dark-text-secondary uppercase tracking-wider mb-4">
          Appearance
        </h2>
        <div className="bg-dark-card border border-dark-border rounded-xl p-5">
          <div>
            <label className="block text-sm font-medium text-dark-text mb-3">Theme</label>
            <div className="flex gap-3">
              <button className="px-4 py-2 bg-gradient-to-r from-accent-blue/20 to-accent-purple/10 border border-accent-blue/30 rounded-lg text-sm text-white font-medium">
                Dark
              </button>
              <button className="px-4 py-2 bg-dark-surface border border-dark-border rounded-lg text-sm text-dark-text-secondary font-medium hover:bg-dark-hover transition-colors">
                Light
              </button>
              <button className="px-4 py-2 bg-dark-surface border border-dark-border rounded-lg text-sm text-dark-text-secondary font-medium hover:bg-dark-hover transition-colors">
                System
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* Save button */}
      <div className="flex justify-end">
        <button className="flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-accent-blue to-accent-purple rounded-lg text-sm font-medium text-white hover:opacity-90 transition-opacity">
          <Save size={16} />
          Save Settings
        </button>
      </div>
    </div>
  );
}

export default Settings;
