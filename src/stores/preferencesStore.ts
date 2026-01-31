// Preferences Store - Zustand state management for user preferences
import { create } from "zustand";
import type { Preferences } from "@/lib/types";
import * as api from "@/lib/tauri";

interface PreferencesState {
  // State
  preferences: Preferences | null;
  loading: boolean;
  error: string | null;

  // Actions
  loadPreferences: () => Promise<void>;
  updatePreferences: (prefs: Preferences) => Promise<void>;
  resetPreferences: () => Promise<void>;
  addExcludedApp: (app: string) => Promise<void>;
  removeExcludedApp: (app: string) => Promise<void>;
}

const DEFAULT_PREFERENCES: Preferences = {
  enabled: true,
  playSound: true,
  showSystemTray: true,
  startAtLogin: false,
  startMinimized: false,
  defaultMatchingMode: "strict",
  defaultCaseSensitive: false,
  comboTriggerShortcut: "",
  pickerShortcut: "Ctrl+Space",
  pasteMethod: "clipboard",
  theme: "system",
  backupEnabled: true,
  backupIntervalHours: 24,
  maxBackups: 10,
  autoCheckUpdates: true,
  excludedApps: [],
};

export const usePreferencesStore = create<PreferencesState>((set, get) => ({
  preferences: null,
  loading: false,
  error: null,

  loadPreferences: async () => {
    set({ loading: true, error: null });
    try {
      const preferences = await api.getPreferences();
      set({ preferences, loading: false });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to load preferences";
      // Use defaults if backend not available
      set({ preferences: DEFAULT_PREFERENCES, error: errorMessage, loading: false });
    }
  },

  updatePreferences: async (prefs) => {
    set({ error: null });
    try {
      await api.updatePreferences(prefs);
      set({ preferences: prefs });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to update preferences";
      set({ error: errorMessage });
      throw error;
    }
  },

  resetPreferences: async () => {
    set({ error: null });
    try {
      const defaults = await api.resetPreferences();
      set({ preferences: defaults });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to reset preferences";
      set({ error: errorMessage });
      throw error;
    }
  },

  addExcludedApp: async (app) => {
    set({ error: null });
    try {
      await api.addExcludedApp(app);
      const prefs = get().preferences;
      if (prefs) {
        set({
          preferences: {
            ...prefs,
            excludedApps: [...prefs.excludedApps, app],
          },
        });
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to add excluded app";
      set({ error: errorMessage });
      throw error;
    }
  },

  removeExcludedApp: async (app) => {
    set({ error: null });
    try {
      await api.removeExcludedApp(app);
      const prefs = get().preferences;
      if (prefs) {
        set({
          preferences: {
            ...prefs,
            excludedApps: prefs.excludedApps.filter((a) => a !== app),
          },
        });
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to remove excluded app";
      set({ error: errorMessage });
      throw error;
    }
  },
}));
