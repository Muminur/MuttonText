import React from "react";
import type { Preferences } from "@/lib/types";

interface BehaviorTabProps {
  preferences: Preferences;
  onChange: (prefs: Preferences) => void;
}

export const BehaviorTab: React.FC<BehaviorTabProps> = ({ preferences, onChange }) => {
  const update = (partial: Partial<Preferences>) => {
    onChange({ ...preferences, ...partial });
  };

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">Behavior</h3>

      <div className="space-y-4">
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={preferences.enabled}
            onChange={(e) => update({ enabled: e.target.checked })}
            className="h-4 w-4 rounded border-gray-300 text-blue-600"
          />
          <div>
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Enable snippet expansion</span>
            <p className="text-xs text-gray-500 dark:text-gray-400">Globally enable or disable combo expansion</p>
          </div>
        </label>

        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={preferences.playSound}
            onChange={(e) => update({ playSound: e.target.checked })}
            className="h-4 w-4 rounded border-gray-300 text-blue-600"
          />
          <div>
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Play sound on expansion</span>
            <p className="text-xs text-gray-500 dark:text-gray-400">Play a sound when a combo is triggered</p>
          </div>
        </label>

        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-700">
            Default matching mode
          </label>
          <select
            value={preferences.defaultMatchingMode}
            onChange={(e) =>
              update({ defaultMatchingMode: e.target.value as "strict" | "loose" })
            }
            className="block w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
          >
            <option value="strict">Strict (word boundary)</option>
            <option value="loose">Loose (match anywhere)</option>
          </select>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            Default matching mode for new combos
          </p>
        </div>

        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={preferences.defaultCaseSensitive}
            onChange={(e) => update({ defaultCaseSensitive: e.target.checked })}
            className="h-4 w-4 rounded border-gray-300 text-blue-600"
          />
          <div>
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Case-sensitive by default</span>
            <p className="text-xs text-gray-500 dark:text-gray-400">Default case sensitivity for new combos</p>
          </div>
        </label>
      </div>
    </div>
  );
};
