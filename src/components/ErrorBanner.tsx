/** Inline error message for forms and tabs. */
interface ErrorBannerProps {
  /** Text to show; nothing is rendered when it is `null` or empty. */
  message: string | null;
  /** Adds a Dismiss button that calls this when pressed. */
  onDismiss?: () => void;
}

/** Shows an error message to the user and announces it to screen readers. */
function ErrorBanner({ message, onDismiss }: ErrorBannerProps) {
  if (!message) {
    return null;
  }
  return (
    <div className="error-banner" role="alert">
      <span>{message}</span>
      {onDismiss && (
        <button
          type="button"
          className="link"
          onClick={onDismiss}
          aria-label="Dismiss error"
        >
          Dismiss
        </button>
      )}
    </div>
  );
}

export default ErrorBanner;
