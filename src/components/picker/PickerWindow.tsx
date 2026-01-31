// PickerWindow - Standalone search-focused window for quick combo selection
import { useEffect, useRef, useCallback } from "react";
import { Search, X, ArrowUpDown } from "lucide-react";
import { usePicker } from "@/hooks/usePicker";
import { useGroupStore } from "@/stores/groupStore";
import * as api from "@/lib/tauri";
import { SearchResultItem } from "./SearchResultItem";

export function PickerWindow() {
  const {
    query,
    results,
    selectedIndex,
    loading,
    error,
    search,
    clearSearch,
    moveSelection,
    setSelectedIndex,
    getSelectedCombo,
  } = usePicker();

  const { groups } = useGroupStore();
  const searchInputRef = useRef<HTMLInputElement>(null);
  const resultsContainerRef = useRef<HTMLDivElement>(null);

  // Auto-focus search input on mount
  useEffect(() => {
    searchInputRef.current?.focus();
  }, []);

  // Scroll selected item into view
  useEffect(() => {
    if (resultsContainerRef.current) {
      const selectedElement = resultsContainerRef.current.querySelector(
        `[data-index="${selectedIndex}"]`
      );
      if (selectedElement && typeof selectedElement.scrollIntoView === "function") {
        selectedElement.scrollIntoView({
          block: "nearest",
          behavior: "smooth",
        });
      }
    }
  }, [selectedIndex]);

  // Handle search input change
  const handleSearchChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      search(e.target.value);
    },
    [search]
  );

  // Handle clear button click
  const handleClear = useCallback(() => {
    clearSearch();
    searchInputRef.current?.focus();
  }, [clearSearch]);

  // Handle Enter key - trigger substitution
  const handleInsert = useCallback(async () => {
    const selectedCombo = getSelectedCombo();
    if (!selectedCombo) return;

    try {
      await api.triggerComboExpansion(selectedCombo.id);
      await api.closePicker();
    } catch (error) {
      console.error("Failed to trigger combo expansion:", error);
    }
  }, [getSelectedCombo]);

  // Handle Ctrl+Enter - copy to clipboard
  const handleCopy = useCallback(async () => {
    const selectedCombo = getSelectedCombo();
    if (!selectedCombo) return;

    try {
      await api.copySnippetToClipboard(selectedCombo.id);
      // Could show a brief toast here
    } catch (error) {
      console.error("Failed to copy snippet:", error);
    }
  }, [getSelectedCombo]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          moveSelection("down");
          break;
        case "ArrowUp":
          e.preventDefault();
          moveSelection("up");
          break;
        case "Enter":
          e.preventDefault();
          if (e.ctrlKey || e.metaKey) {
            handleCopy();
          } else {
            handleInsert();
          }
          break;
        case "Escape":
          e.preventDefault();
          api.closePicker();
          break;
      }
    },
    [moveSelection, handleInsert, handleCopy]
  );

  // Handle result item click
  const handleResultClick = useCallback(
    (index: number) => {
      setSelectedIndex(index);
    },
    [setSelectedIndex]
  );

  // Handle result item double-click
  const handleResultDoubleClick = useCallback(() => {
    handleInsert();
  }, [handleInsert]);

  // Get group name for a combo
  const getGroupName = useCallback(
    (groupId: string) => {
      const group = groups.find((g) => g.id === groupId);
      return group?.name || "Unknown";
    },
    [groups]
  );

  return (
    <div className="flex flex-col h-screen bg-white">
      {/* Search Header */}
      <div className="flex-shrink-0 p-4 border-b bg-gray-50">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search combos..."
            value={query}
            onChange={handleSearchChange}
            onKeyDown={handleKeyDown}
            className="w-full pl-10 pr-10 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-base"
            autoComplete="off"
            spellCheck={false}
          />
          {query && (
            <button
              onClick={handleClear}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600 focus:outline-none"
              aria-label="Clear search"
            >
              <X className="w-5 h-5" />
            </button>
          )}
        </div>

        {/* Keyboard hints */}
        <div className="mt-3 flex items-center gap-4 text-xs text-gray-500">
          <div className="flex items-center gap-1">
            <ArrowUpDown className="w-3 h-3" />
            <span>Navigate</span>
          </div>
          <div className="flex items-center gap-1">
            <kbd className="px-1.5 py-0.5 bg-white border border-gray-300 rounded text-xs">Enter</kbd>
            <span>Insert</span>
          </div>
          <div className="flex items-center gap-1">
            <kbd className="px-1.5 py-0.5 bg-white border border-gray-300 rounded text-xs">Ctrl+Enter</kbd>
            <span>Copy</span>
          </div>
          <div className="flex items-center gap-1">
            <kbd className="px-1.5 py-0.5 bg-white border border-gray-300 rounded text-xs">Esc</kbd>
            <span>Close</span>
          </div>
        </div>
      </div>

      {/* Results List */}
      <div
        ref={resultsContainerRef}
        className="flex-1 overflow-y-auto overscroll-contain"
      >
        {loading && (
          <div className="flex items-center justify-center h-32 text-gray-500">
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
              <span>Searching...</span>
            </div>
          </div>
        )}

        {error && (
          <div className="flex items-center justify-center h-32 text-red-500">
            <div className="text-center">
              <div className="font-semibold">Error</div>
              <div className="text-sm">{error}</div>
            </div>
          </div>
        )}

        {!loading && !error && results.length === 0 && (
          <div className="flex items-center justify-center h-32 text-gray-400">
            {query ? "No combos found" : "Type to search..."}
          </div>
        )}

        {!loading && !error && results.length > 0 && (
          <div className="divide-y divide-gray-100">
            {results.map((combo, index) => (
              <SearchResultItem
                key={combo.id}
                combo={combo}
                groupName={getGroupName(combo.groupId)}
                index={index}
                isSelected={index === selectedIndex}
                onClick={() => handleResultClick(index)}
                onDoubleClick={handleResultDoubleClick}
                onInsert={handleInsert}
                onCopy={handleCopy}
              />
            ))}
          </div>
        )}
      </div>

      {/* Footer - Result count */}
      {results.length > 0 && !loading && (
        <div className="flex-shrink-0 px-4 py-2 border-t bg-gray-50 text-xs text-gray-500">
          {results.length} {results.length === 1 ? "result" : "results"}
        </div>
      )}
    </div>
  );
}
