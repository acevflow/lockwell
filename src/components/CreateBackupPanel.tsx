/** Settings panel that creates an encrypted backup of the vault. */
import { useState } from "react";
import { createBackup } from "../api/backup";
import ErrorBanner from "./ErrorBanner";

interface CreateBackupPanelProps {
  /** Reports an error; returns the message to show, or null if handled. */
  onError: (error: unknown) => string | null;
}

/**
 * Creates an encrypted backup of the whole vault. The core asks where to save
 * the file; cancelling the dialog is not an error and shows nothing.
 */
function CreateBackupPanel({ onError }: CreateBackupPanelProps) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  async function create() {
    setBusy(true);
    setError(null);
    setSuccess(null);
    try {
      if (await createBackup()) {
        setSuccess("Backup saved.");
      }
    } catch (backupError) {
      setError(onError(backupError));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="panel" aria-labelledby="create-backup-title">
      <h2 id="create-backup-title">Encrypted backup</h2>
      <p className="hint">
        Saves all your passwords and files in one encrypted file. It opens with
        your current master password, so keep it in mind when you restore.
      </p>
      <ErrorBanner message={error} />
      {success && (
        <p className="success" role="status">
          {success}
        </p>
      )}
      <div className="actions">
        <button type="button" onClick={() => void create()} disabled={busy}>
          {busy ? "Saving backup…" : "Create backup"}
        </button>
      </div>
    </section>
  );
}

export default CreateBackupPanel;
