import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MenuBar } from "../MenuBar";

describe("MenuBar", () => {
  it("should render all menu buttons", () => {
    render(<MenuBar />);

    expect(screen.getByTestId("file-menu-trigger")).toBeInTheDocument();
    expect(screen.getByTestId("edit-menu-trigger")).toBeInTheDocument();
    expect(screen.getByTestId("combos-menu-trigger")).toBeInTheDocument();
    expect(screen.getByTestId("groups-menu-trigger")).toBeInTheDocument();
    expect(screen.getByTestId("help-menu-trigger")).toBeInTheDocument();
  });

  it("should display menu labels", () => {
    render(<MenuBar />);

    expect(screen.getByText("File")).toBeInTheDocument();
    expect(screen.getByText("Edit")).toBeInTheDocument();
    expect(screen.getByText("Combos")).toBeInTheDocument();
    expect(screen.getByText("Groups")).toBeInTheDocument();
    expect(screen.getByText("Help")).toBeInTheDocument();
  });

  it("should open File menu on click", async () => {
    const user = userEvent.setup();
    render(<MenuBar />);

    await user.click(screen.getByTestId("file-menu-trigger"));

    expect(screen.getByText("New Combo")).toBeInTheDocument();
    expect(screen.getByText("New Group")).toBeInTheDocument();
    expect(screen.getByText("Import...")).toBeInTheDocument();
    expect(screen.getByText("Export...")).toBeInTheDocument();
    expect(screen.getByText("Exit")).toBeInTheDocument();
  });

  it("should open Edit menu on click", async () => {
    const user = userEvent.setup();
    render(<MenuBar />);

    await user.click(screen.getByTestId("edit-menu-trigger"));

    expect(screen.getByText("Undo")).toBeInTheDocument();
    expect(screen.getByText("Redo")).toBeInTheDocument();
    expect(screen.getByText("Cut")).toBeInTheDocument();
    expect(screen.getByText("Copy")).toBeInTheDocument();
    expect(screen.getByText("Paste")).toBeInTheDocument();
  });

  it("should open Combos menu on click", async () => {
    const user = userEvent.setup();
    render(<MenuBar />);

    await user.click(screen.getByTestId("combos-menu-trigger"));

    expect(screen.getByText("Enable All")).toBeInTheDocument();
    expect(screen.getByText("Disable All")).toBeInTheDocument();
  });

  it("should open Groups menu on click", async () => {
    const user = userEvent.setup();
    render(<MenuBar />);

    await user.click(screen.getByTestId("groups-menu-trigger"));

    expect(screen.getByText("New Group")).toBeInTheDocument();
  });

  it("should open Help menu on click", async () => {
    const user = userEvent.setup();
    render(<MenuBar />);

    await user.click(screen.getByTestId("help-menu-trigger"));

    expect(screen.getByText("About")).toBeInTheDocument();
    expect(screen.getByText("Check for Updates")).toBeInTheDocument();
  });

  it("should call console.log when menu items are clicked", async () => {
    const user = userEvent.setup();
    const consoleSpy = vi.spyOn(console, "log");
    render(<MenuBar />);

    await user.click(screen.getByTestId("file-menu-trigger"));
    await user.click(screen.getByText("New Combo"));

    expect(consoleSpy).toHaveBeenCalledWith("New Combo");
  });
});
