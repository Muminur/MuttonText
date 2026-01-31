import React, { useEffect, useState } from "react";
import { XIcon } from "lucide-react";
import type { Preferences } from "@/lib/types";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { BehaviorTab } from "./BehaviorTab";
import { AppearanceTab } from "./AppearanceTab";
import { ShortcutsTab } from "./ShortcutsTab";
import { DataTab } from "./DataTab";
import { UpdatesTab } from "./UpdatesTab";
import { AdvancedTab } from "./AdvancedTab";

const TABS = [
  { id: "behavior", label: "Behavior" },
  { id: "appearance", label: "Appearance" },
  { id: "shortcuts", label: "Shortcuts" },
  { id: "data", label: "Data" },
  { id: "updates", label: "Updates" },
  { id: "advanced", label: "Advanced" },
] as const;

type TabId = (typeof TABS)[number]["id"];

interface PreferencesDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export const PreferencesDialog: React.FC<PreferencesDialogProps> = ({ isOpen, onClose }) => {
  const { preferences, loading, loadPreferences, updatePreferences, resetPreferences } =
    usePreferencesStore();

  const [activeTab, setActiveTab] = useState<TabId>("behavior");
  const [draft, setDraft] = useState<Preferences | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadPreferences();
    }
  }, [isOpen, loadPreferences]);

  useEffect(() => {
    if (preferences) {
      setDraft({ ...preferences });
    }
  }, [preferences]);

  if (!isOpen) return null;

  const handleSave = async () => {
    if (!draft) return;
    setSaving(true);
    try {
      await updatePreferences(draft);
      onClose();
    } catch {
      // error is set in store
    } finally {
      setSaving(false);
    }
  };

  const handleReset = async () => {
    try {
      await resetPreferences();
    } catch {
      // error is set in store
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onClose();
    }
  };

  const renderTab = () => {
    if (!draft) return null;
    switch (activeTab) {
      case "behavior":
        return <BehaviorTab preferences={draft} onChange={setDraft} />;
      case "appearance":
        return <AppearanceTab preferences={draft} onChange={setDraft} />;
      case "shortcuts":
        return <ShortcutsTab preferences={draft} onChange={setDraft} />;
      case "data":
        return <DataTab preferences={draft} onChange={setDraft} />;
      case "updates":
        return <UpdatesTab preferences={draft} onChange={setDraft} />;
      case "advanced":
        return <AdvancedTab preferences={draft} onChange={setDraft} />;
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onKeyDown={handleKeyDown}
      role="dialog"
      aria-modal="true"
      aria-label="Preferences"
    >
      <div className="flex h-[520px] w-[680px] flex-col rounded-lg bg-white shadow-xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b px-6 py-4">
          <h2 className="text-lg font-semibold text-gray-900">Preferences</h2>
          <button
            onClick={onClose}
            className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
            aria-label="Close"
          >
            <XIcon size={18} />
          </button>
        </div>

        {/* Body */}
        <div className="flex flex-1 overflow-hidden">
          {/* Tab sidebar */}
          <nav className="w-40 border-r bg-gray-50 py-2">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`block w-full px-4 py-2 text-left text-sm transition-colors ${
                  activeTab === tab.id
                    ? "bg-blue-50 font-medium text-blue-700"
                    : "text-gray-600 hover:bg-gray-100"
                }`}
              >
                {tab.label}
              </button>
            ))}
          </nav>

          {/* Tab content */}
          <div className="flex-1 overflow-y-auto p-6">
            {loading ? (
              <p className="text-sm text-gray-500">Loading preferences...</p>
            ) : (
              renderTab()
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between border-t px-6 py-3">
          <button
            onClick={handleReset}
            className="rounded border border-gray-300 px-3 py-1.5 text-sm text-gray-600 hover:bg-gray-50"
          >
            Reset to Defaults
          </button>
          <div className="flex gap-2">
            <button
              onClick={onClose}
              className="rounded border border-gray-300 px-4 py-1.5 text-sm text-gray-600 hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={saving}
              className="rounded bg-blue-600 px-4 py-1.5 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
            >
              {saving ? "Saving..." : "Save"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
