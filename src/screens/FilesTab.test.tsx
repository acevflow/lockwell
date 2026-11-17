import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ApiError } from "../api/client";
import type { FileSummary } from "../api/types";
import {
  callsTo,
  commandError,
  failWith,
  mockCommands,
} from "../test/tauriMock";
import FilesTab from "./FilesTab";

const REPORT: FileSummary = {
  id: "a".repeat(32),
  name: "report.pdf",
  size: 2_500_000,
  createdAt: 0,
};

describe("FilesTab", () => {
  it("lists files with their size", async () => {
    mockCommands({ list_files: () => [REPORT] });
    render(<FilesTab onLocked={vi.fn()} />);
    expect(await screen.findByText("report.pdf")).toBeVisible();
    expect(screen.getByText("2.5 MB")).toBeVisible();
  });

  it("shows an empty state", async () => {
    mockCommands({ list_files: () => [] });
    render(<FilesTab onLocked={vi.fn()} />);
    expect(await screen.findByText(/No files yet/)).toBeVisible();
  });

  it("adds files and reloads the list", async () => {
    const stored: FileSummary[] = [];
    mockCommands({
      list_files: () => [...stored],
      import_files: () => {
        stored.push(REPORT);
        return [REPORT];
      },
    });
    render(<FilesTab onLocked={vi.fn()} />);
    await screen.findByText(/No files yet/);
    fireEvent.click(screen.getByRole("button", { name: "Add files" }));
    expect(await screen.findByText("Added “report.pdf”.")).toBeVisible();
    expect(screen.getByText("report.pdf")).toBeVisible();
  });

  it("reloads the list when adding fails after some files were added", async () => {
    const stored: FileSummary[] = [];
    mockCommands({
      list_files: () => [...stored],
      // The core adds files one by one and stops at the first that fails.
      import_files: () => {
        stored.push(REPORT);
        throw commandError("invalid_input", "could not add “b.txt”: too long");
      },
    });
    render(<FilesTab onLocked={vi.fn()} />);
    await screen.findByText(/No files yet/);
    fireEvent.click(screen.getByRole("button", { name: "Add files" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("b.txt");
    expect(await screen.findByText("report.pdf")).toBeVisible();
  });

  it("says nothing when adding is cancelled", async () => {
    mockCommands({ list_files: () => [], import_files: () => [] });
    render(<FilesTab onLocked={vi.fn()} />);
    await screen.findByText(/No files yet/);
    fireEvent.click(screen.getByRole("button", { name: "Add files" }));
    await vi.waitFor(() => expect(callsTo("list_files")).toHaveLength(2));
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("saves a copy", async () => {
    mockCommands({ list_files: () => [REPORT], export_file: () => true });
    render(<FilesTab onLocked={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Save a copy of report.pdf" }),
    );
    expect(
      await screen.findByText("Saved a copy of “report.pdf”."),
    ).toBeVisible();
    expect(callsTo("export_file")).toEqual([{ id: REPORT.id }]);
  });

  it("deletes after confirmation", async () => {
    const stored = [REPORT];
    mockCommands({
      list_files: () => [...stored],
      delete_file: () => {
        stored.pop();
        return null;
      },
    });
    render(<FilesTab onLocked={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Delete report.pdf" }),
    );
    const dialog = screen.getByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
    expect(await screen.findByText("Deleted “report.pdf”.")).toBeVisible();
    expect(screen.getByText(/No files yet/)).toBeVisible();
  });

  it("shows errors from the core", async () => {
    mockCommands({
      list_files: () => [REPORT],
      export_file: failWith("corrupt", "the encrypted file failed to decrypt"),
    });
    render(<FilesTab onLocked={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Save a copy of report.pdf" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The encrypted file failed to decrypt.",
    );
  });

  it("calls onLocked when the vault is locked", async () => {
    mockCommands({ list_files: failWith("locked", "the vault is locked") });
    const onLocked = vi.fn();
    render(<FilesTab onLocked={onLocked} />);
    await vi.waitFor(() => expect(onLocked).toHaveBeenCalledOnce());
  });

  it("ignores a list that fails after leaving the tab", async () => {
    let fail!: (reason: unknown) => void;
    mockCommands({
      list_files: () => new Promise((_, reject) => (fail = reject)),
    });
    const onLocked = vi.fn();
    const { unmount } = render(<FilesTab onLocked={onLocked} />);
    unmount();
    fail(new ApiError("locked", "the vault is locked"));
    await act(() => Promise.resolve());
    expect(onLocked).not.toHaveBeenCalled();
  });

  it("shows that the list is loading", () => {
    mockCommands({ list_files: () => new Promise(() => {}) });
    render(<FilesTab onLocked={vi.fn()} />);
    expect(screen.getByText("Loading…")).toBeVisible();
  });

  it("shows load errors", async () => {
    mockCommands({ list_files: failWith("storage", "the database failed") });
    render(<FilesTab onLocked={vi.fn()} />);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The database failed.",
    );
  });

  it("counts several added files", async () => {
    const other = { ...REPORT, id: "b".repeat(32), name: "photo.jpg" };
    mockCommands({ list_files: () => [], import_files: () => [REPORT, other] });
    render(<FilesTab onLocked={vi.fn()} />);
    await screen.findByText(/No files yet/);
    fireEvent.click(screen.getByRole("button", { name: "Add files" }));
    expect(await screen.findByText("Added 2 files.")).toBeVisible();
  });

  it("says nothing when saving a copy is cancelled", async () => {
    mockCommands({ list_files: () => [REPORT], export_file: () => false });
    render(<FilesTab onLocked={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Save a copy of report.pdf" }),
    );
    await vi.waitFor(() => expect(callsTo("export_file")).toHaveLength(1));
    await vi.waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Save a copy of report.pdf" }),
      ).toBeEnabled(),
    );
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("shows what it is doing and disables the actions meanwhile", async () => {
    mockCommands({
      list_files: () => [REPORT],
      export_file: () => new Promise(() => {}),
    });
    render(<FilesTab onLocked={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Save a copy of report.pdf" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Decrypting “report.pdf”…",
    );
    expect(screen.getByRole("button", { name: "Add files" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Delete report.pdf" }),
    ).toBeDisabled();
  });

  it("dismisses an error", async () => {
    mockCommands({
      list_files: () => [REPORT],
      export_file: failWith("io", "the disk is full"),
    });
    render(<FilesTab onLocked={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Save a copy of report.pdf" }),
    );
    await screen.findByRole("alert");
    fireEvent.click(screen.getByRole("button", { name: "Dismiss error" }));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("keeps the file when deletion is cancelled", async () => {
    mockCommands({ list_files: () => [REPORT] });
    render(<FilesTab onLocked={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Delete report.pdf" }),
    );
    const dialog = screen.getByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(screen.getByText("report.pdf")).toBeVisible();
    expect(callsTo("delete_file")).toHaveLength(0);
  });

  it("keeps the file when deletion fails", async () => {
    mockCommands({
      list_files: () => [REPORT],
      delete_file: failWith("io", "access denied"),
    });
    render(<FilesTab onLocked={vi.fn()} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Delete report.pdf" }),
    );
    const dialog = screen.getByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Access denied.",
    );
    expect(screen.getByText("report.pdf")).toBeVisible();
  });
});
