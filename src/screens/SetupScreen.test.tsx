import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { callsTo, failWith, mockCommands } from "../test/tauriMock";
import SetupScreen from "./SetupScreen";

const GOOD_PASSWORD = "a long test phrase";

function fill(password: string, confirmation: string, acknowledge = true) {
  fireEvent.change(screen.getByLabelText("Master password"), {
    target: { value: password },
  });
  fireEvent.change(screen.getByLabelText("Confirm master password"), {
    target: { value: confirmation },
  });
  if (acknowledge) {
    fireEvent.click(screen.getByRole("checkbox"));
  }
  fireEvent.click(screen.getByRole("button", { name: "Create vault" }));
}

describe("SetupScreen", () => {
  it("creates the vault and reports success", async () => {
    mockCommands({ create_vault: () => null });
    const onCreated = vi.fn();
    render(<SetupScreen onCreated={onCreated} onRestored={vi.fn()} />);
    fill(GOOD_PASSWORD, GOOD_PASSWORD);
    await waitFor(() => expect(onCreated).toHaveBeenCalledOnce());
    expect(callsTo("create_vault")).toEqual([{ password: GOOD_PASSWORD }]);
  });

  it("rejects a short password without calling the core", () => {
    mockCommands({});
    render(<SetupScreen onCreated={vi.fn()} onRestored={vi.fn()} />);
    fill("short", "short");
    expect(screen.getByRole("alert")).toHaveTextContent("at least 12");
    expect(callsTo("create_vault")).toHaveLength(0);
  });

  it("rejects mismatched passwords", () => {
    mockCommands({});
    render(<SetupScreen onCreated={vi.fn()} onRestored={vi.fn()} />);
    fill(GOOD_PASSWORD, `${GOOD_PASSWORD}!`);
    expect(screen.getByRole("alert")).toHaveTextContent("do not match");
  });

  it("requires acknowledging that there is no recovery", () => {
    mockCommands({});
    render(<SetupScreen onCreated={vi.fn()} onRestored={vi.fn()} />);
    fill(GOOD_PASSWORD, GOOD_PASSWORD, false);
    expect(screen.getByRole("alert")).toHaveTextContent("cannot be recovered");
    expect(callsTo("create_vault")).toHaveLength(0);
  });

  it("can restore a backup instead", async () => {
    mockCommands({ restore_backup: () => true });
    const onRestored = vi.fn();
    render(<SetupScreen onCreated={vi.fn()} onRestored={onRestored} />);
    fireEvent.click(
      screen.getByRole("button", { name: "Restore from a backup instead" }),
    );
    fireEvent.change(screen.getByLabelText("Backup master password"), {
      target: { value: GOOD_PASSWORD },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Choose backup and restore" }),
    );
    await waitFor(() => expect(onRestored).toHaveBeenCalledOnce());
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("can go back to creating a vault", () => {
    mockCommands({});
    render(<SetupScreen onCreated={vi.fn()} onRestored={vi.fn()} />);
    fireEvent.click(
      screen.getByRole("button", { name: "Restore from a backup instead" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Create a new vault instead" }),
    );
    expect(
      screen.getByRole("heading", { name: "Create your vault" }),
    ).toBeVisible();
  });

  it("shows errors from the core", async () => {
    mockCommands({
      create_vault: failWith("vault_exists", "a vault already exists"),
    });
    const onCreated = vi.fn();
    render(<SetupScreen onCreated={onCreated} onRestored={vi.fn()} />);
    fill(GOOD_PASSWORD, GOOD_PASSWORD);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "A vault already exists.",
    );
    expect(onCreated).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Create vault" })).toBeEnabled();
  });
});
