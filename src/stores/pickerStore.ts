// Picker Store - Zustand state management for combo picker window
import { create } from "zustand";
import type { Combo } from "@/lib/types";

interface PickerState {
  // State
  isOpen: boolean;
  query: string;
  results: Combo[];
  selectedIndex: number;
  loading: boolean;
  error: string | null;
  windowSize: { width: number; height: number };

  // Actions
  setIsOpen: (isOpen: boolean) => void;
  setQuery: (query: string) => void;
  setResults: (results: Combo[]) => void;
  setSelectedIndex: (index: number) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setWindowSize: (size: { width: number; height: number }) => void;
  moveSelection: (direction: "up" | "down") => void;
  reset: () => void;
}

const DEFAULT_WINDOW_SIZE = { width: 500, height: 400 };

// Load window size from localStorage
function loadWindowSize(): { width: number; height: number } {
  try {
    const stored = localStorage.getItem("picker-window-size");
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        width: parsed.width || DEFAULT_WINDOW_SIZE.width,
        height: parsed.height || DEFAULT_WINDOW_SIZE.height,
      };
    }
  } catch (error) {
    console.error("Failed to load picker window size:", error);
  }
  return DEFAULT_WINDOW_SIZE;
}

// Save window size to localStorage
function saveWindowSize(size: { width: number; height: number }) {
  try {
    localStorage.setItem("picker-window-size", JSON.stringify(size));
  } catch (error) {
    console.error("Failed to save picker window size:", error);
  }
}

export const usePickerStore = create<PickerState>((set, get) => ({
  // Initial state
  isOpen: false,
  query: "",
  results: [],
  selectedIndex: 0,
  loading: false,
  error: null,
  windowSize: loadWindowSize(),

  // Set picker open state
  setIsOpen: (isOpen) => {
    set({ isOpen });
    if (!isOpen) {
      // Reset state when closing
      get().reset();
    }
  },

  // Set search query
  setQuery: (query) => set({ query }),

  // Set search results
  setResults: (results) => {
    set({ results, selectedIndex: 0 }); // Reset selection to first result
  },

  // Set selected index
  setSelectedIndex: (index) => {
    const { results } = get();
    if (index >= 0 && index < results.length) {
      set({ selectedIndex: index });
    }
  },

  // Set loading state
  setLoading: (loading) => set({ loading }),

  // Set error
  setError: (error) => set({ error }),

  // Set window size and persist
  setWindowSize: (size) => {
    set({ windowSize: size });
    saveWindowSize(size);
  },

  // Move selection up or down with wrapping
  moveSelection: (direction) => {
    const { selectedIndex, results } = get();
    if (results.length === 0) return;

    let newIndex = selectedIndex;
    if (direction === "up") {
      newIndex = selectedIndex > 0 ? selectedIndex - 1 : results.length - 1;
    } else {
      newIndex = selectedIndex < results.length - 1 ? selectedIndex + 1 : 0;
    }
    set({ selectedIndex: newIndex });
  },

  // Reset to initial state
  reset: () =>
    set({
      query: "",
      results: [],
      selectedIndex: 0,
      loading: false,
      error: null,
    }),
}));
