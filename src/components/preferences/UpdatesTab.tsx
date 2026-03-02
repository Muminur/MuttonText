import React, { useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import type { Preferences, VersionInfo } from "@/lib/types";
import * as api from "@/lib/tauri";

interface UpdatesTabProps {
  preferences: Preferences;
  onChange: (prefs: Preferences) => void;
}

interface ReleaseAsset {
  name: string;
  browser_download_url: string;
}

interface GitHubRelease {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  html_url: string;
  assets: ReleaseAsset[];
}

function getPlatformAsset(assets: ReleaseAsset[]): ReleaseAsset | undefined {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return assets.find((a) => a.name.endsWith(".dmg"));
  if (ua.includes("win")) return assets.find((a) => a.name.endsWith(".exe") || a.name.endsWith(".msi"));
  // Linux: prefer AppImage, then deb
  return (
    assets.find((a) => a.name.endsWith(".AppImage")) ||
    assets.find((a) => a.name.endsWith(".deb"))
  );
}

export const UpdatesTab: React.FC<UpdatesTabProps> = ({ preferences, onChange }) => {
  const [checking, setChecking] = useState(false);
  const [updateAvailable, setUpdateAvailable] = useState<VersionInfo | null>(null);
  const [releaseUrl, setReleaseUrl] = useState<string>("");
  const [downloadUrl, setDownloadUrl] = useState<string>("");
  const [message, setMessage] = useState<string | null>(null);
  const [currentVersion, setCurrentVersion] = useState<string>("");

  const update = (partial: Partial<Preferences>) => {
    onChange({ ...preferences, ...partial });
  };

  React.useEffect(() => {
    getVersion().then(setCurrentVersion).catch(() => setCurrentVersion("unknown"));
  }, []);

  const handleCheckUpdates = async () => {
    setChecking(true);
    setMessage(null);
    setUpdateAvailable(null);
    try {
      const response = await fetch(
        "https://api.github.com/repos/Muminur/MuttonText/releases/latest",
        { headers: { Accept: "application/vnd.github+json" } }
      );
      if (!response.ok) throw new Error(`GitHub API returned ${response.status}`);

      const release: GitHubRelease = await response.json();
      const version = release.tag_name.replace(/^v/, "");

      const versionInfo: VersionInfo = {
        version,
        releaseUrl: release.html_url,
        releaseNotes: release.body || "",
        publishedAt: release.published_at,
      };

      const isNewer = await api.checkForUpdates(versionInfo);
      if (isNewer) {
        setUpdateAvailable(versionInfo);
        setReleaseUrl(release.html_url);
        const asset = getPlatformAsset(release.assets);
        setDownloadUrl(asset?.browser_download_url || release.html_url);
      } else {
        setMessage("You are running the latest version.");
      }
    } catch (error) {
      setMessage(
        error instanceof Error ? `Update check failed: ${error.message}` : "Failed to check for updates."
      );
    } finally {
      setChecking(false);
    }
  };

  const handleDownload = () => {
    api.openUrl(downloadUrl || releaseUrl);
  };

  const handleSkip = async () => {
    if (!updateAvailable) return;
    await api.skipUpdateVersion(updateAvailable.version);
    setUpdateAvailable(null);
    setMessage("Version skipped. You will not be notified about this release again.");
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
            {checking ? "Checking…" : "Check for Updates"}
          </button>
        </div>

        {message && (
          <p className="text-sm text-gray-600 dark:text-gray-400">{message}</p>
        )}

        {updateAvailable && (
          <div className="rounded border border-blue-200 dark:border-blue-700 bg-blue-50 dark:bg-blue-900/30 p-4 space-y-3">
            <div>
              <p className="text-sm font-semibold text-blue-800 dark:text-blue-300">
                Version {updateAvailable.version} is available
              </p>
              {updateAvailable.releaseNotes && (
                <p className="mt-1 text-xs text-blue-700 dark:text-blue-400 whitespace-pre-line line-clamp-4">
                  {updateAvailable.releaseNotes}
                </p>
              )}
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleDownload}
                className="rounded bg-blue-600 px-3 py-1.5 text-sm text-white hover:bg-blue-700"
              >
                Download Update
              </button>
              <button
                onClick={() => api.openUrl(releaseUrl)}
                className="rounded border border-blue-300 dark:border-blue-600 px-3 py-1.5 text-sm text-blue-700 dark:text-blue-300 hover:bg-blue-50 dark:hover:bg-blue-900/50"
              >
                View Release Notes
              </button>
              <button
                onClick={handleSkip}
                className="rounded border border-gray-300 dark:border-gray-600 px-3 py-1.5 text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800"
              >
                Skip This Version
              </button>
            </div>
          </div>
        )}

        <div className="rounded border border-gray-200 dark:border-gray-600 bg-gray-50 dark:bg-gray-900 p-3">
          <p className="text-xs text-gray-500 dark:text-gray-400">
            Current version: {currentVersion || "loading…"}
          </p>
        </div>
      </div>
    </div>
  );
};
