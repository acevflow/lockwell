/** Password field with a show/hide toggle, used wherever a password is typed. */
import { useState } from "react";

interface PasswordInputProps {
  /** Id of the input; the label points to it. */
  id: string;
  /** Visible label, also used in the toggle button's accessible name. */
  label: string;
  /** Current text of the field (controlled component). */
  value: string;
  /** Called with the new text on every change. */
  onChange: (value: string) => void;
  /** `current-password`, `new-password` or `off`. */
  autoComplete: string;
  /** Focuses the field when it mounts. */
  autoFocus?: boolean;
  /** Marks the field as required for assistive technology. */
  required?: boolean;
  /** Id of an element that describes the field, such as a hint. */
  describedBy?: string;
}

/**
 * A labelled password field with a show/hide toggle. Hidden by default.
 * Spell checking, autocapitalization and autocorrection are off so the typed
 * text is not sent to or altered by the browser.
 */
function PasswordInput({
  id,
  label,
  value,
  onChange,
  autoComplete,
  autoFocus = false,
  required = false,
  describedBy,
}: PasswordInputProps) {
  const [visible, setVisible] = useState(false);

  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      <div className="password-input">
        <input
          id={id}
          type={visible ? "text" : "password"}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          autoComplete={autoComplete}
          autoFocus={autoFocus}
          required={required}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          aria-describedby={describedBy}
        />
        <button
          type="button"
          className="secondary"
          onClick={() => setVisible((shown) => !shown)}
          aria-pressed={visible}
          aria-label={
            visible
              ? `Hide ${label.toLowerCase()}`
              : `Show ${label.toLowerCase()}`
          }
        >
          {visible ? "Hide" : "Show"}
        </button>
      </div>
    </div>
  );
}

export default PasswordInput;
