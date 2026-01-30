import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { GroupItem } from "../GroupItem";
import { Group } from "@/lib/types";

const mockGroup: Group = {
  id: "group-1",
  name: "Work",
  description: "Work-related combos",
  enabled: true,
  createdAt: "2024-01-01T00:00:00Z",
  modifiedAt: "2024-01-01T00:00:00Z",
};

describe("GroupItem", () => {
  it("should render group name and combo count", () => {
    render(
      <GroupItem
        group={mockGroup}
        comboCount={5}
        isSelected={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onToggle={vi.fn()}
      />
    );

    expect(screen.getByText("Work")).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
  });

  it("should render description if provided", () => {
    render(
      <GroupItem
        group={mockGroup}
        comboCount={5}
        isSelected={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onToggle={vi.fn()}
      />
    );

    expect(screen.getByText("Work-related combos")).toBeInTheDocument();
  });

  it("should highlight when selected", () => {
    render(
      <GroupItem
        group={mockGroup}
        comboCount={5}
        isSelected={true}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onToggle={vi.fn()}
      />
    );

    const item = screen.getByTestId("group-item-group-1");
    expect(item).toHaveClass("bg-blue-500");
  });

  it("should apply opacity when disabled", () => {
    const disabledGroup = { ...mockGroup, enabled: false };

    render(
      <GroupItem
        group={disabledGroup}
        comboCount={5}
        isSelected={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onToggle={vi.fn()}
      />
    );

    const item = screen.getByTestId("group-item-group-1");
    expect(item).toHaveClass("opacity-50");
  });

  it("should call onSelect when clicked", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();

    render(
      <GroupItem
        group={mockGroup}
        comboCount={5}
        isSelected={false}
        onSelect={onSelect}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onToggle={vi.fn()}
      />
    );

    await user.click(screen.getByTestId("group-item-group-1"));
    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it("should call onEdit when double-clicked", async () => {
    const user = userEvent.setup();
    const onEdit = vi.fn();

    render(
      <GroupItem
        group={mockGroup}
        comboCount={5}
        isSelected={false}
        onSelect={vi.fn()}
        onEdit={onEdit}
        onDelete={vi.fn()}
        onToggle={vi.fn()}
      />
    );

    await user.dblClick(screen.getByTestId("group-item-group-1"));
    expect(onEdit).toHaveBeenCalledTimes(1);
  });

  it("should show context menu with Edit, Enable/Disable, and Delete", async () => {
    const user = userEvent.setup();

    render(
      <GroupItem
        group={mockGroup}
        comboCount={5}
        isSelected={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onToggle={vi.fn()}
      />
    );

    await user.pointer({
      keys: "[MouseRight>]",
      target: screen.getByTestId("group-item-group-1"),
    });

    expect(screen.getByText("Edit")).toBeInTheDocument();
    expect(screen.getByText("Disable")).toBeInTheDocument();
    expect(screen.getByText("Delete")).toBeInTheDocument();
  });

  it("should show 'Enable' in context menu when group is disabled", async () => {
    const user = userEvent.setup();
    const disabledGroup = { ...mockGroup, enabled: false };

    render(
      <GroupItem
        group={disabledGroup}
        comboCount={5}
        isSelected={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onToggle={vi.fn()}
      />
    );

    await user.pointer({
      keys: "[MouseRight>]",
      target: screen.getByTestId("group-item-group-1"),
    });

    expect(screen.getByText("Enable")).toBeInTheDocument();
  });
});
