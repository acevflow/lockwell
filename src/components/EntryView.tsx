/** Read-only view of a password entry, with copy buttons. */
import { useState } from "react";
import { CLIPBOARD_CLEAR_SECONDS, type CopyableField } from "../api/security";
import type { Entry } from "../api/types";

interface EntryViewProps {
  /** The complete entry, including its password. */
  entry: Entry;
  /** Called by the Edit button. */
  onEdit: () => void;
  /** Called by the Delete button; the caller asks for confirmation. */
  onDelete: () => void;
  /** Copies a field to the clipboard; a rejected promise is reported by the caller. */
  onCopy: (field: CopyableField) => Promise<void>;
}

/** Names of the copyable fields, for the confirmation message. */
const FIELD_NAMES: Record<CopyableField, string> = {
  username: "Username",
  password: "Password",
};

/**
 * Read-only view of an entry. The password stays masked until shown, and the
 * copy buttons ask the core to put the value on the clipboard, so copying does
 * not need to reveal it. The caller gives each entry its own instance (a
 * `key`), so the "shown" state never carries over to another entry.
 */
function EntryView({ entry, onEdit, onDelete, onCopy }: EntryViewProps) {
  const [showPassword, setShowPassword] = useState(false);
  const [copied, setCopied] = useState<CopyableField | null>(null);

  async function copy(field: CopyableField) {
    setCopied(null);
    try {
      await onCopy(field);
      setCopied(field);
    } catch {
      // The caller has already shown the error.
    }
  }

  return (
    <section className="panel" aria-labelledby="entry-view-title">
      <h2 id="entry-view-title">{entry.title}</h2>
      <dl className="details">
        <dt>Username</dt>
        <dd>
          {entry.username ? (
            <span className="secret-row">
              <span>{entry.username}</span>
              <button
                type="button"
                className="secondary small"
                onClick={() => void copy("username")}
                aria-label="Copy username"
              >
                Copy
              </button>
            </span>
          ) : (
            <span className="muted">None</span>
          )}
        </dd>

        <dt>Password</dt>
        <dd>
          {entry.password ? (
            <span className="secret-row">
              <code data-testid="entry-password">
                {showPassword ? entry.password : "••••••••••••"}
              </code>
              <button
                type="button"
                className="secondary small"
                onClick={() => setShowPassword((shown) => !shown)}
                aria-pressed={showPassword}
              >
                {showPassword ? "Hide" : "Show"}
              </button>
              <button
                type="button"
                className="secondary small"
                onClick={() => void copy("password")}
                aria-label="Copy password"
              >
                Copy
              </button>
            </span>
          ) : (
            <span className="muted">None</span>
          )}
        </dd>

        <dt>Website</dt>
        <dd>{entry.url || <span className="muted">None</span>}</dd>

        <dt>Notes</dt>
        <dd className="notes">
          {entry.notes || <span className="muted">None</span>}
        </dd>

        <dt>Last changed</dt>
        <dd>{new Date(entry.updatedAt).toLocaleString()}</dd>
      </dl>
      {copied && (
        <p className="success" role="status">
          {FIELD_NAMES[copied]} copied. It will be cleared from the clipboard in{" "}
          {CLIPBOARD_CLEAR_SECONDS} seconds.
        </p>
      )}
      <div className="actions">
        <button type="button" className="danger" onClick={onDelete}>
          Delete
        </button>
        <button type="button" onClick={onEdit}>
          Edit
        </button>
      </div>
    </section>
  );
}

export default EntryView;
