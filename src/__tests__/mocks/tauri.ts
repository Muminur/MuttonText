import { vi } from "vitest";

/**
 * Mock for @tauri-apps/api/core invoke function
 */
export const mockInvoke = vi.fn();

/**
 * Mock for @tauri-apps/api/event listen function
 */
export const mockListen = vi.fn(() => Promise.resolve(() => {}));

/**
 * Mock for @tauri-apps/api/event emit function
 */
export const mockEmit = vi.fn(() => Promise.resolve());

/**
 * Setup Tauri API mocks
 * Call this in your test setup or individual test files
 */
export const setupTauriMocks = () => {
  vi.mock("@tauri-apps/api/core", () => ({
    invoke: mockInvoke,
  }));

  vi.mock("@tauri-apps/api/event", () => ({
    listen: mockListen,
    emit: mockEmit,
  }));
};

/**
 * Reset all Tauri mocks
 * Call this in afterEach to clean up between tests
 */
export const resetTauriMocks = () => {
  mockInvoke.mockReset();
  mockListen.mockReset();
  mockEmit.mockReset();
};

/**
 * Mock successful invoke response
 */
export const mockInvokeSuccess = <T>(returnValue: T) => {
  mockInvoke.mockResolvedValueOnce(returnValue);
};

/**
 * Mock invoke error
 */
export const mockInvokeError = (error: string) => {
  mockInvoke.mockRejectedValueOnce(new Error(error));
};

/**
 * Helper to create a mock event listener
 */
export const createMockListener = () => {
  const handlers: Array<(event: unknown) => void> = [];
  const unlisten = vi.fn();

  const listen = vi.fn((callback: (event: unknown) => void) => {
    handlers.push(callback);
    return Promise.resolve(unlisten);
  });

  const emit = (event: unknown) => {
    handlers.forEach((handler) => handler(event));
  };

  return { listen, emit, unlisten, handlers };
};
