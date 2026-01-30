import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { GroupEditor } from "../GroupEditor";
import { Group } from "@/lib/types";

const mockGroup: Group = {
  id: "group-1",
  name: "Work",
  description: "Work-related combos",
  enabled: true,
  createdAt: "2024-01-01T00:00:00Z",
  modifiedAt: "2024-01-01T00:00:00Z",
};

describe("GroupEditor", () => {
  it("should not render when open is false", () => {
    render(
      <GroupEditor
        open={false}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />
    );

    expect(screen.queryByTestId("group-editor-dialog")).not.toBeInTheDocument();
  });

  it("should render when open is true", () => {
    render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />
    );

    expect(screen.getByTestId("group-editor-dialog")).toBeInTheDocument();
  });

  it("should show 'New Group' title when creating", () => {
    render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />
    );

    expect(screen.getByText("New Group")).toBeInTheDocument();
  });

  it("should show 'Edit Group' title when editing", () => {
    render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={vi.fn()}
        group={mockGroup}
      />
    );

    expect(screen.getByText("Edit Group")).toBeInTheDocument();
  });

  it("should populate fields when editing existing group", () => {
    render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={vi.fn()}
        group={mockGroup}
      />
    );

    const nameInput = screen.getByTestId("group-name-input") as HTMLInputElement;
    const descInput = screen.getByTestId("group-description-input") as HTMLTextAreaElement;

    expect(nameInput.value).toBe("Work");
    expect(descInput.value).toBe("Work-related combos");
  });

  it("should have empty fields when creating new group", () => {
    render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />
    );

    const nameInput = screen.getByTestId("group-name-input") as HTMLInputElement;
    const descInput = screen.getByTestId("group-description-input") as HTMLTextAreaElement;

    expect(nameInput.value).toBe("");
    expect(descInput.value).toBe("");
  });

  it("should call onClose when cancel button is clicked", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();

    render(
      <GroupEditor
        open={true}
        onClose={onClose}
        onSave={vi.fn()}
      />
    );

    await user.click(screen.getByTestId("group-cancel-button"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("should validate that name is required", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();

    render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={onSave}
      />
    );

    await user.click(screen.getByTestId("group-save-button"));

    expect(screen.getByText("Name is required")).toBeInTheDocument();
    expect(onSave).not.toHaveBeenCalled();
  });

  it("should call onSave with trimmed data when valid", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const onClose = vi.fn();

    render(
      <GroupEditor
        open={true}
        onClose={onClose}
        onSave={onSave}
      />
    );

    await user.type(screen.getByTestId("group-name-input"), "  New Group  ");
    await user.type(screen.getByTestId("group-description-input"), "  My description  ");
    await user.click(screen.getByTestId("group-save-button"));

    expect(onSave).toHaveBeenCalledWith({
      name: "New Group",
      description: "My description",
      enabled: true,
    });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("should preserve enabled state when editing", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const disabledGroup = { ...mockGroup, enabled: false };

    render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={onSave}
        group={disabledGroup}
      />
    );

    await user.click(screen.getByTestId("group-save-button"));

    expect(onSave).toHaveBeenCalledWith({
      name: "Work",
      description: "Work-related combos",
      enabled: false,
    });
  });

  it("should clear validation errors when dialog reopens", async () => {
    const user = userEvent.setup();
    const { rerender } = render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />
    );

    // Trigger validation error
    await user.click(screen.getByTestId("group-save-button"));
    expect(screen.getByText("Name is required")).toBeInTheDocument();

    // Close and reopen
    rerender(
      <GroupEditor
        open={false}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />
    );
    rerender(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />
    );

    expect(screen.queryByText("Name is required")).not.toBeInTheDocument();
  });

  it("should allow entering multi-line description", async () => {
    const user = userEvent.setup();

    render(
      <GroupEditor
        open={true}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />
    );

    const descInput = screen.getByTestId("group-description-input") as HTMLTextAreaElement;
    await user.type(descInput, "Line 1{Enter}Line 2");

    expect(descInput.value).toContain("Line 1\nLine 2");
  });
});
