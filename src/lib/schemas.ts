// Zod schemas for runtime validation of data models
import { z } from "zod";

/**
 * Matching mode enum
 */
export const matchingModeSchema = z.enum(["strict", "loose"]);

/**
 * Paste method enum
 */
export const pasteMethodSchema = z.enum(["clipboard", "simulateKeystrokes"]);

/**
 * Theme enum
 */
export const themeSchema = z.enum(["system", "light", "dark"]);

/**
 * Combo schema with full validation
 */
export const comboSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  description: z.string(),
  keyword: z.string(),
  snippet: z.string(),
  groupId: z.string().uuid(),
  matchingMode: matchingModeSchema,
  caseSensitive: z.boolean(),
  enabled: z.boolean(),
  useCount: z.number().int().min(0),
  lastUsed: z.string().nullable(),
  createdAt: z.string(),
  modifiedAt: z.string(),
});

/**
 * Group schema with full validation
 */
export const groupSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  description: z.string(),
  enabled: z.boolean(),
  createdAt: z.string(),
  modifiedAt: z.string(),
});

/**
 * Combo library schema
 */
export const comboLibrarySchema = z.object({
  version: z.string(),
  groups: z.array(groupSchema),
  combos: z.array(comboSchema),
});

/**
 * Preferences schema
 */
export const preferencesSchema = z.object({
  enabled: z.boolean(),
  playSound: z.boolean(),
  showSystemTray: z.boolean(),
  startAtLogin: z.boolean(),
  startMinimized: z.boolean(),
  defaultMatchingMode: matchingModeSchema,
  defaultCaseSensitive: z.boolean(),
  comboTriggerShortcut: z.string(),
  pickerShortcut: z.string(),
  pasteMethod: pasteMethodSchema,
  theme: themeSchema,
  backupEnabled: z.boolean(),
  backupIntervalHours: z.number().int().min(0),
  maxBackups: z.number().int().min(0),
  autoCheckUpdates: z.boolean(),
  excludedApps: z.array(z.string()),
});

/**
 * Create combo schema with validation rules
 * - Keyword must have no spaces
 * - Keyword must be at least 2 characters
 * - Snippet cannot be empty or only whitespace
 */
export const createComboSchema = z.object({
  name: z.string(),
  description: z.string(),
  keyword: z
    .string()
    .min(2, "Keyword must be at least 2 characters")
    .refine((val) => !val.includes(" "), {
      message: "Keyword cannot contain spaces",
    }),
  snippet: z
    .string()
    .min(1, "Snippet cannot be empty")
    .refine((val) => val.trim().length > 0, {
      message: "Snippet cannot be only whitespace",
    }),
  groupId: z.string().uuid(),
  matchingMode: matchingModeSchema,
  caseSensitive: z.boolean(),
  enabled: z.boolean(),
});

/**
 * Update combo schema (partial, with same validation rules)
 */
export const updateComboSchema = createComboSchema.partial();

/**
 * Create group schema
 */
export const createGroupSchema = z.object({
  name: z.string().min(1, "Group name cannot be empty"),
  description: z.string(),
  enabled: z.boolean(),
});

/**
 * Update group schema (partial)
 */
export const updateGroupSchema = createGroupSchema.partial();

/**
 * Inferred TypeScript types from schemas
 */
export type ComboSchema = z.infer<typeof comboSchema>;
export type GroupSchema = z.infer<typeof groupSchema>;
export type ComboLibrarySchema = z.infer<typeof comboLibrarySchema>;
export type PreferencesSchema = z.infer<typeof preferencesSchema>;
export type CreateComboSchema = z.infer<typeof createComboSchema>;
export type UpdateComboSchema = z.infer<typeof updateComboSchema>;
export type CreateGroupSchema = z.infer<typeof createGroupSchema>;
export type UpdateGroupSchema = z.infer<typeof updateGroupSchema>;
