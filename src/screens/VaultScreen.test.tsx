import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { VAULT_LOCKED_EVENT } from "../api/security";
import { callsTo, emitEvent, failWith, mockCommands } from "../test/tauriMock";
import VaultScreen from "./VaultScreen";

function renderScreen() {
  const props = { onLock: vi.fn(), onLocked: vi.fn(), onRestored: vi.fn() };
  render(<VaultScreen {...props} />);
  return props;
}

describe("VaultScreen", () => {
  beforeEach(() => {
    mockCommands({
      list_entries: () => [],
      list_files: () => [],
      touch_activity: () => undefined,
    });
  });

  it("opens on the passwords tab", async () => {
    renderScreen();

    expect(
      await screen.findByRole("searchbox", { name: "Search entries" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Passwords" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("tabpanel")).toHaveAttribute(
      "aria-labelledby",
      "tab-passwords",
    );
  });

  it("switches between tabs", async () => {
    renderScreen();

    fireEvent.click(screen.getByRole("tab", { name: "Files" }));
    expect(
      await screen.findByRole("heading", { name: "Files" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Files" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("tab", { name: "Passwords" })).toHaveAttribute(
      "aria-selected",
      "false",
    );

    fireEvent.click(screen.getByRole("tab", { name: "Settings" }));
    expect(
      screen.getByRole("heading", { name: "Change master password" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("tabpanel")).toHaveAttribute(
      "id",
      "panel-settings",
    );
  });

  it("locks with the Lock button", () => {
    const { onLock } = renderScreen();

    fireEvent.click(screen.getByRole("button", { name: "Lock" }));

    expect(onLock).toHaveBeenCalledOnce();
  });

  it("reports a lock by the core with its reason", async () => {
    const { onLocked } = renderScreen();
    await act(() => Promise.resolve());

    emitEvent(VAULT_LOCKED_EVENT, { reason: "systemLock" });

    expect(onLocked).toHaveBeenCalledExactlyOnceWith("systemLock");
  });

  it("reports a locked error from a tab without a reason", async () => {
    mockCommands({
      list_entries: failWith("locked", "the vault is locked"),
      touch_activity: () => undefined,
    });
    const { onLocked } = renderScreen();

    await vi.waitFor(() => expect(onLocked).toHaveBeenCalledOnce());
    expect(onLocked.mock.calls[0]).toEqual([]);
  });

  it("reports user activity to the core", () => {
    renderScreen();

    fireEvent.keyDown(window);

    expect(callsTo("touch_activity")).toHaveLength(1);
  });
});
