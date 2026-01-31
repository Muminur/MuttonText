import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PreferencesDialog } from "../PreferencesDialog";
import { usePreferencesStore } from "@/stores/preferencesStore";
import type { Preferences } from "@/lib/types";

// Mock the preferences store
vi.mock("@/stores/preferencesStore");

// Mock Tauri API
vi.mock("@/lib/tauri", () => ({
  checkForUpdates: vi.fn(),
}));

const mockPreferences: Preferences = {
  enabled: true,
  playSound: false,
  defaultMatchingMode: "strict",
  defaultCaseSensitive: false,
  theme: "system",
  showSystemTray: true,
  startAtLogin: false,
  startMinimized: false,
  pickerShortcut: "Ctrl+Space",
  comboTriggerShortcut: "Ctrl+Shift+E",
  backupEnabled: true,
  backupIntervalHours: 24,
  maxBackups: 10,
  autoCheckUpdates: true,
  pasteMethod: "clipboard",
  excludedApps: [],
};

const createMockPreferencesStore = (overrides = {}) => ({
  preferences: mockPreferences,
  loading: false,
  error: null,
  loadPreferences: vi.fn(),
  updatePreferences: vi.fn(),
  resetPreferences: vi.fn(),
  ...overrides,
});

describe("PreferencesDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
      createMockPreferencesStore()
    );
  });

  describe("Dialog Structure", () => {
    it("renders nothing when closed", () => {
      const { container } = render(<PreferencesDialog isOpen={false} onClose={vi.fn()} />);
      expect(container.firstChild).toBeNull();
    });

    it("renders dialog when open", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByRole("dialog", { name: "Preferences" })).toBeInTheDocument();
    });

    it("renders dialog header with title", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByText("Preferences")).toBeInTheDocument();
    });

    it("renders close button", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByLabelText("Close")).toBeInTheDocument();
    });

    it("renders all tab navigation buttons", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByRole("button", { name: "Behavior" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Appearance" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Shortcuts" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Data" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Updates" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Advanced" })).toBeInTheDocument();
    });

    it("renders footer buttons", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByRole("button", { name: "Reset to Defaults" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
    });
  });

  describe("Tab Switching", () => {
    it("shows behavior tab by default", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByText("Enable snippet expansion")).toBeInTheDocument();
    });

    it("switches to appearance tab when clicked", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      await user.click(screen.getByRole("button", { name: "Appearance" }));

      expect(screen.getByText("Theme")).toBeInTheDocument();
    });

    it("switches to shortcuts tab when clicked", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      await user.click(screen.getByRole("button", { name: "Shortcuts" }));

      expect(screen.getByText("Picker shortcut")).toBeInTheDocument();
    });

    it("switches to data tab when clicked", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      await user.click(screen.getByRole("button", { name: "Data" }));

      expect(screen.getByText("Data & Backups")).toBeInTheDocument();
    });

    it("switches to updates tab when clicked", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      await user.click(screen.getByRole("button", { name: "Updates" }));

      expect(screen.getByText("Automatically check for updates")).toBeInTheDocument();
    });

    it("switches to advanced tab when clicked", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      await user.click(screen.getByRole("button", { name: "Advanced" }));

      expect(screen.getByText("Paste method")).toBeInTheDocument();
    });

    it("highlights active tab", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      const appearanceTab = screen.getByRole("button", { name: "Appearance" });
      await user.click(appearanceTab);

      expect(appearanceTab).toHaveClass("bg-blue-50");
    });
  });

  describe("Dialog Actions", () => {
    it("calls onClose when close button clicked", async () => {
      const user = userEvent.setup();
      const onClose = vi.fn();
      render(<PreferencesDialog isOpen={true} onClose={onClose} />);

      await user.click(screen.getByLabelText("Close"));

      expect(onClose).toHaveBeenCalled();
    });

    it("calls onClose when Cancel button clicked", async () => {
      const user = userEvent.setup();
      const onClose = vi.fn();
      render(<PreferencesDialog isOpen={true} onClose={onClose} />);

      await user.click(screen.getByRole("button", { name: "Cancel" }));

      expect(onClose).toHaveBeenCalled();
    });

    it("calls updatePreferences and onClose when Save clicked", async () => {
      const user = userEvent.setup();
      const mockUpdatePreferences = vi.fn().mockResolvedValue(undefined);
      const onClose = vi.fn();
      (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPreferencesStore({ updatePreferences: mockUpdatePreferences })
      );

      render(<PreferencesDialog isOpen={true} onClose={onClose} />);

      await user.click(screen.getByRole("button", { name: "Save" }));

      expect(mockUpdatePreferences).toHaveBeenCalled();
      expect(onClose).toHaveBeenCalled();
    });

    it("shows 'Saving...' while saving", async () => {
      const user = userEvent.setup();
      const mockUpdatePreferences = vi
        .fn()
        .mockImplementation(() => new Promise((resolve) => setTimeout(resolve, 100)));
      (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPreferencesStore({ updatePreferences: mockUpdatePreferences })
      );

      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      await user.click(screen.getByRole("button", { name: "Save" }));

      expect(screen.getByText("Saving...")).toBeInTheDocument();
    });

    it("calls resetPreferences when Reset to Defaults clicked", async () => {
      const user = userEvent.setup();
      const mockResetPreferences = vi.fn();
      (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPreferencesStore({ resetPreferences: mockResetPreferences })
      );

      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      await user.click(screen.getByRole("button", { name: "Reset to Defaults" }));

      expect(mockResetPreferences).toHaveBeenCalled();
    });

    it("closes dialog on Escape key", async () => {
      const onClose = vi.fn();
      render(<PreferencesDialog isOpen={true} onClose={onClose} />);

      const dialog = screen.getByRole("dialog", { name: "Preferences" });

      // Trigger keyboard event directly on the dialog
      const event = new KeyboardEvent("keydown", { key: "Escape", bubbles: true });
      dialog.dispatchEvent(event);

      expect(onClose).toHaveBeenCalled();
    });
  });

  describe("Loading State", () => {
    it("shows loading message when loading", () => {
      (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPreferencesStore({ loading: true })
      );

      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      expect(screen.getByText("Loading preferences...")).toBeInTheDocument();
    });

    it("loads preferences when dialog opens", () => {
      const mockLoadPreferences = vi.fn();
      (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPreferencesStore({ loadPreferences: mockLoadPreferences })
      );

      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      expect(mockLoadPreferences).toHaveBeenCalled();
    });
  });

  describe("BehaviorTab", () => {
    it("renders enabled toggle", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByText("Enable snippet expansion")).toBeInTheDocument();
    });

    it("renders play sound toggle", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByText("Play sound on expansion")).toBeInTheDocument();
    });

    it("renders matching mode select", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByText("Default matching mode")).toBeInTheDocument();
      expect(screen.getByText("Strict (word boundary)")).toBeInTheDocument();
      expect(screen.getByText("Loose (match anywhere)")).toBeInTheDocument();
    });

    it("renders case sensitive toggle", () => {
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      expect(screen.getByText("Case-sensitive by default")).toBeInTheDocument();
    });

    it("toggles enabled checkbox", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);

      const checkbox = screen
        .getByText("Enable snippet expansion")
        .closest("label")!
        .querySelector("input[type='checkbox']") as HTMLInputElement;

      expect(checkbox.checked).toBe(true);
      await user.click(checkbox);
      expect(checkbox.checked).toBe(false);
    });
  });

  describe("AppearanceTab", () => {
    it("renders theme selector", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Appearance" }));

      expect(screen.getByText("Theme")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "system" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "light" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "dark" })).toBeInTheDocument();
    });

    it("renders system tray toggle", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Appearance" }));

      expect(screen.getByText("Show in system tray")).toBeInTheDocument();
    });

    it("renders start at login toggle", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Appearance" }));

      expect(screen.getByText("Start at login")).toBeInTheDocument();
    });

    it("renders start minimized toggle", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Appearance" }));

      expect(screen.getByText("Start minimized")).toBeInTheDocument();
    });

    it("selects theme button", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Appearance" }));

      const darkButton = screen.getByRole("button", { name: "dark" });
      await user.click(darkButton);

      expect(darkButton).toHaveClass("border-blue-500");
    });
  });

  describe("ShortcutsTab", () => {
    it("renders picker shortcut input", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Shortcuts" }));

      expect(screen.getByText("Picker shortcut")).toBeInTheDocument();
      const input = screen.getByPlaceholderText("e.g. Ctrl+Space");
      expect(input).toBeInTheDocument();
      expect(input).toHaveValue("Ctrl+Space");
    });

    it("renders combo trigger shortcut input", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Shortcuts" }));

      expect(screen.getByText("Combo trigger shortcut")).toBeInTheDocument();
      const input = screen.getByPlaceholderText("e.g. Ctrl+Shift+E");
      expect(input).toBeInTheDocument();
      expect(input).toHaveValue("Ctrl+Shift+E");
    });

    it("accepts input for picker shortcut", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Shortcuts" }));

      const input = screen.getByPlaceholderText("e.g. Ctrl+Space");
      await user.clear(input);
      await user.type(input, "Alt+S");

      expect(input).toHaveValue("Alt+S");
    });
  });

  describe("DataTab", () => {
    it("renders backup enabled toggle", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Data" }));

      expect(screen.getByText("Enable automatic backups")).toBeInTheDocument();
    });

    it("renders backup interval input when enabled", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Data" }));

      expect(screen.getByText("Backup interval (hours)")).toBeInTheDocument();
      const input = screen.getByDisplayValue("24");
      expect(input).toBeInTheDocument();
    });

    it("renders max backups input when enabled", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Data" }));

      expect(screen.getByText("Maximum backups to keep")).toBeInTheDocument();
      const input = screen.getByDisplayValue("10");
      expect(input).toBeInTheDocument();
    });

    it("shows data directory info message", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Data" }));

      expect(
        screen.getByText(/Data is stored in the application data directory/)
      ).toBeInTheDocument();
    });
  });

  describe("DataTab - Disabled State", () => {
    it("hides backup inputs when disabled", async () => {
      const user = userEvent.setup();
      (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPreferencesStore({
          preferences: { ...mockPreferences, backupEnabled: false },
        })
      );

      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Data" }));

      expect(screen.queryByText("Backup interval (hours)")).not.toBeInTheDocument();
      expect(screen.queryByText("Maximum backups to keep")).not.toBeInTheDocument();
    });
  });

  describe("UpdatesTab", () => {
    it("renders auto-check updates toggle", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Updates" }));

      expect(screen.getByText("Automatically check for updates")).toBeInTheDocument();
    });

    it("renders check for updates button", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Updates" }));

      expect(screen.getByRole("button", { name: "Check for Updates" })).toBeInTheDocument();
    });

    it("displays current version", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Updates" }));

      expect(screen.getByText("Current version: 0.1.0")).toBeInTheDocument();
    });

    it("shows 'Checking...' when checking for updates", async () => {
      const user = userEvent.setup();
      const { checkForUpdates } = await import("@/lib/tauri");
      (checkForUpdates as ReturnType<typeof vi.fn>).mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve(null), 100))
      );

      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Updates" }));

      const checkButton = screen.getByRole("button", { name: "Check for Updates" });
      await user.click(checkButton);

      expect(screen.getByText("Checking...")).toBeInTheDocument();
    });
  });

  describe("AdvancedTab", () => {
    it("renders paste method select", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      expect(screen.getByText("Paste method")).toBeInTheDocument();
      expect(screen.getByText("Clipboard (recommended)")).toBeInTheDocument();
      expect(screen.getByText("Simulate keystrokes")).toBeInTheDocument();
    });

    it("renders excluded applications section", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      expect(screen.getByText("Excluded applications")).toBeInTheDocument();
      expect(
        screen.getByText("Combos will not expand in these applications.")
      ).toBeInTheDocument();
    });

    it("renders add application input and button", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      expect(
        screen.getByPlaceholderText("Application name (e.g. code.exe)")
      ).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Add" })).toBeInTheDocument();
    });

    it("shows empty state when no apps excluded", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      expect(screen.getByText("No excluded applications")).toBeInTheDocument();
    });

    it("adds application to excluded list", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      const input = screen.getByPlaceholderText("Application name (e.g. code.exe)");
      const addButton = screen.getByRole("button", { name: "Add" });

      await user.type(input, "vscode.exe");
      await user.click(addButton);

      expect(screen.getByText("vscode.exe")).toBeInTheDocument();
      expect(input).toHaveValue("");
    });

    it("displays excluded apps with remove buttons", async () => {
      const user = userEvent.setup();
      (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPreferencesStore({
          preferences: { ...mockPreferences, excludedApps: ["code.exe", "notepad.exe"] },
        })
      );

      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      expect(screen.getByText("code.exe")).toBeInTheDocument();
      expect(screen.getByText("notepad.exe")).toBeInTheDocument();

      const removeButtons = screen.getAllByText("Remove");
      expect(removeButtons).toHaveLength(2);
    });

    it("removes application from excluded list", async () => {
      const user = userEvent.setup();
      (usePreferencesStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPreferencesStore({
          preferences: { ...mockPreferences, excludedApps: ["code.exe"] },
        })
      );

      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      const removeButton = screen.getByText("Remove");
      await user.click(removeButton);

      expect(screen.queryByText("code.exe")).not.toBeInTheDocument();
    });

    it("adds app on Enter key press", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      const input = screen.getByPlaceholderText("Application name (e.g. code.exe)");
      await user.type(input, "vscode.exe{Enter}");

      expect(screen.getByText("vscode.exe")).toBeInTheDocument();
      expect(input).toHaveValue("");
    });

    it("disables add button when input is empty", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      const addButton = screen.getByRole("button", { name: "Add" }) as HTMLButtonElement;
      expect(addButton.disabled).toBe(true);
    });

    it("enables add button when input has text", async () => {
      const user = userEvent.setup();
      render(<PreferencesDialog isOpen={true} onClose={vi.fn()} />);
      await user.click(screen.getByRole("button", { name: "Advanced" }));

      const input = screen.getByPlaceholderText("Application name (e.g. code.exe)");
      const addButton = screen.getByRole("button", { name: "Add" }) as HTMLButtonElement;

      await user.type(input, "vscode.exe");

      expect(addButton.disabled).toBe(false);
    });
  });
});
