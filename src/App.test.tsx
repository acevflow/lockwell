import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";
import { callsTo, failWith, mockCommands } from "./test/tauriMock";

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

  it("goes to the unlock screen when the core reports a locked vault", async () => {
    mockCommands({
      vault_status: () => ({ exists: true, unlocked: true }),
      list_entries: failWith("locked", "the vault is locked"),
    });
    render(<App />);
    expect(await screen.findByText("Your vault is locked.")).toBeVisible();
  });
});
