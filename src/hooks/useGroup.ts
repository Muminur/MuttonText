import { useEffect } from "react";
import { useGroupStore } from "@/stores/groupStore";

/**
 * Hook to access a specific group by ID
 * Also provides access to all store methods and state
 */
export function useGroup(id?: string) {
  const store = useGroupStore();
  const group = id ? store.groups.find((g) => g.id === id) : undefined;

  return {
    group,
    ...store,
  };
}

/**
 * Hook to load and access all groups
 * Automatically loads groups on mount
 */
export function useGroups() {
  const { groups, loading, error, loadGroups } = useGroupStore();

  useEffect(() => {
    loadGroups();
  }, [loadGroups]);

  return {
    groups,
    loading,
    error,
    refetch: loadGroups,
  };
}
