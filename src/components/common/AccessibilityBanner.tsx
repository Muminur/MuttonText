import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function AccessibilityBanner() {
  const [granted, setGranted] = useState(true);

  useEffect(() => {
    let interval: number | undefined;

    const check = async () => {
      try {
        const result = await invoke<boolean>("check_accessibility");
        setGranted(result);
        if (result && interval) {
          clearInterval(interval);
          interval = undefined;
        }
      } catch {
        // Not on macOS or command not available
        setGranted(true);
      }
    };

    check();
    interval = window.setInterval(check, 3000);
    return () => {
      if (interval) clearInterval(interval);
    };
  }, []);

  if (granted) return null;

  const handleRequest = async () => {
    try {
      await invoke("request_accessibility");
    } catch {
      // ignore
    }
  };

  return (
    <div className="bg-yellow-500/90 text-black px-4 py-2 text-sm flex items-center justify-between">
      <span>
        Accessibility permission required. Grant access in{" "}
        <strong>System Settings → Privacy & Security → Accessibility</strong>.
      </span>
      <button
        onClick={handleRequest}
        className="ml-3 px-3 py-1 bg-black/20 rounded hover:bg-black/30 text-sm font-medium whitespace-nowrap"
      >
        Open Settings
      </button>
    </div>
  );
}
