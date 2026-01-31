import React, { useState, useEffect, useCallback } from "react";
import {
  XIcon,
  PlusIcon,
  RotateCcwIcon,
  TrashIcon,
  AlertTriangleIcon,
} from "lucide-react";
import type { BackupInfo } from "@/lib/types";
import {
  listBackups,
  createBackup,
  restoreBackup,
  deleteBackup,
} from "@/lib/tauri";

interface BackupManagerProps {
  isOpen: boolean;
  onClose: () => void;
}

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export const BackupManager: React.FC<BackupManagerProps> = ({ isOpen, onClose }) => {
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [confirmAction, setConfirmAction] = useState<{
    type: "restore" | "delete";
    id: string;
  } | null>(null);

  const loadBackups = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const list = await listBackups();
      setBackups(list);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      loadBackups();
    }
  }, [isOpen, loadBackups]);

  const handleCreate = useCallback(async () => {
    setError("");
    try {
      await createBackup();
      await loadBackups();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [loadBackups]);

  const handleRestore = useCallback(
    async (id: string) => {
      setError("");
      setConfirmAction(null);
      try {
        await restoreBackup(id);
        await loadBackups();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
    },
    [loadBackups]
  );

  const handleDelete = useCallback(
    async (id: string) => {
      setError("");
      setConfirmAction(null);
      try {
        await deleteBackup(id);
        await loadBackups();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
    },
    [loadBackups]
  );

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      role="dialog"
      aria-modal="true"
      aria-label="Backup Manager"
    >
      <div className="flex h-[480px] w-[520px] flex-col rounded-lg bg-white shadow-xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b px-6 py-4">
          <h2 className="text-lg font-semibold text-gray-900">Backup Manager</h2>
          <button
            onClick={onClose}
            className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
            aria-label="Close"
          >
            <XIcon size={18} />
          </button>
        </div>

        {/* Toolbar */}
        <div className="flex items-center border-b px-6 py-2">
          <button
            onClick={handleCreate}
            className="flex items-center gap-1.5 rounded bg-blue-600 px-3 py-1.5 text-sm text-white hover:bg-blue-700"
          >
            <PlusIcon size={14} />
            Create Backup
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-4">
          {loading && (
            <p className="text-center text-sm text-gray-500">Loading backups...</p>
          )}

          {!loading && backups.length === 0 && (
            <p className="text-center text-sm text-gray-500">No backups found.</p>
          )}

          {!loading && backups.length > 0 && (
            <div className="space-y-2">
              {backups.map((backup) => (
                <div
                  key={backup.id}
                  className="flex items-center justify-between rounded border p-3"
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium text-gray-900">
                      {formatDate(backup.timestamp)}
                    </p>
                    <p className="text-xs text-gray-500">
                      {backup.comboCount} combos &middot; {formatSize(backup.sizeBytes)}
                    </p>
                  </div>
                  <div className="flex gap-1">
                    <button
                      onClick={() =>
                        setConfirmAction({ type: "restore", id: backup.id })
                      }
                      className="rounded p-1.5 text-gray-400 hover:bg-blue-50 hover:text-blue-600"
                      title="Restore"
                    >
                      <RotateCcwIcon size={14} />
                    </button>
                    <button
                      onClick={() =>
                        setConfirmAction({ type: "delete", id: backup.id })
                      }
                      className="rounded p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600"
                      title="Delete"
                    >
                      <TrashIcon size={14} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {error && (
            <div className="mt-3 rounded border border-red-200 bg-red-50 p-2">
              <p className="text-xs text-red-700">{error}</p>
            </div>
          )}
        </div>

        {/* Confirmation overlay */}
        {confirmAction && (
          <div className="border-t bg-yellow-50 px-6 py-3">
            <div className="flex items-center gap-2">
              <AlertTriangleIcon size={16} className="text-yellow-600" />
              <p className="flex-1 text-sm text-yellow-800">
                {confirmAction.type === "restore"
                  ? "Restore this backup? Current data will be replaced."
                  : "Delete this backup? This cannot be undone."}
              </p>
              <button
                onClick={() => setConfirmAction(null)}
                className="rounded border border-gray-300 px-3 py-1 text-xs text-gray-600 hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={() =>
                  confirmAction.type === "restore"
                    ? handleRestore(confirmAction.id)
                    : handleDelete(confirmAction.id)
                }
                className={`rounded px-3 py-1 text-xs text-white ${
                  confirmAction.type === "restore"
                    ? "bg-blue-600 hover:bg-blue-700"
                    : "bg-red-600 hover:bg-red-700"
                }`}
              >
                {confirmAction.type === "restore" ? "Restore" : "Delete"}
              </button>
            </div>
          </div>
        )}

        {/* Footer */}
        <div className="flex justify-end border-t px-6 py-3">
          <button
            onClick={onClose}
            className="rounded border border-gray-300 px-4 py-1.5 text-sm text-gray-600 hover:bg-gray-50"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
