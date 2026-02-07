import React, { useState } from "react";
import type { Preferences, VersionInfo } from "@/lib/types";
import * as api from "@/lib/tauri";

interface UpdatesTabProps {
  preferences: Preferences;
  onChange: (prefs: Preferences) => void;
}

export const UpdatesTab: React.FC<UpdatesTabProps> = ({ preferences, onChange }) => {
  const [checking, setChecking] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<VersionInfo | null>(null);
  const [checkError, setCheckError] = useState<string | null>(null);

  const update = (partial: Partial<Preferences>) => {
    onChange({ ...preferences, ...partial });
  };

  const handleCheckUpdates = async () => {
    setChecking(true);
    setCheckError(null);
    setUpdateInfo(null);
    try {
      const info = await api.checkForUpdates();
      setUpdateInfo(info);
      if (!info) {
        setCheckError("You are running the latest version.");
      }
    } catch (error) {
      setCheckError(
        error instanceof Error ? error.message : "Failed to check for updates"
      );
    } finally {
      setChecking(false);
    }
  };

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">Updates</h3>

      <div className="space-y-4">
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={preferences.autoCheckUpdates}
            onChange={(e) => update({ autoCheckUpdates: e.target.checked })}
            className="h-4 w-4 rounded border-gray-300 text-blue-600"
          />
          <div>
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Automatically check for updates
            </span>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              Check for new versions when the app starts
            </p>
          </div>
        </label>

        <div>
          <button
            onClick={handleCheckUpdates}
            disabled={checking}
            className="rounded bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
          >
            {checking ? "Checking..." : "Check for Updates"}
          </button>
        </div>

        {checkError && (
          <p className="text-sm text-gray-600">{checkError}</p>
        )}

        {updateInfo && (
          <div className="rounded border border-blue-200 bg-blue-50 p-3">
            <p className="text-sm font-medium text-blue-800">
              Version {updateInfo.version} available
            </p>
            <p className="mt-1 text-xs text-blue-700">{updateInfo.releaseNotes}</p>
          </div>
        )}

        <div className="rounded border border-gray-200 dark:border-gray-600 bg-gray-50 dark:bg-gray-900 p-3">
          <p className="text-xs text-gray-500 dark:text-gray-400">Current version: 0.1.0</p>
        </div>
      </div>
    </div>
  );
};
