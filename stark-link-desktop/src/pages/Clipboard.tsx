import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Search, ClipboardList, Copy, Check, RefreshCw, Trash2 } from "lucide-react";
import type { ClipboardEntry } from "../types";

function Clipboard() {
  const [entries, setEntries] = useState<ClipboardEntry[]>([]);
  const [search, setSearch] = useState("");
  const [copiedId, setCopiedId] = useState<string | null>(null);

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

  useEffect(() => {
    loadClipboard();
    const interval = setInterval(loadClipboard, 5000);
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
    <div className="max-w-4xl">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white mb-1">Clipboard</h1>
          <p className="text-dark-text-secondary text-sm">
            Clipboard history synced across devices
          </p>
        </div>
        <button
          onClick={loadClipboard}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg text-sm text-dark-text hover:bg-dark-hover transition-colors"
        >
          <RefreshCw size={16} />
        </button>
      </div>

      {/* Search */}
      <div className="relative mb-6">
        <Search
          size={16}
          className="absolute left-3 top-1/2 -translate-y-1/2 text-dark-text-secondary"
        />
        <input
          type="text"
          placeholder="Search clipboard history..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full bg-dark-card border border-dark-border rounded-lg pl-10 pr-4 py-2.5 text-sm text-dark-text placeholder-dark-text-secondary focus:outline-none focus:border-accent-blue/50 transition-colors"
        />
      </div>

      {/* Clipboard list */}
      {filtered.length > 0 ? (
        <div className="space-y-2">
          {filtered.map((entry) => (
            <div
              key={entry.id}
              className="bg-dark-card border border-dark-border rounded-xl p-4 hover:border-accent-blue/30 transition-all duration-200 group cursor-pointer"
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
                    <span className="text-[11px] text-dark-text-secondary font-mono">
                      from {entry.source_device.slice(0, 8)}
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
              : "Copy something to your clipboard and it will appear here. Connect devices to sync."}
          </p>
        </div>
      )}
    </div>
  );
}

export default Clipboard;
