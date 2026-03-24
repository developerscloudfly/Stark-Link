import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Mouse,
  Keyboard,
  Terminal,
  Play,
  Pause,
  SkipForward,
  SkipBack,
  Volume2,
  VolumeX,
  ChevronUp,
  Lock,
  AppWindow,
  Send,
  Gamepad2,
} from "lucide-react";

interface ConnectedPeer {
  id: string;
  state: string;
  address: string;
}

function RemoteControl() {
  const [connectedPeers, setConnectedPeers] = useState<ConnectedPeer[]>([]);
  const [selectedPeer, setSelectedPeer] = useState<string>("");
  const [status, setStatus] = useState("");
  const [commandInput, setCommandInput] = useState("");
  const [appInput, setAppInput] = useState("");
  const [activeTab, setActiveTab] = useState<"touchpad" | "media" | "command" | "apps">("touchpad");

  async function loadPeers() {
    try {
      const peers = await invoke<ConnectedPeer[]>("get_connected_peers");
      setConnectedPeers(peers);
      const paired = peers.filter((p) => p.state === "Paired" || p.state === "Controlling");
      if (paired.length > 0 && !selectedPeer) {
        setSelectedPeer(paired[0].id);
      }
    } catch (e) {
      console.error("Failed to get peers:", e);
    }
  }

  useEffect(() => {
    loadPeers();
    const interval = setInterval(loadPeers, 3000);
    return () => clearInterval(interval);
  }, []);

  const pairedPeers = connectedPeers.filter(
    (p) => p.state === "Paired" || p.state === "Controlling"
  );

  function showStatus(msg: string, isError = false) {
    setStatus(isError ? `Error: ${msg}` : msg);
    setTimeout(() => setStatus(""), 3000);
  }

  async function sendMouse(action: string, x = 0, y = 0, button = "left") {
    if (!selectedPeer) return showStatus("Select a device first", true);
    try {
      await invoke("send_mouse_event", {
        peerId: selectedPeer,
        x,
        y,
        button,
        action,
      });
      showStatus(`Mouse ${action} sent`);
    } catch (e) {
      showStatus(`${e}`, true);
    }
  }

  async function sendKey(key: string, modifiers: string[] = []) {
    if (!selectedPeer) return showStatus("Select a device first", true);
    try {
      await invoke("send_keyboard_event", {
        peerId: selectedPeer,
        key,
        action: "down",
        modifiers,
      });
      showStatus(`Key "${key}" sent`);
    } catch (e) {
      showStatus(`${e}`, true);
    }
  }

  async function sendMediaAction(action: string) {
    if (!selectedPeer) return showStatus("Select a device first", true);
    try {
      await invoke("send_media_control", {
        peerId: selectedPeer,
        action,
      });
      showStatus(`Media: ${action}`);
    } catch (e) {
      showStatus(`${e}`, true);
    }
  }

  async function handleSendCommand() {
    if (!selectedPeer) return showStatus("Select a device first", true);
    if (!commandInput.trim()) return;
    try {
      await invoke("send_remote_command", {
        peerId: selectedPeer,
        command: commandInput.trim(),
        timeoutSecs: 30,
      });
      showStatus(`Command sent: ${commandInput.trim()}`);
      setCommandInput("");
    } catch (e) {
      showStatus(`${e}`, true);
    }
  }

  async function handleLockDevice() {
    if (!selectedPeer) return showStatus("Select a device first", true);
    try {
      await invoke("lock_remote_device", { peerId: selectedPeer });
      showStatus("Lock command sent");
    } catch (e) {
      showStatus(`${e}`, true);
    }
  }

  async function handleLaunchApp() {
    if (!selectedPeer) return showStatus("Select a device first", true);
    if (!appInput.trim()) return;
    try {
      await invoke("launch_remote_app", {
        peerId: selectedPeer,
        appId: appInput.trim(),
        args: [],
      });
      showStatus(`Launch: ${appInput.trim()}`);
      setAppInput("");
    } catch (e) {
      showStatus(`${e}`, true);
    }
  }

  return (
    <div className="max-w-4xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold text-white mb-1">Remote Control</h1>
          <p className="text-dark-text-secondary text-sm">
            Control connected devices remotely
          </p>
        </div>
      </div>

      {/* Device selector */}
      {pairedPeers.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <div className="w-16 h-16 rounded-2xl bg-dark-card border border-dark-border flex items-center justify-center mb-4">
            <Gamepad2 size={28} className="text-dark-text-secondary" />
          </div>
          <h3 className="text-lg font-medium text-dark-text mb-1">No devices connected</h3>
          <p className="text-sm text-dark-text-secondary max-w-sm">
            Go to the Devices page and connect to a device first.
          </p>
        </div>
      ) : (
        <>
          <div className="bg-dark-card border border-dark-border rounded-2xl p-4 mb-6">
            <label className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-2 block">
              Controlling Device
            </label>
            <div className="flex gap-2">
              {pairedPeers.map((peer) => (
                <button
                  key={peer.id}
                  onClick={() => setSelectedPeer(peer.id)}
                  className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-all ${
                    selectedPeer === peer.id
                      ? "bg-gradient-to-r from-accent-blue to-accent-purple text-white"
                      : "bg-dark-surface border border-dark-border text-dark-text-secondary hover:text-dark-text hover:bg-dark-hover"
                  }`}
                >
                  <span className="w-2 h-2 rounded-full bg-status-online"></span>
                  <span className="font-mono">{peer.id.slice(0, 8)}...</span>
                  <span className="text-xs opacity-70">{peer.address}</span>
                </button>
              ))}
            </div>
          </div>

          {status && (
            <div
              className={`mb-4 px-4 py-2 rounded-xl text-sm ${
                status.startsWith("Error")
                  ? "bg-status-error/10 text-status-error border border-status-error/20"
                  : "bg-status-online/10 text-status-online border border-status-online/20"
              }`}
            >
              {status}
            </div>
          )}

          {/* Tab bar */}
          <div className="flex gap-1 bg-dark-card border border-dark-border rounded-xl p-1 mb-6">
            {[
              { id: "touchpad" as const, icon: Mouse, label: "Touchpad" },
              { id: "media" as const, icon: Play, label: "Media" },
              { id: "command" as const, icon: Terminal, label: "Terminal" },
              { id: "apps" as const, icon: AppWindow, label: "Apps" },
            ].map(({ id, icon: Icon, label }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id)}
                className={`flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium transition-all ${
                  activeTab === id
                    ? "bg-gradient-to-r from-accent-blue/20 to-accent-purple/10 text-white border border-accent-blue/30"
                    : "text-dark-text-secondary hover:text-dark-text hover:bg-dark-hover"
                }`}
              >
                <Icon size={16} />
                {label}
              </button>
            ))}
          </div>

          {/* Touchpad tab */}
          {activeTab === "touchpad" && (
            <div className="space-y-4">
              {/* Virtual touchpad */}
              <div className="bg-dark-card border border-dark-border rounded-2xl p-6">
                <h3 className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-4">
                  Virtual Touchpad
                </h3>
                <div
                  className="w-full h-48 bg-dark-surface border border-dark-border rounded-xl flex items-center justify-center cursor-crosshair relative select-none"
                  onMouseMove={(e) => {
                    const rect = e.currentTarget.getBoundingClientRect();
                    const x = ((e.clientX - rect.left) / rect.width) * 1920;
                    const y = ((e.clientY - rect.top) / rect.height) * 1080;
                    sendMouse("move", x, y);
                  }}
                  onClick={(e) => {
                    const rect = e.currentTarget.getBoundingClientRect();
                    const x = ((e.clientX - rect.left) / rect.width) * 1920;
                    const y = ((e.clientY - rect.top) / rect.height) * 1080;
                    sendMouse("click", x, y);
                  }}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    const rect = e.currentTarget.getBoundingClientRect();
                    const x = ((e.clientX - rect.left) / rect.width) * 1920;
                    const y = ((e.clientY - rect.top) / rect.height) * 1080;
                    sendMouse("click", x, y, "right");
                  }}
                >
                  <div className="text-dark-text-secondary text-sm pointer-events-none">
                    <Mouse size={24} className="mx-auto mb-2 opacity-30" />
                    Move mouse here &middot; Click to left-click &middot; Right-click for right-click
                  </div>
                </div>
                <div className="flex gap-3 mt-4">
                  <button
                    onClick={() => sendMouse("click", 960, 540, "left")}
                    className="flex-1 py-3 bg-dark-surface border border-dark-border rounded-xl text-sm text-dark-text hover:bg-dark-hover transition-colors"
                  >
                    Left Click
                  </button>
                  <button
                    onClick={() => sendMouse("click", 960, 540, "right")}
                    className="flex-1 py-3 bg-dark-surface border border-dark-border rounded-xl text-sm text-dark-text hover:bg-dark-hover transition-colors"
                  >
                    Right Click
                  </button>
                  <button
                    onClick={() => sendMouse("doubleclick", 960, 540, "left")}
                    className="flex-1 py-3 bg-dark-surface border border-dark-border rounded-xl text-sm text-dark-text hover:bg-dark-hover transition-colors"
                  >
                    Double Click
                  </button>
                </div>
              </div>

              {/* Keyboard shortcuts */}
              <div className="bg-dark-card border border-dark-border rounded-2xl p-6">
                <h3 className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-4">
                  <Keyboard size={14} className="inline mr-2" />
                  Quick Keys
                </h3>
                <div className="grid grid-cols-4 gap-2">
                  {[
                    { label: "Enter", key: "Return" },
                    { label: "Esc", key: "Escape" },
                    { label: "Tab", key: "Tab" },
                    { label: "Space", key: "space" },
                    { label: "Backspace", key: "BackSpace" },
                    { label: "Delete", key: "Delete" },
                    { label: "Home", key: "Home" },
                    { label: "End", key: "End" },
                    { label: "Up", key: "Up" },
                    { label: "Down", key: "Down" },
                    { label: "Left", key: "Left" },
                    { label: "Right", key: "Right" },
                  ].map(({ label, key }) => (
                    <button
                      key={key}
                      onClick={() => sendKey(key)}
                      className="py-2.5 bg-dark-surface border border-dark-border rounded-lg text-sm text-dark-text hover:bg-dark-hover hover:border-accent-blue/30 transition-all font-mono"
                    >
                      {label}
                    </button>
                  ))}
                </div>

                <h4 className="text-xs text-dark-text-secondary mt-4 mb-2">Combos</h4>
                <div className="grid grid-cols-3 gap-2">
                  {[
                    { label: "Ctrl+C", key: "c", mods: ["ctrl"] },
                    { label: "Ctrl+V", key: "v", mods: ["ctrl"] },
                    { label: "Ctrl+Z", key: "z", mods: ["ctrl"] },
                    { label: "Ctrl+A", key: "a", mods: ["ctrl"] },
                    { label: "Ctrl+S", key: "s", mods: ["ctrl"] },
                    { label: "Alt+Tab", key: "Tab", mods: ["alt"] },
                    { label: "Alt+F4", key: "F4", mods: ["alt"] },
                    { label: "Win+D", key: "d", mods: ["meta"] },
                    { label: "Win+L", key: "l", mods: ["meta"] },
                  ].map(({ label, key, mods }) => (
                    <button
                      key={label}
                      onClick={() => sendKey(key, mods)}
                      className="py-2.5 bg-dark-surface border border-dark-border rounded-lg text-sm text-dark-text hover:bg-dark-hover hover:border-accent-purple/30 transition-all font-mono"
                    >
                      {label}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Media tab */}
          {activeTab === "media" && (
            <div className="bg-dark-card border border-dark-border rounded-2xl p-6">
              <h3 className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-6">
                Media Controls
              </h3>
              <div className="flex flex-col items-center gap-6">
                {/* Playback */}
                <div className="flex items-center gap-4">
                  <button
                    onClick={() => sendMediaAction("previous")}
                    className="w-14 h-14 rounded-2xl bg-dark-surface border border-dark-border flex items-center justify-center hover:bg-dark-hover hover:border-accent-blue/30 transition-all"
                  >
                    <SkipBack size={22} className="text-dark-text" />
                  </button>
                  <button
                    onClick={() => sendMediaAction("play")}
                    className="w-16 h-16 rounded-2xl bg-gradient-to-r from-accent-blue to-accent-purple flex items-center justify-center hover:opacity-90 transition-opacity shadow-lg shadow-accent-blue/20"
                  >
                    <Play size={28} className="text-white ml-1" />
                  </button>
                  <button
                    onClick={() => sendMediaAction("pause")}
                    className="w-14 h-14 rounded-2xl bg-dark-surface border border-dark-border flex items-center justify-center hover:bg-dark-hover hover:border-accent-blue/30 transition-all"
                  >
                    <Pause size={22} className="text-dark-text" />
                  </button>
                  <button
                    onClick={() => sendMediaAction("next")}
                    className="w-14 h-14 rounded-2xl bg-dark-surface border border-dark-border flex items-center justify-center hover:bg-dark-hover hover:border-accent-blue/30 transition-all"
                  >
                    <SkipForward size={22} className="text-dark-text" />
                  </button>
                </div>

                {/* Volume */}
                <div className="flex items-center gap-3">
                  <button
                    onClick={() => sendMediaAction("mute")}
                    className="w-12 h-12 rounded-xl bg-dark-surface border border-dark-border flex items-center justify-center hover:bg-dark-hover transition-colors"
                  >
                    <VolumeX size={18} className="text-dark-text" />
                  </button>
                  <button
                    onClick={() => sendMediaAction("volume_down")}
                    className="w-12 h-12 rounded-xl bg-dark-surface border border-dark-border flex items-center justify-center hover:bg-dark-hover transition-colors text-xl text-dark-text font-bold"
                  >
                    &minus;
                  </button>
                  <div className="w-20 text-center">
                    <Volume2 size={20} className="mx-auto text-accent-blue" />
                    <span className="text-xs text-dark-text-secondary mt-1 block">Volume</span>
                  </div>
                  <button
                    onClick={() => sendMediaAction("volume_up")}
                    className="w-12 h-12 rounded-xl bg-dark-surface border border-dark-border flex items-center justify-center hover:bg-dark-hover transition-colors text-xl text-dark-text font-bold"
                  >
                    +
                  </button>
                </div>

                {/* Lock device */}
                <button
                  onClick={handleLockDevice}
                  className="flex items-center gap-2 px-6 py-3 bg-status-error/10 border border-status-error/20 rounded-xl text-sm text-status-error hover:bg-status-error/20 transition-colors"
                >
                  <Lock size={16} />
                  Lock Remote Device
                </button>
              </div>
            </div>
          )}

          {/* Command tab */}
          {activeTab === "command" && (
            <div className="bg-dark-card border border-dark-border rounded-2xl p-6">
              <h3 className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-4">
                <Terminal size={14} className="inline mr-2" />
                Remote Command
              </h3>
              <p className="text-xs text-dark-text-secondary mb-4">
                Execute a shell command on the connected device.
              </p>
              <div className="flex gap-3">
                <input
                  type="text"
                  placeholder="Enter command (e.g., dir, ls, ipconfig)..."
                  value={commandInput}
                  onChange={(e) => setCommandInput(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleSendCommand()}
                  className="flex-1 bg-dark-surface border border-dark-border rounded-xl px-4 py-3 text-sm text-dark-text placeholder-dark-text-secondary focus:outline-none focus:border-accent-blue/30 transition-colors font-mono"
                />
                <button
                  onClick={handleSendCommand}
                  disabled={!commandInput.trim()}
                  className="flex items-center gap-2 px-5 py-3 bg-gradient-to-r from-accent-blue to-accent-purple rounded-xl text-sm font-semibold text-white hover:opacity-90 transition-opacity disabled:opacity-30"
                >
                  <Send size={14} />
                  Run
                </button>
              </div>

              <div className="mt-6 space-y-2">
                <h4 className="text-xs text-dark-text-secondary mb-2">Quick Commands</h4>
                <div className="grid grid-cols-2 gap-2">
                  {[
                    { label: "System Info", cmd: "systeminfo" },
                    { label: "IP Config", cmd: "ipconfig" },
                    { label: "Task List", cmd: "tasklist" },
                    { label: "Disk Usage", cmd: "wmic logicaldisk get size,freespace,caption" },
                    { label: "Shutdown", cmd: "shutdown /s /t 60" },
                    { label: "Cancel Shutdown", cmd: "shutdown /a" },
                  ].map(({ label, cmd }) => (
                    <button
                      key={cmd}
                      onClick={() => {
                        setCommandInput(cmd);
                      }}
                      className="py-2.5 px-4 bg-dark-surface border border-dark-border rounded-lg text-sm text-dark-text hover:bg-dark-hover hover:border-accent-blue/30 transition-all text-left"
                    >
                      <span className="text-dark-text">{label}</span>
                      <span className="block text-xs text-dark-text-secondary font-mono mt-0.5 truncate">
                        {cmd}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Apps tab */}
          {activeTab === "apps" && (
            <div className="bg-dark-card border border-dark-border rounded-2xl p-6">
              <h3 className="text-xs font-semibold text-dark-text-secondary uppercase tracking-widest mb-4">
                <AppWindow size={14} className="inline mr-2" />
                Launch Application
              </h3>
              <p className="text-xs text-dark-text-secondary mb-4">
                Launch an application on the connected device by name or path.
              </p>
              <div className="flex gap-3">
                <input
                  type="text"
                  placeholder="App name or path (e.g., notepad, calc, explorer)..."
                  value={appInput}
                  onChange={(e) => setAppInput(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleLaunchApp()}
                  className="flex-1 bg-dark-surface border border-dark-border rounded-xl px-4 py-3 text-sm text-dark-text placeholder-dark-text-secondary focus:outline-none focus:border-accent-blue/30 transition-colors font-mono"
                />
                <button
                  onClick={handleLaunchApp}
                  disabled={!appInput.trim()}
                  className="flex items-center gap-2 px-5 py-3 bg-gradient-to-r from-accent-blue to-accent-purple rounded-xl text-sm font-semibold text-white hover:opacity-90 transition-opacity disabled:opacity-30"
                >
                  <ChevronUp size={14} />
                  Launch
                </button>
              </div>

              <div className="mt-6">
                <h4 className="text-xs text-dark-text-secondary mb-2">Common Apps</h4>
                <div className="grid grid-cols-3 gap-2">
                  {[
                    "notepad",
                    "calc",
                    "explorer",
                    "mspaint",
                    "cmd",
                    "powershell",
                    "taskmgr",
                    "control",
                    "msedge",
                  ].map((app) => (
                    <button
                      key={app}
                      onClick={() => setAppInput(app)}
                      className="py-2.5 bg-dark-surface border border-dark-border rounded-lg text-sm text-dark-text hover:bg-dark-hover hover:border-accent-blue/30 transition-all font-mono capitalize"
                    >
                      {app}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default RemoteControl;
