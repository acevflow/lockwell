/** Tests that each command wrapper calls the right command with the right arguments, and the lock-event listener. */
import { listen } from "@tauri-apps/api/event";
import { describe, expect, it, vi } from "vitest";
import { callsTo, emitEvent, failWith, mockCommands } from "../test/tauriMock";
import { createBackup, restoreBackup } from "./backup";
import { ApiError } from "./client";
import {
  createEntry,
  deleteEntry,
  getEntry,
  listEntries,
  updateEntry,
} from "./entries";
import { deleteFile, exportFile, importFiles, listFiles } from "./files";
import {
  VAULT_LOCKED_EVENT,
  copyEntryField,
  onVaultLocked,
  touchActivity,
} from "./security";
import type { EntryInput } from "./types";
import {
  changeMasterPassword,
  createVault,
  getVaultStatus,
  lockVault,
  unlockVault,
} from "./vault";

const ID = "0123456789abcdef0123456789abcdef";

const INPUT: EntryInput = {
  title: "Example",
  username: "user@example.test",
  password: "synthetic-password",
  url: "https://example.test",
  notes: "",
};

/** Each wrapper, the command it must call and the arguments it must send. */
const wrappers: [string, () => Promise<unknown>, string, object][] = [
  ["getVaultStatus", () => getVaultStatus(), "vault_status", {}],
  [
    "createVault",
    () => createVault("synthetic master"),
    "create_vault",
    { password: "synthetic master" },
  ],
  [
    "unlockVault",
    () => unlockVault("synthetic master"),
    "unlock_vault",
    { password: "synthetic master" },
  ],
  ["lockVault", () => lockVault(), "lock_vault", {}],
  [
    "changeMasterPassword",
    () => changeMasterPassword("synthetic old", "synthetic new"),
    "change_master_password",
    { current: "synthetic old", new: "synthetic new" },
  ],
  ["listEntries", () => listEntries(), "list_entries", {}],
  ["getEntry", () => getEntry(ID), "get_entry", { id: ID }],
  ["createEntry", () => createEntry(INPUT), "create_entry", { entry: INPUT }],
  [
    "updateEntry",
    () => updateEntry(ID, INPUT),
    "update_entry",
    { id: ID, entry: INPUT },
  ],
  ["deleteEntry", () => deleteEntry(ID), "delete_entry", { id: ID }],
  ["listFiles", () => listFiles(), "list_files", {}],
  ["importFiles", () => importFiles(), "import_files", {}],
  ["exportFile", () => exportFile(ID), "export_file", { id: ID }],
  ["deleteFile", () => deleteFile(ID), "delete_file", { id: ID }],
  ["createBackup", () => createBackup(), "create_backup", {}],
  [
    "restoreBackup",
    () => restoreBackup("synthetic master"),
    "restore_backup",
    { password: "synthetic master" },
  ],
  ["touchActivity", () => touchActivity(), "touch_activity", {}],
  [
    "copyEntryField",
    () => copyEntryField(ID, "password"),
    "copy_entry_field",
    { id: ID, field: "password" },
  ],
];

describe("command wrappers", () => {
  it.each(wrappers)(
    "%s calls %s with the expected arguments",
    async (_name, run, command, args) => {
      mockCommands({ [command]: () => "result" });

      await expect(run()).resolves.toBe("result");

      expect(callsTo(command)).toEqual([args]);
    },
  );

  it.each(wrappers)(
    "%s turns a core error into an ApiError",
    async (_name, run, command) => {
      mockCommands({ [command]: failWith("locked", "the vault is locked") });

      const failure = run();

      await expect(failure).rejects.toBeInstanceOf(ApiError);
      await expect(failure).rejects.toMatchObject({
        code: "locked",
        message: "the vault is locked",
      });
    },
  );
});

describe("onVaultLocked", () => {
  it("listens for the lock event and passes on the reason", async () => {
    const handler = vi.fn();

    await onVaultLocked(handler);
    emitEvent(VAULT_LOCKED_EVENT, { reason: "systemLock" });

    expect(vi.mocked(listen)).toHaveBeenCalledWith(
      "vault-locked",
      expect.any(Function),
    );
    expect(handler).toHaveBeenCalledExactlyOnceWith("systemLock");
  });

  it("ignores other events", async () => {
    const handler = vi.fn();

    await onVaultLocked(handler);
    emitEvent("something-else", { reason: "idle" });

    expect(handler).not.toHaveBeenCalled();
  });

  it("resolves to the function that stops listening", async () => {
    const stop = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(stop);

    const unlisten = await onVaultLocked(vi.fn());
    unlisten();

    expect(stop).toHaveBeenCalledOnce();
  });
});
