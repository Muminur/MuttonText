// Combo Store - Zustand state management for combos
// This store interfaces with Tauri IPC commands to manage combo data
import { create } from "zustand";
import type { Combo, CreateComboInput, UpdateComboInput } from "@/lib/types";
import * as api from "@/lib/tauri";

interface SelectOptions {
  ctrl?: boolean;
  shift?: boolean;
}

/** Helper: derive selectedId from selectedIds */
function deriveSelectedId(ids: Set<string>): string | null {
  const arr = Array.from(ids);
  return arr.length > 0 ? arr[0] : null;
}

interface ComboState {
  // State
  combos: Combo[];
  selectedIds: Set<string>;
  /** Backward compat: first selected ID or null */
  selectedId: string | null;
  loading: boolean;
  error: string | null;

  // Actions
  loadCombos: () => Promise<void>;
  selectCombo: (id: string | null, options?: SelectOptions) => void;
  selectAll: (ids: string[]) => void;
  clearSelection: () => void;
  createCombo: (input: CreateComboInput) => Promise<Combo>;
  updateCombo: (id: string, input: UpdateComboInput) => Promise<Combo>;
  deleteCombo: (id: string) => Promise<void>;
  duplicateCombo: (id: string) => Promise<Combo>;
  moveComboToGroup: (comboId: string, groupId: string) => Promise<void>;
  toggleCombo: (id: string) => Promise<boolean>;
  getSelectedCombo: () => Combo | undefined;
  getSelectedCombos: () => Combo[];
}

export const useComboStore = create<ComboState>((set, get) => ({
  // Initial state
  combos: [],
  selectedIds: new Set<string>(),
  selectedId: null,
  loading: false,
  error: null,

  // Load all combos from backend
  loadCombos: async () => {
    set({ loading: true, error: null });
    try {
      const combos = await api.getAllCombos();
      set({ combos, loading: false });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to load combos";
      set({ error: errorMessage, loading: false });
    }
  },

  // Select a combo by ID with multi-select support
  selectCombo: (id, options = {}) => {
    if (!id) {
      set({ selectedIds: new Set<string>(), selectedId: null });
      return;
    }

    const { ctrl, shift } = options;
    const state = get();
    const newSelectedIds = new Set(state.selectedIds);

    if (shift && newSelectedIds.size > 0) {
      const comboIds = state.combos.map((c) => c.id);
      const lastSelectedId = Array.from(newSelectedIds)[newSelectedIds.size - 1];
      const lastIndex = comboIds.indexOf(lastSelectedId);
      const currentIndex = comboIds.indexOf(id);

      if (lastIndex !== -1 && currentIndex !== -1) {
        const start = Math.min(lastIndex, currentIndex);
        const end = Math.max(lastIndex, currentIndex);
        for (let i = start; i <= end; i++) {
          newSelectedIds.add(comboIds[i]);
        }
      }
    } else if (ctrl) {
      if (newSelectedIds.has(id)) {
        newSelectedIds.delete(id);
      } else {
        newSelectedIds.add(id);
      }
    } else {
      newSelectedIds.clear();
      newSelectedIds.add(id);
    }

    set({ selectedIds: newSelectedIds, selectedId: deriveSelectedId(newSelectedIds) });
  },

  // Select all combos
  selectAll: (ids: string[]) => {
    const newIds = new Set(ids);
    set({ selectedIds: newIds, selectedId: deriveSelectedId(newIds) });
  },

  // Clear all selections
  clearSelection: () => {
    set({ selectedIds: new Set<string>(), selectedId: null });
  },

  // Create a new combo
  createCombo: async (input) => {
    set({ error: null });
    try {
      const newCombo = await api.createCombo(input);
      set((state) => ({
        combos: [...state.combos, newCombo],
      }));
      return newCombo;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to create combo";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Update an existing combo
  updateCombo: async (id, input) => {
    set({ error: null });
    try {
      const updatedCombo = await api.updateCombo(id, input);
      set((state) => ({
        combos: state.combos.map((combo) =>
          combo.id === id ? updatedCombo : combo
        ),
      }));
      return updatedCombo;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to update combo";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Delete a combo
  deleteCombo: async (id) => {
    set({ error: null });
    try {
      await api.deleteCombo(id);
      set((state) => {
        const newSelectedIds = new Set(state.selectedIds);
        newSelectedIds.delete(id);
        return {
          combos: state.combos.filter((combo) => combo.id !== id),
          selectedIds: newSelectedIds,
          selectedId: deriveSelectedId(newSelectedIds),
        };
      });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to delete combo";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Duplicate a combo
  duplicateCombo: async (id) => {
    set({ error: null });
    try {
      const duplicatedCombo = await api.duplicateCombo(id);
      set((state) => ({
        combos: [...state.combos, duplicatedCombo],
      }));
      return duplicatedCombo;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to duplicate combo";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Move a combo to a different group
  moveComboToGroup: async (comboId, groupId) => {
    set({ error: null });
    try {
      await api.moveComboToGroup(comboId, groupId);
      const updatedCombo = await api.getCombo(comboId);
      if (updatedCombo) {
        set((state) => ({
          combos: state.combos.map((combo) =>
            combo.id === comboId ? updatedCombo : combo
          ),
        }));
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : "Failed to move combo to group";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Toggle a combo's enabled state
  toggleCombo: async (id) => {
    set({ error: null });
    try {
      const newEnabledState = await api.toggleCombo(id);
      set((state) => ({
        combos: state.combos.map((combo) =>
          combo.id === id ? { ...combo, enabled: newEnabledState } : combo
        ),
      }));
      return newEnabledState;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to toggle combo";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Get the currently selected combo (backward compatibility)
  getSelectedCombo: () => {
    const state = get();
    const ids = Array.from(state.selectedIds);
    if (ids.length === 0) return undefined;
    return state.combos.find((combo) => combo.id === ids[0]);
  },

  // Get all selected combos
  getSelectedCombos: () => {
    const state = get();
    const selectedIdsArray = Array.from(state.selectedIds);
    return state.combos.filter((combo) => selectedIdsArray.includes(combo.id));
  },
}));
