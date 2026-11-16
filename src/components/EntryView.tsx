import { useState } from "react";
import type { Entry } from "../api/types";

interface EntryViewProps {
  entry: Entry;
  onEdit: () => void;
  onDelete: () => void;
}

/** Read-only view of an entry. The password stays masked until shown. */
function EntryView({ entry, onEdit, onDelete }: EntryViewProps) {
  const [showPassword, setShowPassword] = useState(false);

  return (
    <section className="panel" aria-labelledby="entry-view-title">
      <h2 id="entry-view-title">{entry.title}</h2>
      <dl className="details">
        <dt>Username</dt>
        <dd>{entry.username || <span className="muted">None</span>}</dd>

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
