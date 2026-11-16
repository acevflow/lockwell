import { useState, type FormEvent } from "react";
import { errorMessage } from "../api/client";
import { createVault } from "../api/vault";
import ErrorBanner from "../components/ErrorBanner";
import PasswordInput from "../components/PasswordInput";
import RestoreBackupForm from "../components/RestoreBackupForm";
import {
  MIN_MASTER_PASSWORD_CHARS,
  masterPasswordProblem,
} from "../passwordRules";

interface SetupScreenProps {
  /** Called once the vault exists and is unlocked. */
  onCreated: () => void;
  /** Called once a backup was restored; the vault is then locked. */
  onRestored: () => void;
}

/**
 * First run: choose the master password and create the vault, or restore
 * an existing backup instead.
 */
function SetupScreen({ onCreated, onRestored }: SetupScreenProps) {
  const [restoring, setRestoring] = useState(false);
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [acknowledged, setAcknowledged] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    const problem = masterPasswordProblem(password);
    if (problem) {
      setError(problem);
      return;
    }
    if (password !== confirmation) {
      setError("The passwords do not match.");
      return;
    }
    if (!acknowledged) {
      setError("Confirm that you understand the password cannot be recovered.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await createVault(password);
      setPassword("");
      setConfirmation("");
      onCreated();
    } catch (createError) {
      setError(errorMessage(createError));
      setBusy(false);
    }
  }

  if (restoring) {
    return (
      <main className="centered">
        <div className="card">
          <RestoreBackupForm replacesVault={false} onRestored={onRestored} />
          <button
            type="button"
            className="link switch-mode"
            onClick={() => setRestoring(false)}
          >
            Create a new vault instead
          </button>
        </div>
      </main>
    );
  }

  return (
    <main className="centered">
      <form className="card" onSubmit={submit} noValidate>
        <h1>Create your vault</h1>
        <p>
          Your passwords and files are encrypted on this computer with a master
          password. Nothing leaves your device.
        </p>
        <ErrorBanner message={error} />
        <PasswordInput
          id="new-master-password"
          label="Master password"
          value={password}
          onChange={setPassword}
          autoComplete="new-password"
          autoFocus
          required
          describedBy="master-password-hint"
        />
        <p id="master-password-hint" className="hint">
          At least {MIN_MASTER_PASSWORD_CHARS} characters. A long phrase is
          easier to remember and harder to guess.
        </p>
        <PasswordInput
          id="confirm-master-password"
          label="Confirm master password"
          value={confirmation}
          onChange={setConfirmation}
          autoComplete="new-password"
          required
        />
        <p className="warning">
          There is no way to recover a forgotten master password. If you lose
          it, your data cannot be decrypted.
        </p>
        <label className="checkbox">
          <input
            type="checkbox"
            checked={acknowledged}
            onChange={(event) => setAcknowledged(event.target.checked)}
          />
          I understand that a forgotten master password cannot be recovered.
        </label>
        <button type="submit" disabled={busy}>
          {busy ? "Creating vault…" : "Create vault"}
        </button>
        <button
          type="button"
          className="link switch-mode"
          onClick={() => setRestoring(true)}
        >
          Restore from a backup instead
        </button>
      </form>
    </main>
  );
}

export default SetupScreen;
