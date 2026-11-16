import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { FileSummary } from "../api/types";
import { callsTo, failWith, mockCommands } from "../test/tauriMock";
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
});
