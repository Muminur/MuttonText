import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { GroupList } from "../GroupList";
import { Group } from "@/lib/types";

const mockGroups: Group[] = [
  {
    id: "group-1",
    name: "Work",
    description: "Work-related combos",
    enabled: true,
    createdAt: "2024-01-01T00:00:00Z",
    modifiedAt: "2024-01-01T00:00:00Z",
  },
  {
    id: "group-2",
    name: "Personal",
    description: "Personal snippets",
    enabled: true,
    createdAt: "2024-01-02T00:00:00Z",
    modifiedAt: "2024-01-02T00:00:00Z",
  },
];

describe("GroupList", () => {
  it("should render 'All Combos' virtual group", () => {
    render(
      <GroupList
        groups={[]}
        selectedGroupId={null}
        onSelectGroup={vi.fn()}
        onEditGroup={vi.fn()}
        onDeleteGroup={vi.fn()}
        onToggleGroup={vi.fn()}
        onReorderGroups={vi.fn()}
        totalComboCount={10}
        getGroupComboCount={vi.fn()}
      />
    );

    expect(screen.getByTestId("all-combos-group")).toBeInTheDocument();
    expect(screen.getByText("All Combos")).toBeInTheDocument();
    expect(screen.getByText("10")).toBeInTheDocument();
  });

  it("should render all groups", () => {
    render(
      <GroupList
        groups={mockGroups}
        selectedGroupId={null}
        onSelectGroup={vi.fn()}
        onEditGroup={vi.fn()}
        onDeleteGroup={vi.fn()}
        onToggleGroup={vi.fn()}
        onReorderGroups={vi.fn()}
        totalComboCount={10}
        getGroupComboCount={() => 5}
      />
    );

    expect(screen.getByTestId("group-item-group-1")).toBeInTheDocument();
    expect(screen.getByTestId("group-item-group-2")).toBeInTheDocument();
    expect(screen.getByText("Work")).toBeInTheDocument();
    expect(screen.getByText("Personal")).toBeInTheDocument();
  });

  it("should highlight selected group", () => {
    render(
      <GroupList
        groups={mockGroups}
        selectedGroupId="group-1"
        onSelectGroup={vi.fn()}
        onEditGroup={vi.fn()}
        onDeleteGroup={vi.fn()}
        onToggleGroup={vi.fn()}
        onReorderGroups={vi.fn()}
        totalComboCount={10}
        getGroupComboCount={() => 5}
      />
    );

    const selectedGroup = screen.getByTestId("group-item-group-1");
    expect(selectedGroup).toHaveClass("bg-blue-500");
  });

  it("should highlight 'All Combos' when no group is selected", () => {
    render(
      <GroupList
        groups={mockGroups}
        selectedGroupId={null}
        onSelectGroup={vi.fn()}
        onEditGroup={vi.fn()}
        onDeleteGroup={vi.fn()}
        onToggleGroup={vi.fn()}
        onReorderGroups={vi.fn()}
        totalComboCount={10}
        getGroupComboCount={() => 5}
      />
    );

    const allCombos = screen.getByTestId("all-combos-group");
    expect(allCombos).toHaveClass("bg-blue-500");
  });

  it("should call onSelectGroup when clicking 'All Combos'", async () => {
    const user = userEvent.setup();
    const onSelectGroup = vi.fn();

    render(
      <GroupList
        groups={mockGroups}
        selectedGroupId="group-1"
        onSelectGroup={onSelectGroup}
        onEditGroup={vi.fn()}
        onDeleteGroup={vi.fn()}
        onToggleGroup={vi.fn()}
        onReorderGroups={vi.fn()}
        totalComboCount={10}
        getGroupComboCount={() => 5}
      />
    );

    await user.click(screen.getByTestId("all-combos-group"));
    expect(onSelectGroup).toHaveBeenCalledWith(null);
  });

  it("should display combo counts for each group", () => {
    const getGroupComboCount = (groupId: string) => {
      if (groupId === "group-1") return 3;
      if (groupId === "group-2") return 7;
      return 0;
    };

    render(
      <GroupList
        groups={mockGroups}
        selectedGroupId={null}
        onSelectGroup={vi.fn()}
        onEditGroup={vi.fn()}
        onDeleteGroup={vi.fn()}
        onToggleGroup={vi.fn()}
        onReorderGroups={vi.fn()}
        totalComboCount={10}
        getGroupComboCount={getGroupComboCount}
      />
    );

    // Note: Multiple "3" and "7" text nodes may exist, so we check they exist
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("7")).toBeInTheDocument();
  });
});
