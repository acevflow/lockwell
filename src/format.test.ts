import { describe, expect, it } from "vitest";
import { formatBytes } from "./format";

describe("formatBytes", () => {
  it("shows small sizes in bytes", () => {
    expect(formatBytes(0)).toBe("0 bytes");
    expect(formatBytes(1)).toBe("1 byte");
    expect(formatBytes(999)).toBe("999 bytes");
  });

  it("uses decimal units with one decimal below 100", () => {
    expect(formatBytes(1000)).toBe("1 KB");
    expect(formatBytes(1500)).toBe("1.5 KB");
    expect(formatBytes(2_340_000)).toBe("2.3 MB");
    expect(formatBytes(150_000_000)).toBe("150 MB");
    expect(formatBytes(3_000_000_000)).toBe("3 GB");
  });

  it("stops at terabytes", () => {
    expect(formatBytes(5_000_000_000_000_000)).toBe("5000 TB");
  });
});
