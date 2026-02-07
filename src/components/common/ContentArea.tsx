import React from "react";
import { SearchIcon, PlusIcon, GridIcon, ListIcon } from "lucide-react";
import { ComboEditor } from "../combo/ComboEditor";
import { useComboStore } from "@/stores/comboStore";
import type { CreateComboInput } from "@/lib/types";

interface ContentAreaProps {
  children: React.ReactNode;
}

/**
 * Main content area that displays combo list and toolbar.
 * Contains search input, new combo button, and view toggle.
 */
export const ContentArea: React.FC<ContentAreaProps> = ({ children }) => {
  const [viewMode, setViewMode] = React.useState<"list" | "grid">("list");
  const [comboEditorOpen, setComboEditorOpen] = React.useState(false);
  const { createCombo } = useComboStore();

  const handleCreateCombo = async (data: CreateComboInput) => {
    try {
      await createCombo(data);
      setComboEditorOpen(false);
    } catch (error) {
      console.error("Failed to create combo:", error);
    }
  };

  const handleNewComboClick = () => {
    setComboEditorOpen(true);
  };

  return (
    <div className="flex h-full flex-col">
      {/* Toolbar */}
      <div className="flex items-center gap-2 border-b bg-white p-2">
        {/* Search input */}
        <div className="relative flex-1">
          <SearchIcon
            size={16}
            className="absolute left-2 top-1/2 -translate-y-1/2 text-gray-400"
          />
          <input
            type="text"
            placeholder="Search combos..."
            className="w-full rounded border border-gray-300 py-1 pl-8 pr-3 text-sm outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
            data-testid="search-input"
          />
        </div>

        {/* New combo button */}
        <button
          className="flex items-center gap-1 rounded bg-blue-500 px-3 py-1 text-sm text-white hover:bg-blue-600"
          onClick={handleNewComboClick}
          data-testid="new-combo-button"
        >
          <PlusIcon size={16} />
          New Combo
        </button>

        {/* View toggle */}
        <div className="flex rounded border border-gray-300">
          <button
            className={`p-1 ${viewMode === "list" ? "bg-gray-200" : "hover:bg-gray-100"}`}
            onClick={() => setViewMode("list")}
            aria-label="List view"
            data-testid="view-list-button"
          >
            <ListIcon size={16} />
          </button>
          <button
            className={`p-1 ${viewMode === "grid" ? "bg-gray-200" : "hover:bg-gray-100"}`}
            onClick={() => setViewMode("grid")}
            aria-label="Grid view"
            data-testid="view-grid-button"
          >
            <GridIcon size={16} />
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-4">{children}</div>

      {/* Combo Editor Dialog */}
      <ComboEditor
        open={comboEditorOpen}
        onSave={handleCreateCombo}
        onCancel={() => setComboEditorOpen(false)}
      />
    </div>
  );
};
