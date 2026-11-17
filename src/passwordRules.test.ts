/** Tests of the master password rules, which mirror those of the core. */
import { describe, expect, it } from "vitest";
import {
  MAX_MASTER_PASSWORD_CHARS,
  charCount,
  masterPasswordProblem,
} from "./passwordRules";

describe("masterPasswordProblem", () => {
  it("accepts 12 characters", () => {
    expect(masterPasswordProblem("abcdefghijkl")).toBeNull();
  });

  it("rejects shorter passwords", () => {
    expect(masterPasswordProblem("")).toMatch(/12/);
    expect(masterPasswordProblem("abcdefghijk")).toMatch(/12/);
  });

  it("counts characters, not UTF-16 units", () => {
    expect(charCount("🔒🔒")).toBe(2);
    expect(masterPasswordProblem("🔒".repeat(11))).not.toBeNull();
    expect(masterPasswordProblem("🔒".repeat(12))).toBeNull();
  });

  it("rejects passwords that are too long or only spaces", () => {
    expect(
      masterPasswordProblem("a".repeat(MAX_MASTER_PASSWORD_CHARS + 1)),
    ).toMatch(/at most/);
    expect(masterPasswordProblem(" ".repeat(20))).toMatch(/spaces/);
  });
});
