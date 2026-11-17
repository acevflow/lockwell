/** Screen shown while the vault is locked: asks for the master password. */
import { useState, type FormEvent } from "react";
import { errorMessage, isWrongPasswordError } from "../api/client";
import { unlockVault } from "../api/vault";
import ErrorBanner from "../components/ErrorBanner";
import PasswordInput from "../components/PasswordInput";

interface UnlockScreenProps {
  /** Called once the vault is unlocked. */
  onUnlocked: () => void;
  /** Why the vault was locked automatically, if it was. */
  notice?: string;
}

/**
 * Asks for the master password to unlock the vault. The typed password is
 * cleared from the field after every attempt, successful or not.
 */
function UnlockScreen({ onUnlocked, notice }: UnlockScreenProps) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (password === "") {
      setError("Enter your master password.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await unlockVault(password);
      setPassword("");
      onUnlocked();
    } catch (unlockError) {
      setError(
        isWrongPasswordError(unlockError)
          ? "Incorrect master password."
          : errorMessage(unlockError),
      );
      setPassword("");
      setBusy(false);
    }
  }

  return (
    <main className="centered">
      <form className="card" onSubmit={submit} noValidate>
        <h1>lockwell</h1>
        <p>Your vault is locked.</p>
        {notice && (
          <p className="notice" role="status">
            {notice}
          </p>
        )}
        <ErrorBanner message={error} />
        <PasswordInput
          id="master-password"
          label="Master password"
          value={password}
          onChange={setPassword}
          autoComplete="current-password"
          autoFocus
          required
        />
        <button type="submit" disabled={busy}>
          {busy ? "Unlocking…" : "Unlock"}
        </button>
      </form>
    </main>
  );
}

export default UnlockScreen;
