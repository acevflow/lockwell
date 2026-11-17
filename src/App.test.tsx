/** Tests of the root component: which screen it shows and how it reacts to locks. */
import { listen } from "@tauri-apps/api/event";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "./App";
import { callsTo, emitEvent, failWith, mockCommands } from "./test/tauriMock";

const noEntries = { list_entries: () => [] };

describe("App", () => {
  it("shows the setup screen when there is no vault", async () => {
    mockCommands({ vault_status: () => ({ exists: false, unlocked: false }) });
    render(<App />);
    expect(
      await screen.findByRole("heading", { name: "Create your vault" }),
    ).toBeInTheDocument();
  });

  it("shows the unlock screen when the vault is locked", async () => {
    mockCommands({ vault_status: () => ({ exists: true, unlocked: false }) });
    render(<App />);
    expect(await screen.findByText("Your vault is locked.")).toBeVisible();
  });

  it("shows the vault when it is already unlocked", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      ...noEntries,
    });
    render(<App />);
    expect(await screen.findByRole("tab", { name: "Passwords" })).toBeVisible();
  });

  it("reports a status failure", async () => {
    mockCommands({
      vault_status: failWith("unsupported_version", "the vault is too new"),
    });
    render(<App />);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The vault is too new.",
    );
  });

  it("locks and returns to the unlock screen", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      lock_vault: () => null,
      ...noEntries,
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Lock" }));
    expect(await screen.findByText("Your vault is locked.")).toBeVisible();
    expect(callsTo("lock_vault")).toHaveLength(1);
  });

  it("leaves the vault screen even if locking fails", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      lock_vault: failWith("storage", "busy"),
      ...noEntries,
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Lock" }));
    expect(await screen.findByText("Your vault is locked.")).toBeVisible();
  });

  it("locks when the core emits the lock event and says why", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      ...noEntries,
    });
    render(<App />);
    await screen.findByRole("tab", { name: "Passwords" });
    await waitFor(() => expect(vi.mocked(listen)).toHaveBeenCalled());
    act(() => emitEvent("vault-locked", { reason: "idle" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "locked after 5 minutes of inactivity",
    );
  });

  it("explains a lock caused by Windows", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      ...noEntries,
    });
    render(<App />);
    await screen.findByRole("tab", { name: "Passwords" });
    await waitFor(() => expect(vi.mocked(listen)).toHaveBeenCalled());
    act(() => emitEvent("vault-locked", { reason: "systemLock" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "because Windows was locked",
    );
  });

  it("returns to the unlock screen after restoring a backup", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      restore_backup: () => true,
      ...noEntries,
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("tab", { name: "Settings" }));
    fireEvent.change(screen.getByLabelText("Backup master password"), {
      target: { value: "old test phrase" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Choose backup and restore" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Replace vault" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Backup restored.",
    );
    expect(screen.getByText("Your vault is locked.")).toBeVisible();
  });

  it("reports activity to the core at most once per interval", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      touch_activity: () => null,
      ...noEntries,
    });
    render(<App />);
    await screen.findByRole("tab", { name: "Passwords" });
    fireEvent.keyDown(window, { key: "a" });
    fireEvent.mouseMove(window);
    fireEvent.keyDown(window, { key: "b" });
    expect(callsTo("touch_activity")).toHaveLength(1);
  });

  it("goes to the unlock screen when the core reports a locked vault", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      list_entries: failWith("locked", "the vault is locked"),
    });
    render(<App />);
    expect(await screen.findByText("Your vault is locked.")).toBeVisible();
  });

  it("shows that it is loading until the core answers", () => {
    mockCommands({ vault_status: () => new Promise(() => {}) });
    render(<App />);
    expect(screen.getByText("Loading…")).toBeVisible();
  });

  it("opens the vault once it is created", async () => {
    mockCommands({
      vault_status: () => ({ exists: false, unlocked: false }),
      create_vault: () => null,
      ...noEntries,
    });
    render(<App />);
    await screen.findByRole("heading", { name: "Create your vault" });
    fireEvent.change(screen.getByLabelText("Master password"), {
      target: { value: "a long test phrase" },
    });
    fireEvent.change(screen.getByLabelText("Confirm master password"), {
      target: { value: "a long test phrase" },
    });
    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.click(screen.getByRole("button", { name: "Create vault" }));
    expect(await screen.findByRole("tab", { name: "Passwords" })).toBeVisible();
  });
});
