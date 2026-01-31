// SearchResultItem - Individual search result in picker window
import {
  ContextMenu,
  ContextMenuTrigger,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "@radix-ui/react-context-menu";
import { Copy, Edit, FileText } from "lucide-react";
import type { Combo } from "@/lib/types";

interface SearchResultItemProps {
  combo: Combo;
  groupName: string;
  index: number;
  isSelected: boolean;
  onClick: () => void;
  onDoubleClick: () => void;
  onInsert: () => void;
  onCopy: () => void;
}

export function SearchResultItem({
  combo,
  groupName,
  index,
  isSelected,
  onClick,
  onDoubleClick,
  onInsert,
  onCopy,
}: SearchResultItemProps) {
  // Truncate snippet to first line and max 60 chars
  const truncateSnippet = (snippet: string) => {
    const firstLine = snippet.split("\n")[0];
    const maxLength = 60;
    if (firstLine.length <= maxLength) return firstLine;
    return firstLine.slice(0, maxLength) + "...";
  };

  // Handle opening combo editor (navigate to main window)
  const handleEdit = () => {
    // This will be implemented when main window has routing
    console.log("Edit combo:", combo.id);
    // For now, just log - later this should switch to main window and open editor
  };

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          data-index={index}
          className={`px-4 py-3 cursor-pointer transition-colors ${
            isSelected
              ? "bg-blue-50 border-l-4 border-l-blue-500"
              : "hover:bg-gray-50 border-l-4 border-l-transparent"
          }`}
          onClick={onClick}
          onDoubleClick={onDoubleClick}
        >
          <div className="flex items-start justify-between gap-3">
            <div className="flex-1 min-w-0">
              {/* Combo name */}
              <div className="font-semibold text-gray-900 truncate">
                {combo.name}
              </div>

              {/* Keyword */}
              <div className="mt-1 flex items-center gap-2">
                <code className="px-2 py-0.5 bg-gray-100 text-blue-600 rounded text-sm font-mono">
                  {combo.keyword}
                </code>
                {/* Group badge */}
                <span className="px-2 py-0.5 bg-purple-100 text-purple-700 rounded text-xs font-medium">
                  {groupName}
                </span>
              </div>

              {/* Snippet preview */}
              <div className="mt-2 text-sm text-gray-500 font-mono truncate">
                {truncateSnippet(combo.snippet)}
              </div>
            </div>

            {/* Enabled indicator */}
            {!combo.enabled && (
              <div className="flex-shrink-0">
                <span className="px-2 py-1 bg-red-100 text-red-700 rounded text-xs font-medium">
                  Disabled
                </span>
              </div>
            )}
          </div>
        </div>
      </ContextMenuTrigger>

      <ContextMenuContent className="bg-white border border-gray-200 rounded-lg shadow-lg p-1 min-w-[180px]">
        <ContextMenuItem
          className="px-3 py-2 hover:bg-gray-100 rounded cursor-pointer flex items-center gap-2 text-sm"
          onSelect={onInsert}
        >
          <FileText className="w-4 h-4" />
          Insert
        </ContextMenuItem>
        <ContextMenuItem
          className="px-3 py-2 hover:bg-gray-100 rounded cursor-pointer flex items-center gap-2 text-sm"
          onSelect={onCopy}
        >
          <Copy className="w-4 h-4" />
          Copy to Clipboard
        </ContextMenuItem>
        <ContextMenuSeparator className="h-px bg-gray-200 my-1" />
        <ContextMenuItem
          className="px-3 py-2 hover:bg-gray-100 rounded cursor-pointer flex items-center gap-2 text-sm"
          onSelect={handleEdit}
        >
          <Edit className="w-4 h-4" />
          Edit Combo
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
