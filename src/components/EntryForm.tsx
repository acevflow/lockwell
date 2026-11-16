import { useState, type FormEvent } from "react";
import { errorMessage } from "../api/client";
import type { EntryInput } from "../api/types";
import { ENTRY_LIMITS } from "../entryRules";
import ErrorBanner from "./ErrorBanner";
import PasswordInput from "./PasswordInput";

const EMPTY_ENTRY: EntryInput = {
  title: "",
  username: "",
  password: "",
  url: "",
  notes: "",
};

interface EntryFormProps {
  heading: string;
  initial?: EntryInput;
  submitLabel: string;
  /** Saves the entry; a rejected promise shows its error in the form. */
  onSubmit: (entry: EntryInput) => Promise<void>;
  onCancel: () => void;
}

/** Form to create or edit an entry. */
function EntryForm({
  heading,
  initial = EMPTY_ENTRY,
  submitLabel,
  onSubmit,
  onCancel,
}: EntryFormProps) {
  // Keep only the editable fields: `initial` may be a full `Entry`, and the
  // core rejects unknown fields such as `id` or `createdAt`.
  const [entry, setEntry] = useState<EntryInput>(() => ({
    title: initial.title,
    username: initial.username,
    password: initial.password,
    url: initial.url,
    notes: initial.notes,
  }));
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  function update(field: keyof EntryInput, value: string) {
    setEntry((current) => ({ ...current, [field]: value }));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (entry.title.trim() === "") {
      setError("The title is required.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await onSubmit(entry);
    } catch (submitError) {
      setError(errorMessage(submitError));
      setBusy(false);
    }
  }

  return (
    <form className="panel" onSubmit={submit} noValidate>
      <h2>{heading}</h2>
      <ErrorBanner message={error} />
      <div className="field">
        <label htmlFor="entry-title">Title</label>
        <input
          id="entry-title"
          value={entry.title}
          onChange={(event) => update("title", event.target.value)}
          maxLength={ENTRY_LIMITS.title}
          required
          autoFocus
          autoComplete="off"
        />
      </div>
      <div className="field">
        <label htmlFor="entry-username">Username</label>
        <input
          id="entry-username"
          value={entry.username}
          onChange={(event) => update("username", event.target.value)}
          maxLength={ENTRY_LIMITS.username}
          autoComplete="off"
          spellCheck={false}
        />
      </div>
      <PasswordInput
        id="entry-password"
        label="Password"
        value={entry.password}
        onChange={(value) => update("password", value)}
        autoComplete="off"
      />
      <div className="field">
        <label htmlFor="entry-url">Website</label>
        <input
          id="entry-url"
          value={entry.url}
          onChange={(event) => update("url", event.target.value)}
          maxLength={ENTRY_LIMITS.url}
          autoComplete="off"
          spellCheck={false}
          placeholder="https://"
        />
      </div>
      <div className="field">
        <label htmlFor="entry-notes">Notes</label>
        <textarea
          id="entry-notes"
          value={entry.notes}
          onChange={(event) => update("notes", event.target.value)}
          maxLength={ENTRY_LIMITS.notes}
          rows={4}
        />
      </div>
      <div className="actions">
        <button
          type="button"
          className="secondary"
          onClick={onCancel}
          disabled={busy}
        >
          Cancel
        </button>
        <button type="submit" disabled={busy}>
          {busy ? "Saving…" : submitLabel}
        </button>
      </div>
    </form>
  );
}

export default EntryForm;
