// Registers the Testing Library DOM matchers, replaces the Tauri bridge and
// event system with mocks (see tauriMock.ts) and cleans up after each test.
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});
