/** Modal confirmation dialog used before destructive or irreversible actions. */
import { useEffect, useId } from "react";

interface ConfirmDialogProps {
  /** Heading of the dialog; it also names the dialog for screen readers. */
  title: string;
  /** Explains what will happen if the user confirms. */
  message: string;
  /** Text of the confirm button, such as "Delete". */
  confirmLabel: string;
  /** Called when the user presses the confirm button. */
  onConfirm: () => void;
  /** Called by the Cancel button and by Escape (unless `busy`). */
  onCancel: () => void;
  /** Disables the buttons while the confirmed action runs. */
  busy?: boolean;
}

/**
 * Modal confirmation for destructive actions. Cancel has the initial focus
 * and Escape cancels, so a stray Enter never confirms by accident.
 */
function ConfirmDialog({
  title,
  message,
  confirmLabel,
  onConfirm,
  onCancel,
  busy = false,
}: ConfirmDialogProps) {
  const titleId = useId();
  const messageId = useId();

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape" && !busy) {
        onCancel();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [busy, onCancel]);

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={messageId}
      >
        <h2 id={titleId}>{title}</h2>
        <p id={messageId}>{message}</p>
        <div className="actions">
          <button
            type="button"
            className="secondary"
            onClick={onCancel}
            disabled={busy}
            autoFocus
          >
            Cancel
          </button>
          <button
            type="button"
            className="danger"
            onClick={onConfirm}
            disabled={busy}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

export default ConfirmDialog;
