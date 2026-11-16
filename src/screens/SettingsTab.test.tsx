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
    render(<SettingsTab onLocked={vi.fn()} />);
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
    render(<SettingsTab onLocked={vi.fn()} />);
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
    render(<SettingsTab onLocked={vi.fn()} />);
    submit("wrong test phrase", "new test phrase!");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The current master password is incorrect.",
    );
  });

  it("calls onLocked when the vault is locked", async () => {
    mockCommands({ change_master_password: failWith("locked", "locked") });
    const onLocked = vi.fn();
    render(<SettingsTab onLocked={onLocked} />);
    submit("old test phrase", "new test phrase!");
    await vi.waitFor(() => expect(onLocked).toHaveBeenCalledOnce());
  });
});
