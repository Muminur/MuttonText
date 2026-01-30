import React from "react";
import { PlusIcon } from "lucide-react";
import { GroupList } from "./GroupList";
import { GroupEditor } from "./GroupEditor";
import { Group, CreateGroupInput } from "@/lib/types";

/**
 * Sidebar component containing the group list and "Add Group" button.
 * Fixed width, scrollable if content overflows.
 */
export const Sidebar: React.FC = () => {
  const [groups] = React.useState<Group[]>([]);
  const [selectedGroupId, setSelectedGroupId] = React.useState<string | null>(null);
  const [editorOpen, setEditorOpen] = React.useState(false);
  const [editingGroup, setEditingGroup] = React.useState<Group | undefined>(undefined);

  const handleSelectGroup = (groupId: string | null) => {
    setSelectedGroupId(groupId);
  };

  const handleEditGroup = (group: Group) => {
    setEditingGroup(group);
    setEditorOpen(true);
  };

  const handleDeleteGroup = (groupId: string) => {
    console.log("Delete group:", groupId);
    // TODO: Implement deletion via groupStore
  };

  const handleToggleGroup = (groupId: string) => {
    console.log("Toggle group:", groupId);
    // TODO: Implement toggle via groupStore
  };

  const handleReorderGroups = (reorderedGroups: Group[]) => {
    console.log("Reorder groups:", reorderedGroups);
    // TODO: Implement reorder via groupStore
  };

  const handleSaveGroup = (data: CreateGroupInput) => {
    console.log("Save group:", data, editingGroup);
    // TODO: Implement create/update via groupStore
    setEditorOpen(false);
    setEditingGroup(undefined);
  };

  const handleAddGroup = () => {
    setEditingGroup(undefined);
    setEditorOpen(true);
  };

  const getGroupComboCount = (_groupId: string) => {
    // TODO: Get from comboStore
    return 0;
  };

  return (
    <>
      <div className="flex h-full flex-col bg-gray-50">
        {/* Group list area */}
        <div className="flex-1 overflow-auto p-2" data-testid="group-list-area">
          <GroupList
            groups={groups}
            selectedGroupId={selectedGroupId}
            onSelectGroup={handleSelectGroup}
            onEditGroup={handleEditGroup}
            onDeleteGroup={handleDeleteGroup}
            onToggleGroup={handleToggleGroup}
            onReorderGroups={handleReorderGroups}
            totalComboCount={0}
            getGroupComboCount={getGroupComboCount}
          />
        </div>

        {/* Add Group button at bottom */}
        <div className="border-t bg-white p-2">
          <button
            className="flex w-full items-center justify-center gap-1 rounded bg-blue-500 py-2 text-sm text-white hover:bg-blue-600"
            onClick={handleAddGroup}
            data-testid="add-group-button"
          >
            <PlusIcon size={16} />
            Add Group
          </button>
        </div>
      </div>

      {/* Group Editor Dialog */}
      <GroupEditor
        open={editorOpen}
        onClose={() => {
          setEditorOpen(false);
          setEditingGroup(undefined);
        }}
        onSave={handleSaveGroup}
        group={editingGroup}
      />
    </>
  );
};
