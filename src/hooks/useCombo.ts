import { useEffect } from "react";
import { useComboStore } from "@/stores/comboStore";

/**
 * Hook to access a specific combo by ID
 * Also provides access to all store methods and state
 */
export function useCombo(id?: string) {
  const store = useComboStore();
  const combo = id ? store.combos.find((c) => c.id === id) : undefined;

  return {
    combo,
    ...store,
  };
}

/**
 * Hook to load and access all combos
 * Automatically loads combos on mount
 */
export function useCombos() {
  const { combos, loading, error, loadCombos } = useComboStore();

  useEffect(() => {
    loadCombos();
  }, [loadCombos]);

  return {
    combos,
    loading,
    error,
    refetch: loadCombos,
  };
}
