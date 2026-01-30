import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useCombo, useCombos } from "@/hooks/useCombo";
import { useComboStore } from "@/stores/comboStore";
// import * as api from "@/lib/tauri";
import type { Combo } from "@/lib/types";

// Mock the tauri API and store
vi.mock("@/lib/tauri");
vi.mock("@/stores/comboStore");

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

describe("useCombo", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return combo when id is provided", () => {
    const mockCombo = createMockCombo({ id: "test-id-1" });
    const mockStore = {
      combos: [mockCombo, createMockCombo({ id: "test-id-2" })],
      selectedId: null,
      loading: false,
      error: null,
      loadCombos: vi.fn(),
      selectCombo: vi.fn(),
      createCombo: vi.fn(),
      updateCombo: vi.fn(),
      deleteCombo: vi.fn(),
      duplicateCombo: vi.fn(),
      moveComboToGroup: vi.fn(),
      toggleCombo: vi.fn(),
      getSelectedCombo: vi.fn(),
    };

    vi.mocked(useComboStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useCombo("test-id-1"));

    expect(result.current.combo).toEqual(mockCombo);
  });

  it("should return undefined when id is not provided", () => {
    const mockStore = {
      combos: [createMockCombo()],
      selectedId: null,
      loading: false,
      error: null,
      loadCombos: vi.fn(),
      selectCombo: vi.fn(),
      createCombo: vi.fn(),
      updateCombo: vi.fn(),
      deleteCombo: vi.fn(),
      duplicateCombo: vi.fn(),
      moveComboToGroup: vi.fn(),
      toggleCombo: vi.fn(),
      getSelectedCombo: vi.fn(),
    };

    vi.mocked(useComboStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useCombo());

    expect(result.current.combo).toBeUndefined();
  });

  it("should return undefined when combo with id does not exist", () => {
    const mockStore = {
      combos: [createMockCombo({ id: "other-id" })],
      selectedId: null,
      loading: false,
      error: null,
      loadCombos: vi.fn(),
      selectCombo: vi.fn(),
      createCombo: vi.fn(),
      updateCombo: vi.fn(),
      deleteCombo: vi.fn(),
      duplicateCombo: vi.fn(),
      moveComboToGroup: vi.fn(),
      toggleCombo: vi.fn(),
      getSelectedCombo: vi.fn(),
    };

    vi.mocked(useComboStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useCombo("non-existent-id"));

    expect(result.current.combo).toBeUndefined();
  });

  it("should return all store methods and state", () => {
    const mockStore = {
      combos: [],
      selectedId: null,
      loading: false,
      error: null,
      loadCombos: vi.fn(),
      selectCombo: vi.fn(),
      createCombo: vi.fn(),
      updateCombo: vi.fn(),
      deleteCombo: vi.fn(),
      duplicateCombo: vi.fn(),
      moveComboToGroup: vi.fn(),
      toggleCombo: vi.fn(),
      getSelectedCombo: vi.fn(),
    };

    vi.mocked(useComboStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useCombo());

    expect(result.current).toMatchObject({
      combos: mockStore.combos,
      selectedId: mockStore.selectedId,
      loading: mockStore.loading,
      error: mockStore.error,
      loadCombos: mockStore.loadCombos,
      selectCombo: mockStore.selectCombo,
      createCombo: mockStore.createCombo,
      updateCombo: mockStore.updateCombo,
      deleteCombo: mockStore.deleteCombo,
      duplicateCombo: mockStore.duplicateCombo,
      moveComboToGroup: mockStore.moveComboToGroup,
      toggleCombo: mockStore.toggleCombo,
      getSelectedCombo: mockStore.getSelectedCombo,
    });
  });
});

describe("useCombos", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should load combos on mount", async () => {
    const loadCombosMock = vi.fn().mockResolvedValue(undefined);
    const mockStore = {
      combos: [],
      selectedId: null,
      loading: true,
      error: null,
      loadCombos: loadCombosMock,
      selectCombo: vi.fn(),
      createCombo: vi.fn(),
      updateCombo: vi.fn(),
      deleteCombo: vi.fn(),
      duplicateCombo: vi.fn(),
      moveComboToGroup: vi.fn(),
      toggleCombo: vi.fn(),
      getSelectedCombo: vi.fn(),
    };

    vi.mocked(useComboStore).mockReturnValue(mockStore);

    renderHook(() => useCombos());

    await waitFor(() => {
      expect(loadCombosMock).toHaveBeenCalledTimes(1);
    });
  });

  it("should return combos, loading, error, and refetch", () => {
    const mockCombos = [createMockCombo(), createMockCombo({ id: "test-id-2" })];
    const loadCombosMock = vi.fn();
    const mockStore = {
      combos: mockCombos,
      selectedId: null,
      loading: false,
      error: null,
      loadCombos: loadCombosMock,
      selectCombo: vi.fn(),
      createCombo: vi.fn(),
      updateCombo: vi.fn(),
      deleteCombo: vi.fn(),
      duplicateCombo: vi.fn(),
      moveComboToGroup: vi.fn(),
      toggleCombo: vi.fn(),
      getSelectedCombo: vi.fn(),
    };

    vi.mocked(useComboStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useCombos());

    expect(result.current.combos).toEqual(mockCombos);
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBe(null);
    expect(result.current.refetch).toBe(loadCombosMock);
  });

  it("should handle loading state", () => {
    const mockStore = {
      combos: [],
      selectedId: null,
      loading: true,
      error: null,
      loadCombos: vi.fn(),
      selectCombo: vi.fn(),
      createCombo: vi.fn(),
      updateCombo: vi.fn(),
      deleteCombo: vi.fn(),
      duplicateCombo: vi.fn(),
      moveComboToGroup: vi.fn(),
      toggleCombo: vi.fn(),
      getSelectedCombo: vi.fn(),
    };

    vi.mocked(useComboStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useCombos());

    expect(result.current.loading).toBe(true);
  });

  it("should handle error state", () => {
    const errorMessage = "Failed to load combos";
    const mockStore = {
      combos: [],
      selectedId: null,
      loading: false,
      error: errorMessage,
      loadCombos: vi.fn(),
      selectCombo: vi.fn(),
      createCombo: vi.fn(),
      updateCombo: vi.fn(),
      deleteCombo: vi.fn(),
      duplicateCombo: vi.fn(),
      moveComboToGroup: vi.fn(),
      toggleCombo: vi.fn(),
      getSelectedCombo: vi.fn(),
    };

    vi.mocked(useComboStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useCombos());

    expect(result.current.error).toBe(errorMessage);
  });
});
