import React, { useState } from "react";
import type { Preferences } from "@/lib/types";

interface AdvancedTabProps {
  preferences: Preferences;
  onChange: (prefs: Preferences) => void;
}

export const AdvancedTab: React.FC<AdvancedTabProps> = ({ preferences, onChange }) => {
  const [newApp, setNewApp] = useState("");

  const update = (partial: Partial<Preferences>) => {
    onChange({ ...preferences, ...partial });
  };

  const handleAddApp = () => {
    const trimmed = newApp.trim();
    if (trimmed && !preferences.excludedApps.includes(trimmed)) {
      update({ excludedApps: [...preferences.excludedApps, trimmed] });
      setNewApp("");
    }
  };

  const handleRemoveApp = (app: string) => {
    update({ excludedApps: preferences.excludedApps.filter((a) => a !== app) });
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAddApp();
    }
  };

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-medium text-gray-900">Advanced</h3>

      <div className="space-y-4">
        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-700">Paste method</label>
          <select
            value={preferences.pasteMethod}
            onChange={(e) =>
              update({ pasteMethod: e.target.value as "clipboard" | "simulateKeystrokes" })
            }
            className="block w-full rounded border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
          >
            <option value="clipboard">Clipboard (recommended)</option>
            <option value="simulateKeystrokes">Simulate keystrokes</option>
          </select>
          <p className="text-xs text-gray-500">
            How expanded text is pasted. Use keystrokes for apps that block clipboard access.
          </p>
        </div>

        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-700">Excluded applications</label>
          <p className="text-xs text-gray-500">
            Combos will not expand in these applications.
          </p>

          <div className="flex gap-2">
            <input
              type="text"
              value={newApp}
              onChange={(e) => setNewApp(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Application name (e.g. code.exe)"
              className="flex-1 rounded border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
            />
            <button
              onClick={handleAddApp}
              disabled={!newApp.trim()}
              className="rounded bg-blue-600 px-3 py-2 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
            >
              Add
            </button>
          </div>

          {preferences.excludedApps.length > 0 ? (
            <ul className="space-y-1">
              {preferences.excludedApps.map((app) => (
                <li
                  key={app}
                  className="flex items-center justify-between rounded border border-gray-200 bg-gray-50 px-3 py-1.5 text-sm"
                >
                  <span>{app}</span>
                  <button
                    onClick={() => handleRemoveApp(app)}
                    className="text-red-500 hover:text-red-700"
                  >
                    Remove
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-xs italic text-gray-400">No excluded applications</p>
          )}
        </div>
      </div>
    </div>
  );
};
