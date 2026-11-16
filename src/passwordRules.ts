/**
 * Master password rules, mirrored from the core
 * (`src-tauri/src/vault/password_policy.rs`) for immediate feedback.
 * The core remains the authority and checks them again.
 */

export const MIN_MASTER_PASSWORD_CHARS = 12;
export const MAX_MASTER_PASSWORD_CHARS = 1024;

/** Number of Unicode characters, as the core counts them. */
export function charCount(text: string): number {
  return [...text].length;
}

/** Returns why `password` is not acceptable, or `null` if it is. */
export function masterPasswordProblem(password: string): string | null {
  const length = charCount(password);
  if (length < MIN_MASTER_PASSWORD_CHARS) {
    return `Use at least ${MIN_MASTER_PASSWORD_CHARS} characters.`;
  }
  if (length > MAX_MASTER_PASSWORD_CHARS) {
    return `Use at most ${MAX_MASTER_PASSWORD_CHARS} characters.`;
  }
  if (password.trim() === "") {
    return "The password cannot be only spaces.";
  }
  return null;
}
