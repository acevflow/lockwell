// Registers the Testing Library DOM matchers, replaces the Tauri bridge with
// a mock (see tauriMock.ts) and cleans up after each test.
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});
