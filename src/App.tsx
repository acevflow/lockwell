import { useCallback, useEffect, useState } from "react";
import { errorMessage } from "./api/client";
import type { LockReason } from "./api/security";
import { getVaultStatus, lockVault } from "./api/vault";
import SetupScreen from "./screens/SetupScreen";
import UnlockScreen from "./screens/UnlockScreen";
import VaultScreen from "./screens/VaultScreen";

type Phase =
  | { kind: "loading" }
  | { kind: "failed"; message: string }
  | { kind: "setup" }
  | { kind: "locked"; reason?: LockReason }
  | { kind: "unlocked" };

const LOCK_NOTICES: Record<LockReason, string> = {
  idle: "The vault was locked after 5 minutes of inactivity.",
  systemLock: "The vault was locked because Windows was locked.",
};

/**
 * Root component of the interface. It asks the core for the vault status
 * and shows the setup, unlock or vault screen.
 */
function App() {
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });

  useEffect(() => {
    getVaultStatus()
      .then((status) =>
        setPhase(
          !status.exists
            ? { kind: "setup" }
            : status.unlocked
              ? { kind: "unlocked" }
              : { kind: "locked" },
        ),
      )
      .catch((error: unknown) =>
        setPhase({ kind: "failed", message: errorMessage(error) }),
      );
  }, []);

  const showLocked = useCallback(
    (reason?: LockReason) => setPhase({ kind: "locked", reason }),
    [],
  );

  const lock = useCallback(() => {
    // Leave the vault screen even if the call fails: the interface must
    // never keep showing decrypted data after the user asked to lock.
    lockVault()
      .catch(() => undefined)
      .finally(() => showLocked());
  }, [showLocked]);

  switch (phase.kind) {
    case "loading":
      return (
        <main className="centered">
          <p className="muted">Loading…</p>
        </main>
      );
    case "failed":
      return (
        <main className="centered">
          <div className="card">
            <h1>lockwell</h1>
            <p role="alert">The vault could not be opened: {phase.message}</p>
          </div>
        </main>
      );
    case "setup":
      return <SetupScreen onCreated={() => setPhase({ kind: "unlocked" })} />;
    case "locked":
      return (
        <UnlockScreen
          notice={phase.reason ? LOCK_NOTICES[phase.reason] : undefined}
          onUnlocked={() => setPhase({ kind: "unlocked" })}
        />
      );
    case "unlocked":
      return <VaultScreen onLock={lock} onLocked={showLocked} />;
  }
}

export default App;
