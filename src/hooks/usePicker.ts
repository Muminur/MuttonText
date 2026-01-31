import { useCallback, useEffect, useRef } from "react";
import { usePickerStore } from "@/stores/pickerStore";
import * as api from "@/lib/tauri";

/**
 * Hook to manage picker state and search functionality
 * Provides debounced search and keyboard navigation
 */
export function usePicker() {
  const {
    setQuery,
    setResults,
    setLoading,
    setError,
    setSelectedIndex,
    ...store
  } = usePickerStore();
  const debounceTimerRef = useRef<number | null>(null);

  // Debounced search function (150ms)
  const search = useCallback(
    async (query: string) => {
      setQuery(query);

      // Clear previous timer
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }

      // Debounce search
      debounceTimerRef.current = setTimeout(async () => {
        if (!query.trim()) {
          setResults([]);
          setLoading(false);
          return;
        }

        setLoading(true);
        setError(null);

        try {
          const results = await api.searchCombos(query);
          setResults(results);
        } catch (error) {
          const errorMessage =
            error instanceof Error ? error.message : "Failed to search combos";
          setError(errorMessage);
          setResults([]);
        } finally {
          setLoading(false);
        }
      }, 150);
    },
    [setQuery, setResults, setLoading, setError]
  );

  // Clear search
  const clearSearch = useCallback(() => {
    setQuery("");
    setResults([]);
    setSelectedIndex(0);
  }, [setQuery, setResults, setSelectedIndex]);

  // Get selected combo
  const getSelectedCombo = useCallback(() => {
    return store.results[store.selectedIndex] || null;
  }, [store.results, store.selectedIndex]);

  // Cleanup debounce timer on unmount
  useEffect(() => {
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, []);

  return {
    ...store,
    setQuery,
    setResults,
    setLoading,
    setError,
    setSelectedIndex,
    search,
    clearSearch,
    getSelectedCombo,
  };
}
