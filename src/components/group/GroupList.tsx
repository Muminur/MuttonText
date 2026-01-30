import React from "react";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { GroupItem } from "./GroupItem";
import { Group } from "@/lib/types";

interface GroupListProps {
  groups: Group[];
  selectedGroupId: string | null;
  onSelectGroup: (groupId: string | null) => void;
  onEditGroup: (group: Group) => void;
  onDeleteGroup: (groupId: string) => void;
  onToggleGroup: (groupId: string) => void;
  onReorderGroups: (groups: Group[]) => void;
  totalComboCount: number;
  getGroupComboCount: (groupId: string) => number;
}

/**
 * Renders list of groups with "All Combos" virtual group at top.
 * Each group shows name and combo count.
 * Supports drag-and-drop reordering of groups.
 */
export const GroupList: React.FC<GroupListProps> = ({
  groups,
  selectedGroupId,
  onSelectGroup,
  onEditGroup,
  onDeleteGroup,
  onToggleGroup,
  onReorderGroups,
  totalComboCount,
  getGroupComboCount,
}) => {
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id) {
      const oldIndex = groups.findIndex((group) => group.id === active.id);
      const newIndex = groups.findIndex((group) => group.id === over.id);

      const reorderedGroups = arrayMove(groups, oldIndex, newIndex);
      onReorderGroups(reorderedGroups);
    }
  };

  return (
    <div className="space-y-1" data-testid="group-list">
      {/* "All Combos" virtual group - not draggable */}
      <div
        className={`cursor-pointer rounded px-3 py-2 ${
          selectedGroupId === null
            ? "bg-blue-500 text-white"
            : "hover:bg-gray-200"
        }`}
        onClick={() => onSelectGroup(null)}
        data-testid="all-combos-group"
      >
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">All Combos</span>
          <span
            className={`rounded-full px-2 py-0.5 text-xs ${
              selectedGroupId === null
                ? "bg-blue-600 text-white"
                : "bg-gray-200 text-gray-700"
            }`}
          >
            {totalComboCount}
          </span>
        </div>
      </div>

      {/* Regular groups - draggable */}
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext
          items={groups.map((group) => group.id)}
          strategy={verticalListSortingStrategy}
        >
          {groups.map((group) => (
            <GroupItem
              key={group.id}
              group={group}
              comboCount={getGroupComboCount(group.id)}
              isSelected={selectedGroupId === group.id}
              onSelect={() => onSelectGroup(group.id)}
              onEdit={() => onEditGroup(group)}
              onDelete={() => onDeleteGroup(group.id)}
              onToggle={() => onToggleGroup(group.id)}
            />
          ))}
        </SortableContext>
      </DndContext>
    </div>
  );
};
