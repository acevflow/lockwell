import { useState } from "react";
import type { LockReason } from "../api/security";
import { useActivityReporter } from "../hooks/useActivityReporter";
import { useVaultLockedEvent } from "../hooks/useVaultLockedEvent";
import FilesTab from "./FilesTab";
import PasswordsTab from "./PasswordsTab";
import SettingsTab from "./SettingsTab";

interface VaultScreenProps {
  /** Locks the vault (the Lock button). */
  onLock: () => void;
  /**
   * Called when the core locked the vault on its own (with the reason) or a
   * command reports that it is locked (without one).
   */
  onLocked: (reason?: LockReason) => void;
}

const TABS = [
  { id: "passwords", label: "Passwords" },
  { id: "files", label: "Files" },
  { id: "settings", label: "Settings" },
] as const;

type TabId = (typeof TABS)[number]["id"];

/**
 * The unlocked vault. Everything decrypted lives in the state of this
 * component tree, so it is discarded when the vault locks and this screen
 * unmounts.
 */
function VaultScreen({ onLock, onLocked }: VaultScreenProps) {
  const [tab, setTab] = useState<TabId>("passwords");
  useActivityReporter();
  useVaultLockedEvent(onLocked);

  return (
    <div className="vault">
      <header className="topbar">
        <span className="brand">lockwell</span>
        <nav role="tablist" aria-label="Vault sections">
          {TABS.map(({ id, label }) => (
            <button
              key={id}
              type="button"
              role="tab"
              id={`tab-${id}`}
              aria-selected={tab === id}
              aria-controls={`panel-${id}`}
              className={tab === id ? "tab active" : "tab"}
              onClick={() => setTab(id)}
            >
              {label}
            </button>
          ))}
        </nav>
        <button type="button" className="secondary" onClick={onLock}>
          Lock
        </button>
      </header>
      <main
        role="tabpanel"
        id={`panel-${tab}`}
        aria-labelledby={`tab-${tab}`}
        className="tab-panel"
      >
        {tab === "passwords" && <PasswordsTab onLocked={onLocked} />}
        {tab === "files" && <FilesTab onLocked={onLocked} />}
        {tab === "settings" && <SettingsTab onLocked={onLocked} />}
      </main>
    </div>
  );
}

export default VaultScreen;
