import React from "react";
import type { Preferences } from "@/lib/types";

interface DataTabProps {
  preferences: Preferences;
  onChange: (prefs: Preferences) => void;
}

export const DataTab: React.FC<DataTabProps> = ({ preferences, onChange }) => {
  const update = (partial: Partial<Preferences>) => {
    onChange({ ...preferences, ...partial });
  };

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-medium text-gray-900">Data & Backups</h3>

      <div className="space-y-4">
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={preferences.backupEnabled}
            onChange={(e) => update({ backupEnabled: e.target.checked })}
            className="h-4 w-4 rounded border-gray-300 text-blue-600"
          />
          <div>
            <span className="text-sm font-medium text-gray-700">Enable automatic backups</span>
            <p className="text-xs text-gray-500">Automatically back up your combos</p>
          </div>
        </label>

        {preferences.backupEnabled && (
          <>
            <div className="space-y-2">
              <label className="block text-sm font-medium text-gray-700">
                Backup interval (hours)
              </label>
              <input
                type="number"
                min={1}
                max={168}
                value={preferences.backupIntervalHours}
                onChange={(e) =>
                  update({ backupIntervalHours: parseInt(e.target.value, 10) || 24 })
                }
                className="block w-32 rounded border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
              />
            </div>

            <div className="space-y-2">
              <label className="block text-sm font-medium text-gray-700">
                Maximum backups to keep
              </label>
              <input
                type="number"
                min={1}
                max={100}
                value={preferences.maxBackups}
                onChange={(e) =>
                  update({ maxBackups: parseInt(e.target.value, 10) || 10 })
                }
                className="block w-32 rounded border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
              />
            </div>
          </>
        )}

        <div className="rounded border border-gray-200 bg-gray-50 p-3">
          <p className="text-xs text-gray-500">
            Data is stored in the application data directory. Backups are saved alongside your combo data.
          </p>
        </div>
      </div>
    </div>
  );
};
