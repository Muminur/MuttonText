import { useEffect } from "react";
import type { Theme } from "@/lib/types";

export function useTheme(theme: Theme | undefined) {
  useEffect(() => {
    if (!theme) return;

    const root = document.documentElement;

    if (theme === "dark") {
      root.classList.add("dark");
    } else if (theme === "light") {
      root.classList.remove("dark");
    } else {
      // system: follow OS preference
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const applySystem = (e: MediaQueryList | MediaQueryListEvent) => {
        if (e.matches) {
          root.classList.add("dark");
        } else {
          root.classList.remove("dark");
        }
      };
      applySystem(mediaQuery);
      const handler = (e: MediaQueryListEvent) => applySystem(e);
      mediaQuery.addEventListener("change", handler);
      return () => mediaQuery.removeEventListener("change", handler);
    }
  }, [theme]);
}
