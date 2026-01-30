import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ContentArea } from "../ContentArea";

describe("ContentArea", () => {
  it("should render search input", () => {
    render(<ContentArea>Content</ContentArea>);

    expect(screen.getByTestId("search-input")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Search combos...")).toBeInTheDocument();
  });

  it("should render new combo button", () => {
    render(<ContentArea>Content</ContentArea>);

    expect(screen.getByTestId("new-combo-button")).toBeInTheDocument();
    expect(screen.getByText("New Combo")).toBeInTheDocument();
  });

  it("should render view toggle buttons", () => {
    render(<ContentArea>Content</ContentArea>);

    expect(screen.getByTestId("view-list-button")).toBeInTheDocument();
    expect(screen.getByTestId("view-grid-button")).toBeInTheDocument();
  });

  it("should render children content", () => {
    render(
      <ContentArea>
        <div data-testid="child-content">My Content</div>
      </ContentArea>
    );

    expect(screen.getByTestId("child-content")).toBeInTheDocument();
    expect(screen.getByText("My Content")).toBeInTheDocument();
  });

  it("should toggle view mode when buttons are clicked", async () => {
    const user = userEvent.setup();
    render(<ContentArea>Content</ContentArea>);

    const listButton = screen.getByTestId("view-list-button");
    const gridButton = screen.getByTestId("view-grid-button");

    // Initially list view is active
    expect(listButton).toHaveClass("bg-gray-200");
    expect(gridButton).not.toHaveClass("bg-gray-200");

    // Click grid button
    await user.click(gridButton);
    expect(gridButton).toHaveClass("bg-gray-200");
    expect(listButton).not.toHaveClass("bg-gray-200");

    // Click list button
    await user.click(listButton);
    expect(listButton).toHaveClass("bg-gray-200");
    expect(gridButton).not.toHaveClass("bg-gray-200");
  });

  it("should call console.log when new combo button is clicked", async () => {
    const user = userEvent.setup();
    const consoleSpy = vi.spyOn(console, "log");
    render(<ContentArea>Content</ContentArea>);

    await user.click(screen.getByTestId("new-combo-button"));

    expect(consoleSpy).toHaveBeenCalledWith("New Combo");
  });

  it("should allow typing in search input", async () => {
    const user = userEvent.setup();
    render(<ContentArea>Content</ContentArea>);

    const searchInput = screen.getByTestId("search-input") as HTMLInputElement;
    await user.type(searchInput, "test query");

    expect(searchInput.value).toBe("test query");
  });
});
