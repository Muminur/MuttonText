// ComboList - Displays combos in a virtualized grid view with sorting and filtering
import { useState, useMemo, useCallback, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Search, ChevronUp, ChevronDown } from "lucide-react";
import { useComboStore } from "../../stores/comboStore";
import { ComboItem } from "./ComboItem";

type SortField = "name" | "keyword" | "lastUsed" | "useCount";
type SortDirection = "asc" | "desc";

export function ComboList() {
  const { combos, loading, error, selectedIds, selectAll, clearSelection } = useComboStore();

  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [sortField, setSortField] = useState<SortField>("name");
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");

  const parentRef = useRef<HTMLDivElement>(null);

  // Debounced search (300ms)
  const handleSearchChange = useCallback((value: string) => {
    setSearchQuery(value);
    const timeout = setTimeout(() => {
      setDebouncedQuery(value);
    }, 300);
    return () => clearTimeout(timeout);
  }, []);

  // Filter and sort combos
  const filteredAndSortedCombos = useMemo(() => {
    let result = [...combos];

    // Filter by search query
    if (debouncedQuery.trim()) {
      const query = debouncedQuery.toLowerCase();
      result = result.filter(
        (combo) =>
          combo.name.toLowerCase().includes(query) ||
          combo.keyword.toLowerCase().includes(query) ||
          combo.description.toLowerCase().includes(query) ||
          combo.snippet.toLowerCase().includes(query)
      );
    }

    // Sort
    result.sort((a, b) => {
      let aValue: string | number = "";
      let bValue: string | number = "";

      switch (sortField) {
        case "name":
          aValue = a.name.toLowerCase();
          bValue = b.name.toLowerCase();
          break;
        case "keyword":
          aValue = a.keyword.toLowerCase();
          bValue = b.keyword.toLowerCase();
          break;
        case "lastUsed":
          aValue = a.lastUsed ? new Date(a.lastUsed).getTime() : 0;
          bValue = b.lastUsed ? new Date(b.lastUsed).getTime() : 0;
          break;
        case "useCount":
          aValue = a.useCount;
          bValue = b.useCount;
          break;
      }

      if (aValue < bValue) return sortDirection === "asc" ? -1 : 1;
      if (aValue > bValue) return sortDirection === "asc" ? 1 : -1;
      return 0;
    });

    return result;
  }, [combos, debouncedQuery, sortField, sortDirection]);

  // Virtualizer for performance
  const rowVirtualizer = useVirtualizer({
    count: filteredAndSortedCombos.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 56,
    overscan: 5,
  });

  // Handle column header click for sorting
  const handleSort = (field: SortField) => {
    if (sortField === field) {
      // Toggle direction
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      // New field, default to ascending
      setSortField(field);
      setSortDirection("asc");
    }
  };

  // Render sort indicator
  const SortIndicator = ({ field }: { field: SortField }) => {
    if (sortField !== field) return null;
    return sortDirection === "asc" ? (
      <ChevronUp className="inline w-4 h-4 ml-1" />
    ) : (
      <ChevronDown className="inline w-4 h-4 ml-1" />
    );
  };

  // Handle select all/none
  const handleSelectAll = () => {
    const allIds = filteredAndSortedCombos.map((c) => c.id);
    if (selectedIds.size === allIds.length) {
      clearSelection();
    } else {
      selectAll(allIds);
    }
  };

  // Check if all visible combos are selected
  const allSelected = filteredAndSortedCombos.length > 0 &&
    filteredAndSortedCombos.every((c) => selectedIds.has(c.id));
  const someSelected = filteredAndSortedCombos.some((c) => selectedIds.has(c.id)) && !allSelected;

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-gray-500">Loading combos...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-red-500">
          <div className="font-semibold">Error loading combos</div>
          <div className="text-sm">{error}</div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Search bar */}
      <div className="p-4 border-b">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search combos..."
            value={searchQuery}
            onChange={(e) => handleSearchChange(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Virtualized list */}
      <div className="flex-1 overflow-hidden flex flex-col">
        {filteredAndSortedCombos.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-gray-500">
              {debouncedQuery ? "No combos found matching your search" : "No combos found"}
            </div>
          </div>
        ) : (
          <>
            {/* Header - grid layout matching rows */}
            <div
              className="grid items-center bg-gray-50 border-b px-4 py-3 sticky top-0 z-10"
              style={{ gridTemplateColumns: "40px 1fr 120px 1fr 120px 120px 80px" }}
            >
              <div className="flex items-center justify-center">
                <input
                  type="checkbox"
                  checked={allSelected}
                  ref={(input) => {
                    if (input) {
                      input.indeterminate = someSelected;
                    }
                  }}
                  onChange={handleSelectAll}
                  className="w-4 h-4 cursor-pointer"
                  title={allSelected ? "Deselect all" : "Select all"}
                />
              </div>
              <button
                onClick={() => handleSort("name")}
                className="text-left font-medium hover:text-blue-600"
              >
                Name
                <SortIndicator field="name" />
              </button>
              <button
                onClick={() => handleSort("keyword")}
                className="text-left font-medium hover:text-blue-600"
              >
                Keyword
                <SortIndicator field="keyword" />
              </button>
              <div className="text-left font-medium">Snippet</div>
              <div className="text-left font-medium">Group</div>
              <button
                onClick={() => handleSort("lastUsed")}
                className="text-left font-medium hover:text-blue-600"
              >
                Last Used
                <SortIndicator field="lastUsed" />
              </button>
              <div className="text-center font-medium">Enabled</div>
            </div>

            {/* Virtualized body */}
            <div
              ref={parentRef}
              className="flex-1 overflow-auto"
            >
              <div
                style={{
                  height: `${rowVirtualizer.getTotalSize()}px`,
                  width: "100%",
                  position: "relative",
                }}
              >
                {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                  const combo = filteredAndSortedCombos[virtualRow.index];
                  return (
                    <div
                      key={virtualRow.key}
                      data-index={virtualRow.index}
                      ref={rowVirtualizer.measureElement}
                      style={{
                        position: "absolute",
                        top: 0,
                        left: 0,
                        width: "100%",
                        transform: `translateY(${virtualRow.start}px)`,
                      }}
                    >
                      <ComboItem combo={combo} />
                    </div>
                  );
                })}
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
