import React from "react";
import { PlusIcon } from "lucide-react";
import { GroupList } from "./GroupList";
import { GroupEditor } from "./GroupEditor";
import { Group, CreateGroupInput } from "@/lib/types";
import { useGroupStore } from "@/stores/groupStore";
import { useComboStore } from "@/stores/comboStore";

/**
 * Sidebar component containing the group list and "Add Group" button.
 * Fixed width, scrollable if content overflows.
 */
export const Sidebar: React.FC = () => {
  const { groups, selectedGroupId, selectGroup, createGroup, updateGroup, deleteGroup, toggleGroup, reorderGroups, loadGroups } = useGroupStore();
  const { combos, loadCombos } = useComboStore();
  const [editorOpen, setEditorOpen] = React.useState(false);
  const [editingGroup, setEditingGroup] = React.useState<Group | undefined>(undefined);

  // Load groups and combos on mount
  React.useEffect(() => {
    loadGroups();
    loadCombos();
  }, [loadGroups, loadCombos]);

  const handleSelectGroup = (groupId: string | null) => {
    selectGroup(groupId);
  };

  const handleEditGroup = (group: Group) => {
    setEditingGroup(group);
    setEditorOpen(true);
  };

  const handleDeleteGroup = async (groupId: string) => {
    try {
      await deleteGroup(groupId);
    } catch (error) {
      console.error("Failed to delete group:", error);
    }
  };

  const handleToggleGroup = async (groupId: string) => {
    try {
      await toggleGroup(groupId);
    } catch (error) {
      console.error("Failed to toggle group:", error);
    }
  };

  const handleReorderGroups = (reorderedGroups: Group[]) => {
    reorderGroups(reorderedGroups);
  };

  const handleSaveGroup = async (data: CreateGroupInput) => {
    try {
      if (editingGroup) {
        await updateGroup(editingGroup.id, data);
      } else {
        await createGroup(data);
      }
      setEditorOpen(false);
      setEditingGroup(undefined);
    } catch (error) {
      console.error("Failed to save group:", error);
    }
  };

  const handleAddGroup = () => {
    setEditingGroup(undefined);
    setEditorOpen(true);
  };

  const getGroupComboCount = (groupId: string) => {
    return combos.filter((combo) => combo.groupId === groupId).length;
  };

  const totalComboCount = combos.length;

  return (
    <>
      <div className="flex h-full flex-col bg-gray-50" role="navigation" aria-label="Group navigation">
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
            totalComboCount={totalComboCount}
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
