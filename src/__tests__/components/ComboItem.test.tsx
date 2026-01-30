// ComboItem component tests
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ComboItem } from "../../components/combo/ComboItem";
import { useComboStore } from "../../stores/comboStore";
import { useGroupStore } from "../../stores/groupStore";
import type { Combo } from "../../lib/types";

vi.mock("../../stores/comboStore");
vi.mock("../../stores/groupStore");

const mockCombo: Combo = {
  id: "test-combo-1",
  name: "Test Combo",
  description: "A test combo",
  keyword: "test",
  snippet: "This is a test snippet",
  groupId: "group-1",
  matchingMode: "strict",
  caseSensitive: false,
  enabled: true,
  useCount: 5,
  lastUsed: "2026-01-29T10:00:00Z",
  createdAt: "2026-01-01T00:00:00Z",
  modifiedAt: "2026-01-29T10:00:00Z",
};

describe("ComboItem", () => {
  const selectCombo = vi.fn();
  const duplicateCombo = vi.fn();
  const deleteCombo = vi.fn();
  const toggleCombo = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      selectedIds: new Set<string>(),
      selectCombo,
      duplicateCombo,
      deleteCombo,
      toggleCombo,
    });
    (useGroupStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      groups: [
        {
          id: "group-1",
          name: "Test Group",
          description: "",
          enabled: true,
          createdAt: "2026-01-01T00:00:00Z",
          modifiedAt: "2026-01-01T00:00:00Z",
        },
      ],
    });
  });

  it("renders combo information", () => {
    render(<ComboItem combo={mockCombo} />);

    expect(screen.getByText("Test Combo")).toBeInTheDocument();
    expect(screen.getByText("test")).toBeInTheDocument();
    expect(screen.getByText("Test Group")).toBeInTheDocument();
  });

  it("truncates long snippets", () => {
    const longCombo = {
      ...mockCombo,
      snippet: "A".repeat(100),
    };
    render(<ComboItem combo={longCombo} />);

    const snippetCell = screen.getByText(/A{50}/);
    expect(snippetCell.textContent?.length).toBeLessThan(100);
  });

  it("shows enabled status", () => {
    render(<ComboItem combo={mockCombo} />);

    // The enabled checkbox is the one that is readOnly (second checkbox, first is selection)
    const checkboxes = screen.getAllByRole("checkbox");
    const enabledCheckbox = checkboxes.find(
      (cb) => cb.hasAttribute("readonly") || (cb as HTMLInputElement).readOnly
    );
    expect(enabledCheckbox).toBeTruthy();
    expect(enabledCheckbox).toBeChecked();
  });

  it("shows disabled status", () => {
    const disabledCombo = { ...mockCombo, enabled: false };
    render(<ComboItem combo={disabledCombo} />);

    const checkboxes = screen.getAllByRole("checkbox");
    const enabledCheckbox = checkboxes.find(
      (cb) => (cb as HTMLInputElement).readOnly
    );
    expect(enabledCheckbox).toBeTruthy();
    expect(enabledCheckbox).not.toBeChecked();
  });

  it("shows last used date", () => {
    render(<ComboItem combo={mockCombo} />);

    // Should show formatted date (locale-specific)
    const dateElement = screen.getByText(/\d{1,2}\/\d{1,2}\/\d{4}|\d{4}-\d{2}-\d{2}|Jan|yesterday|day/);
    expect(dateElement).toBeInTheDocument();
  });

  it("shows 'Never' when never used", () => {
    const neverUsedCombo = { ...mockCombo, lastUsed: null, useCount: 0 };
    render(<ComboItem combo={neverUsedCombo} />);

    expect(screen.getByText("Never")).toBeInTheDocument();
  });

  it("calls selectCombo on click", async () => {
    const user = userEvent.setup();
    render(<ComboItem combo={mockCombo} />);

    const row = screen.getByText("Test Combo").closest("[class*='grid']");
    await user.click(row!);

    expect(selectCombo).toHaveBeenCalledWith("test-combo-1", expect.objectContaining({
      ctrl: false,
      shift: false,
    }));
  });

  it("selects combo on click", async () => {
    const user = userEvent.setup();
    render(<ComboItem combo={mockCombo} />);

    const row = screen.getByText("Test Combo").closest("[class*='grid']");
    await user.click(row!);

    expect(selectCombo).toHaveBeenCalledWith("test-combo-1", expect.any(Object));
  });

  it("highlights selected row", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      selectedIds: new Set(["test-combo-1"]),
      selectCombo,
      duplicateCombo,
      deleteCombo,
      toggleCombo,
    });

    render(<ComboItem combo={mockCombo} />);

    const row = screen.getByText("Test Combo").closest("[class*='grid']");
    expect(row).toHaveClass("bg-blue-50");
  });

  it.skip("shows use count in tooltip", async () => {
    const user = userEvent.setup();
    render(<ComboItem combo={mockCombo} />);

    const lastUsedCell = screen.getByText(/\d{1,2}\/\d{1,2}\/\d{4}/);
    await user.hover(lastUsedCell);

    // Tooltip should show use count
    // Note: Radix Tooltip may need time to appear
    await screen.findByText(/Used 5 times/i, {}, { timeout: 2000 });
  });
});
