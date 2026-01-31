import React, { useState, useCallback, useRef } from "react";
import { XIcon, UploadIcon, FileIcon, AlertCircleIcon, CheckCircleIcon } from "lucide-react";
import type { ImportPreview, ImportResult, ConflictResolution } from "@/lib/types";
import { previewImport, importCombos } from "@/lib/tauri";

interface ImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onImportComplete: () => void;
}

type Stage = "select" | "preview" | "importing" | "results";

const FORMAT_LABELS: Record<string, string> = {
  beeftextJson: "Beeftext JSON",
  beeftextCsv: "Beeftext CSV",
  textExpanderCsv: "TextExpander CSV",
  muttonTextJson: "MuttonText JSON",
};

export const ImportDialog: React.FC<ImportDialogProps> = ({
  isOpen,
  onClose,
  onImportComplete,
}) => {
  const [stage, setStage] = useState<Stage>("select");
  const [fileContent, setFileContent] = useState<string>("");
  const [fileName, setFileName] = useState<string>("");
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [conflict, setConflict] = useState<ConflictResolution>("skip");
  const [result, setResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState<string>("");
  const [isDragging, setIsDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const reset = useCallback(() => {
    setStage("select");
    setFileContent("");
    setFileName("");
    setPreview(null);
    setConflict("skip");
    setResult(null);
    setError("");
    setIsDragging(false);
  }, []);

  const handleClose = useCallback(() => {
    reset();
    onClose();
  }, [reset, onClose]);

  const handleFileRead = useCallback(async (content: string, name: string) => {
    setFileContent(content);
    setFileName(name);
    setError("");
    try {
      const p = await previewImport(content);
      setPreview(p);
      setStage("preview");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        handleFileRead(reader.result as string, file.name);
      };
      reader.readAsText(file);
    },
    [handleFileRead]
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragging(false);
      const file = e.dataTransfer.files[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        handleFileRead(reader.result as string, file.name);
      };
      reader.readAsText(file);
    },
    [handleFileRead]
  );

  const handleImport = useCallback(async () => {
    if (!preview || !fileContent) return;
    setStage("importing");
    setError("");
    try {
      const r = await importCombos(fileContent, preview.format, conflict);
      setResult(r);
      setStage("results");
      onImportComplete();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStage("preview");
    }
  }, [preview, fileContent, conflict, onImportComplete]);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      role="dialog"
      aria-modal="true"
      aria-label="Import Combos"
    >
      <div className="flex w-[480px] flex-col rounded-lg bg-white shadow-xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b px-6 py-4">
          <h2 className="text-lg font-semibold text-gray-900">Import Combos</h2>
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
          {stage === "select" && (
            <div className="space-y-4">
              <div
                className={`flex flex-col items-center justify-center rounded-lg border-2 border-dashed p-8 transition-colors ${
                  isDragging
                    ? "border-blue-500 bg-blue-50"
                    : "border-gray-300 hover:border-gray-400"
                }`}
                onDragOver={(e) => {
                  e.preventDefault();
                  setIsDragging(true);
                }}
                onDragLeave={() => setIsDragging(false)}
                onDrop={handleDrop}
              >
                <UploadIcon size={32} className="mb-2 text-gray-400" />
                <p className="text-sm text-gray-600">
                  Drag and drop a file here, or
                </p>
                <button
                  onClick={() => fileInputRef.current?.click()}
                  className="mt-2 rounded bg-blue-600 px-4 py-1.5 text-sm text-white hover:bg-blue-700"
                >
                  Choose File
                </button>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".json,.csv,.txt"
                  onChange={handleFileSelect}
                  className="hidden"
                />
              </div>
              <p className="text-xs text-gray-500">
                Supported formats: MuttonText JSON, Beeftext JSON/CSV, TextExpander CSV
              </p>
            </div>
          )}

          {stage === "preview" && preview && (
            <div className="space-y-4">
              <div className="flex items-center gap-2 rounded border bg-gray-50 p-3">
                <FileIcon size={16} className="text-gray-500" />
                <span className="text-sm text-gray-700">{fileName}</span>
              </div>
              <div className="grid grid-cols-3 gap-3 text-center">
                <div className="rounded border p-2">
                  <div className="text-lg font-semibold text-gray-900">
                    {preview.comboCount}
                  </div>
                  <div className="text-xs text-gray-500">Combos</div>
                </div>
                <div className="rounded border p-2">
                  <div className="text-lg font-semibold text-gray-900">
                    {preview.groupCount}
                  </div>
                  <div className="text-xs text-gray-500">Groups</div>
                </div>
                <div className="rounded border p-2">
                  <div className="text-sm font-medium text-gray-900">
                    {FORMAT_LABELS[preview.format] ?? preview.format}
                  </div>
                  <div className="text-xs text-gray-500">Format</div>
                </div>
              </div>

              <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-700">
                  Conflict Resolution
                </label>
                <select
                  value={conflict}
                  onChange={(e) => setConflict(e.target.value as ConflictResolution)}
                  className="block w-full rounded border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                >
                  <option value="skip">Skip duplicates</option>
                  <option value="overwrite">Overwrite existing</option>
                  <option value="rename">Rename imported</option>
                </select>
              </div>
            </div>
          )}

          {stage === "importing" && (
            <div className="flex flex-col items-center py-8">
              <div className="h-8 w-8 animate-spin rounded-full border-4 border-blue-600 border-t-transparent" />
              <p className="mt-3 text-sm text-gray-600">Importing combos...</p>
            </div>
          )}

          {stage === "results" && result && (
            <div className="space-y-3">
              <div className="flex items-center gap-2">
                <CheckCircleIcon size={20} className="text-green-600" />
                <span className="font-medium text-gray-900">Import Complete</span>
              </div>
              <div className="grid grid-cols-3 gap-3 text-center">
                <div className="rounded border border-green-200 bg-green-50 p-2">
                  <div className="text-lg font-semibold text-green-700">
                    {result.importedCount}
                  </div>
                  <div className="text-xs text-green-600">Imported</div>
                </div>
                <div className="rounded border border-yellow-200 bg-yellow-50 p-2">
                  <div className="text-lg font-semibold text-yellow-700">
                    {result.skippedCount}
                  </div>
                  <div className="text-xs text-yellow-600">Skipped</div>
                </div>
                <div className="rounded border border-red-200 bg-red-50 p-2">
                  <div className="text-lg font-semibold text-red-700">
                    {result.errors.length}
                  </div>
                  <div className="text-xs text-red-600">Errors</div>
                </div>
              </div>
              {result.errors.length > 0 && (
                <div className="max-h-32 overflow-y-auto rounded border border-red-200 bg-red-50 p-2">
                  {result.errors.map((err, i) => (
                    <p key={i} className="text-xs text-red-700">
                      {err}
                    </p>
                  ))}
                </div>
              )}
            </div>
          )}

          {error && (
            <div className="mt-3 flex items-center gap-2 rounded border border-red-200 bg-red-50 p-2">
              <AlertCircleIcon size={16} className="text-red-600" />
              <p className="text-xs text-red-700">{error}</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 border-t px-6 py-3">
          {stage === "results" ? (
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
              {stage === "preview" && (
                <button
                  onClick={handleImport}
                  className="rounded bg-blue-600 px-4 py-1.5 text-sm text-white hover:bg-blue-700"
                >
                  Import
                </button>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};
