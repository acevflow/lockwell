import { listen } from "@tauri-apps/api/event";
import { act, fireEvent, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ApiError } from "../api/client";
import { VAULT_LOCKED_EVENT } from "../api/security";
import { callsTo, emitEvent, failWith, mockCommands } from "../test/tauriMock";
import {
  ACTIVITY_REPORT_INTERVAL_MS,
  useActivityReporter,
} from "./useActivityReporter";
import { useApiError } from "./useApiError";
import { useInitialLoad } from "./useInitialLoad";
import { useVaultLockedEvent } from "./useVaultLockedEvent";

/** A fixed, arbitrary point in time, so the tests never read the real clock. */
const START = new Date("2026-01-01T00:00:00Z");

/** A promise that the test settles by hand. */
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("useActivityReporter", () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(START);
    mockCommands({ touch_activity: () => undefined });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it.each([
    "keyDown",
    "mouseDown",
    "mouseMove",
    "wheel",
    "touchStart",
  ] as const)("reports a %s event", (event) => {
    renderHook(() => useActivityReporter());

    fireEvent[event](window);

    expect(callsTo("touch_activity")).toHaveLength(1);
  });

  it("reports at most once per interval", () => {
    renderHook(() => useActivityReporter());

    fireEvent.keyDown(window);
    vi.setSystemTime(START.getTime() + ACTIVITY_REPORT_INTERVAL_MS - 1);
    fireEvent.keyDown(window);

    expect(callsTo("touch_activity")).toHaveLength(1);
  });

  it("reports again once the interval has passed", () => {
    renderHook(() => useActivityReporter());

    fireEvent.keyDown(window);
    vi.setSystemTime(START.getTime() + ACTIVITY_REPORT_INTERVAL_MS);
    fireEvent.keyDown(window);

    expect(callsTo("touch_activity")).toHaveLength(2);
  });

  it("stops listening when unmounted", () => {
    const { unmount } = renderHook(() => useActivityReporter());

    unmount();
    fireEvent.keyDown(window);

    expect(callsTo("touch_activity")).toHaveLength(0);
  });

  it("keeps reporting after a failed report", async () => {
    // Vitest fails the run on an unhandled rejection, so a failure that is
    // not swallowed would also surface here.
    mockCommands({ touch_activity: failWith("io", "unavailable") });
    renderHook(() => useActivityReporter());

    fireEvent.keyDown(window);
    await act(() => Promise.resolve());
    vi.setSystemTime(START.getTime() + ACTIVITY_REPORT_INTERVAL_MS);
    fireEvent.keyDown(window);
    await act(() => Promise.resolve());

    expect(callsTo("touch_activity")).toHaveLength(2);
  });
});

describe("useApiError", () => {
  it("starts without an error", () => {
    const { result } = renderHook(() => useApiError(vi.fn()));

    expect(result.current.error).toBeNull();
  });

  it("stores the message of other errors", () => {
    const onLocked = vi.fn();
    const { result } = renderHook(() => useApiError(onLocked));

    act(() => result.current.report(new ApiError("io", "the disk is full")));

    expect(result.current.error).toBe("The disk is full.");
    expect(onLocked).not.toHaveBeenCalled();
  });

  it("calls onLocked instead of showing a locked error", () => {
    const onLocked = vi.fn();
    const { result } = renderHook(() => useApiError(onLocked));

    act(() => result.current.report(new ApiError("locked", "locked")));

    expect(onLocked).toHaveBeenCalledOnce();
    expect(result.current.error).toBeNull();
  });

  it("clears the error", () => {
    const { result } = renderHook(() => useApiError(vi.fn()));
    act(() => result.current.report(new Error("boom")));

    act(() => result.current.clear());

    expect(result.current.error).toBeNull();
  });
});

describe("useVaultLockedEvent", () => {
  it("passes the lock reason to the handler", async () => {
    const onLocked = vi.fn();
    renderHook(() => useVaultLockedEvent(onLocked));
    await act(() => Promise.resolve());

    emitEvent(VAULT_LOCKED_EVENT, { reason: "idle" });

    expect(onLocked).toHaveBeenCalledExactlyOnceWith("idle");
  });

  it("stops listening when unmounted", async () => {
    const stop = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(stop);
    const { unmount } = renderHook(() => useVaultLockedEvent(vi.fn()));
    await act(() => Promise.resolve());

    unmount();

    expect(stop).toHaveBeenCalledOnce();
  });

  it("stops listening at once if unmounted before listening starts", async () => {
    const stop = vi.fn();
    const pending = deferred<() => void>();
    vi.mocked(listen).mockReturnValueOnce(pending.promise);
    const { unmount } = renderHook(() => useVaultLockedEvent(vi.fn()));

    unmount();
    await act(async () => pending.resolve(stop));

    expect(stop).toHaveBeenCalledOnce();
  });

  it("logs when it cannot listen", async () => {
    const logged = vi.spyOn(console, "error").mockImplementation(() => {});
    const failure = new Error("no event system");
    vi.mocked(listen).mockRejectedValueOnce(failure);

    renderHook(() => useVaultLockedEvent(vi.fn()));
    await act(() => Promise.resolve());

    expect(logged).toHaveBeenCalledExactlyOnceWith(
      "lockwell: cannot listen for vault lock events",
      failure,
    );
    logged.mockRestore();
  });
});

describe("useInitialLoad", () => {
  it("passes the result to onLoaded", async () => {
    const onLoaded = vi.fn();
    const onError = vi.fn();
    const load = vi.fn().mockResolvedValue(["a"]);

    renderHook(() => useInitialLoad(load, onLoaded, onError));
    await act(async () => {});

    expect(onLoaded).toHaveBeenCalledWith(["a"]);
    expect(onError).not.toHaveBeenCalled();
  });

  it("passes a failure to onError", async () => {
    const onLoaded = vi.fn();
    const onError = vi.fn();
    const failure = new Error("nope");
    const load = vi.fn().mockRejectedValue(failure);

    renderHook(() => useInitialLoad(load, onLoaded, onError));
    await act(async () => {});

    expect(onError).toHaveBeenCalledWith(failure);
    expect(onLoaded).not.toHaveBeenCalled();
  });

  it("ignores results that arrive after unmounting", async () => {
    const onLoaded = vi.fn();
    const onError = vi.fn();
    const pending = deferred<string>();

    const { unmount } = renderHook(() =>
      useInitialLoad(() => pending.promise, onLoaded, onError),
    );
    unmount();
    await act(async () => pending.resolve("late"));

    expect(onLoaded).not.toHaveBeenCalled();
  });
});
