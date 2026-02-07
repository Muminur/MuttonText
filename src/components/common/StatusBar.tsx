// StatusBar - Shows combo statistics at bottom of main layout
import { useMemo } from "react";
import { useComboStore } from "../../stores/comboStore";

export function StatusBar() {
  const { combos } = useComboStore();

  // Find most recently used combo
  const lastUsedInfo = useMemo(() => {
    const usedCombos = combos.filter((c) => c.lastUsed !== null);
    if (usedCombos.length === 0) return null;

    const mostRecent = usedCombos.reduce((prev, current) => {
      const prevTime = new Date(prev.lastUsed!).getTime();
      const currentTime = new Date(current.lastUsed!).getTime();
      return currentTime > prevTime ? current : prev;
    });

    return {
      name: mostRecent.name,
      time: formatTimeAgo(mostRecent.lastUsed!),
    };
  }, [combos]);

  // Format time ago
  function formatTimeAgo(isoString: string): string {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSeconds = Math.floor(diffMs / 1000);
    const diffMinutes = Math.floor(diffSeconds / 60);
    const diffHours = Math.floor(diffMinutes / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffSeconds < 60) return "just now";
    if (diffMinutes < 60) return `${diffMinutes} minute${diffMinutes !== 1 ? "s" : ""} ago`;
    if (diffHours < 24) return `${diffHours} hour${diffHours !== 1 ? "s" : ""} ago`;
    if (diffDays < 7) return `${diffDays} day${diffDays !== 1 ? "s" : ""} ago`;

    return date.toLocaleDateString();
  }

  return (
    <div
      role="contentinfo"
      className="border-t border-gray-200 dark:border-gray-700 px-4 py-2 bg-gray-50 dark:bg-gray-800 text-sm text-gray-600 dark:text-gray-400 flex items-center justify-between"
    >
      <div>
        <span className="font-medium">{combos.length}</span>{" "}
        {combos.length === 1 ? "combo" : "combos"}
      </div>

      {lastUsedInfo && (
        <div className="text-xs">
          Last used: <span className="font-medium">{lastUsedInfo.name}</span>{" "}
          <span className="text-gray-500 dark:text-gray-400">{lastUsedInfo.time}</span>
        </div>
      )}
    </div>
  );
}
