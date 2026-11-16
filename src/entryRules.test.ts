import { describe, expect, it } from "vitest";
import type { EntrySummary } from "./api/types";
import { filterEntries } from "./entryRules";

const entries: EntrySummary[] = [
  {
    id: "1",
    title: "Mail",
    username: "me@example.com",
    url: "https://mail.example",
    updatedAt: 0,
  },
  {
    id: "2",
    title: "Bank",
    username: "customer",
    url: "https://bank.example",
    updatedAt: 0,
  },
];

describe("filterEntries", () => {
  it("returns everything for an empty or blank query", () => {
    expect(filterEntries(entries, "")).toEqual(entries);
    expect(filterEntries(entries, "   ")).toEqual(entries);
  });

  it("matches title, username and website without case", () => {
    expect(filterEntries(entries, "MAIL").map((e) => e.id)).toEqual(["1"]);
    expect(filterEntries(entries, "customer").map((e) => e.id)).toEqual(["2"]);
    expect(filterEntries(entries, "bank.example").map((e) => e.id)).toEqual([
      "2",
    ]);
    expect(filterEntries(entries, "example").length).toBe(2);
  });

  it("returns nothing when nothing matches", () => {
    expect(filterEntries(entries, "zzz")).toEqual([]);
  });
});
