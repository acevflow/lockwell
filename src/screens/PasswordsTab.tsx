/**
 * Passwords tab: entry list with search on the left, and the selected entry
 * (view, edit or create form) on the right. The full entry, with its password,
 * is only fetched when the user opens it.
 */
import { useCallback, useMemo, useState } from "react";
import {
  createEntry,
  deleteEntry,
  getEntry,
  listEntries,
  updateEntry,
} from "../api/entries";
import { copyEntryField, type CopyableField } from "../api/security";
import type { Entry, EntryInput, EntrySummary } from "../api/types";
import ConfirmDialog from "../components/ConfirmDialog";
import EntryForm from "../components/EntryForm";
import EntryList from "../components/EntryList";
import EntryView from "../components/EntryView";
import ErrorBanner from "../components/ErrorBanner";
import { filterEntries } from "../entryRules";
import { useApiError } from "../hooks/useApiError";
import { useInitialLoad } from "../hooks/useInitialLoad";

interface PasswordsTabProps {
  /** Called when the core reports that the vault is locked. */
  onLocked: () => void;
}

/** What the right-hand panel shows. */
type Mode =
  | { kind: "empty" }
  | { kind: "view"; entry: Entry }
  | { kind: "create" }
  | { kind: "edit"; entry: Entry };

/** Lists, searches, shows, creates, edits and deletes password entries. */
function PasswordsTab({ onLocked }: PasswordsTabProps) {
  const [entries, setEntries] = useState<EntrySummary[] | null>(null);
  const [query, setQuery] = useState("");
  const [mode, setMode] = useState<Mode>({ kind: "empty" });
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const { error, report, clear } = useApiError(onLocked);

  const reload = useCallback(async () => {
    try {
      setEntries(await listEntries());
    } catch (loadError) {
      report(loadError);
    }
  }, [report]);

  useInitialLoad(listEntries, setEntries, report);

  const visible = useMemo(
    () => filterEntries(entries ?? [], query),
    [entries, query],
  );

  async function select(id: string) {
    clear();
    try {
      setMode({ kind: "view", entry: await getEntry(id) });
    } catch (selectError) {
      report(selectError);
    }
  }

  async function create(input: EntryInput) {
    const id = await createEntry(input);
    await reload();
    setMode({ kind: "view", entry: await getEntry(id) });
  }

  async function update(id: string, input: EntryInput) {
    await updateEntry(id, input);
    await reload();
    setMode({ kind: "view", entry: await getEntry(id) });
  }

  async function copy(id: string, field: CopyableField) {
    clear();
    try {
      await copyEntryField(id, field);
    } catch (copyError) {
      report(copyError);
      throw copyError;
    }
  }

  async function remove(id: string) {
    setDeleting(true);
    try {
      await deleteEntry(id);
      setMode({ kind: "empty" });
      await reload();
    } catch (deleteError) {
      report(deleteError);
    } finally {
      setDeleting(false);
      setConfirmDelete(false);
    }
  }

  const selectedId =
    mode.kind === "view" || mode.kind === "edit" ? mode.entry.id : null;

  return (
    <div className="split">
      <aside className="sidebar">
        <div className="toolbar">
          <input
            type="search"
            placeholder="Search"
            aria-label="Search entries"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
          <button
            type="button"
            onClick={() => {
              clear();
              setMode({ kind: "create" });
            }}
          >
            New entry
          </button>
        </div>
        {entries === null ? (
          <p className="muted">Loading…</p>
        ) : (
          <EntryList
            entries={visible}
            selectedId={selectedId}
            onSelect={(id) => void select(id)}
            emptyMessage={
              entries.length === 0
                ? "No entries yet. Add your first one."
                : "No entries match your search."
            }
          />
        )}
      </aside>

      <section className="content">
        <ErrorBanner message={error} onDismiss={clear} />
        {mode.kind === "empty" && (
          <p className="muted">Select an entry or add a new one.</p>
        )}
        {mode.kind === "create" && (
          <EntryForm
            heading="New entry"
            submitLabel="Add entry"
            onSubmit={create}
            onCancel={() => setMode({ kind: "empty" })}
          />
        )}
        {mode.kind === "edit" && (
          <EntryForm
            key={mode.entry.id}
            heading="Edit entry"
            initial={mode.entry}
            submitLabel="Save changes"
            onSubmit={(input) => update(mode.entry.id, input)}
            onCancel={() => setMode({ kind: "view", entry: mode.entry })}
          />
        )}
        {mode.kind === "view" && (
          <EntryView
            key={mode.entry.id}
            entry={mode.entry}
            onEdit={() => setMode({ kind: "edit", entry: mode.entry })}
            onDelete={() => setConfirmDelete(true)}
            onCopy={(field) => copy(mode.entry.id, field)}
          />
        )}
      </section>

      {confirmDelete && mode.kind === "view" && (
        <ConfirmDialog
          title="Delete entry?"
          message={`“${mode.entry.title}” will be deleted permanently.`}
          confirmLabel="Delete"
          busy={deleting}
          onCancel={() => setConfirmDelete(false)}
          onConfirm={() => void remove(mode.entry.id)}
        />
      )}
    </div>
  );
}

export default PasswordsTab;
