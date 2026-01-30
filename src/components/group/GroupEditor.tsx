import React from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { Group, CreateGroupInput } from "@/lib/types";

interface GroupEditorProps {
  open: boolean;
  onClose: () => void;
  onSave: (data: CreateGroupInput) => void;
  group?: Group;
}

/**
 * Dialog for creating or editing a group.
 * Name field (required), Description field (optional).
 */
export const GroupEditor: React.FC<GroupEditorProps> = ({
  open,
  onClose,
  onSave,
  group,
}) => {
  const [name, setName] = React.useState(group?.name || "");
  const [description, setDescription] = React.useState(group?.description || "");
  const [errors, setErrors] = React.useState<{ name?: string }>({});

  React.useEffect(() => {
    if (group) {
      setName(group.name);
      setDescription(group.description);
    } else {
      setName("");
      setDescription("");
    }
    setErrors({});
  }, [group, open]);

  const handleSave = () => {
    // Validation
    const newErrors: { name?: string } = {};
    if (!name.trim()) {
      newErrors.name = "Name is required";
    }

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    onSave({
      name: name.trim(),
      description: description.trim(),
      enabled: group?.enabled ?? true,
    });

    onClose();
  };

  const handleCancel = () => {
    setErrors({});
    onClose();
  };

  return (
    <Dialog.Root open={open} onOpenChange={(open) => !open && handleCancel()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content
          className="fixed left-1/2 top-1/2 w-full max-w-md -translate-x-1/2 -translate-y-1/2 rounded-lg bg-white p-6 shadow-lg"
          data-testid="group-editor-dialog"
        >
          <div className="mb-4 flex items-center justify-between">
            <Dialog.Title className="text-lg font-semibold">
              {group ? "Edit Group" : "New Group"}
            </Dialog.Title>
            <Dialog.Close asChild>
              <button
                className="rounded p-1 hover:bg-gray-100"
                aria-label="Close"
              >
                <X size={20} />
              </button>
            </Dialog.Close>
          </div>

          <div className="space-y-4">
            {/* Name field */}
            <div>
              <label htmlFor="group-name" className="mb-1 block text-sm font-medium">
                Name <span className="text-red-500">*</span>
              </label>
              <input
                id="group-name"
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className={`w-full rounded border px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-blue-500 ${
                  errors.name ? "border-red-500" : "border-gray-300"
                }`}
                placeholder="Enter group name"
                data-testid="group-name-input"
              />
              {errors.name && (
                <p className="mt-1 text-xs text-red-500">{errors.name}</p>
              )}
            </div>

            {/* Description field */}
            <div>
              <label htmlFor="group-description" className="mb-1 block text-sm font-medium">
                Description
              </label>
              <textarea
                id="group-description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="w-full rounded border border-gray-300 px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Optional description"
                rows={3}
                data-testid="group-description-input"
              />
            </div>
          </div>

          {/* Actions */}
          <div className="mt-6 flex justify-end gap-2">
            <button
              onClick={handleCancel}
              className="rounded border border-gray-300 px-4 py-2 text-sm hover:bg-gray-100"
              data-testid="group-cancel-button"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              className="rounded bg-blue-500 px-4 py-2 text-sm text-white hover:bg-blue-600"
              data-testid="group-save-button"
            >
              Save
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
