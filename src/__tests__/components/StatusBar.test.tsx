// StatusBar component tests
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatusBar } from "../../components/common/StatusBar";
import { useComboStore } from "../../stores/comboStore";
import type { Combo } from "../../lib/types";

vi.mock("../../stores/comboStore");

const mockCombos: Combo[] = [
  {
    id: "1",
    name: "Signature",
    description: "",
    keyword: "sig",
    snippet: "Best regards",
    groupId: "group-1",
    matchingMode: "strict",
    caseSensitive: false,
    enabled: true,
    useCount: 5,
    lastUsed: "2026-01-30T10:00:00Z",
    createdAt: "2026-01-01T00:00:00Z",
    modifiedAt: "2026-01-30T10:00:00Z",
  },
  {
    id: "2",
    name: "Email",
    description: "",
    keyword: "email",
    snippet: "john@example.com",
    groupId: "group-1",
    matchingMode: "strict",
    caseSensitive: false,
    enabled: true,
    useCount: 10,
    lastUsed: "2026-01-29T10:00:00Z",
    createdAt: "2026-01-01T00:00:00Z",
    modifiedAt: "2026-01-29T10:00:00Z",
  },
  {
    id: "3",
    name: "Never Used",
    description: "",
    keyword: "never",
    snippet: "test",
    groupId: "group-1",
    matchingMode: "strict",
    caseSensitive: false,
    enabled: true,
    useCount: 0,
    lastUsed: null,
    createdAt: "2026-01-01T00:00:00Z",
    modifiedAt: "2026-01-01T00:00:00Z",
  },
];

describe("StatusBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders status bar", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: mockCombos,
    });

    render(<StatusBar />);
    const statusBar = screen.getByRole("contentinfo");
    expect(statusBar).toBeInTheDocument();
  });

  it("displays total combo count", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: mockCombos,
    });

    render(<StatusBar />);
    const statusBar = screen.getByRole("contentinfo");
    expect(statusBar.textContent).toContain("3");
    expect(statusBar.textContent).toContain("combos");
  });

  it("displays singular 'combo' when count is 1", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: [mockCombos[0]],
    });

    render(<StatusBar />);
    const statusBar = screen.getByRole("contentinfo");
    expect(statusBar.textContent).toMatch(/1\s*combo/);
  });

  it("displays most recently used combo", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: mockCombos,
    });

    render(<StatusBar />);
    // "Signature" was used most recently (2026-01-30)
    expect(screen.getByText(/last used/i)).toBeInTheDocument();
    expect(screen.getByText("Signature")).toBeInTheDocument();
  });

  it("shows time ago for last used combo", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: mockCombos,
    });

    render(<StatusBar />);
    // Should show relative time
    expect(screen.getByText(/ago|minutes?|hours?|days?|just now/i)).toBeInTheDocument();
  });

  it("does not show last used when no combos have been used", () => {
    const neverUsedCombos = mockCombos.map((c) => ({
      ...c,
      lastUsed: null,
      useCount: 0,
    }));

    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: neverUsedCombos,
    });

    render(<StatusBar />);
    expect(screen.queryByText(/last used/i)).not.toBeInTheDocument();
  });

  it("shows 0 combos when list is empty", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: [],
    });

    render(<StatusBar />);
    const statusBar = screen.getByRole("contentinfo");
    expect(statusBar.textContent).toContain("0");
    expect(statusBar.textContent).toContain("combos");
  });

  it("has muted styling", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: mockCombos,
    });

    render(<StatusBar />);
    const statusBar = screen.getByRole("contentinfo");
    expect(statusBar).toHaveClass("text-sm", "text-gray-600");
  });

  it("is fixed at bottom", () => {
    (useComboStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      combos: mockCombos,
    });

    render(<StatusBar />);
    const statusBar = screen.getByRole("contentinfo");
    expect(statusBar).toHaveClass("border-t");
  });
});
