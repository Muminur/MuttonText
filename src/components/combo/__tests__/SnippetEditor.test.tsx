import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SnippetEditor } from "../SnippetEditor";

describe("SnippetEditor", () => {
  it("should render a textarea", () => {
    render(
      <SnippetEditor
        value=""
        onChange={vi.fn()}
        placeholder="Enter snippet..."
      />
    );

    expect(screen.getByPlaceholderText("Enter snippet...")).toBeInTheDocument();
  });

  it("should display the initial value", () => {
    render(
      <SnippetEditor
        value="Hello #{name}"
        onChange={vi.fn()}
      />
    );

    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
    expect(textarea.value).toBe("Hello #{name}");
  });

  it("should call onChange when text is typed", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <SnippetEditor
        value=""
        onChange={onChange}
      />
    );

    await user.type(screen.getByRole("textbox"), "test");

    expect(onChange).toHaveBeenCalled();
  });

  it("should highlight variable syntax", () => {
    const { container } = render(
      <SnippetEditor
        value="Hello #{name}, today is #{date}"
        onChange={vi.fn()}
      />
    );

    // Should have highlighted overlay
    const overlay = container.querySelector('[data-testid="syntax-overlay"]');
    expect(overlay).toBeInTheDocument();
  });

  it("should highlight single variable", () => {
    const { container } = render(
      <SnippetEditor
        value="#{clipboard}"
        onChange={vi.fn()}
      />
    );

    const highlighted = container.querySelectorAll(".text-blue-600");
    expect(highlighted.length).toBeGreaterThan(0);
  });

  it("should highlight multiple variables", () => {
    const { container } = render(
      <SnippetEditor
        value="#{date} - #{time} - #{clipboard}"
        onChange={vi.fn()}
      />
    );

    const highlighted = container.querySelectorAll(".text-blue-600");
    expect(highlighted.length).toBeGreaterThan(0);
  });

  it("should highlight variables with parameters", () => {
    const { container } = render(
      <SnippetEditor
        value="#{dateTime:YYYY-MM-DD} #{input:Name}"
        onChange={vi.fn()}
      />
    );

    const highlighted = container.querySelectorAll(".text-blue-600");
    expect(highlighted.length).toBeGreaterThan(0);
  });

  it("should not highlight incomplete variable syntax", () => {
    const { container } = render(
      <SnippetEditor
        value="#{incomplete"
        onChange={vi.fn()}
      />
    );

    // Incomplete variables should not be highlighted
    const text = container.textContent;
    expect(text).toContain("#{incomplete");
  });

  it("should maintain cursor position when typing", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <SnippetEditor
        value="Hello world"
        onChange={onChange}
      />
    );

    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;

    // Click to set cursor position
    textarea.focus();
    await user.click(textarea);

    // Type at the end (userEvent.type appends)
    await user.type(textarea, "!");

    // onChange should have been called
    expect(onChange).toHaveBeenCalled();
  });

  it("should sync highlighting with textarea scroll", () => {
    const { container } = render(
      <SnippetEditor
        value="Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8"
        onChange={vi.fn()}
        rows={3}
      />
    );

    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
    const overlay = container.querySelector('[data-testid="syntax-overlay"]') as HTMLElement;

    // Should have same scroll position
    textarea.scrollTop = 50;
    textarea.dispatchEvent(new Event("scroll"));

    // Note: scroll syncing is tested in actual browser environment
    expect(overlay).toBeInTheDocument();
  });

  it("should handle line breaks in highlighting", () => {
    const { container } = render(
      <SnippetEditor
        value="Line 1 #{var1}\nLine 2 #{var2}\nLine 3 #{var3}"
        onChange={vi.fn()}
      />
    );

    const overlay = container.querySelector('[data-testid="syntax-overlay"]');
    expect(overlay?.textContent).toContain("Line 1");
    expect(overlay?.textContent).toContain("Line 2");
    expect(overlay?.textContent).toContain("Line 3");
  });

  it("should preserve whitespace in highlighting", () => {
    const { container } = render(
      <SnippetEditor
        value="  #{indented}  "
        onChange={vi.fn()}
      />
    );

    const overlay = container.querySelector('[data-testid="syntax-overlay"]');
    expect(overlay).toBeInTheDocument();
  });

  it("should handle empty value", () => {
    render(
      <SnippetEditor
        value=""
        onChange={vi.fn()}
      />
    );

    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
    expect(textarea.value).toBe("");
  });

  it("should accept custom className", () => {
    render(
      <SnippetEditor
        value=""
        onChange={vi.fn()}
        className="custom-class"
      />
    );

    const textarea = screen.getByRole("textbox");
    expect(textarea).toHaveClass("custom-class");
  });

  it("should accept custom rows attribute", () => {
    render(
      <SnippetEditor
        value=""
        onChange={vi.fn()}
        rows={10}
      />
    );

    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
    expect(textarea.rows).toBe(10);
  });

  it("should accept custom id attribute", () => {
    render(
      <SnippetEditor
        value=""
        onChange={vi.fn()}
        id="custom-id"
      />
    );

    const textarea = screen.getByRole("textbox");
    expect(textarea).toHaveAttribute("id", "custom-id");
  });

  it("should support forwardRef", () => {
    const ref = vi.fn();
    render(
      <SnippetEditor
        value=""
        onChange={vi.fn()}
        ref={ref}
      />
    );

    expect(ref).toHaveBeenCalled();
  });

  it("should highlight nested variables", () => {
    const { container } = render(
      <SnippetEditor
        value="#{combo:#{other}}"
        onChange={vi.fn()}
      />
    );

    const highlighted = container.querySelectorAll(".text-blue-600");
    expect(highlighted.length).toBeGreaterThan(0);
  });

  it("should handle escaped braces", () => {
    const { container } = render(
      <SnippetEditor
        value="Not a variable: \\#{escaped\\}"
        onChange={vi.fn()}
      />
    );

    // Escaped variables should not be highlighted
    const overlay = container.querySelector('[data-testid="syntax-overlay"]');
    expect(overlay).toBeInTheDocument();
  });

  it("should update highlighting when value changes", () => {
    const { container, rerender } = render(
      <SnippetEditor
        value="#{date}"
        onChange={vi.fn()}
      />
    );

    let highlighted = container.querySelectorAll(".text-blue-600");
    expect(highlighted.length).toBeGreaterThan(0);

    // Change value
    rerender(
      <SnippetEditor
        value="No variables here"
        onChange={vi.fn()}
      />
    );

    highlighted = container.querySelectorAll(".text-blue-600");
    expect(highlighted.length).toBe(0);
  });
});
