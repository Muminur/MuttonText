import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  mockInvoke,
  mockListen,
  mockEmit,
  resetTauriMocks,
  mockInvokeSuccess,
  mockInvokeError,
  createMockListener,
} from "./tauri";

describe("Tauri Mocks", () => {
  beforeEach(() => {
    resetTauriMocks();
  });

  describe("mockInvokeSuccess", () => {
    it("should mock successful invoke with return value", async () => {
      mockInvokeSuccess({ data: "test" });

      const result = await mockInvoke();

      expect(result).toEqual({ data: "test" });
    });
  });

  describe("mockInvokeError", () => {
    it("should mock invoke error", async () => {
      mockInvokeError("Test error");

      await expect(mockInvoke()).rejects.toThrow("Test error");
    });
  });

  describe("createMockListener", () => {
    it("should create a mock event listener", async () => {
      const { listen, emit } = createMockListener();

      const handler = vi.fn();
      await listen(handler);

      emit({ payload: "test" });

      expect(handler).toHaveBeenCalledWith({ payload: "test" });
    });

    it("should support multiple handlers", async () => {
      const { listen, emit } = createMockListener();

      const handler1 = vi.fn();
      const handler2 = vi.fn();

      await listen(handler1);
      await listen(handler2);

      emit({ payload: "test" });

      expect(handler1).toHaveBeenCalledWith({ payload: "test" });
      expect(handler2).toHaveBeenCalledWith({ payload: "test" });
    });
  });

  describe("resetTauriMocks", () => {
    it("should reset all mocks", () => {
      mockInvoke.mockReturnValue("test");
      mockListen.mockReturnValue(Promise.resolve(() => {}));
      mockEmit.mockReturnValue(Promise.resolve());

      resetTauriMocks();

      expect(mockInvoke.mock.calls.length).toBe(0);
      expect(mockListen.mock.calls.length).toBe(0);
      expect(mockEmit.mock.calls.length).toBe(0);
    });
  });
});
