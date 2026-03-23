import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, ArrowUpDown } from "lucide-react";
import type { TransferInfo } from "../types";
import TransferItem from "../components/TransferItem";

function Transfers() {
  const [transfers, setTransfers] = useState<TransferInfo[]>([]);

  async function loadTransfers() {
    try {
      const txs = await invoke<TransferInfo[]>("get_transfers");
      setTransfers(txs);
    } catch (e) {
      console.error("Failed to get transfers:", e);
    }
  }

  async function handlePause(id: string) {
    try {
      await invoke("pause_transfer", { transferId: id });
      loadTransfers();
    } catch (e) {
      console.error("Failed to pause transfer:", e);
    }
  }

  async function handleResume(id: string) {
    try {
      await invoke("resume_transfer", { transferId: id });
      loadTransfers();
    } catch (e) {
      console.error("Failed to resume transfer:", e);
    }
  }

  async function handleCancel(id: string) {
    try {
      await invoke("cancel_transfer", { transferId: id });
      loadTransfers();
    } catch (e) {
      console.error("Failed to cancel transfer:", e);
    }
  }

  useEffect(() => {
    loadTransfers();
    const interval = setInterval(loadTransfers, 2000);
    return () => clearInterval(interval);
  }, []);

  const active = transfers.filter(
    (t) => t.state === "InProgress" || t.state === "Paused" || t.state === "Pending"
  );
  const completed = transfers.filter(
    (t) => t.state === "Completed" || t.state === "Failed" || t.state === "Cancelled"
  );

  return (
    <div className="max-w-4xl">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white mb-1">Transfers</h1>
          <p className="text-dark-text-secondary text-sm">
            Manage file transfers with connected devices
          </p>
        </div>
        <button
          onClick={loadTransfers}
          className="flex items-center gap-2 px-4 py-2 bg-dark-card border border-dark-border rounded-lg text-sm text-dark-text hover:bg-dark-hover transition-colors"
        >
          <RefreshCw size={16} />
        </button>
      </div>

      {/* Active transfers */}
      {active.length > 0 && (
        <div className="mb-8">
          <h3 className="text-sm font-semibold text-dark-text-secondary uppercase tracking-wider mb-3">
            Active ({active.length})
          </h3>
          <div className="space-y-3">
            {active.map((transfer) => (
              <TransferItem
                key={transfer.id}
                transfer={transfer}
                onPause={handlePause}
                onResume={handleResume}
                onCancel={handleCancel}
              />
            ))}
          </div>
        </div>
      )}

      {/* Completed transfers */}
      {completed.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-dark-text-secondary uppercase tracking-wider mb-3">
            History ({completed.length})
          </h3>
          <div className="space-y-3">
            {completed.map((transfer) => (
              <TransferItem
                key={transfer.id}
                transfer={transfer}
                onPause={handlePause}
                onResume={handleResume}
                onCancel={handleCancel}
              />
            ))}
          </div>
        </div>
      )}

      {/* Empty state */}
      {transfers.length === 0 && (
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <div className="w-16 h-16 rounded-2xl bg-dark-card border border-dark-border flex items-center justify-center mb-4">
            <ArrowUpDown size={28} className="text-dark-text-secondary" />
          </div>
          <h3 className="text-lg font-medium text-dark-text mb-1">No transfers yet</h3>
          <p className="text-sm text-dark-text-secondary max-w-sm">
            Send a file from the Dashboard or Devices page to start a transfer.
          </p>
        </div>
      )}
    </div>
  );
}

export default Transfers;
