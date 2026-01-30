// Shared utility functions
import type { Combo, Group, Preferences, MatchingMode, PasteMethod, Theme } from "./types";

/**
 * Type guard to check if an object is a valid Combo
 * @param obj - Object to check
 * @returns true if object has all required Combo fields
 */
export function isCombo(obj: unknown): obj is Combo {
  if (!obj || typeof obj !== "object") return false;

  const c = obj as Record<string, unknown>;

  return (
    typeof c.id === "string" &&
    typeof c.name === "string" &&
    typeof c.description === "string" &&
    typeof c.keyword === "string" &&
    typeof c.snippet === "string" &&
    typeof c.groupId === "string" &&
    (c.matchingMode === "strict" || c.matchingMode === "loose") &&
    typeof c.caseSensitive === "boolean" &&
    typeof c.enabled === "boolean" &&
    typeof c.useCount === "number" &&
    (c.lastUsed === null || typeof c.lastUsed === "string") &&
    typeof c.createdAt === "string" &&
    typeof c.modifiedAt === "string"
  );
}

/**
 * Type guard to check if an object is a valid Group
 * @param obj - Object to check
 * @returns true if object has all required Group fields
 */
export function isGroup(obj: unknown): obj is Group {
  if (!obj || typeof obj !== "object") return false;

  const g = obj as Record<string, unknown>;

  return (
    typeof g.id === "string" &&
    typeof g.name === "string" &&
    typeof g.description === "string" &&
    typeof g.enabled === "boolean" &&
    typeof g.createdAt === "string" &&
    typeof g.modifiedAt === "string"
  );
}

/**
 * Create default preferences with sensible values
 * @returns Preferences object with defaults
 */
export function createDefaultPreferences(): Preferences {
  return {
    enabled: true,
    playSound: false,
    showSystemTray: true,
    startAtLogin: false,
    startMinimized: false,
    defaultMatchingMode: "strict" as MatchingMode,
    defaultCaseSensitive: false,
    comboTriggerShortcut: "",
    pickerShortcut: "",
    pasteMethod: "clipboard" as PasteMethod,
    theme: "system" as Theme,
    backupEnabled: true,
    backupIntervalHours: 24,
    maxBackups: 10,
    autoCheckUpdates: true,
    excludedApps: [],
  };
}

/**
 * Generate a unique ID using crypto.randomUUID()
 * @returns UUID v4 string
 */
export function generateId(): string {
  return crypto.randomUUID();
}

/**
 * Format a lastUsed timestamp for display
 * @param date - ISO 8601 timestamp or null
 * @returns Formatted string like "2 hours ago" or "Never"
 */
export function formatLastUsed(date: string | null): string {
  if (!date) return "Never";

  const now = new Date();
  const used = new Date(date);
  const diffMs = now.getTime() - used.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);
  const diffMonths = Math.floor(diffDays / 30);
  const diffYears = Math.floor(diffDays / 365);

  if (diffSeconds < 60) {
    return diffSeconds === 1 ? "1 second ago" : `${diffSeconds} seconds ago`;
  }

  if (diffMinutes < 60) {
    return diffMinutes === 1 ? "1 minute ago" : `${diffMinutes} minutes ago`;
  }

  if (diffHours < 24) {
    return diffHours === 1 ? "1 hour ago" : `${diffHours} hours ago`;
  }

  if (diffDays < 30) {
    return diffDays === 1 ? "1 day ago" : `${diffDays} days ago`;
  }

  if (diffMonths < 12) {
    return diffMonths === 1 ? "1 month ago" : `${diffMonths} months ago`;
  }

  return diffYears === 1 ? "1 year ago" : `${diffYears} years ago`;
}
