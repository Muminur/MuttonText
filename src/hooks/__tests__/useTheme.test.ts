import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook } from "@testing-library/react";
import { useTheme } from "../useTheme";

describe("useTheme", () => {
  beforeEach(() => {
    document.documentElement.classList.remove("dark");
  });

  it("adds dark class when theme is dark", () => {
    renderHook(() => useTheme("dark"));
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("removes dark class when theme is light", () => {
    document.documentElement.classList.add("dark");
    renderHook(() => useTheme("light"));
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });

  it("does nothing when theme is undefined", () => {
    renderHook(() => useTheme(undefined));
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });

  it("follows system preference when theme is system", () => {
    // Mock matchMedia
    const mockMatchMedia = vi.fn().mockReturnValue({
      matches: true,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    });
    window.matchMedia = mockMatchMedia;

    renderHook(() => useTheme("system"));
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });
});
