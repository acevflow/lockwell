/** Selectable list of password entries. */
import type { EntrySummary } from "../api/types";

interface EntryListProps {
  /** Entries to show, already filtered and sorted. */
  entries: EntrySummary[];
  /** Id of the entry that is open, if any. */
  selectedId: string | null;
  /** Called with the id of the entry the user clicks. */
  onSelect: (id: string) => void;
  /** Shown when `entries` is empty. */
  emptyMessage: string;
}

/** The list of entries: title and username, never the password. */
function EntryList({
  entries,
  selectedId,
  onSelect,
  emptyMessage,
}: EntryListProps) {
  if (entries.length === 0) {
    return <p className="muted">{emptyMessage}</p>;
  }
  return (
    <ul className="item-list" aria-label="Entries">
      {entries.map((entry) => (
        <li key={entry.id}>
          <button
            type="button"
            className={entry.id === selectedId ? "item selected" : "item"}
            aria-current={entry.id === selectedId ? "true" : undefined}
            onClick={() => onSelect(entry.id)}
          >
            <span className="item-title">{entry.title}</span>
            {entry.username && (
              <span className="item-subtitle">{entry.username}</span>
            )}
          </button>
        </li>
      ))}
    </ul>
  );
}

export default EntryList;
