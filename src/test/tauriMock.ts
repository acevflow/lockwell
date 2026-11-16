/**
 * Test double for the Tauri commands. `@tauri-apps/api/core` is mocked in
 * `setup.ts`; tests describe how each command answers with `mockCommands`.
 * A command without a handler fails the call, so unexpected calls surface.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { vi } from "vitest";

type Handler = (args: Record<string, unknown>) => unknown;

/** Makes `invoke` answer with `handlers`; returns the mock to inspect calls. */
export function mockCommands(handlers: Record<string, Handler>) {
  const mocked = vi.mocked(invoke);
  mocked.mockImplementation(async (command, args) => {
    const handler = handlers[command];
    if (!handler) {
      throw { code: "unknown", message: `unexpected command ${command}` };
    }
    return handler((args ?? {}) as Record<string, unknown>);
  });
  return mocked;
}

/** A command error as the core serializes it. */
export function commandError(code: string, message: string) {
  return { code, message };
}

/** Handler that rejects with a command error. */
export function failWith(code: string, message: string): Handler {
  return () => {
    throw commandError(code, message);
  };
}

/**
 * Delivers an event from the core to every listener registered for `name`
 * with the mocked `listen`.
 */
export function emitEvent(name: string, payload: unknown) {
  for (const [event, handler] of vi.mocked(listen).mock.calls) {
    if (event === name) {
      handler({ event: name, id: 0, payload });
    }
  }
}

/** The arguments of every call to `command`. */
export function callsTo(command: string): Record<string, unknown>[] {
  return vi
    .mocked(invoke)
    .mock.calls.filter(([name]) => name === command)
    .map(([, args]) => (args ?? {}) as Record<string, unknown>);
}
