import { useState, type FormEvent } from "react";
import { ApiError, errorMessage, isLockedError } from "../api/client";
import { changeMasterPassword } from "../api/vault";
import ErrorBanner from "../components/ErrorBanner";
import PasswordInput from "../components/PasswordInput";
import {
  MIN_MASTER_PASSWORD_CHARS,
  masterPasswordProblem,
} from "../passwordRules";

interface SettingsTabProps {
  /** Called when the core reports that the vault is locked. */
  onLocked: () => void;
}

/** Vault settings: change the master password. */
function SettingsTab({ onLocked }: SettingsTabProps) {
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSuccess(null);
    if (current === "") {
      setError("Enter your current master password.");
      return;
    }
    const problem = masterPasswordProblem(next);
    if (problem) {
      setError(`New password: ${problem}`);
      return;
    }
    if (next !== confirmation) {
      setError("The new passwords do not match.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await changeMasterPassword(current, next);
      setCurrent("");
      setNext("");
      setConfirmation("");
      setSuccess("Master password changed.");
    } catch (changeError) {
      if (isLockedError(changeError)) {
        onLocked();
        return;
      }
      setError(
        changeError instanceof ApiError && changeError.code === "wrong_password"
          ? "The current master password is incorrect."
          : errorMessage(changeError),
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="settings">
      <form className="panel" onSubmit={submit} noValidate>
        <h2>Change master password</h2>
        <p className="hint">
          Your data stays the same; only the password that unlocks it changes.
          There is no recovery if you forget the new one.
        </p>
        <ErrorBanner message={error} />
        {success && (
          <p className="success" role="status">
            {success}
          </p>
        )}
        <PasswordInput
          id="current-master-password"
          label="Current master password"
          value={current}
          onChange={setCurrent}
          autoComplete="current-password"
          required
        />
        <PasswordInput
          id="next-master-password"
          label="New master password"
          value={next}
          onChange={setNext}
          autoComplete="new-password"
          required
          describedBy="next-master-password-hint"
        />
        <p id="next-master-password-hint" className="hint">
          At least {MIN_MASTER_PASSWORD_CHARS} characters.
        </p>
        <PasswordInput
          id="confirm-next-master-password"
          label="Confirm new master password"
          value={confirmation}
          onChange={setConfirmation}
          autoComplete="new-password"
          required
        />
        <div className="actions">
          <button type="submit" disabled={busy}>
            {busy ? "Changing…" : "Change password"}
          </button>
        </div>
      </form>
    </div>
  );
}

export default SettingsTab;
