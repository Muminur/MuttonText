// InsertVariableMenu - Dropdown menu for inserting variables into snippets
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { Plus } from "lucide-react";

interface Variable {
  syntax: string;
  name: string;
  description: string;
}

interface VariableCategory {
  name: string;
  variables: Variable[];
  advanced?: boolean;
}

interface InsertVariableMenuProps {
  onInsert: (variable: string) => void;
}

const VARIABLE_CATEGORIES: VariableCategory[] = [
  {
    name: "Date/Time",
    variables: [
      {
        syntax: "#{date}",
        name: "date",
        description: "Current date in locale format",
      },
      {
        syntax: "#{time}",
        name: "time",
        description: "Current time in locale format",
      },
      {
        syntax: "#{dateTime}",
        name: "dateTime",
        description: "Current date and time in locale format",
      },
      {
        syntax: "#{dateTime:format}",
        name: "dateTime:format",
        description: "Date/time with custom format string",
      },
      {
        syntax: "#{dateTime:shift:format}",
        name: "dateTime:shift:format",
        description: "Date/time with offset (e.g., +1d, -2h)",
      },
    ],
  },
  {
    name: "Clipboard",
    variables: [
      {
        syntax: "#{clipboard}",
        name: "clipboard",
        description: "Current clipboard content",
      },
    ],
  },
  {
    name: "References",
    variables: [
      {
        syntax: "#{combo:keyword}",
        name: "combo:keyword",
        description: "Insert another combo by keyword",
      },
      {
        syntax: "#{lower:keyword}",
        name: "lower:keyword",
        description: "Insert combo in lowercase",
      },
      {
        syntax: "#{upper:keyword}",
        name: "upper:keyword",
        description: "Insert combo in uppercase",
      },
    ],
  },
  {
    name: "Interactive",
    variables: [
      {
        syntax: "#{cursor}",
        name: "cursor",
        description: "Cursor position after expansion",
      },
      {
        syntax: "#{input:prompt}",
        name: "input:prompt",
        description: "Prompt user for input",
      },
    ],
  },
  {
    name: "System",
    variables: [
      {
        syntax: "#{envVar:name}",
        name: "envVar:name",
        description: "Environment variable value",
      },
    ],
  },
  {
    name: "Keys",
    variables: [
      {
        syntax: "#{key:name}",
        name: "key:name",
        description: "Press a single key (e.g., Enter, Tab)",
      },
      {
        syntax: "#{key:name:count}",
        name: "key:name:count",
        description: "Press a key multiple times",
      },
      {
        syntax: "#{shortcut:keys}",
        name: "shortcut:keys",
        description: "Press key combination (e.g., Ctrl+C)",
      },
      {
        syntax: "#{delay:ms}",
        name: "delay:ms",
        description: "Pause expansion for milliseconds",
      },
    ],
  },
  {
    name: "Script",
    variables: [
      {
        syntax: "#{shell:path}",
        name: "shell:path",
        description: "Execute shell script and insert output",
      },
    ],
    advanced: true,
  },
];

export function InsertVariableMenu({ onInsert }: InsertVariableMenuProps) {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button
          type="button"
          className="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded flex items-center gap-1"
          aria-label="Insert Variable"
        >
          <Plus className="w-4 h-4" />
          Insert Variable
        </button>
      </DropdownMenu.Trigger>

      <DropdownMenu.Portal>
        <DropdownMenu.Content
          className="bg-white border rounded-lg shadow-lg p-1 min-w-[350px] max-h-[500px] overflow-y-auto z-50"
          sideOffset={5}
        >
          {VARIABLE_CATEGORIES.map((category, categoryIndex) => (
            <div key={category.name}>
              {categoryIndex > 0 && <DropdownMenu.Separator className="h-px bg-gray-200 my-1" />}

              <DropdownMenu.Label className="px-3 py-2 text-xs font-semibold text-gray-500 uppercase flex items-center gap-2">
                {category.name}
                {category.advanced && (
                  <span className="text-orange-500 text-xs font-normal normal-case">
                    (Advanced)
                  </span>
                )}
              </DropdownMenu.Label>

              {category.variables.map((variable) => (
                <DropdownMenu.Item
                  key={variable.syntax}
                  className="px-3 py-2 hover:bg-gray-100 rounded cursor-pointer focus:bg-gray-100 focus:outline-none"
                  onSelect={() => onInsert(variable.syntax)}
                >
                  <div className="font-mono text-sm font-medium text-blue-600">
                    {variable.syntax}
                  </div>
                  <div className="text-xs text-gray-600 mt-0.5">
                    {variable.description}
                  </div>
                </DropdownMenu.Item>
              ))}
            </div>
          ))}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}
