/** Tests of the unlock screen. */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { callsTo, failWith, mockCommands } from "../test/tauriMock";
import UnlockScreen from "./UnlockScreen";

function submit(password: string) {
  fireEvent.change(screen.getByLabelText("Master password"), {
    target: { value: password },
  });
  fireEvent.click(screen.getByRole("button", { name: "Unlock" }));
}

describe("UnlockScreen", () => {
  it("unlocks with the password", async () => {
    mockCommands({ unlock_vault: () => null });
    const onUnlocked = vi.fn();
    render(<UnlockScreen onUnlocked={onUnlocked} />);
    submit("the test password");
    await waitFor(() => expect(onUnlocked).toHaveBeenCalledOnce());
    expect(callsTo("unlock_vault")).toEqual([
      { password: "the test password" },
    ]);
  });

  it("reports a wrong password and clears the field", async () => {
    mockCommands({
      unlock_vault: failWith(
        "wrong_password",
        "the master password is incorrect",
      ),
    });
    const onUnlocked = vi.fn();
    render(<UnlockScreen onUnlocked={onUnlocked} />);
    submit("not it");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Incorrect master password.",
    );
    expect(screen.getByLabelText("Master password")).toHaveValue("");
    expect(onUnlocked).not.toHaveBeenCalled();
  });

  it("reports other errors with their message", async () => {
    mockCommands({
      unlock_vault: failWith("corrupt", "the vault data is damaged"),
    });
    render(<UnlockScreen onUnlocked={vi.fn()} />);
    submit("anything");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The vault data is damaged.",
    );
  });

  it("asks for a password before calling the core", () => {
    mockCommands({});
    render(<UnlockScreen onUnlocked={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Unlock" }));
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Enter your master password.",
    );
    expect(callsTo("unlock_vault")).toHaveLength(0);
  });

  it("shows a notice when given one", () => {
    render(
      <UnlockScreen onUnlocked={vi.fn()} notice="The vault was locked." />,
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "The vault was locked.",
    );
  });

  it("shows no notice by default", () => {
    render(<UnlockScreen onUnlocked={vi.fn()} />);
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("disables the button while unlocking", () => {
    mockCommands({ unlock_vault: () => new Promise(() => {}) });
    render(<UnlockScreen onUnlocked={vi.fn()} />);
    submit("the test password");
    expect(screen.getByRole("button", { name: "Unlocking…" })).toBeDisabled();
  });

  it("never shows the typed password in an error", async () => {
    mockCommands({
      unlock_vault: failWith("wrong_password", "incorrect"),
    });
    render(<UnlockScreen onUnlocked={vi.fn()} />);
    submit("synthetic typed password");
    expect(await screen.findByRole("alert")).not.toHaveTextContent(
      "synthetic typed password",
    );
  });
});
