/**
 * Typed access to the Tauri commands and their errors.
 *
 * Every command module in this folder goes through {@link call}, so errors
 * from the Rust core (`{ code, message }`) reach the components as
 * {@link ApiError} instances. Error messages never contain secrets.
 */
import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
import type { ErrorCode } from "./types";

/** An error returned by a command, with its stable code. */
export class ApiError extends Error {
  readonly code: ErrorCode;

  constructor(code: ErrorCode, message: string) {
    super(message);
    this.name = "ApiError";
    this.code = code;
  }
}

/**
 * Calls a command and converts its error into an {@link ApiError}.
 *
 * @param command - Name of the Rust command, in snake_case.
 * @param args - Arguments of the command, with camelCase keys.
 * @returns What the command returns.
 * @throws {ApiError} If the command fails.
 */
export async function call<T>(command: string, args?: InvokeArgs): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toApiError(error);
  }
}

/**
 * Normalizes anything thrown by `invoke` into an {@link ApiError}. A payload
 * with `code` and `message` keeps its code; any other value becomes the
 * client-side code `unknown`.
 */
export function toApiError(error: unknown): ApiError {
  if (error instanceof ApiError) {
    return error;
  }
  if (isErrorPayload(error)) {
    return new ApiError(error.code as ErrorCode, error.message);
  }
  if (error instanceof Error) {
    return new ApiError("unknown", error.message);
  }
  return new ApiError("unknown", String(error));
}

/** True if the error means the vault is locked. */
export function isLockedError(error: unknown): boolean {
  return error instanceof ApiError && error.code === "locked";
}

/** True if the error means the master password was wrong. */
export function isWrongPasswordError(error: unknown): boolean {
  return error instanceof ApiError && error.code === "wrong_password";
}

/**
 * A message for people: capitalized and ending with a period. Falls back to
 * "Something went wrong." when the error has no message.
 */
export function errorMessage(error: unknown): string {
  const text = toApiError(error).message.trim() || "Something went wrong";
  const sentence = text.charAt(0).toUpperCase() + text.slice(1);
  return /[.!?]$/.test(sentence) ? sentence : `${sentence}.`;
}

/** Whether `value` has the `{ code, message }` shape the core serializes. */
function isErrorPayload(
  value: unknown,
): value is { code: string; message: string } {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as { code?: unknown }).code === "string" &&
    typeof (value as { message?: unknown }).message === "string"
  );
}
