import type { EntrySummary } from "../api/types";

interface EntryListProps {
  entries: EntrySummary[];
  selectedId: string | null;
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
