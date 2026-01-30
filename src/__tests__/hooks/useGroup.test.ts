import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useGroup, useGroups } from "@/hooks/useGroup";
import { useGroupStore } from "@/stores/groupStore";
import type { Group } from "@/lib/types";

// Mock the tauri API and store
vi.mock("@/lib/tauri");
vi.mock("@/stores/groupStore");

const createMockGroup = (overrides: Partial<Group> = {}): Group => ({
  id: "group-id-1",
  name: "Test Group",
  description: "Test group description",
  enabled: true,
  createdAt: "2024-01-01T00:00:00Z",
  modifiedAt: "2024-01-01T00:00:00Z",
  ...overrides,
});

describe("useGroup", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return group when id is provided", () => {
    const mockGroup = createMockGroup({ id: "group-id-1" });
    const mockStore = {
      groups: [mockGroup, createMockGroup({ id: "group-id-2" })],
      selectedGroupId: null,
      loading: false,
      error: null,
      loadGroups: vi.fn(),
      selectGroup: vi.fn(),
      createGroup: vi.fn(),
      updateGroup: vi.fn(),
      deleteGroup: vi.fn(),
      toggleGroup: vi.fn(),
    };

    vi.mocked(useGroupStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useGroup("group-id-1"));

    expect(result.current.group).toEqual(mockGroup);
  });

  it("should return undefined when id is not provided", () => {
    const mockStore = {
      groups: [createMockGroup()],
      selectedGroupId: null,
      loading: false,
      error: null,
      loadGroups: vi.fn(),
      selectGroup: vi.fn(),
      createGroup: vi.fn(),
      updateGroup: vi.fn(),
      deleteGroup: vi.fn(),
      toggleGroup: vi.fn(),
    };

    vi.mocked(useGroupStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useGroup());

    expect(result.current.group).toBeUndefined();
  });

  it("should return undefined when group with id does not exist", () => {
    const mockStore = {
      groups: [createMockGroup({ id: "other-id" })],
      selectedGroupId: null,
      loading: false,
      error: null,
      loadGroups: vi.fn(),
      selectGroup: vi.fn(),
      createGroup: vi.fn(),
      updateGroup: vi.fn(),
      deleteGroup: vi.fn(),
      toggleGroup: vi.fn(),
    };

    vi.mocked(useGroupStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useGroup("non-existent-id"));

    expect(result.current.group).toBeUndefined();
  });

  it("should return all store methods and state", () => {
    const mockStore = {
      groups: [],
      selectedGroupId: null,
      loading: false,
      error: null,
      loadGroups: vi.fn(),
      selectGroup: vi.fn(),
      createGroup: vi.fn(),
      updateGroup: vi.fn(),
      deleteGroup: vi.fn(),
      toggleGroup: vi.fn(),
    };

    vi.mocked(useGroupStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useGroup());

    expect(result.current).toMatchObject({
      groups: mockStore.groups,
      selectedGroupId: mockStore.selectedGroupId,
      loading: mockStore.loading,
      error: mockStore.error,
      loadGroups: mockStore.loadGroups,
      selectGroup: mockStore.selectGroup,
      createGroup: mockStore.createGroup,
      updateGroup: mockStore.updateGroup,
      deleteGroup: mockStore.deleteGroup,
      toggleGroup: mockStore.toggleGroup,
    });
  });
});

describe("useGroups", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should load groups on mount", async () => {
    const loadGroupsMock = vi.fn().mockResolvedValue(undefined);
    const mockStore = {
      groups: [],
      selectedGroupId: null,
      loading: true,
      error: null,
      loadGroups: loadGroupsMock,
      selectGroup: vi.fn(),
      createGroup: vi.fn(),
      updateGroup: vi.fn(),
      deleteGroup: vi.fn(),
      toggleGroup: vi.fn(),
    };

    vi.mocked(useGroupStore).mockReturnValue(mockStore);

    renderHook(() => useGroups());

    await waitFor(() => {
      expect(loadGroupsMock).toHaveBeenCalledTimes(1);
    });
  });

  it("should return groups, loading, error, and refetch", () => {
    const mockGroups = [createMockGroup(), createMockGroup({ id: "group-id-2" })];
    const loadGroupsMock = vi.fn();
    const mockStore = {
      groups: mockGroups,
      selectedGroupId: null,
      loading: false,
      error: null,
      loadGroups: loadGroupsMock,
      selectGroup: vi.fn(),
      createGroup: vi.fn(),
      updateGroup: vi.fn(),
      deleteGroup: vi.fn(),
      toggleGroup: vi.fn(),
    };

    vi.mocked(useGroupStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useGroups());

    expect(result.current.groups).toEqual(mockGroups);
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBe(null);
    expect(result.current.refetch).toBe(loadGroupsMock);
  });

  it("should handle loading state", () => {
    const mockStore = {
      groups: [],
      selectedGroupId: null,
      loading: true,
      error: null,
      loadGroups: vi.fn(),
      selectGroup: vi.fn(),
      createGroup: vi.fn(),
      updateGroup: vi.fn(),
      deleteGroup: vi.fn(),
      toggleGroup: vi.fn(),
    };

    vi.mocked(useGroupStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useGroups());

    expect(result.current.loading).toBe(true);
  });

  it("should handle error state", () => {
    const errorMessage = "Failed to load groups";
    const mockStore = {
      groups: [],
      selectedGroupId: null,
      loading: false,
      error: errorMessage,
      loadGroups: vi.fn(),
      selectGroup: vi.fn(),
      createGroup: vi.fn(),
      updateGroup: vi.fn(),
      deleteGroup: vi.fn(),
      toggleGroup: vi.fn(),
    };

    vi.mocked(useGroupStore).mockReturnValue(mockStore);

    const { result } = renderHook(() => useGroups());

    expect(result.current.error).toBe(errorMessage);
  });
});
