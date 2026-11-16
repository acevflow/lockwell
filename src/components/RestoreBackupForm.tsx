import { useState, type FormEvent } from "react";
import { restoreBackup } from "../api/backup";
import { ApiError, errorMessage } from "../api/client";
import ConfirmDialog from "./ConfirmDialog";
import ErrorBanner from "./ErrorBanner";
import PasswordInput from "./PasswordInput";

interface RestoreBackupFormProps {
  /** Whether a vault exists and will be replaced (asks for confirmation). */
  replacesVault: boolean;
  /** Called after a successful restore; the vault is then locked. */
  onRestored: () => void;
}

/**
 * Restores an encrypted backup: asks for the master password the vault had
 * when the backup was made, then the core asks for the backup file.
 */
function RestoreBackupForm({
  replacesVault,
  onRestored,
}: RestoreBackupFormProps) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirming, setConfirming] = useState(false);

  function submit(event: FormEvent) {
    event.preventDefault();
    if (password === "") {
      setError("Enter the master password of the backup.");
      return;
    }
    setError(null);
    if (replacesVault) {
      setConfirming(true);
    } else {
      void restore();
    }
  }

  async function restore() {
    setConfirming(false);
    setBusy(true);
    try {
      const restored = await restoreBackup(password);
      if (restored) {
        setPassword("");
        onRestored();
        return;
      }
    } catch (restoreError) {
      setError(
        restoreError instanceof ApiError &&
          restoreError.code === "wrong_password"
          ? "That password does not open this backup."
          : errorMessage(restoreError),
      );
    }
    setBusy(false);
  }

  return (
    <form className="panel" onSubmit={submit} noValidate>
      <h2>Restore from a backup</h2>
      <p className="hint">
        Use the master password the vault had when the backup was made. You will
        then choose the backup file.
      </p>
      {replacesVault && (
        <p className="warning">
          Restoring replaces everything in this vault with the content of the
          backup.
        </p>
      )}
      <ErrorBanner message={error} />
      <PasswordInput
        id="backup-password"
        label="Backup master password"
        value={password}
        onChange={setPassword}
        autoComplete="off"
        required
      />
      <div className="actions">
        <button type="submit" disabled={busy}>
          {busy ? "Restoring…" : "Choose backup and restore"}
        </button>
      </div>
      {confirming && (
        <ConfirmDialog
          title="Replace this vault?"
          message="All current passwords and files will be replaced by the backup. This cannot be undone."
          confirmLabel="Replace vault"
          onCancel={() => setConfirming(false)}
          onConfirm={() => void restore()}
        />
      )}
    </form>
  );
}

export default RestoreBackupForm;
