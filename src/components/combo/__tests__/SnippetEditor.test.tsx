import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { SnippetEditor } from "../SnippetEditor";

describe("SnippetEditor", () => {
  it("renders non-variable text visibly in the overlay", () => {
    render(<SnippetEditor value="hello world" onChange={vi.fn()} />);
    const overlay = screen.getByTestId("syntax-overlay");
    const spans = overlay.querySelectorAll("span");
    // No span should have text-transparent class
    spans.forEach((span) => {
      expect(span.className).not.toContain("text-transparent");
    });
  });

  it("highlights variable syntax with blue color", () => {
    render(<SnippetEditor value="Hello #{name}!" onChange={vi.fn()} />);
    const overlay = screen.getByTestId("syntax-overlay");
    const blueSpan = overlay.querySelector(".text-blue-600");
    expect(blueSpan).not.toBeNull();
    expect(blueSpan?.textContent).toBe("#{name}");
  });

  it("renders mixed text with visible non-variable and highlighted variable parts", () => {
    render(
      <SnippetEditor value="Dear #{name}, welcome!" onChange={vi.fn()} />
    );
    const overlay = screen.getByTestId("syntax-overlay");
    // Non-variable text should be visible (not transparent)
    const allSpans = overlay.querySelectorAll("span");
    const transparentSpans = Array.from(allSpans).filter((s) =>
      s.className.includes("text-transparent")
    );
    expect(transparentSpans.length).toBe(0);
  });
});
