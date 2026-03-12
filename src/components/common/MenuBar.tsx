import React, { useState } from "react";
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
  onNewCombo?: () => void;
  onNewGroup?: () => void;
}

/**
 * Menu bar component with File, Edit, Combos, Groups, and Help menus.
 * Uses Radix UI DropdownMenu for accessible menu implementation.
 */
export const MenuBar: React.FC<MenuBarProps> = ({ onOpenPreferences, onOpenImport, onOpenExport, onOpenBackups, onNewCombo, onNewGroup }) => {
  const [aboutOpen, setAboutOpen] = useState(false);

  return (
    <div className="flex h-8 items-center gap-1 border-b border-gray-200 dark:border-gray-700 bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-gray-100 px-2" role="menubar" aria-label="Main menu">
      {/* File Menu */}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <button
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200 dark:hover:bg-gray-700"
            data-testid="file-menu-trigger"
            role="menuitem"
            aria-haspopup="true"
          >
            <FileIcon size={14} aria-hidden="true" />
            File
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => onNewCombo?.()}
            >
              New Combo
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => onNewGroup?.()}
            >
              New Group
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 h-px bg-gray-200 dark:bg-gray-600" />
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => onOpenImport?.()}
            >
              Import...
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => onOpenExport?.()}
            >
              Export...
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => onOpenBackups?.()}
            >
              Backups...
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 h-px bg-gray-200 dark:bg-gray-600" />
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => onOpenPreferences?.()}
            >
              Preferences
              <span className="ml-auto pl-4 text-xs text-gray-400">Ctrl+,</span>
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 h-px bg-gray-200 dark:bg-gray-600" />
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
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
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200 dark:hover:bg-gray-700"
            data-testid="edit-menu-trigger"
            role="menuitem"
            aria-haspopup="true"
          >
            <EditIcon size={14} aria-hidden="true" />
            Edit
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => console.log("Undo")}
            >
              Undo
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => console.log("Redo")}
            >
              Redo
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 h-px bg-gray-200 dark:bg-gray-600" />
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => console.log("Cut")}
            >
              Cut
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => console.log("Copy")}
            >
              Copy
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
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
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200 dark:hover:bg-gray-700"
            data-testid="combos-menu-trigger"
            role="menuitem"
            aria-haspopup="true"
          >
            <ListIcon size={14} aria-hidden="true" />
            Combos
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => console.log("Enable All")}
            >
              Enable All
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
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
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200 dark:hover:bg-gray-700"
            data-testid="groups-menu-trigger"
            role="menuitem"
            aria-haspopup="true"
          >
            <FolderIcon size={14} aria-hidden="true" />
            Groups
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => onNewGroup?.()}
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
            className="flex items-center gap-1 rounded px-2 py-1 text-sm hover:bg-gray-200 dark:hover:bg-gray-700"
            data-testid="help-menu-trigger"
            role="menuitem"
            aria-haspopup="true"
          >
            <HelpCircleIcon size={14} aria-hidden="true" />
            Help
            <ChevronDownIcon size={12} />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="min-w-[180px] rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 shadow-md"
            sideOffset={2}
          >
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => setAboutOpen(true)}
            >
              About
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className="cursor-pointer px-3 py-2 text-sm outline-none hover:bg-gray-100 dark:hover:bg-gray-700 dark:text-gray-100"
              onSelect={() => console.log("Check for Updates")}
            >
              Check for Updates
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
      {aboutOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
          onClick={() => setAboutOpen(false)}
        >
          <div
            className="rounded-lg border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 p-6 shadow-xl w-80 text-center"
            onClick={(e) => e.stopPropagation()}
          >
            <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">MuttonText</h2>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">A fast, cross-platform text expansion tool</p>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">Version: 0.1.0</p>
            <button
              className="rounded bg-blue-500 px-4 py-2 text-sm text-white hover:bg-blue-600"
              onClick={() => setAboutOpen(false)}
            >
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
