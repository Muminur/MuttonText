// Tauri IPC wrapper functions for invoking Rust backend commands
import { invoke } from "@tauri-apps/api/core";
import type {
  Combo,
  Group,
  Preferences,
  CreateComboInput,
  UpdateComboInput,
  CreateGroupInput,
  UpdateGroupInput,
  TrayState,
  ImportResult,
  ImportPreview,
  BackupInfo,
  VersionInfo,
} from "./types";

// ========================================
// Combo Operations
// ========================================

/**
 * Get all combos from the backend
 */
export async function getAllCombos(): Promise<Combo[]> {
  return invoke("get_all_combos");
}

/**
 * Get a single combo by ID
 */
export async function getCombo(id: string): Promise<Combo | null> {
  return invoke("get_combo", { id });
}

/**
 * Create a new combo
 */
export async function createCombo(input: CreateComboInput): Promise<Combo> {
  return invoke("create_combo", {
    name: input.name,
    description: input.description,
    keyword: input.keyword,
    snippet: input.snippet,
    groupId: input.groupId,
    matchingMode: input.matchingMode,
    caseSensitive: input.caseSensitive,
    enabled: input.enabled,
  });
}

/**
 * Update an existing combo
 */
export async function updateCombo(
  id: string,
  input: UpdateComboInput
): Promise<Combo> {
  return invoke("update_combo", { id, ...input });
}

/**
 * Delete a combo
 */
export async function deleteCombo(id: string): Promise<void> {
  return invoke("delete_combo", { id });
}

/**
 * Duplicate a combo (creates a copy with " (Copy)" appended to name)
 */
export async function duplicateCombo(id: string): Promise<Combo> {
  return invoke("duplicate_combo", { id });
}

/**
 * Move a combo to a different group
 */
export async function moveComboToGroup(
  comboId: string,
  groupId: string
): Promise<void> {
  return invoke("move_combo_to_group", { comboId, groupId });
}

/**
 * Toggle a combo's enabled state
 * Returns the new enabled state
 */
export async function toggleCombo(id: string): Promise<boolean> {
  return invoke("toggle_combo", { id });
}

// ========================================
// Group Operations
// ========================================

/**
 * Get all groups from the backend
 */
export async function getAllGroups(): Promise<Group[]> {
  return invoke("get_all_groups");
}

/**
 * Get a single group by ID
 */
export async function getGroup(id: string): Promise<Group | null> {
  return invoke("get_group", { id });
}

/**
 * Create a new group
 */
export async function createGroup(input: CreateGroupInput): Promise<Group> {
  return invoke("create_group", {
    name: input.name,
    description: input.description,
    enabled: input.enabled,
  });
}

/**
 * Update an existing group
 */
export async function updateGroup(
  id: string,
  input: UpdateGroupInput
): Promise<Group> {
  return invoke("update_group", { id, ...input });
}

/**
 * Delete a group
 */
export async function deleteGroup(id: string): Promise<void> {
  return invoke("delete_group", { id });
}

/**
 * Toggle a group's enabled state
 * Returns the new enabled state
 */
export async function toggleGroup(id: string): Promise<boolean> {
  return invoke("toggle_group", { id });
}

// ========================================
// Picker Operations
// ========================================

/**
 * Search combos by query (searches name, keyword, description, snippet)
 */
export async function searchCombos(query: string): Promise<Combo[]> {
  return invoke("search_combos", { query });
}

/**
 * Trigger combo expansion (paste snippet)
 */
export async function triggerComboExpansion(comboId: string): Promise<void> {
  return invoke("trigger_combo_expansion", { comboId });
}

/**
 * Copy snippet to clipboard without expansion
 */
export async function copySnippetToClipboard(comboId: string): Promise<void> {
  return invoke("copy_snippet_to_clipboard", { comboId });
}

/**
 * Open the picker window
 */
export async function openPicker(): Promise<void> {
  return invoke("open_picker");
}

/**
 * Close the picker window
 */
export async function closePicker(): Promise<void> {
  return invoke("close_picker");
}

// ========================================
// Preferences Operations
// ========================================

export async function getPreferences(): Promise<Preferences> {
  return invoke("get_preferences");
}

export async function updatePreferences(preferences: Preferences): Promise<void> {
  return invoke("update_preferences", { preferences });
}

export async function resetPreferences(): Promise<Preferences> {
  return invoke("reset_preferences");
}

export async function getExcludedApps(): Promise<string[]> {
  return invoke("get_excluded_apps");
}

export async function addExcludedApp(app: string): Promise<void> {
  return invoke("add_excluded_app", { app });
}

export async function removeExcludedApp(app: string): Promise<boolean> {
  return invoke("remove_excluded_app", { app });
}

// ========================================
// Tray Operations
// ========================================

export async function getTrayState(): Promise<TrayState> {
  return invoke("get_tray_state");
}

export async function setTrayEnabled(enabled: boolean): Promise<void> {
  return invoke("set_tray_enabled", { enabled });
}

// ========================================
// Import/Export Operations
// ========================================

export async function importCombos(
  content: string,
  format: string,
  conflictResolution: string
): Promise<ImportResult> {
  return invoke("import_combos", { content, format, conflictResolution });
}

export async function previewImport(content: string): Promise<ImportPreview> {
  return invoke("preview_import", { content });
}

export async function exportCombos(format: string): Promise<string> {
  return invoke("export_combos", { format });
}

// ========================================
// Backup Operations
// ========================================

export async function createBackup(): Promise<BackupInfo> {
  return invoke("create_backup");
}

export async function restoreBackup(backupId: string): Promise<void> {
  return invoke("restore_backup", { backupId });
}

export async function listBackups(): Promise<BackupInfo[]> {
  return invoke("list_backups");
}

export async function deleteBackup(backupId: string): Promise<void> {
  return invoke("delete_backup", { backupId });
}

// ========================================
// Update Operations
// ========================================

export async function checkForUpdates(): Promise<VersionInfo | null> {
  return invoke("check_for_updates");
}

export async function skipUpdateVersion(version: string): Promise<void> {
  return invoke("skip_update_version", { version });
}
