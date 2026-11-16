interface ErrorBannerProps {
  message: string | null;
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
