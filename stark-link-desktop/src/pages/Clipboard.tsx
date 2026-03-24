import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Search, ClipboardList, Copy, Check, RefreshCw, Send, ClipboardPaste } from "lucide-react";
import type { ClipboardEntry } from "../types";

function Clipboard() {
  const [entries, setEntries] = useState<ClipboardEntry[]>([]);
  const [search, setSearch] = useState("");
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [syncStatus, setSyncStatus] = useState("");
  const [manualText, setManualText] = useState("");

  async function loadClipboard() {
    try {
      const clips = await invoke<ClipboardEntry[]>("get_clipboard_history");
      setEntries(clips);
    } catch (e) {
      console.error("Failed to get clipboard:", e);
    }
  }

  function handleCopy(entry: ClipboardEntry) {
    if (entry.text) {
      navigator.clipboard.writeText(entry.text).then(() => {
        setCopiedId(entry.id);
        setTimeout(() => setCopiedId(null), 2000);
      });
    }
  }

  async function handleReadAndSync() {
    try {
      const text = await navigator.clipboard.readText();
      if (!text) {
        setSyncStatus("Clipboard is empty");
        setTimeout(() => setSyncStatus(""), 3000);
        return;
      }
      await invoke("add_clipboard_entry", { text });
      setSyncStatus("Clipboard synced!");
      loadClipboard();
      setTimeout(() => setSyncStatus(""), 3000);
    } catch (e) {
      setSyncStatus(`Failed: ${e}`);
      setTimeout(() => setSyncStatus(""), 4000);
    }
  }

  async function handleManualSync() {
    if (!manualText.trim()) return;
    try {
      await invoke("add_clipboard_entry", { text: manualText.trim() });
      setSyncStatus("Added to clipboard history!");
      setManualText("");
      loadClipboard();
      setTimeout(() => setSyncStatus(""), 3000);
    } catch (e) {
      setSyncStatus(`Failed: ${e}`);
      setTimeout(() => setSyncStatus(""), 4000);
    }
  }

  useEffect(() => {
    loadClipboard();
    const interval = setInterval(loadClipboard, 3000);
    return () => clearInterval(interval);
  }, []);

  const filtered = entries.filter((e) => {
    if (!search) return true;
    const text = e.text || "";
    return text.toLowerCase().includes(search.toLowerCase());
  });

  function formatTimestamp(ts: string): string {
    try {
      const date = new Date(ts);
      return date.toLocaleString();
    } catch {
      return ts;
    }
  }

  return (
    <div className="max-w-4xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold text-white mb-1">Clipboard</h1>
          <p className="text-dark-text-secondary text-sm">
            Clipboard history synced across devices
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleReadAndSync}
            className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-accent-blue to-accent-purple rounded-xl text-sm font-semibold text-white hover:opacity-90 transition-opacity"
          >
            <Send size={14} />
            Sync Clipboard
          </button>
          <button
            onClick={loadClipboard}
            className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-xl text-sm text-dark-text hover:bg-dark-hover transition-colors"
          >
            <RefreshCw size={16} />
          </button>
        </div>
      </div>

      {syncStatus && (
        <div
          className={`mb-4 px-4 py-2 rounded-xl text-sm ${
            syncStatus.startsWith("Failed")
              ? "bg-status-error/10 text-status-error border border-status-error/20"
              : "bg-status-online/10 text-status-online border border-status-online/20"
          }`}
        >
          {syncStatus}
        </div>
      )}

      {/* Manual text input */}
      <div className="bg-dark-card border border-dark-border rounded-2xl p-4 mb-6">
        <div className="flex gap-3">
          <div className="w-10 h-10 rounded-xl bg-accent-purple/10 flex items-center justify-center shrink-0">
            <ClipboardPaste size={18} className="text-accent-purple" />
          </div>
          <input
            type="text"
            placeholder="Type or paste text to add to clipboard history..."
            value={manualText}
            onChange={(e) => setManualText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleManualSync()}
            className="flex-1 bg-transparent text-sm text-dark-text placeholder-dark-text-secondary focus:outline-none"
          />
          <button
            onClick={handleManualSync}
            disabled={!manualText.trim()}
            className="px-4 py-1.5 bg-accent-purple/20 text-accent-purple rounded-lg text-sm font-medium hover:bg-accent-purple/30 transition-colors disabled:opacity-30"
          >
            Add
          </button>
        </div>
      </div>

      {/* Search */}
      <div className="relative mb-6">
        <Search
          size={16}
          className="absolute left-4 top-1/2 -translate-y-1/2 text-dark-text-secondary"
        />
        <input
          type="text"
          placeholder="Search clipboard history..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full bg-dark-card border border-dark-border rounded-xl pl-11 pr-4 py-3 text-sm text-dark-text placeholder-dark-text-secondary focus:outline-none focus:border-accent-blue/30 transition-colors"
        />
      </div>

      {/* Clipboard list */}
      {filtered.length > 0 ? (
        <div className="space-y-2">
          {filtered.map((entry) => (
            <div
              key={entry.id}
              className="bg-dark-card border border-dark-border rounded-xl p-4 hover:border-accent-blue/20 transition-all group cursor-pointer"
              onClick={() => handleCopy(entry)}
            >
              <div className="flex items-start gap-3">
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-dark-text leading-relaxed line-clamp-3">
                    {entry.text || `[${entry.content_type}]`}
                  </p>
                  <div className="flex items-center gap-3 mt-2">
                    <span className="text-[11px] text-dark-text-secondary">
                      {formatTimestamp(entry.timestamp)}
                    </span>
                    <span className="text-[11px] px-1.5 py-0.5 bg-dark-surface rounded text-dark-text-secondary">
                      {entry.content_type}
                    </span>
                  </div>
                </div>
                <button
                  className="w-8 h-8 rounded-lg bg-dark-surface flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                  title="Copy to clipboard"
                >
                  {copiedId === entry.id ? (
                    <Check size={14} className="text-status-online" />
                  ) : (
                    <Copy size={14} className="text-dark-text-secondary" />
                  )}
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <div className="w-16 h-16 rounded-2xl bg-dark-card border border-dark-border flex items-center justify-center mb-4">
            <ClipboardList size={28} className="text-dark-text-secondary" />
          </div>
          <h3 className="text-lg font-medium text-dark-text mb-1">
            {search ? "No matching items" : "Clipboard is empty"}
          </h3>
          <p className="text-sm text-dark-text-secondary max-w-sm">
            {search
              ? "Try a different search term."
              : 'Click "Sync Clipboard" to capture your current clipboard, or type text above.'}
          </p>
        </div>
      )}
    </div>
  );
}

export default Clipboard;
