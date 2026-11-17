import { describe, expect, it } from "vitest";
import { commandError, failWith, mockCommands } from "../test/tauriMock";
import {
  ApiError,
  call,
  errorMessage,
  isLockedError,
  isWrongPasswordError,
  toApiError,
} from "./client";

describe("toApiError", () => {
  it("keeps the code and message of a command error", () => {
    const error = toApiError(commandError("locked", "the vault is locked"));
    expect(error).toBeInstanceOf(ApiError);
    expect(error.code).toBe("locked");
    expect(error.message).toBe("the vault is locked");
  });

  it("wraps plain errors and other values as unknown", () => {
    expect(toApiError(new Error("boom")).code).toBe("unknown");
    expect(toApiError("text").message).toBe("text");
    expect(toApiError(null).message).toBe("null");
    expect(toApiError({ code: 1, message: "x" }).code).toBe("unknown");
  });

  it("returns an ApiError unchanged", () => {
    const original = new ApiError("io", "disk");
    expect(toApiError(original)).toBe(original);
  });
});

describe("errorMessage", () => {
  it("capitalizes and ends the sentence", () => {
    expect(errorMessage(commandError("io", "disk full"))).toBe("Disk full.");
    expect(errorMessage(commandError("io", "Done!"))).toBe("Done!");
  });

  it("falls back for empty messages", () => {
    expect(errorMessage(commandError("io", " "))).toBe("Something went wrong.");
  });
});

describe("isWrongPasswordError", () => {
  it("detects only the wrong_password code", () => {
    expect(isWrongPasswordError(new ApiError("wrong_password", "x"))).toBe(
      true,
    );
    expect(isWrongPasswordError(new ApiError("io", "x"))).toBe(false);
    expect(isWrongPasswordError(new Error("wrong_password"))).toBe(false);
  });
});

describe("isLockedError", () => {
  it("detects only the locked code", () => {
    expect(isLockedError(new ApiError("locked", "x"))).toBe(true);
    expect(isLockedError(new ApiError("io", "x"))).toBe(false);
    expect(isLockedError(new Error("locked"))).toBe(false);
  });
});

describe("call", () => {
  it("passes the command and arguments and returns the result", async () => {
    const invoke = mockCommands({ get_entry: ({ id }) => `entry ${id}` });
    await expect(call("get_entry", { id: "abc" })).resolves.toBe("entry abc");
    expect(invoke).toHaveBeenCalledWith("get_entry", { id: "abc" });
  });

  it("converts rejections into ApiError", async () => {
    mockCommands({ lock_vault: failWith("storage", "database is busy") });
    await expect(call("lock_vault")).rejects.toMatchObject({
      code: "storage",
      message: "database is busy",
    });
  });
});
