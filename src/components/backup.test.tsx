import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { callsTo, failWith, mockCommands } from "../test/tauriMock";
import CreateBackupPanel from "./CreateBackupPanel";
import RestoreBackupForm from "./RestoreBackupForm";

function submitRestore(password: string) {
  fireEvent.change(screen.getByLabelText("Backup master password"), {
    target: { value: password },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Choose backup and restore" }),
  );
}

describe("CreateBackupPanel", () => {
  it("creates a backup and confirms it", async () => {
    mockCommands({ create_backup: () => true });
    render(<CreateBackupPanel onError={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Create backup" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Backup saved.",
    );
  });

  it("says nothing when cancelled", async () => {
    mockCommands({ create_backup: () => false });
    render(<CreateBackupPanel onError={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Create backup" }));
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Create backup" }),
      ).toBeEnabled(),
    );
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("shows the message chosen by onError", async () => {
    mockCommands({ create_backup: failWith("io", "disk full") });
    const onError = vi.fn(() => "Disk full.");
    render(<CreateBackupPanel onError={onError} />);
    fireEvent.click(screen.getByRole("button", { name: "Create backup" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Disk full.");
    expect(onError).toHaveBeenCalledOnce();
  });
});

describe("RestoreBackupForm", () => {
  it("restores without confirmation when there is no vault", async () => {
    mockCommands({ restore_backup: () => true });
    const onRestored = vi.fn();
    render(<RestoreBackupForm replacesVault={false} onRestored={onRestored} />);
    submitRestore("old test phrase");
    await waitFor(() => expect(onRestored).toHaveBeenCalledOnce());
    expect(callsTo("restore_backup")).toEqual([
      { password: "old test phrase" },
    ]);
  });

  it("asks before replacing an existing vault", async () => {
    mockCommands({ restore_backup: () => true });
    const onRestored = vi.fn();
    render(<RestoreBackupForm replacesVault onRestored={onRestored} />);
    submitRestore("old test phrase");
    expect(callsTo("restore_backup")).toHaveLength(0);
    fireEvent.click(screen.getByRole("button", { name: "Replace vault" }));
    await waitFor(() => expect(onRestored).toHaveBeenCalledOnce());
  });

  it("does nothing when the replacement is cancelled", () => {
    mockCommands({});
    render(<RestoreBackupForm replacesVault onRestored={vi.fn()} />);
    submitRestore("old test phrase");
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(callsTo("restore_backup")).toHaveLength(0);
  });

  it("requires the password", () => {
    mockCommands({});
    render(<RestoreBackupForm replacesVault={false} onRestored={vi.fn()} />);
    submitRestore("");
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Enter the master password of the backup.",
    );
  });

  it("explains a wrong password", async () => {
    mockCommands({ restore_backup: failWith("wrong_password", "incorrect") });
    const onRestored = vi.fn();
    render(<RestoreBackupForm replacesVault={false} onRestored={onRestored} />);
    submitRestore("wrong phrase");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "That password does not open this backup.",
    );
    expect(onRestored).not.toHaveBeenCalled();
  });

  it("shows other errors and stays usable when cancelled", async () => {
    mockCommands({
      restore_backup: failWith(
        "corrupt",
        "invalid backup: the backup is truncated",
      ),
    });
    render(<RestoreBackupForm replacesVault={false} onRestored={vi.fn()} />);
    submitRestore("phrase");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Invalid backup: the backup is truncated.",
    );
    expect(
      screen.getByRole("button", { name: "Choose backup and restore" }),
    ).toBeEnabled();
  });
});
