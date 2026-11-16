import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { callsTo, failWith, mockCommands } from "../test/tauriMock";
import SettingsTab from "./SettingsTab";

function submit(current: string, next: string, confirmation = next) {
  fireEvent.change(screen.getByLabelText("Current master password"), {
    target: { value: current },
  });
  fireEvent.change(screen.getByLabelText("New master password"), {
    target: { value: next },
  });
  fireEvent.change(screen.getByLabelText("Confirm new master password"), {
    target: { value: confirmation },
  });
  fireEvent.click(screen.getByRole("button", { name: "Change password" }));
}

describe("SettingsTab", () => {
  it("changes the master password", async () => {
    mockCommands({ change_master_password: () => null });
    render(<SettingsTab onLocked={vi.fn()} onRestored={vi.fn()} />);
    submit("old test phrase", "new test phrase!");
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Master password changed.",
    );
    expect(callsTo("change_master_password")).toEqual([
      { current: "old test phrase", new: "new test phrase!" },
    ]);
    expect(screen.getByLabelText("Current master password")).toHaveValue("");
  });

  it("validates the new password locally", () => {
    mockCommands({});
    render(<SettingsTab onLocked={vi.fn()} onRestored={vi.fn()} />);
    submit("old test phrase", "short");
    expect(screen.getByRole("alert")).toHaveTextContent("at least 12");
    submit("old test phrase", "new test phrase!", "different phrase");
    expect(screen.getByRole("alert")).toHaveTextContent("do not match");
    submit("", "new test phrase!");
    expect(screen.getByRole("alert")).toHaveTextContent(
      "current master password",
    );
    expect(callsTo("change_master_password")).toHaveLength(0);
  });

  it("reports a wrong current password", async () => {
    mockCommands({
      change_master_password: failWith("wrong_password", "incorrect"),
    });
    render(<SettingsTab onLocked={vi.fn()} onRestored={vi.fn()} />);
    submit("wrong test phrase", "new test phrase!");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The current master password is incorrect.",
    );
  });

  it("calls onLocked when the vault is locked", async () => {
    mockCommands({ change_master_password: failWith("locked", "locked") });
    const onLocked = vi.fn();
    render(<SettingsTab onLocked={onLocked} onRestored={vi.fn()} />);
    submit("old test phrase", "new test phrase!");
    await vi.waitFor(() => expect(onLocked).toHaveBeenCalledOnce());
  });

  it("clears every field after a change", async () => {
    mockCommands({ change_master_password: () => null });
    render(<SettingsTab onLocked={vi.fn()} onRestored={vi.fn()} />);
    submit("old test phrase", "new test phrase!");
    await screen.findByRole("status");
    expect(screen.getByLabelText("New master password")).toHaveValue("");
    expect(screen.getByLabelText("Confirm new master password")).toHaveValue(
      "",
    );
  });

  it("disables the form while changing", () => {
    mockCommands({ change_master_password: () => new Promise(() => {}) });
    render(<SettingsTab onLocked={vi.fn()} onRestored={vi.fn()} />);
    submit("old test phrase", "new test phrase!");
    expect(screen.getByRole("button", { name: "Changing…" })).toBeDisabled();
  });

  it("shows other errors with their message and no password", async () => {
    mockCommands({
      change_master_password: failWith("storage", "the database is busy"),
    });
    render(<SettingsTab onLocked={vi.fn()} onRestored={vi.fn()} />);
    submit("old test phrase", "new test phrase!");
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("The database is busy.");
    expect(alert).not.toHaveTextContent("old test phrase");
    expect(alert).not.toHaveTextContent("new test phrase!");
    expect(
      screen.getByRole("button", { name: "Change password" }),
    ).toBeEnabled();
  });

  it("calls onLocked when a backup finds the vault locked", async () => {
    mockCommands({ create_backup: failWith("locked", "locked") });
    const onLocked = vi.fn();
    render(<SettingsTab onLocked={onLocked} onRestored={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Create backup" }));
    await vi.waitFor(() => expect(onLocked).toHaveBeenCalledOnce());
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows other backup errors", async () => {
    mockCommands({ create_backup: failWith("io", "the disk is full") });
    render(<SettingsTab onLocked={vi.fn()} onRestored={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Create backup" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The disk is full.",
    );
  });
});
