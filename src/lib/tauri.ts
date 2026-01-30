// Tauri IPC wrapper functions for invoking Rust backend commands
import { invoke } from "@tauri-apps/api/core";
import type {
  Combo,
  Group,
  CreateComboInput,
  UpdateComboInput,
  CreateGroupInput,
  UpdateGroupInput,
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
