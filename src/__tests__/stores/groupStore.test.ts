import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useGroupStore } from "@/stores/groupStore";
import * as api from "@/lib/tauri";
import type { Group, CreateGroupInput, UpdateGroupInput } from "@/lib/types";

// Mock the tauri API
vi.mock("@/lib/tauri");

const createMockGroup = (overrides: Partial<Group> = {}): Group => ({
  id: "group-id-1",
  name: "Test Group",
  description: "Test group description",
  enabled: true,
  createdAt: "2024-01-01T00:00:00Z",
  modifiedAt: "2024-01-01T00:00:00Z",
  ...overrides,
});

describe("groupStore", () => {
  beforeEach(() => {
    // Reset store state before each test
    const { getState } = useGroupStore;
    act(() => {
      getState().groups = [];
      getState().selectedGroupId = null;
      getState().loading = false;
      getState().error = null;
    });
    vi.clearAllMocks();
  });

  describe("loadGroups", () => {
    it("should load groups successfully", async () => {
      const mockGroups = [createMockGroup(), createMockGroup({ id: "group-id-2" })];
      vi.mocked(api.getAllGroups).mockResolvedValue(mockGroups);

      const { result } = renderHook(() => useGroupStore());

      await act(async () => {
        await result.current.loadGroups();
      });

      expect(result.current.groups).toEqual(mockGroups);
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBe(null);
    });

    it("should set loading state during fetch", async () => {
      vi.mocked(api.getAllGroups).mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve([]), 100))
      );

      const { result } = renderHook(() => useGroupStore());

      act(() => {
        result.current.loadGroups();
      });

      expect(result.current.loading).toBe(true);

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });
    });

    it("should handle errors when loading groups fails", async () => {
      const errorMessage = "Failed to load groups";
      vi.mocked(api.getAllGroups).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useGroupStore());

      await act(async () => {
        await result.current.loadGroups();
      });

      expect(result.current.error).toBe(errorMessage);
      expect(result.current.loading).toBe(false);
    });
  });

  describe("selectGroup", () => {
    it("should select a group by id", () => {
      const { result } = renderHook(() => useGroupStore());

      act(() => {
        result.current.selectGroup("group-id-1");
      });

      expect(result.current.selectedGroupId).toBe("group-id-1");
    });

    it("should deselect when passed null", () => {
      const { result } = renderHook(() => useGroupStore());

      act(() => {
        result.current.selectGroup("group-id-1");
        result.current.selectGroup(null);
      });

      expect(result.current.selectedGroupId).toBe(null);
    });
  });

  describe("createGroup", () => {
    it("should create a new group and add to store", async () => {
      const input: CreateGroupInput = {
        name: "New Group",
        description: "New group description",
        enabled: true,
      };
      const createdGroup = createMockGroup({ ...input, id: "new-group-id" });
      vi.mocked(api.createGroup).mockResolvedValue(createdGroup);

      const { result } = renderHook(() => useGroupStore());

      let returnedGroup: Group | undefined;
      await act(async () => {
        returnedGroup = await result.current.createGroup(input);
      });

      expect(returnedGroup).toEqual(createdGroup);
      expect(result.current.groups).toContainEqual(createdGroup);
    });

    it("should handle errors when creating group fails", async () => {
      const errorMessage = "Failed to create group";
      vi.mocked(api.createGroup).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useGroupStore());

      const input: CreateGroupInput = {
        name: "New Group",
        description: "",
        enabled: true,
      };

      await expect(
        act(async () => {
          await result.current.createGroup(input);
        })
      ).rejects.toThrow();
    });
  });

  describe("updateGroup", () => {
    it("should update an existing group", async () => {
      const originalGroup = createMockGroup();
      const update: UpdateGroupInput = { name: "Updated Group Name" };
      const updatedGroup = { ...originalGroup, ...update };

      vi.mocked(api.updateGroup).mockResolvedValue(updatedGroup);

      const { result } = renderHook(() => useGroupStore());

      act(() => {
        result.current.groups = [originalGroup];
      });

      let returnedGroup: Group | undefined;
      await act(async () => {
        returnedGroup = await result.current.updateGroup(originalGroup.id, update);
      });

      expect(returnedGroup).toEqual(updatedGroup);
      expect(result.current.groups[0]).toEqual(updatedGroup);
    });

    it("should handle errors when updating group fails", async () => {
      const errorMessage = "Failed to update group";
      vi.mocked(api.updateGroup).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useGroupStore());

      await expect(
        act(async () => {
          await result.current.updateGroup("test-id", { name: "Updated" });
        })
      ).rejects.toThrow();
    });
  });

  describe("deleteGroup", () => {
    it("should delete a group from the store", async () => {
      const group1 = createMockGroup({ id: "group-1" });
      const group2 = createMockGroup({ id: "group-2" });

      vi.mocked(api.deleteGroup).mockResolvedValue(undefined);

      const { result } = renderHook(() => useGroupStore());

      act(() => {
        result.current.groups = [group1, group2];
      });

      await act(async () => {
        await result.current.deleteGroup("group-1");
      });

      expect(result.current.groups).toEqual([group2]);
      expect(result.current.groups).not.toContainEqual(group1);
    });

    it("should deselect group if it was selected", async () => {
      const group = createMockGroup({ id: "group-1" });
      vi.mocked(api.deleteGroup).mockResolvedValue(undefined);

      const { result } = renderHook(() => useGroupStore());

      act(() => {
        result.current.groups = [group];
        result.current.selectedGroupId = "group-1";
      });

      await act(async () => {
        await result.current.deleteGroup("group-1");
      });

      expect(result.current.selectedGroupId).toBe(null);
    });

    it("should handle errors when deleting group fails", async () => {
      const errorMessage = "Failed to delete group";
      vi.mocked(api.deleteGroup).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useGroupStore());

      await expect(
        act(async () => {
          await result.current.deleteGroup("test-id");
        })
      ).rejects.toThrow();
    });
  });

  describe("toggleGroup", () => {
    it("should toggle group enabled state", async () => {
      const group = createMockGroup({ id: "group-1", enabled: true });
      vi.mocked(api.toggleGroup).mockResolvedValue(false);

      const { result } = renderHook(() => useGroupStore());

      act(() => {
        result.current.groups = [group];
      });

      let newState: boolean | undefined;
      await act(async () => {
        newState = await result.current.toggleGroup("group-1");
      });

      expect(newState).toBe(false);
      expect(result.current.groups[0].enabled).toBe(false);
    });

    it("should handle errors when toggling group fails", async () => {
      const errorMessage = "Failed to toggle group";
      vi.mocked(api.toggleGroup).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useGroupStore());

      await expect(
        act(async () => {
          await result.current.toggleGroup("test-id");
        })
      ).rejects.toThrow();
    });
  });
});
