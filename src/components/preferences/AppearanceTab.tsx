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
      <h3 className="text-lg font-medium text-gray-900">Appearance</h3>

      <div className="space-y-4">
        <div className="space-y-2">
          <label className="block text-sm font-medium text-gray-700">Theme</label>
          <div className="flex gap-3">
            {(["system", "light", "dark"] as const).map((theme) => (
              <button
                key={theme}
                onClick={() => update({ theme })}
                className={`flex-1 rounded border px-4 py-2 text-sm capitalize transition-colors ${
                  preferences.theme === theme
                    ? "border-blue-500 bg-blue-50 text-blue-700"
                    : "border-gray-300 bg-white text-gray-700 hover:bg-gray-50"
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
            <span className="text-sm font-medium text-gray-700">Show in system tray</span>
            <p className="text-xs text-gray-500">Display icon in the system notification area</p>
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
            <span className="text-sm font-medium text-gray-700">Start at login</span>
            <p className="text-xs text-gray-500">Launch MuttonText when you log in</p>
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
            <span className="text-sm font-medium text-gray-700">Start minimized</span>
            <p className="text-xs text-gray-500">Start minimized to system tray</p>
          </div>
        </label>
      </div>
    </div>
  );
};
