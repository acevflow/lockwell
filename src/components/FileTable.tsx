/** Table of the files stored in the vault. */
import type { FileSummary } from "../api/types";
import { formatBytes } from "../format";

interface FileTableProps {
  /** Files to list, in display order. */
  files: FileSummary[];
  /** Disables the row buttons while an operation runs. */
  disabled: boolean;
  /** Called when the user asks for a decrypted copy of `file`. */
  onSaveCopy: (file: FileSummary) => void;
  /** Called when the user asks to delete `file`; the caller confirms. */
  onDelete: (file: FileSummary) => void;
}

/** Table of stored files with a "Save a copy" and a "Delete" button per row. */
function FileTable({ files, disabled, onSaveCopy, onDelete }: FileTableProps) {
  return (
    <table className="file-table">
      <thead>
        <tr>
          <th scope="col">Name</th>
          <th scope="col">Size</th>
          <th scope="col">Added</th>
          <th scope="col">
            <span className="visually-hidden">Actions</span>
          </th>
        </tr>
      </thead>
      <tbody>
        {files.map((file) => (
          <tr key={file.id}>
            <td className="file-name">{file.name}</td>
            <td>{formatBytes(file.size)}</td>
            <td>{new Date(file.createdAt).toLocaleDateString()}</td>
            <td className="row-actions">
              <button
                type="button"
                className="secondary small"
                onClick={() => onSaveCopy(file)}
                disabled={disabled}
                aria-label={`Save a copy of ${file.name}`}
              >
                Save a copy
              </button>
              <button
                type="button"
                className="danger small"
                onClick={() => onDelete(file)}
                disabled={disabled}
                aria-label={`Delete ${file.name}`}
              >
                Delete
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export default FileTable;
