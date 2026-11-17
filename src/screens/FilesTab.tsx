/** Files tab: list, add, save a copy of and delete encrypted files. */
import { useCallback, useState } from "react";
import { deleteFile, exportFile, importFiles, listFiles } from "../api/files";
import type { FileSummary } from "../api/types";
import ConfirmDialog from "../components/ConfirmDialog";
import ErrorBanner from "../components/ErrorBanner";
import FileTable from "../components/FileTable";
import { useApiError } from "../hooks/useApiError";
import { useInitialLoad } from "../hooks/useInitialLoad";

interface FilesTabProps {
  /** Called when the core reports that the vault is locked. */
  onLocked: () => void;
}

/** Lists, adds, saves copies of and deletes encrypted files. */
function FilesTab({ onLocked }: FilesTabProps) {
  const [files, setFiles] = useState<FileSummary[] | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [toDelete, setToDelete] = useState<FileSummary | null>(null);
  const { error, report, clear } = useApiError(onLocked);

  const reload = useCallback(async () => {
    setFiles(await listFiles());
  }, []);

  useInitialLoad(listFiles, setFiles, report);

  /**
   * Runs one file operation: shows `label` while it runs, then the message it
   * returns (or nothing for `null`). Only one operation runs at a time; the
   * buttons are disabled while `busy`.
   */
  async function run(label: string, action: () => Promise<string | null>) {
    clear();
    setStatus(null);
    setBusy(label);
    try {
      setStatus(await action());
    } catch (actionError) {
      report(actionError);
    } finally {
      setBusy(null);
    }
  }

  function add() {
    void run("Encrypting files…", async () => {
      let added: FileSummary[];
      try {
        added = await importFiles();
      } catch (importError) {
        // The core adds files one by one, so some may be stored even though
        // a later one failed: show what is really in the vault.
        await reload().catch(() => undefined);
        throw importError;
      }
      await reload();
      if (added.length === 0) return null;
      return added.length === 1
        ? `Added “${added[0].name}”.`
        : `Added ${added.length} files.`;
    });
  }

  function saveCopy(file: FileSummary) {
    void run(`Decrypting “${file.name}”…`, async () =>
      (await exportFile(file.id)) ? `Saved a copy of “${file.name}”.` : null,
    );
  }

  function remove(file: FileSummary) {
    setToDelete(null);
    void run(`Deleting “${file.name}”…`, async () => {
      await deleteFile(file.id);
      await reload();
      return `Deleted “${file.name}”.`;
    });
  }

  return (
    <div className="files">
      <div className="files-header">
        <h2>Files</h2>
        <button type="button" onClick={add} disabled={busy !== null}>
          Add files
        </button>
      </div>
      <p className="hint">
        Files are encrypted inside the vault, names included. Saving a copy
        writes a decrypted file wherever you choose.
      </p>
      <ErrorBanner message={error} onDismiss={clear} />
      {busy && (
        <p className="muted" role="status">
          {busy}
        </p>
      )}
      {!busy && status && (
        <p className="success" role="status">
          {status}
        </p>
      )}
      {files === null ? (
        <p className="muted">Loading…</p>
      ) : files.length === 0 ? (
        <p className="muted">No files yet. Add your first one.</p>
      ) : (
        <FileTable
          files={files}
          disabled={busy !== null}
          onSaveCopy={saveCopy}
          onDelete={setToDelete}
        />
      )}
      {toDelete && (
        <ConfirmDialog
          title="Delete file?"
          message={`“${toDelete.name}” will be deleted from the vault permanently.`}
          confirmLabel="Delete"
          onCancel={() => setToDelete(null)}
          onConfirm={() => remove(toDelete)}
        />
      )}
    </div>
  );
}

export default FilesTab;
