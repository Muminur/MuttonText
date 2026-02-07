// ComboItem - Grid row for a single combo with context menu
import {
  ContextMenu,
  ContextMenuTrigger,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "@radix-ui/react-context-menu";
import * as Tooltip from "@radix-ui/react-tooltip";
import { Edit, Copy, FolderOpen, Trash2, Power, PowerOff } from "lucide-react";
import { useComboStore } from "../../stores/comboStore";
import { useGroupStore } from "../../stores/groupStore";
import type { Combo } from "../../lib/types";

interface ComboItemProps {
  combo: Combo;
  onEdit?: (combo: Combo) => void;
}

export function ComboItem({ combo, onEdit }: ComboItemProps) {
  const {
    selectedIds,
    selectCombo,
    duplicateCombo,
    deleteCombo,
    toggleCombo,
  } = useComboStore();
  const { groups } = useGroupStore();

  const group = groups.find((g) => g.id === combo.groupId);
  const isSelected = selectedIds.has(combo.id);

  // Format last used date
  const formatLastUsed = (lastUsed: string | null) => {
    if (!lastUsed) return "Never";
    const date = new Date(lastUsed);
    return date.toLocaleDateString();
  };

  // Truncate snippet for display
  const truncateSnippet = (snippet: string, maxLength = 50) => {
    if (snippet.length <= maxLength) return snippet;
    return snippet.slice(0, maxLength) + "...";
  };

  // Handle row click with multi-select support
  const handleClick = (e: React.MouseEvent) => {
    // Ctrl+Click: toggle selection
    // Shift+Click: range selection
    // Regular click: single select
    selectCombo(combo.id, {
      ctrl: e.ctrlKey || e.metaKey,
      shift: e.shiftKey,
    });
  };

  // Handle checkbox click
  const handleCheckboxClick = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent row click
    selectCombo(combo.id, { ctrl: true }); // Toggle selection
  };

  // Handle double click to edit
  const handleDoubleClick = () => {
    if (onEdit) {
      onEdit(combo);
    }
  };

  // Context menu actions
  const handleDuplicate = async () => {
    await duplicateCombo(combo.id);
  };

  const handleDelete = async () => {
    if (confirm(`Delete combo "${combo.name}"?`)) {
      await deleteCombo(combo.id);
    }
  };

  const handleToggleEnabled = async () => {
    await toggleCombo(combo.id);
  };

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          className={`grid items-center border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer px-4 py-2 ${
            isSelected ? "bg-blue-50 dark:bg-blue-900/30" : ""
          }`}
          style={{ gridTemplateColumns: "40px 1fr 120px 1fr 120px 120px 80px" }}
          onClick={handleClick}
          onDoubleClick={handleDoubleClick}
          role="listitem"
          aria-selected={isSelected}
          aria-label={`${combo.name}, keyword: ${combo.keyword}`}
        >
          <div className="flex items-center justify-center py-1">
            <input
              type="checkbox"
              checked={isSelected}
              onChange={() => {}} // Controlled by onClick
              onClick={handleCheckboxClick}
              className="w-4 h-4 cursor-pointer"
            />
          </div>
          <div className="py-1">
            <div className="font-medium">{combo.name}</div>
            {combo.description && (
              <div className="text-sm text-gray-500">{combo.description}</div>
            )}
          </div>
          <div className="py-1">
            <code className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-sm font-mono">
              {combo.keyword}
            </code>
          </div>
          <div className="text-sm text-gray-600 font-mono py-1">
            {truncateSnippet(combo.snippet)}
          </div>
          <div className="text-sm py-1">{group?.name || "Unknown"}</div>
          <div className="text-sm py-1">
            <Tooltip.Provider>
              <Tooltip.Root>
                <Tooltip.Trigger asChild>
                  <span className="cursor-help">{formatLastUsed(combo.lastUsed)}</span>
                </Tooltip.Trigger>
                <Tooltip.Portal>
                  <Tooltip.Content
                    className="bg-gray-900 text-white px-3 py-2 rounded text-sm"
                    sideOffset={5}
                  >
                    Used {combo.useCount} times
                    <Tooltip.Arrow className="fill-gray-900" />
                  </Tooltip.Content>
                </Tooltip.Portal>
              </Tooltip.Root>
            </Tooltip.Provider>
          </div>
          <div className="text-center py-1">
            <input
              type="checkbox"
              checked={combo.enabled}
              readOnly
              className="w-4 h-4"
            />
          </div>
        </div>
      </ContextMenuTrigger>

      <ContextMenuContent className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600 rounded-lg shadow-lg p-1 min-w-[200px]">
        <ContextMenuItem
          className="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded cursor-pointer flex items-center gap-2 dark:text-gray-100"
          onSelect={() => onEdit?.(combo)}
        >
          <Edit className="w-4 h-4" />
          Edit
        </ContextMenuItem>
        <ContextMenuItem
          className="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded cursor-pointer flex items-center gap-2 dark:text-gray-100"
          onSelect={handleDuplicate}
        >
          <Copy className="w-4 h-4" />
          Duplicate
        </ContextMenuItem>
        <ContextMenuItem
          className="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded cursor-pointer flex items-center gap-2 dark:text-gray-100"
          onSelect={() => console.log("Move to Group")}
        >
          <FolderOpen className="w-4 h-4" />
          Move to Group
        </ContextMenuItem>
        <ContextMenuSeparator className="h-px bg-gray-200 dark:bg-gray-600 my-1" />
        <ContextMenuItem
          className="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded cursor-pointer flex items-center gap-2 dark:text-gray-100"
          onSelect={handleToggleEnabled}
        >
          {combo.enabled ? (
            <>
              <PowerOff className="w-4 h-4" />
              Disable
            </>
          ) : (
            <>
              <Power className="w-4 h-4" />
              Enable
            </>
          )}
        </ContextMenuItem>
        <ContextMenuSeparator className="h-px bg-gray-200 dark:bg-gray-600 my-1" />
        <ContextMenuItem
          className="px-3 py-2 hover:bg-red-100 dark:hover:bg-red-900/30 text-red-600 dark:text-red-400 rounded cursor-pointer flex items-center gap-2"
          onSelect={handleDelete}
        >
          <Trash2 className="w-4 h-4" />
          Delete
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
