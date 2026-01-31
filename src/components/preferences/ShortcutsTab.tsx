import React from "react";
import type { Preferences } from "@/lib/types";

interface ShortcutsTabProps {
  preferences: Preferences;
  onChange: (prefs: Preferences) => void;
}

export const ShortcutsTab: React.FC<ShortcutsTabProps> = ({ preferences, onChange }) => {
  const update = (partial: Partial<Preferences>) => {
    onChange({ ...preferences, ...partial });
  };

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-medium text-gray-900">Shortcuts</h3>

      <div className="space-y-4">
        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-700">
            Picker shortcut
          </label>
          <input
            type="text"
            value={preferences.pickerShortcut}
            onChange={(e) => update({ pickerShortcut: e.target.value })}
            placeholder="e.g. Ctrl+Space"
            className="block w-full rounded border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
          />
          <p className="text-xs text-gray-500">
            Global shortcut to open the combo picker
          </p>
        </div>

        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-700">
            Combo trigger shortcut
          </label>
          <input
            type="text"
            value={preferences.comboTriggerShortcut}
            onChange={(e) => update({ comboTriggerShortcut: e.target.value })}
            placeholder="e.g. Ctrl+Shift+E"
            className="block w-full rounded border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
          />
          <p className="text-xs text-gray-500">
            Global shortcut to trigger combo expansion manually
          </p>
        </div>
      </div>
    </div>
  );
};
