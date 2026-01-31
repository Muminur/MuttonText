import React, { useState, useCallback } from "react";
import { XIcon, DownloadIcon, CheckCircleIcon } from "lucide-react";
import type { ExportFormat } from "@/lib/types";
import { exportCombos } from "@/lib/tauri";

interface ExportDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const FORMATS: { value: ExportFormat; label: string; extension: string }[] = [
  { value: "muttonTextJson", label: "MuttonText JSON", extension: ".json" },
  { value: "textExpanderCsv", label: "TextExpander CSV", extension: ".csv" },
  { value: "cheatsheetCsv", label: "Cheatsheet CSV", extension: ".csv" },
];

export const ExportDialog: React.FC<ExportDialogProps> = ({ isOpen, onClose }) => {
  const [format, setFormat] = useState<ExportFormat>("muttonTextJson");
  const [exporting, setExporting] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState("");

  const handleClose = useCallback(() => {
    setFormat("muttonTextJson");
    setExporting(false);
    setSuccess(false);
    setError("");
    onClose();
  }, [onClose]);

  const handleExport = useCallback(async () => {
    setExporting(true);
    setError("");
    try {
      const content = await exportCombos(format);
      const selected = FORMATS.find((f) => f.value === format);
      const ext = selected?.extension ?? ".json";
      const blob = new Blob([content], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `muttontext-export${ext}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      setSuccess(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setExporting(false);
    }
  }, [format]);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      role="dialog"
      aria-modal="true"
      aria-label="Export Combos"
    >
      <div className="flex w-[400px] flex-col rounded-lg bg-white shadow-xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b px-6 py-4">
          <h2 className="text-lg font-semibold text-gray-900">Export Combos</h2>
          <button
            onClick={handleClose}
            className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
            aria-label="Close"
          >
            <XIcon size={18} />
          </button>
        </div>

        {/* Body */}
        <div className="p-6">
          {success ? (
            <div className="flex flex-col items-center py-4">
              <CheckCircleIcon size={32} className="text-green-600" />
              <p className="mt-2 font-medium text-gray-900">Export Successful</p>
              <p className="mt-1 text-sm text-gray-500">
                Your combos have been exported.
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-700">
                  Export Format
                </label>
                {FORMATS.map((f) => (
                  <label
                    key={f.value}
                    className="flex cursor-pointer items-center gap-3 rounded border p-3 hover:bg-gray-50"
                  >
                    <input
                      type="radio"
                      name="export-format"
                      value={f.value}
                      checked={format === f.value}
                      onChange={() => setFormat(f.value)}
                      className="h-4 w-4 text-blue-600"
                    />
                    <div>
                      <span className="text-sm font-medium text-gray-700">
                        {f.label}
                      </span>
                      <span className="ml-2 text-xs text-gray-400">{f.extension}</span>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          )}

          {error && (
            <div className="mt-3 rounded border border-red-200 bg-red-50 p-2">
              <p className="text-xs text-red-700">{error}</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 border-t px-6 py-3">
          {success ? (
            <button
              onClick={handleClose}
              className="rounded bg-blue-600 px-4 py-1.5 text-sm text-white hover:bg-blue-700"
            >
              Done
            </button>
          ) : (
            <>
              <button
                onClick={handleClose}
                className="rounded border border-gray-300 px-4 py-1.5 text-sm text-gray-600 hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleExport}
                disabled={exporting}
                className="flex items-center gap-1.5 rounded bg-blue-600 px-4 py-1.5 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
              >
                <DownloadIcon size={14} />
                {exporting ? "Exporting..." : "Export"}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
};
