// PickerWindow component tests
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PickerWindow } from "../PickerWindow";
import { usePicker } from "../../../hooks/usePicker";
import { useGroupStore } from "../../../stores/groupStore";
import * as api from "../../../lib/tauri";
import type { Combo } from "../../../lib/types";

// Mock the hooks
vi.mock("../../../hooks/usePicker");
vi.mock("../../../stores/groupStore");

// Mock Tauri API
vi.mock("../../../lib/tauri", () => ({
  searchCombos: vi.fn(),
  triggerComboExpansion: vi.fn(),
  copySnippetToClipboard: vi.fn(),
  closePicker: vi.fn(),
  openPicker: vi.fn(),
}));

const mockCombos: Combo[] = [
  {
    id: "1",
    name: "Email Signature",
    description: "Professional email signature",
    keyword: "sig",
    snippet: "Best regards,\nJohn Doe\nSenior Developer",
    groupId: "group-1",
    matchingMode: "strict",
    caseSensitive: false,
    enabled: true,
    useCount: 10,
    lastUsed: "2026-01-30T10:00:00Z",
    createdAt: "2026-01-01T00:00:00Z",
    modifiedAt: "2026-01-30T10:00:00Z",
  },
  {
    id: "2",
    name: "Email Address",
    description: "My work email",
    keyword: "email",
    snippet: "john.doe@example.com",
    groupId: "group-1",
    matchingMode: "loose",
    caseSensitive: false,
    enabled: true,
    useCount: 25,
    lastUsed: "2026-01-31T09:00:00Z",
    createdAt: "2026-01-02T00:00:00Z",
    modifiedAt: "2026-01-31T09:00:00Z",
  },
  {
    id: "3",
    name: "Phone Number",
    description: "Contact number",
    keyword: "phone",
    snippet: "+1 (555) 123-4567",
    groupId: "group-2",
    matchingMode: "strict",
    caseSensitive: false,
    enabled: false,
    useCount: 5,
    lastUsed: "2026-01-28T14:00:00Z",
    createdAt: "2026-01-03T00:00:00Z",
    modifiedAt: "2026-01-28T14:00:00Z",
  },
];

// Helper to create default mock picker hook return value
const createMockPickerHook = (overrides = {}) => ({
  query: "",
  results: [],
  selectedIndex: 0,
  loading: false,
  error: null,
  isOpen: false,
  windowSize: { width: 500, height: 400 },
  setIsOpen: vi.fn(),
  setQuery: vi.fn(),
  setResults: vi.fn(),
  setLoading: vi.fn(),
  setError: vi.fn(),
  setWindowSize: vi.fn(),
  setSelectedIndex: vi.fn(),
  moveSelection: vi.fn(),
  reset: vi.fn(),
  search: vi.fn(),
  clearSearch: vi.fn(),
  getSelectedCombo: vi.fn(() => null),
  ...overrides,
});

describe("PickerWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Default mock for usePicker
    (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
      createMockPickerHook()
    );

    // Default mock for useGroupStore
    (useGroupStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      groups: [
        {
          id: "group-1",
          name: "Work",
          description: "",
          enabled: true,
          createdAt: "2026-01-01T00:00:00Z",
          modifiedAt: "2026-01-01T00:00:00Z",
        },
        {
          id: "group-2",
          name: "Personal",
          description: "",
          enabled: true,
          createdAt: "2026-01-01T00:00:00Z",
          modifiedAt: "2026-01-01T00:00:00Z",
        },
      ],
    });
  });

  describe("Search Input", () => {
    it("renders search input with placeholder", () => {
      render(<PickerWindow />);
      const input = screen.getByPlaceholderText("Search combos...");
      expect(input).toBeInTheDocument();
    });

    it("auto-focuses search input on mount", () => {
      render(<PickerWindow />);
      const input = screen.getByPlaceholderText("Search combos...");
      expect(input).toHaveFocus();
    });

    it("accepts text input and triggers search", async () => {
      const mockSearch = vi.fn();
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ search: mockSearch })
      );

      const user = userEvent.setup();
      render(<PickerWindow />);

      const input = screen.getByPlaceholderText("Search combos...");
      await user.type(input, "sig");

      // userEvent.type calls onChange for each character, so search is called with each character
      expect(mockSearch).toHaveBeenCalledWith("s");
      expect(mockSearch).toHaveBeenCalledWith("i");
      expect(mockSearch).toHaveBeenCalledWith("g");
    });

    it("shows clear button (X) when text is present", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "test" })
      );

      render(<PickerWindow />);
      const clearButton = screen.getByLabelText("Clear search");
      expect(clearButton).toBeInTheDocument();
    });

    it("does not show clear button when text is empty", () => {
      render(<PickerWindow />);
      const clearButton = screen.queryByLabelText("Clear search");
      expect(clearButton).not.toBeInTheDocument();
    });

    it("clears search when clear button is clicked", async () => {
      const mockClearSearch = vi.fn();
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "test", clearSearch: mockClearSearch })
      );

      const user = userEvent.setup();
      render(<PickerWindow />);

      const clearButton = screen.getByLabelText("Clear search");
      await user.click(clearButton);

      expect(mockClearSearch).toHaveBeenCalled();
    });
  });

  describe("Keyboard Navigation", () => {
    it("handles ArrowDown key to move selection down", async () => {
      const mockMoveSelection = vi.fn();
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ moveSelection: mockMoveSelection })
      );

      const user = userEvent.setup();
      render(<PickerWindow />);

      const input = screen.getByPlaceholderText("Search combos...");
      await user.type(input, "{ArrowDown}");

      expect(mockMoveSelection).toHaveBeenCalledWith("down");
    });

    it("handles ArrowUp key to move selection up", async () => {
      const mockMoveSelection = vi.fn();
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ moveSelection: mockMoveSelection })
      );

      const user = userEvent.setup();
      render(<PickerWindow />);

      const input = screen.getByPlaceholderText("Search combos...");
      await user.type(input, "{ArrowUp}");

      expect(mockMoveSelection).toHaveBeenCalledWith("up");
    });

    it("handles Enter key to trigger insertion", async () => {
      const mockGetSelectedCombo = vi.fn(() => mockCombos[0]);
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ getSelectedCombo: mockGetSelectedCombo })
      );

      const user = userEvent.setup();
      render(<PickerWindow />);

      const input = screen.getByPlaceholderText("Search combos...");
      await user.type(input, "{Enter}");

      expect(api.triggerComboExpansion).toHaveBeenCalledWith("1");
      expect(api.closePicker).toHaveBeenCalled();
    });

    it("handles Ctrl+Enter to copy to clipboard", async () => {
      const mockGetSelectedCombo = vi.fn(() => mockCombos[0]);
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ getSelectedCombo: mockGetSelectedCombo })
      );

      const user = userEvent.setup();
      render(<PickerWindow />);

      const input = screen.getByPlaceholderText("Search combos...");
      await user.type(input, "{Control>}{Enter}{/Control}");

      expect(api.copySnippetToClipboard).toHaveBeenCalledWith("1");
      expect(api.closePicker).not.toHaveBeenCalled();
    });

    it("handles Escape key to close picker", async () => {
      const user = userEvent.setup();
      render(<PickerWindow />);

      const input = screen.getByPlaceholderText("Search combos...");
      await user.type(input, "{Escape}");

      expect(api.closePicker).toHaveBeenCalled();
    });
  });

  describe("Search Results", () => {
    it("displays search results", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", results: mockCombos })
      );

      render(<PickerWindow />);

      expect(screen.getByText("Email Signature")).toBeInTheDocument();
      expect(screen.getByText("Email Address")).toBeInTheDocument();
      expect(screen.getByText("Phone Number")).toBeInTheDocument();
    });

    it("displays combo keywords", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", results: [mockCombos[0]] })
      );

      render(<PickerWindow />);
      expect(screen.getByText("sig")).toBeInTheDocument();
    });

    it("displays group names as badges", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", results: [mockCombos[0]] })
      );

      render(<PickerWindow />);
      expect(screen.getByText("Work")).toBeInTheDocument();
    });

    it("displays snippet preview (truncated)", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", results: [mockCombos[0]] })
      );

      render(<PickerWindow />);
      expect(screen.getByText(/Best regards/)).toBeInTheDocument();
    });

    it("shows disabled indicator for disabled combos", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "phone", results: [mockCombos[2]] })
      );

      render(<PickerWindow />);
      expect(screen.getByText("Disabled")).toBeInTheDocument();
    });

    it("highlights selected result", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", results: mockCombos, selectedIndex: 1 })
      );

      render(<PickerWindow />);

      // Get the parent container with data-index="1"
      const selectedElement = screen.getByText("Email Address").closest("[data-index='1']");
      expect(selectedElement).toHaveClass("bg-blue-50");
    });

    it("shows result count in footer", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", results: mockCombos })
      );

      render(<PickerWindow />);
      expect(screen.getByText("3 results")).toBeInTheDocument();
    });

    it("shows singular 'result' for one result", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", results: [mockCombos[0]] })
      );

      render(<PickerWindow />);
      expect(screen.getByText("1 result")).toBeInTheDocument();
    });
  });

  describe("Empty States", () => {
    it("shows 'Type to search...' when query is empty", () => {
      render(<PickerWindow />);
      expect(screen.getByText("Type to search...")).toBeInTheDocument();
    });

    it("shows 'No combos found' when query has no results", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "nonexistent", results: [] })
      );

      render(<PickerWindow />);
      expect(screen.getByText("No combos found")).toBeInTheDocument();
    });
  });

  describe("Loading State", () => {
    it("shows loading indicator when searching", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", loading: true })
      );

      render(<PickerWindow />);
      expect(screen.getByText("Searching...")).toBeInTheDocument();
    });
  });

  describe("Error State", () => {
    it("shows error message when search fails", () => {
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({ query: "sig", error: "Failed to search combos" })
      );

      render(<PickerWindow />);
      expect(screen.getByText("Error")).toBeInTheDocument();
      expect(screen.getByText("Failed to search combos")).toBeInTheDocument();
    });
  });

  describe("Result Item Interaction", () => {
    it("selects result on click", async () => {
      const mockSetSelectedIndex = vi.fn();
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({
          query: "sig",
          results: mockCombos,
          setSelectedIndex: mockSetSelectedIndex,
        })
      );

      const user = userEvent.setup();
      render(<PickerWindow />);

      const emailResult = screen.getByText("Email Address");
      await user.click(emailResult);

      expect(mockSetSelectedIndex).toHaveBeenCalledWith(1);
    });

    it("triggers insertion on double-click", async () => {
      const mockGetSelectedCombo = vi.fn(() => mockCombos[0]);
      (usePicker as unknown as ReturnType<typeof vi.fn>).mockReturnValue(
        createMockPickerHook({
          query: "sig",
          results: mockCombos,
          getSelectedCombo: mockGetSelectedCombo,
        })
      );

      const user = userEvent.setup();
      render(<PickerWindow />);

      const signatureResult = screen.getByText("Email Signature");
      await user.dblClick(signatureResult);

      expect(api.triggerComboExpansion).toHaveBeenCalledWith("1");
      expect(api.closePicker).toHaveBeenCalled();
    });
  });

  describe("Keyboard Hints", () => {
    it("displays keyboard navigation hints", () => {
      render(<PickerWindow />);

      expect(screen.getByText("Navigate")).toBeInTheDocument();
      expect(screen.getByText("Insert")).toBeInTheDocument();
      expect(screen.getByText("Copy")).toBeInTheDocument();
      expect(screen.getByText("Close")).toBeInTheDocument();
    });
  });
});
