import React from "react";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import {
  FileIcon,
  EditIcon,
  ListIcon,
  FolderIcon,
  HelpCircleIcon,
  ChevronDownIcon,
} from "lucide-react";

interface MenuBarProps {
  onOpenPreferences?: () => void;
  onOpenImport?: () => void;
  onOpenExport?: () => void;
  onOpenBackups?: () => void;
}

/**
 * Menu bar component with File, Edit, Combos, Groups, and Help menus.
 * Uses Radix UI DropdownMenu for accessible menu implementation.
 */
export const MenuBar: React.FC<MenuBarProps> = ({ onOpenPreferences, onOpenImport, onOpenExport, onOpenBackups }) => {
  return (
    <div className="flex h-8 items-center gap-1 border-b bg-gray-100 px-2">
      {/* File Menu */}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <button
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200"
            data-testid="file-menu-trigger"
          >
            <FileIcon size={14} />
            File
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border bg-white shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("New Combo")}
            >
              New Combo
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("New Group")}
            >
              New Group
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 h-px bg-gray-200" />
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => onOpenImport?.()}
            >
              Import...
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => onOpenExport?.()}
            >
              Export...
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => onOpenBackups?.()}
            >
              Backups...
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 h-px bg-gray-200" />
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => onOpenPreferences?.()}
            >
              Preferences
              <span className="ml-auto pl-4 text-xs text-gray-400">Ctrl+,</span>
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 h-px bg-gray-200" />
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Exit")}
            >
              Exit
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>

      {/* Edit Menu */}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <button
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200"
            data-testid="edit-menu-trigger"
          >
            <EditIcon size={14} />
            Edit
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border bg-white shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Undo")}
            >
              Undo
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Redo")}
            >
              Redo
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 h-px bg-gray-200" />
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Cut")}
            >
              Cut
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Copy")}
            >
              Copy
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Paste")}
            >
              Paste
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>

      {/* Combos Menu */}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <button
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200"
            data-testid="combos-menu-trigger"
          >
            <ListIcon size={14} />
            Combos
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border bg-white shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Enable All")}
            >
              Enable All
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Disable All")}
            >
              Disable All
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>

      {/* Groups Menu */}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <button
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200"
            data-testid="groups-menu-trigger"
          >
            <FolderIcon size={14} />
            Groups
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border bg-white shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("New Group")}
            >
              New Group
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>

      {/* Help Menu */}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <button
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200"
            data-testid="help-menu-trigger"
          >
            <HelpCircleIcon size={14} />
            Help
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border bg-white shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("About")}
            >
              About
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100"
              onSelect={() => console.log("Check for Updates")}
            >
              Check for Updates
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    </div>
  );
};
