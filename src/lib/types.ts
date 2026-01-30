// TypeScript types matching Rust models
// These types mirror the Rust data structures in src-tauri/src/models/

/**
 * Matching mode for combo keywords
 * - strict: Match only after word boundaries (spaces, punctuation)
 * - loose: Match anywhere (ends-with matching)
 */
export type MatchingMode = "strict" | "loose";

/**
 * Method for pasting snippets
 * - clipboard: Use system clipboard (faster, more reliable)
 * - simulateKeystrokes: Type out the snippet (for apps that block clipboard)
 */
export type PasteMethod = "clipboard" | "simulateKeystrokes";

/**
 * Application theme
 * - system: Follow system theme
 * - light: Force light theme
 * - dark: Force dark theme
 */
export type Theme = "system" | "light" | "dark";

/**
 * A text snippet combo
 */
export interface Combo {
  /** Unique identifier (UUID v4) */
  id: string;

  /** Display name of the combo */
  name: string;

  /** Optional description */
  description: string;

  /** Trigger keyword (no spaces allowed) */
  keyword: string;

  /** The snippet text to expand (can contain variables) */
  snippet: string;

  /** ID of the group this combo belongs to */
  groupId: string;

  /** Matching mode for the keyword */
  matchingMode: MatchingMode;

  /** Whether keyword matching is case-sensitive */
  caseSensitive: boolean;

  /** Whether this combo is enabled */
  enabled: boolean;

  /** Number of times this combo has been used */
  useCount: number;

  /** ISO 8601 timestamp of last use (null if never used) */
  lastUsed: string | null;

  /** ISO 8601 timestamp of creation */
  createdAt: string;

  /** ISO 8601 timestamp of last modification */
  modifiedAt: string;
}

/**
 * A group to organize combos
 */
export interface Group {
  /** Unique identifier (UUID v4) */
  id: string;

  /** Display name of the group */
  name: string;

  /** Optional description */
  description: string;

  /** Whether this group (and all its combos) is enabled */
  enabled: boolean;

  /** ISO 8601 timestamp of creation */
  createdAt: string;

  /** ISO 8601 timestamp of last modification */
  modifiedAt: string;
}

/**
 * The complete combo library (persisted to combos.json)
 */
export interface ComboLibrary {
  /** Schema version for migration support */
  version: string;

  /** All groups */
  groups: Group[];

  /** All combos */
  combos: Combo[];
}

/**
 * User preferences (persisted to preferences.json)
 */
export interface Preferences {
  /** Whether snippet expansion is enabled globally */
  enabled: boolean;

  /** Play sound on substitution */
  playSound: boolean;

  /** Show icon in system tray */
  showSystemTray: boolean;

  /** Start application on login */
  startAtLogin: boolean;

  /** Start application minimized to tray */
  startMinimized: boolean;

  /** Default matching mode for new combos */
  defaultMatchingMode: MatchingMode;

  /** Default case sensitivity for new combos */
  defaultCaseSensitive: boolean;

  /** Global shortcut to trigger combo manually */
  comboTriggerShortcut: string;

  /** Global shortcut to open combo picker */
  pickerShortcut: string;

  /** Method for pasting snippets */
  pasteMethod: PasteMethod;

  /** Application theme */
  theme: Theme;

  /** Enable automatic backups */
  backupEnabled: boolean;

  /** Interval between automatic backups (in hours) */
  backupIntervalHours: number;

  /** Maximum number of backup files to keep */
  maxBackups: number;

  /** Automatically check for updates */
  autoCheckUpdates: boolean;

  /** List of application names to exclude from expansion */
  excludedApps: string[];
}

/**
 * Input for creating a new combo (without id, timestamps, stats)
 */
export type CreateComboInput = Omit<
  Combo,
  "id" | "createdAt" | "modifiedAt" | "useCount" | "lastUsed"
>;

/**
 * Input for updating an existing combo (all fields optional)
 */
export type UpdateComboInput = Partial<
  Omit<Combo, "id" | "createdAt" | "modifiedAt" | "useCount" | "lastUsed">
>;

/**
 * Input for creating a new group (without id, timestamps)
 */
export type CreateGroupInput = Omit<Group, "id" | "createdAt" | "modifiedAt">;

/**
 * Input for updating an existing group (all fields optional)
 */
export type UpdateGroupInput = Partial<
  Omit<Group, "id" | "createdAt" | "modifiedAt">
>;
