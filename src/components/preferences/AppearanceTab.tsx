import React from "react";
import type { Preferences } from "@/lib/types";

interface AppearanceTabProps {
  preferences: Preferences;
  onChange: (prefs: Preferences) => void;
}

export const AppearanceTab: React.FC<AppearanceTabProps> = ({ preferences, onChange }) => {
  const update = (partial: Partial<Preferences>) => {
    onChange({ ...preferences, ...partial });
  };

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">Appearance</h3>

      <div className="space-y-4">
        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">Theme</label>
          <div className="flex gap-3">
            {(["system", "light", "dark"] as const).map((theme) => (
              <button
                key={theme}
                onClick={() => update({ theme })}
                className={`flex-1 rounded border px-4 py-2 text-sm capitalize transition-colors ${
                  preferences.theme === theme
                    ? "border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400"
                    : "border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600"
                }`}
              >
                {theme}
              </button>
            ))}
          </div>
        </div>

        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={preferences.showSystemTray}
            onChange={(e) => update({ showSystemTray: e.target.checked })}
            className="h-4 w-4 rounded border-gray-300 text-blue-600"
          />
          <div>
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Show in system tray</span>
            <p className="text-xs text-gray-500 dark:text-gray-400">Display icon in the system notification area</p>
          </div>
        </label>

        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={preferences.startAtLogin}
            onChange={(e) => update({ startAtLogin: e.target.checked })}
            className="h-4 w-4 rounded border-gray-300 text-blue-600"
          />
          <div>
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Start at login</span>
            <p className="text-xs text-gray-500 dark:text-gray-400">Launch MuttonText when you log in</p>
          </div>
        </label>

        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={preferences.startMinimized}
            onChange={(e) => update({ startMinimized: e.target.checked })}
            className="h-4 w-4 rounded border-gray-300 text-blue-600"
          />
          <div>
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Start minimized</span>
            <p className="text-xs text-gray-500 dark:text-gray-400">Start minimized to system tray</p>
          </div>
        </label>
      </div>
    </div>
  );
};
