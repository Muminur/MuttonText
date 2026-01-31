import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InsertVariableMenu } from "../InsertVariableMenu";

describe("InsertVariableMenu", () => {
  it("should render the trigger button", () => {
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    expect(screen.getByRole("button", { name: /insert variable/i })).toBeInTheDocument();
  });

  it("should show variable categories when opened", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));

    // Check for category headers
    expect(screen.getByText("Date/Time")).toBeInTheDocument();
    expect(screen.getByText("Clipboard")).toBeInTheDocument();
    expect(screen.getByText("References")).toBeInTheDocument();
    expect(screen.getByText("Interactive")).toBeInTheDocument();
    expect(screen.getByText("System")).toBeInTheDocument();
    expect(screen.getByText("Keys")).toBeInTheDocument();
    expect(screen.getByText("Script")).toBeInTheDocument();
  });

  it("should show variable names and descriptions", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));

    // Check for variable names
    expect(screen.getByText("#{date}")).toBeInTheDocument();
    expect(screen.getByText("#{time}")).toBeInTheDocument();
    expect(screen.getByText("#{clipboard}")).toBeInTheDocument();

    // Check for descriptions
    expect(screen.getByText(/current date in locale format/i)).toBeInTheDocument();
    expect(screen.getByText(/current time in locale format/i)).toBeInTheDocument();
    expect(screen.getByText(/current clipboard content/i)).toBeInTheDocument();
  });

  it("should call onInsert with variable syntax when clicked", async () => {
    const user = userEvent.setup();
    const onInsert = vi.fn();
    render(<InsertVariableMenu onInsert={onInsert} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));
    await user.click(screen.getByText("#{date}"));

    expect(onInsert).toHaveBeenCalledWith("#{date}");
  });

  it("should close menu after variable selection", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));
    expect(screen.getByText("#{date}")).toBeInTheDocument();

    await user.click(screen.getByText("#{date}"));

    // Menu should close after selection
    await new Promise(resolve => setTimeout(resolve, 100));
    expect(screen.queryByText("#{date}")).not.toBeInTheDocument();
  });

  it("should show all date/time variables", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));

    expect(screen.getByText("#{date}")).toBeInTheDocument();
    expect(screen.getByText("#{time}")).toBeInTheDocument();
    expect(screen.getByText("#{dateTime}")).toBeInTheDocument();
    expect(screen.getByText("#{dateTime:format}")).toBeInTheDocument();
    expect(screen.getByText("#{dateTime:shift:format}")).toBeInTheDocument();
  });

  it("should show all reference variables", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));

    expect(screen.getByText("#{combo:keyword}")).toBeInTheDocument();
    expect(screen.getByText("#{lower:keyword}")).toBeInTheDocument();
    expect(screen.getByText("#{upper:keyword}")).toBeInTheDocument();
  });

  it("should show all interactive variables", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));

    expect(screen.getByText("#{cursor}")).toBeInTheDocument();
    expect(screen.getByText("#{input:prompt}")).toBeInTheDocument();
  });

  it("should show key simulation variables", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));

    expect(screen.getByText("#{key:name}")).toBeInTheDocument();
    expect(screen.getByText("#{key:name:count}")).toBeInTheDocument();
    expect(screen.getByText("#{shortcut:keys}")).toBeInTheDocument();
    expect(screen.getByText("#{delay:ms}")).toBeInTheDocument();
  });

  it("should mark script variable as advanced", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));

    const scriptSection = screen.getByText("Script").closest("div");
    expect(scriptSection).toHaveTextContent(/advanced/i);
  });

  it("should handle keyboard navigation", async () => {
    const user = userEvent.setup();
    const onInsert = vi.fn();
    render(<InsertVariableMenu onInsert={onInsert} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));

    // Keyboard navigation should work in dropdown
    await user.keyboard("{ArrowDown}");
    await user.keyboard("{Enter}");

    expect(onInsert).toHaveBeenCalled();
  });

  it("should close on Escape key", async () => {
    const user = userEvent.setup();
    render(<InsertVariableMenu onInsert={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /insert variable/i }));
    expect(screen.getByText("#{date}")).toBeInTheDocument();

    await user.keyboard("{Escape}");

    await new Promise(resolve => setTimeout(resolve, 100));
    expect(screen.queryByText("#{date}")).not.toBeInTheDocument();
  });
});
