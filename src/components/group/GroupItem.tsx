import React from "react";
import * as ContextMenu from "@radix-ui/react-context-menu";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical } from "lucide-react";
import { Group } from "@/lib/types";

interface GroupItemProps {
  group: Group;
  comboCount: number;
  isSelected: boolean;
  onSelect: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onToggle: () => void;
}

/**
 * Individual group item with name, combo count badge, and context menu.
 * Highlighted when selected, double-click to edit.
 * Supports drag-and-drop reordering with drag handle.
 */
export const GroupItem: React.FC<GroupItemProps> = ({
  group,
  comboCount,
  isSelected,
  onSelect,
  onEdit,
  onDelete,
  onToggle,
}) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: group.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={isDragging ? "opacity-50" : ""}
    >
      <ContextMenu.Root>
        <ContextMenu.Trigger asChild>
          <div
            className={`cursor-pointer rounded px-3 py-2 ${
              isSelected ? "bg-blue-500 text-white" : "hover:bg-gray-200 dark:hover:bg-gray-700"
            } ${!group.enabled ? "opacity-50" : ""}`}
            onClick={onSelect}
            onDoubleClick={onEdit}
            data-testid={`group-item-${group.id}`}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {/* Drag handle */}
                <button
                  className={`cursor-grab active:cursor-grabbing ${
                    isSelected ? "text-white" : "text-gray-400"
                  } hover:text-gray-600 focus:outline-none`}
                  {...attributes}
                  {...listeners}
                  data-testid={`group-drag-handle-${group.id}`}
                >
                  <GripVertical size={16} />
                </button>
                <span className="text-sm font-medium">{group.name}</span>
              </div>
              <span
                className={`rounded-full px-2 py-0.5 text-xs ${
                  isSelected
                    ? "bg-blue-600 text-white"
                    : "bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300"
                }`}
              >
                {comboCount}
              </span>
            </div>
            {group.description && (
              <p
                className={`mt-1 truncate text-xs ${
                  isSelected ? "text-blue-100" : "text-gray-500 dark:text-gray-400"
                } ml-6`}
              >
                {group.description}
              </p>
            )}
          </div>
        </ContextMenu.Trigger>

        <ContextMenu.Portal>
          <ContextMenu.Content
            className="min-w-[180px] rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 shadow-md"
            data-testid="group-context-menu"
          >
            <ContextMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={onEdit}
            >
              Edit
            </ContextMenu.Item>
            <ContextMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={onToggle}
            >
              {group.enabled ? "Disable" : "Enable"}
            </ContextMenu.Item>
            <ContextMenu.Separator className="my-1 h-px bg-gray-200 dark:bg-gray-600" />
            <ContextMenu.Item
              className="cursor-pointer px-3 py-2 text-sm text-red-600 dark:text-red-400 outline-none hover:bg-gray-100 dark:hover:bg-gray-700"
              onSelect={onDelete}
            >
              Delete
            </ContextMenu.Item>
          </ContextMenu.Content>
        </ContextMenu.Portal>
      </ContextMenu.Root>
    </div>
  );
};
