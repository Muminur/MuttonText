import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useComboStore } from "@/stores/comboStore";
import * as api from "@/lib/tauri";
import type { Combo, CreateComboInput, UpdateComboInput } from "@/lib/types";

// Mock the tauri API
vi.mock("@/lib/tauri");

const createMockCombo = (overrides: Partial<Combo> = {}): Combo => ({
  id: "test-id-1",
  name: "Test Combo",
  description: "Test description",
  keyword: "test",
  snippet: "Test snippet",
  groupId: "group-1",
  matchingMode: "strict",
  caseSensitive: false,
  enabled: true,
  useCount: 0,
  lastUsed: null,
  createdAt: "2024-01-01T00:00:00Z",
  modifiedAt: "2024-01-01T00:00:00Z",
  ...overrides,
});

describe("comboStore", () => {
  beforeEach(() => {
    // Reset store state before each test using store actions
    const { setState } = useComboStore;
    act(() => {
      setState({
        combos: [],
        selectedIds: new Set<string>(),
        loading: false,
        error: null,
      });
    });
    vi.clearAllMocks();
  });

  describe("loadCombos", () => {
    it("should load combos successfully", async () => {
      const mockCombos = [createMockCombo(), createMockCombo({ id: "test-id-2" })];
      vi.mocked(api.getAllCombos).mockResolvedValue(mockCombos);

      const { result } = renderHook(() => useComboStore());

      await act(async () => {
        await result.current.loadCombos();
      });

      expect(result.current.combos).toEqual(mockCombos);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBe(null);
    });

    it("should set loading state during fetch", async () => {
      vi.mocked(api.getAllCombos).mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve([]), 100))
      );

      const { result } = renderHook(() => useComboStore());

      act(() => {
        result.current.loadCombos();
      });

      expect(result.current.loading).toBe(true);

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });
    });

    it("should handle errors when loading combos fails", async () => {
      const errorMessage = "Failed to load combos";
      vi.mocked(api.getAllCombos).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useComboStore());

      await act(async () => {
        await result.current.loadCombos();
      });

      expect(result.current.error).toBe(errorMessage);
      expect(result.current.loading).toBe(false);
    });
  });

  describe("selectCombo", () => {
    it("should select a combo by id", () => {
      const { result } = renderHook(() => useComboStore());

      act(() => {
        result.current.selectCombo("test-id-1");
      });

      expect(result.current.selectedIds.has("test-id-1")).toBe(true);
      expect(result.current.selectedId).toBe("test-id-1");
    });

    it("should deselect when passed null", () => {
      const { result } = renderHook(() => useComboStore());

      act(() => {
        result.current.selectCombo("test-id-1");
        result.current.selectCombo(null);
      });

      expect(result.current.selectedIds.size).toBe(0);
      expect(result.current.selectedId).toBe(null);
    });

    it("should support ctrl+click toggle", () => {
      const { result } = renderHook(() => useComboStore());

      act(() => {
        result.current.selectCombo("id-1");
      });
      act(() => {
        result.current.selectCombo("id-2", { ctrl: true });
      });

      expect(result.current.selectedIds.has("id-1")).toBe(true);
      expect(result.current.selectedIds.has("id-2")).toBe(true);
      expect(result.current.selectedIds.size).toBe(2);

      // Toggle off
      act(() => {
        result.current.selectCombo("id-1", { ctrl: true });
      });

      expect(result.current.selectedIds.has("id-1")).toBe(false);
      expect(result.current.selectedIds.has("id-2")).toBe(true);
    });

    it("should support shift+click range select", () => {
      const combo1 = createMockCombo({ id: "id-1" });
      const combo2 = createMockCombo({ id: "id-2" });
      const combo3 = createMockCombo({ id: "id-3" });

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({ combos: [combo1, combo2, combo3] });
      });
      act(() => {
        result.current.selectCombo("id-1");
      });
      act(() => {
        result.current.selectCombo("id-3", { shift: true });
      });

      expect(result.current.selectedIds.size).toBe(3);
      expect(result.current.selectedIds.has("id-1")).toBe(true);
      expect(result.current.selectedIds.has("id-2")).toBe(true);
      expect(result.current.selectedIds.has("id-3")).toBe(true);
    });
  });

  describe("selectAll and clearSelection", () => {
    it("should select all provided ids", () => {
      const { result } = renderHook(() => useComboStore());

      act(() => {
        result.current.selectAll(["id-1", "id-2", "id-3"]);
      });

      expect(result.current.selectedIds.size).toBe(3);
    });

    it("should clear all selections", () => {
      const { result } = renderHook(() => useComboStore());

      act(() => {
        result.current.selectAll(["id-1", "id-2"]);
      });
      act(() => {
        result.current.clearSelection();
      });

      expect(result.current.selectedIds.size).toBe(0);
    });
  });

  describe("createCombo", () => {
    it("should create a new combo and add to store", async () => {
      const input: CreateComboInput = {
        name: "New Combo",
        description: "New description",
        keyword: "new",
        snippet: "New snippet",
        groupId: "group-1",
        matchingMode: "strict",
        caseSensitive: false,
        enabled: true,
      };
      const createdCombo = createMockCombo({ ...input, id: "new-id" });
      vi.mocked(api.createCombo).mockResolvedValue(createdCombo);

      const { result } = renderHook(() => useComboStore());

      let returnedCombo: Combo | undefined;
      await act(async () => {
        returnedCombo = await result.current.createCombo(input);
      });

      expect(returnedCombo).toEqual(createdCombo);
      expect(result.current.combos).toContainEqual(createdCombo);
    });

    it("should handle errors when creating combo fails", async () => {
      const errorMessage = "Failed to create combo";
      vi.mocked(api.createCombo).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useComboStore());

      const input: CreateComboInput = {
        name: "New Combo",
        description: "",
        keyword: "new",
        snippet: "New snippet",
        groupId: "group-1",
        matchingMode: "strict",
        caseSensitive: false,
        enabled: true,
      };

      await expect(
        act(async () => {
          await result.current.createCombo(input);
        })
      ).rejects.toThrow();
    });
  });

  describe("updateCombo", () => {
    it("should update an existing combo", async () => {
      const originalCombo = createMockCombo();
      const update: UpdateComboInput = { name: "Updated Name" };
      const updatedCombo = { ...originalCombo, ...update };

      vi.mocked(api.updateCombo).mockResolvedValue(updatedCombo);

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({ combos: [originalCombo] });
      });

      let returnedCombo: Combo | undefined;
      await act(async () => {
        returnedCombo = await result.current.updateCombo(originalCombo.id, update);
      });

      expect(returnedCombo).toEqual(updatedCombo);
      expect(result.current.combos[0]).toEqual(updatedCombo);
    });

    it("should handle errors when updating combo fails", async () => {
      const errorMessage = "Failed to update combo";
      vi.mocked(api.updateCombo).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useComboStore());

      await expect(
        act(async () => {
          await result.current.updateCombo("test-id", { name: "Updated" });
        })
      ).rejects.toThrow();
    });
  });

  describe("deleteCombo", () => {
    it("should delete a combo from the store", async () => {
      const combo1 = createMockCombo({ id: "id-1" });
      const combo2 = createMockCombo({ id: "id-2" });

      vi.mocked(api.deleteCombo).mockResolvedValue(undefined);

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({ combos: [combo1, combo2] });
      });

      await act(async () => {
        await result.current.deleteCombo("id-1");
      });

      expect(result.current.combos).toEqual([combo2]);
      expect(result.current.combos).not.toContainEqual(combo1);
    });

    it("should deselect combo if it was selected", async () => {
      const combo = createMockCombo({ id: "id-1" });
      vi.mocked(api.deleteCombo).mockResolvedValue(undefined);

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({
          combos: [combo],
          selectedIds: new Set(["id-1"]),
        });
      });

      await act(async () => {
        await result.current.deleteCombo("id-1");
      });

      expect(result.current.selectedIds.has("id-1")).toBe(false);
      expect(result.current.selectedId).toBe(null);
    });

    it("should handle errors when deleting combo fails", async () => {
      const errorMessage = "Failed to delete combo";
      vi.mocked(api.deleteCombo).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useComboStore());

      await expect(
        act(async () => {
          await result.current.deleteCombo("test-id");
        })
      ).rejects.toThrow();
    });
  });

  describe("duplicateCombo", () => {
    it("should duplicate a combo and add to store", async () => {
      const originalCombo = createMockCombo({ id: "original-id" });
      const duplicatedCombo = createMockCombo({ id: "duplicated-id", name: "Test Combo (Copy)" });

      vi.mocked(api.duplicateCombo).mockResolvedValue(duplicatedCombo);

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({ combos: [originalCombo] });
      });

      let returnedCombo: Combo | undefined;
      await act(async () => {
        returnedCombo = await result.current.duplicateCombo("original-id");
      });

      expect(returnedCombo).toEqual(duplicatedCombo);
      expect(result.current.combos).toContainEqual(duplicatedCombo);
      expect(result.current.combos).toHaveLength(2);
    });
  });

  describe("moveComboToGroup", () => {
    it("should move a combo to a different group", async () => {
      const combo = createMockCombo({ id: "combo-1", groupId: "group-1" });
      const updatedCombo = { ...combo, groupId: "group-2" };

      vi.mocked(api.moveComboToGroup).mockResolvedValue(undefined);
      vi.mocked(api.getCombo).mockResolvedValue(updatedCombo);

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({ combos: [combo] });
      });

      await act(async () => {
        await result.current.moveComboToGroup("combo-1", "group-2");
      });

      expect(result.current.combos[0].groupId).toBe("group-2");
    });
  });

  describe("toggleCombo", () => {
    it("should toggle combo enabled state", async () => {
      const combo = createMockCombo({ id: "combo-1", enabled: true });
      vi.mocked(api.toggleCombo).mockResolvedValue(false);

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({ combos: [combo] });
      });

      let newState: boolean | undefined;
      await act(async () => {
        newState = await result.current.toggleCombo("combo-1");
      });

      expect(newState).toBe(false);
      expect(result.current.combos[0].enabled).toBe(false);
    });
  });

  describe("getSelectedCombo", () => {
    it("should return the selected combo", () => {
      const combo1 = createMockCombo({ id: "id-1" });
      const combo2 = createMockCombo({ id: "id-2" });

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({
          combos: [combo1, combo2],
          selectedIds: new Set(["id-2"]),
        });
      });

      expect(result.current.getSelectedCombo()).toEqual(combo2);
    });

    it("should return undefined when no combo is selected", () => {
      const { result } = renderHook(() => useComboStore());

      expect(result.current.getSelectedCombo()).toBeUndefined();
    });

    it("should return undefined when selected combo does not exist", () => {
      const combo = createMockCombo({ id: "id-1" });

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({
          combos: [combo],
          selectedIds: new Set(["non-existent-id"]),
        });
      });

      expect(result.current.getSelectedCombo()).toBeUndefined();
    });
  });

  describe("getSelectedCombos", () => {
    it("should return all selected combos", () => {
      const combo1 = createMockCombo({ id: "id-1" });
      const combo2 = createMockCombo({ id: "id-2" });
      const combo3 = createMockCombo({ id: "id-3" });

      const { result } = renderHook(() => useComboStore());

      act(() => {
        useComboStore.setState({
          combos: [combo1, combo2, combo3],
          selectedIds: new Set(["id-1", "id-3"]),
        });
      });

      const selected = result.current.getSelectedCombos();
      expect(selected).toHaveLength(2);
      expect(selected).toContainEqual(combo1);
      expect(selected).toContainEqual(combo3);
    });
  });
});
