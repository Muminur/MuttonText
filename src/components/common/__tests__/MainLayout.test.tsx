import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { MainLayout } from "../MainLayout";

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, "localStorage", {
  value: localStorageMock,
  writable: true,
});

describe("MainLayout", () => {
  beforeEach(() => {
    localStorageMock.clear();
  });

  afterEach(() => {
    localStorageMock.clear();
  });

  it("should render menu bar, sidebar, and content area", () => {
    render(
      <MainLayout>
        <div data-testid="test-content">Test Content</div>
      </MainLayout>
    );

    expect(screen.getByTestId("sidebar-slot")).toBeInTheDocument();
    expect(screen.getByTestId("content-slot")).toBeInTheDocument();
    expect(screen.getByTestId("test-content")).toBeInTheDocument();
  });

  it("should render children in content slot", () => {
    render(
      <MainLayout>
        <p>My custom content</p>
      </MainLayout>
    );

    expect(screen.getByText("My custom content")).toBeInTheDocument();
  });

  it("should have default sidebar width", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    const sidebar = screen.getByTestId("sidebar-slot");
    expect(sidebar).toHaveStyle({ width: "250px" });
  });

  it("should restore sidebar width from localStorage", () => {
    localStorageMock.setItem("muttontext-sidebar-width", "300");

    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    const sidebar = screen.getByTestId("sidebar-slot");
    expect(sidebar).toHaveStyle({ width: "300px" });
  });

  it("should render resize handle", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    expect(screen.getByTestId("sidebar-resize-handle")).toBeInTheDocument();
  });

  it("should render MenuBar component", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    expect(screen.getByTestId("file-menu-trigger")).toBeInTheDocument();
    expect(screen.getByTestId("edit-menu-trigger")).toBeInTheDocument();
  });

  it("should render Sidebar component", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    expect(screen.getByTestId("add-group-button")).toBeInTheDocument();
  });
});
