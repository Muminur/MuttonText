import { describe, it, expect } from "vitest";
import { render, screen } from "./utils";

// Simple test to verify test utilities work
describe("Test Utilities", () => {
  it("should render component with custom render function", () => {
    const TestComponent = () => <div>Test Content</div>;

    render(<TestComponent />);

    expect(screen.getByText("Test Content")).toBeInTheDocument();
  });
});
