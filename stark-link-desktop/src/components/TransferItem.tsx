import {
  ArrowUp,
  ArrowDown,
  Pause,
  Play,
  X,
  CheckCircle2,
  AlertCircle,
  FileIcon,
} from "lucide-react";
import type { TransferInfo } from "../types";

interface TransferItemProps {
  transfer: TransferInfo;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

function formatEta(seconds: number): string {
  if (!isFinite(seconds) || seconds <= 0) return "--";
  if (seconds < 60) return `${Math.ceil(seconds)}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${Math.ceil(seconds % 60)}s`;
  return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
}

function getStateInfo(state: string) {
  switch (state) {
    case "InProgress":
      return { label: "Transferring", color: "text-accent-blue" };
    case "Paused":
      return { label: "Paused", color: "text-status-warning" };
    case "Completed":
      return { label: "Completed", color: "text-status-online" };
    case "Failed":
      return { label: "Failed", color: "text-status-error" };
    case "Cancelled":
      return { label: "Cancelled", color: "text-status-offline" };
    case "Pending":
      return { label: "Pending", color: "text-dark-text-secondary" };
    default:
      return { label: state, color: "text-dark-text-secondary" };
  }
}

function TransferItem({ transfer, onPause, onResume, onCancel }: TransferItemProps) {
  const stateInfo = getStateInfo(transfer.state);
  const isActive = transfer.state === "InProgress" || transfer.state === "Paused";
  const progressPercent = Math.round(transfer.progress * 100);

  return (
    <div className="bg-dark-card border border-dark-border rounded-xl p-4 transition-all duration-200">
      <div className="flex items-center gap-4">
        {/* Icon */}
        <div className="w-10 h-10 rounded-lg bg-dark-surface flex items-center justify-center shrink-0">
          {transfer.state === "Completed" ? (
            <CheckCircle2 size={20} className="text-status-online" />
          ) : transfer.state === "Failed" ? (
            <AlertCircle size={20} className="text-status-error" />
          ) : (
            <FileIcon size={20} className="text-accent-blue" />
          )}
        </div>

        {/* Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-sm font-medium text-dark-text truncate">
              {transfer.file_name}
            </span>
            {transfer.direction === "Outgoing" ? (
              <ArrowUp size={14} className="text-accent-blue shrink-0" />
            ) : (
              <ArrowDown size={14} className="text-accent-purple shrink-0" />
            )}
          </div>

          <div className="flex items-center gap-3 text-xs text-dark-text-secondary">
            <span className={stateInfo.color}>{stateInfo.label}</span>
            <span>
              {formatBytes(transfer.bytes_transferred)} / {formatBytes(transfer.file_size)}
            </span>
            {transfer.state === "InProgress" && (
              <>
                <span>{formatBytes(transfer.speed_bps)}/s</span>
                <span>ETA {formatEta(transfer.eta_secs)}</span>
              </>
            )}
          </div>

          {/* Progress bar */}
          {isActive && (
            <div className="mt-2 h-1.5 bg-dark-surface rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-accent-blue to-accent-purple rounded-full transition-all duration-300"
                style={{ width: `${progressPercent}%` }}
              />
            </div>
          )}
        </div>

        {/* Controls */}
        {isActive && (
          <div className="flex items-center gap-1.5 shrink-0">
            {transfer.state === "InProgress" ? (
              <button
                onClick={() => onPause(transfer.id)}
                className="w-8 h-8 rounded-lg bg-dark-surface hover:bg-dark-hover flex items-center justify-center transition-colors"
                title="Pause"
              >
                <Pause size={14} className="text-dark-text-secondary" />
              </button>
            ) : (
              <button
                onClick={() => onResume(transfer.id)}
                className="w-8 h-8 rounded-lg bg-dark-surface hover:bg-dark-hover flex items-center justify-center transition-colors"
                title="Resume"
              >
                <Play size={14} className="text-dark-text-secondary" />
              </button>
            )}
            <button
              onClick={() => onCancel(transfer.id)}
              className="w-8 h-8 rounded-lg bg-dark-surface hover:bg-status-error/20 flex items-center justify-center transition-colors"
              title="Cancel"
            >
              <X size={14} className="text-dark-text-secondary hover:text-status-error" />
            </button>
          </div>
        )}

        {/* Progress label */}
        <span className="text-sm font-medium text-dark-text-secondary w-12 text-right shrink-0">
          {progressPercent}%
        </span>
      </div>
    </div>
  );
}

export default TransferItem;
