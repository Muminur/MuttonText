import { describe, it, expect } from "vitest";
import {
  comboSchema,
  groupSchema,
  comboLibrarySchema,
  preferencesSchema,
  createComboSchema,
  updateComboSchema,
} from "../lib/schemas";
import {
  isCombo,
  isGroup,
  createDefaultPreferences,
  generateId,
  formatLastUsed,
} from "../lib/utils";
import type {
  Combo,
  Group,
  MatchingMode,
  PasteMethod,
  Theme,
} from "../lib/types";

describe("Zod Schema Validation", () => {
  describe("comboSchema", () => {
    it("should validate a valid combo", () => {
      const validCombo = {
        id: "550e8400-e29b-41d4-a716-446655440000",
        name: "Test Combo",
        description: "A test combo",
        keyword: "test",
        snippet: "This is a test snippet",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "strict" as MatchingMode,
        caseSensitive: false,
        enabled: true,
        useCount: 5,
        lastUsed: "2024-01-30T12:00:00Z",
        createdAt: "2024-01-01T12:00:00Z",
        modifiedAt: "2024-01-30T12:00:00Z",
      };

      const result = comboSchema.safeParse(validCombo);
      expect(result.success).toBe(true);
    });

    it("should validate combo with null lastUsed", () => {
      const combo = {
        id: "550e8400-e29b-41d4-a716-446655440000",
        name: "Test",
        description: "",
        keyword: "test",
        snippet: "snippet",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "loose" as MatchingMode,
        caseSensitive: true,
        enabled: true,
        useCount: 0,
        lastUsed: null,
        createdAt: "2024-01-01T12:00:00Z",
        modifiedAt: "2024-01-01T12:00:00Z",
      };

      const result = comboSchema.safeParse(combo);
      expect(result.success).toBe(true);
    });

    it("should reject combo with invalid matching mode", () => {
      const invalidCombo = {
        id: "550e8400-e29b-41d4-a716-446655440000",
        name: "Test",
        description: "",
        keyword: "test",
        snippet: "snippet",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "invalid",
        caseSensitive: false,
        enabled: true,
        useCount: 0,
        lastUsed: null,
        createdAt: "2024-01-01T12:00:00Z",
        modifiedAt: "2024-01-01T12:00:00Z",
      };

      const result = comboSchema.safeParse(invalidCombo);
      expect(result.success).toBe(false);
    });

    it("should reject combo with missing required fields", () => {
      const invalidCombo = {
        id: "550e8400-e29b-41d4-a716-446655440000",
        name: "Test",
        keyword: "test",
      };

      const result = comboSchema.safeParse(invalidCombo);
      expect(result.success).toBe(false);
    });
  });

  describe("createComboSchema", () => {
    it("should validate valid combo creation input", () => {
      const input = {
        name: "My Combo",
        description: "Description",
        keyword: "mc",
        snippet: "Hello World",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "strict" as MatchingMode,
        caseSensitive: false,
        enabled: true,
      };

      const result = createComboSchema.safeParse(input);
      expect(result.success).toBe(true);
    });

    it("should reject keyword with spaces", () => {
      const input = {
        name: "My Combo",
        description: "",
        keyword: "has space",
        snippet: "Hello",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "strict" as MatchingMode,
        caseSensitive: false,
        enabled: true,
      };

      const result = createComboSchema.safeParse(input);
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.errors[0].path).toContain("keyword");
      }
    });

    it("should reject keyword shorter than 2 characters", () => {
      const input = {
        name: "My Combo",
        description: "",
        keyword: "x",
        snippet: "Hello",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "strict" as MatchingMode,
        caseSensitive: false,
        enabled: true,
      };

      const result = createComboSchema.safeParse(input);
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.errors[0].path).toContain("keyword");
      }
    });

    it("should reject empty snippet", () => {
      const input = {
        name: "My Combo",
        description: "",
        keyword: "test",
        snippet: "",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "strict" as MatchingMode,
        caseSensitive: false,
        enabled: true,
      };

      const result = createComboSchema.safeParse(input);
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.errors[0].path).toContain("snippet");
      }
    });

    it("should reject snippet with only whitespace", () => {
      const input = {
        name: "My Combo",
        description: "",
        keyword: "test",
        snippet: "   \n  \t  ",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "strict" as MatchingMode,
        caseSensitive: false,
        enabled: true,
      };

      const result = createComboSchema.safeParse(input);
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.errors[0].path).toContain("snippet");
      }
    });
  });

  describe("updateComboSchema", () => {
    it("should allow partial updates", () => {
      const input = {
        name: "Updated Name",
      };

      const result = updateComboSchema.safeParse(input);
      expect(result.success).toBe(true);
    });

    it("should validate keyword when provided", () => {
      const input = {
        keyword: "invalid keyword",
      };

      const result = updateComboSchema.safeParse(input);
      expect(result.success).toBe(false);
    });

    it("should allow empty object", () => {
      const result = updateComboSchema.safeParse({});
      expect(result.success).toBe(true);
    });
  });

  describe("groupSchema", () => {
    it("should validate a valid group", () => {
      const validGroup = {
        id: "660e8400-e29b-41d4-a716-446655440000",
        name: "My Group",
        description: "Group description",
        enabled: true,
        createdAt: "2024-01-01T12:00:00Z",
        modifiedAt: "2024-01-30T12:00:00Z",
      };

      const result = groupSchema.safeParse(validGroup);
      expect(result.success).toBe(true);
    });

    it("should reject group with missing name", () => {
      const invalidGroup = {
        id: "660e8400-e29b-41d4-a716-446655440000",
        description: "Description",
        enabled: true,
        createdAt: "2024-01-01T12:00:00Z",
        modifiedAt: "2024-01-01T12:00:00Z",
      };

      const result = groupSchema.safeParse(invalidGroup);
      expect(result.success).toBe(false);
    });
  });

  describe("comboLibrarySchema", () => {
    it("should validate a valid combo library", () => {
      const library = {
        version: "1.0.0",
        groups: [
          {
            id: "660e8400-e29b-41d4-a716-446655440000",
            name: "Group 1",
            description: "",
            enabled: true,
            createdAt: "2024-01-01T12:00:00Z",
            modifiedAt: "2024-01-01T12:00:00Z",
          },
        ],
        combos: [
          {
            id: "550e8400-e29b-41d4-a716-446655440000",
            name: "Combo 1",
            description: "",
            keyword: "c1",
            snippet: "Snippet 1",
            groupId: "660e8400-e29b-41d4-a716-446655440000",
            matchingMode: "strict" as MatchingMode,
            caseSensitive: false,
            enabled: true,
            useCount: 0,
            lastUsed: null,
            createdAt: "2024-01-01T12:00:00Z",
            modifiedAt: "2024-01-01T12:00:00Z",
          },
        ],
      };

      const result = comboLibrarySchema.safeParse(library);
      expect(result.success).toBe(true);
    });
  });

  describe("preferencesSchema", () => {
    it("should validate valid preferences", () => {
      const prefs = {
        enabled: true,
        playSound: false,
        showSystemTray: true,
        startAtLogin: false,
        startMinimized: false,
        defaultMatchingMode: "strict" as MatchingMode,
        defaultCaseSensitive: false,
        comboTriggerShortcut: "Ctrl+Space",
        pickerShortcut: "Alt+Space",
        pasteMethod: "clipboard" as PasteMethod,
        theme: "system" as Theme,
        backupEnabled: true,
        backupIntervalHours: 24,
        maxBackups: 10,
        autoCheckUpdates: true,
        excludedApps: ["password-manager", "keepass"],
      };

      const result = preferencesSchema.safeParse(prefs);
      expect(result.success).toBe(true);
    });

    it("should reject invalid theme", () => {
      const prefs = {
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
        theme: "invalid",
        backupEnabled: false,
        backupIntervalHours: 0,
        maxBackups: 0,
        autoCheckUpdates: false,
        excludedApps: [],
      };

      const result = preferencesSchema.safeParse(prefs);
      expect(result.success).toBe(false);
    });

    it("should reject invalid paste method", () => {
      const prefs = {
        enabled: true,
        playSound: false,
        showSystemTray: true,
        startAtLogin: false,
        startMinimized: false,
        defaultMatchingMode: "strict" as MatchingMode,
        defaultCaseSensitive: false,
        comboTriggerShortcut: "",
        pickerShortcut: "",
        pasteMethod: "invalid",
        theme: "system" as Theme,
        backupEnabled: false,
        backupIntervalHours: 0,
        maxBackups: 0,
        autoCheckUpdates: false,
        excludedApps: [],
      };

      const result = preferencesSchema.safeParse(prefs);
      expect(result.success).toBe(false);
    });
  });
});

describe("Type Guards", () => {
  describe("isCombo", () => {
    it("should return true for valid combo object", () => {
      const combo: Combo = {
        id: "550e8400-e29b-41d4-a716-446655440000",
        name: "Test",
        description: "",
        keyword: "test",
        snippet: "snippet",
        groupId: "660e8400-e29b-41d4-a716-446655440000",
        matchingMode: "strict",
        caseSensitive: false,
        enabled: true,
        useCount: 0,
        lastUsed: null,
        createdAt: "2024-01-01T12:00:00Z",
        modifiedAt: "2024-01-01T12:00:00Z",
      };

      expect(isCombo(combo)).toBe(true);
    });

    it("should return false for object missing required fields", () => {
      const notCombo = {
        id: "550e8400-e29b-41d4-a716-446655440000",
        name: "Test",
      };

      expect(isCombo(notCombo)).toBe(false);
    });

    it("should return false for null", () => {
      expect(isCombo(null)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isCombo(undefined)).toBe(false);
    });

    it("should return false for non-object", () => {
      expect(isCombo("string")).toBe(false);
      expect(isCombo(123)).toBe(false);
      expect(isCombo(true)).toBe(false);
    });
  });

  describe("isGroup", () => {
    it("should return true for valid group object", () => {
      const group: Group = {
        id: "660e8400-e29b-41d4-a716-446655440000",
        name: "Test Group",
        description: "Description",
        enabled: true,
        createdAt: "2024-01-01T12:00:00Z",
        modifiedAt: "2024-01-01T12:00:00Z",
      };

      expect(isGroup(group)).toBe(true);
    });

    it("should return false for object missing required fields", () => {
      const notGroup = {
        id: "660e8400-e29b-41d4-a716-446655440000",
        name: "Test",
      };

      expect(isGroup(notGroup)).toBe(false);
    });

    it("should return false for null", () => {
      expect(isGroup(null)).toBe(false);
    });
  });
});

describe("Utility Functions", () => {
  describe("createDefaultPreferences", () => {
    it("should return preferences with all required fields", () => {
      const prefs = createDefaultPreferences();

      expect(prefs).toHaveProperty("enabled");
      expect(prefs).toHaveProperty("playSound");
      expect(prefs).toHaveProperty("showSystemTray");
      expect(prefs).toHaveProperty("startAtLogin");
      expect(prefs).toHaveProperty("startMinimized");
      expect(prefs).toHaveProperty("defaultMatchingMode");
      expect(prefs).toHaveProperty("defaultCaseSensitive");
      expect(prefs).toHaveProperty("comboTriggerShortcut");
      expect(prefs).toHaveProperty("pickerShortcut");
      expect(prefs).toHaveProperty("pasteMethod");
      expect(prefs).toHaveProperty("theme");
      expect(prefs).toHaveProperty("backupEnabled");
      expect(prefs).toHaveProperty("backupIntervalHours");
      expect(prefs).toHaveProperty("maxBackups");
      expect(prefs).toHaveProperty("autoCheckUpdates");
      expect(prefs).toHaveProperty("excludedApps");
    });

    it("should return valid preferences per schema", () => {
      const prefs = createDefaultPreferences();
      const result = preferencesSchema.safeParse(prefs);
      expect(result.success).toBe(true);
    });

    it("should have sensible defaults", () => {
      const prefs = createDefaultPreferences();

      expect(prefs.enabled).toBe(true);
      expect(prefs.showSystemTray).toBe(true);
      expect(prefs.defaultMatchingMode).toBe("strict");
      expect(prefs.pasteMethod).toBe("clipboard");
      expect(prefs.theme).toBe("system");
      expect(prefs.excludedApps).toEqual([]);
    });
  });

  describe("generateId", () => {
    it("should return a valid UUID v4 string", () => {
      const id = generateId();
      const uuidRegex =
        /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
      expect(id).toMatch(uuidRegex);
    });

    it("should generate unique IDs", () => {
      const id1 = generateId();
      const id2 = generateId();
      const id3 = generateId();

      expect(id1).not.toBe(id2);
      expect(id2).not.toBe(id3);
      expect(id1).not.toBe(id3);
    });

    it("should generate many unique IDs", () => {
      const ids = new Set();
      for (let i = 0; i < 1000; i++) {
        ids.add(generateId());
      }
      expect(ids.size).toBe(1000);
    });
  });

  describe("formatLastUsed", () => {
    it("should return 'Never' for null", () => {
      expect(formatLastUsed(null)).toBe("Never");
    });

    it("should format a valid ISO date string", () => {
      const date = "2024-01-30T12:30:00Z";
      const formatted = formatLastUsed(date);
      expect(formatted).toBeTruthy();
      expect(formatted).not.toBe("Never");
    });

    it("should handle different date formats", () => {
      const date1 = "2024-01-01T00:00:00Z";
      const date2 = "2024-12-31T23:59:59Z";

      const formatted1 = formatLastUsed(date1);
      const formatted2 = formatLastUsed(date2);

      expect(formatted1).toBeTruthy();
      expect(formatted2).toBeTruthy();
      expect(formatted1).not.toBe(formatted2);
    });

    it("should produce human-readable output", () => {
      const recentDate = new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(); // 2 hours ago
      const formatted = formatLastUsed(recentDate);

      // Should contain time-related words like "ago", "hours", etc.
      expect(formatted.toLowerCase()).toMatch(/ago|hour|minute|second|day|month|year/);
    });
  });
});
