// Group Store - Zustand state management for groups
// This store interfaces with Tauri IPC commands to manage group data
import { create } from "zustand";
import type { Group, CreateGroupInput, UpdateGroupInput } from "@/lib/types";
import * as api from "@/lib/tauri";

interface GroupState {
  // State
  groups: Group[];
  selectedGroupId: string | null;
  loading: boolean;
  error: string | null;

  // Actions
  loadGroups: () => Promise<void>;
  selectGroup: (id: string | null) => void;
  createGroup: (input: CreateGroupInput) => Promise<Group>;
  updateGroup: (id: string, input: UpdateGroupInput) => Promise<Group>;
  deleteGroup: (id: string) => Promise<void>;
  toggleGroup: (id: string) => Promise<boolean>;
  reorderGroups: (groups: Group[]) => void;
}

export const useGroupStore = create<GroupState>((set) => ({
  // Initial state
  groups: [],
  selectedGroupId: null,
  loading: false,
  error: null,

  // Load all groups from backend
  loadGroups: async () => {
    set({ loading: true, error: null });
    try {
      const groups = await api.getAllGroups();
      set({ groups, loading: false });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to load groups";
      set({ error: errorMessage, loading: false });
    }
  },

  // Select a group by ID
  selectGroup: (id) => {
    set({ selectedGroupId: id });
  },

  // Create a new group
  createGroup: async (input) => {
    set({ error: null });
    try {
      const newGroup = await api.createGroup(input);
      set((state) => ({
        groups: [...state.groups, newGroup],
      }));
      return newGroup;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to create group";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Update an existing group
  updateGroup: async (id, input) => {
    set({ error: null });
    try {
      const updatedGroup = await api.updateGroup(id, input);
      set((state) => ({
        groups: state.groups.map((group) =>
          group.id === id ? updatedGroup : group
        ),
      }));
      return updatedGroup;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to update group";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Delete a group
  deleteGroup: async (id) => {
    set({ error: null });
    try {
      await api.deleteGroup(id);
      set((state) => ({
        groups: state.groups.filter((group) => group.id !== id),
        selectedGroupId: state.selectedGroupId === id ? null : state.selectedGroupId,
      }));
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to delete group";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Toggle a group's enabled state
  toggleGroup: async (id) => {
    set({ error: null });
    try {
      const newEnabledState = await api.toggleGroup(id);
      set((state) => ({
        groups: state.groups.map((group) =>
          group.id === id ? { ...group, enabled: newEnabledState } : group
        ),
      }));
      return newEnabledState;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to toggle group";
      set({ error: errorMessage });
      throw error;
    }
  },

  // Reorder groups (local state update only - persistence can be added later)
  reorderGroups: (groups) => {
    set({ groups });
  },
}));
