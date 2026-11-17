// @vitest-environment node
import { describe, expect, it } from "vitest";
import {
  licenseOf,
  parseTree,
  render,
  selectCrates,
  sourceOf,
} from "./third-party-licenses.mjs";

const TREE = `lockwell v0.1.0 (D:\\x)
serde v1.0.1
serde_derive v1.0.1 (proc-macro) (*)
serde v1.0.1 (*)
`;

const crate = (name, version, extra = {}) => ({ name, version, ...extra });

describe("parseTree", () => {
  it("reads name and version and ignores markers and repeats", () => {
    expect([...parseTree(TREE)].sort()).toEqual([
      "lockwell 0.1.0",
      "serde 1.0.1",
      "serde_derive 1.0.1",
    ]);
  });

  it("ignores empty and unrelated lines", () => {
    expect(parseTree("\n  \nnot a crate line").size).toBe(0);
  });
});

describe("selectCrates", () => {
  const metadata = {
    packages: [
      crate("zeta", "1.0.0"),
      crate("lockwell", "0.1.0"),
      crate("alpha", "2.0.0"),
      crate("alpha", "1.0.0"),
      crate("test-only", "1.0.0"),
    ],
  };

  it("keeps wanted crates, drops the root and sorts by name and version", () => {
    const wanted = new Set([
      "zeta 1.0.0",
      "lockwell 0.1.0",
      "alpha 2.0.0",
      "alpha 1.0.0",
      "windows-sys 0.61.2",
    ]);
    expect(
      selectCrates(metadata, wanted).map((c) => `${c.name} ${c.version}`),
    ).toEqual(["alpha 1.0.0", "alpha 2.0.0", "zeta 1.0.0"]);
  });
});

describe("licenseOf and sourceOf", () => {
  it("rewrites the old slash form", () => {
    expect(licenseOf(crate("a", "1", { license: "MIT/Apache-2.0" }))).toBe(
      "MIT OR Apache-2.0",
    );
  });

  it("flags a missing license instead of hiding it", () => {
    expect(licenseOf(crate("a", "1"))).toBe("see source");
  });

  it("falls back from repository to homepage to crates.io", () => {
    expect(sourceOf(crate("a", "1", { repository: "https://r" }))).toBe(
      "https://r",
    );
    expect(sourceOf(crate("a", "1", { homepage: "https://h" }))).toBe(
      "https://h",
    );
    expect(sourceOf(crate("a", "1"))).toBe("https://crates.io/crates/a");
  });
});

describe("render", () => {
  it("lists every crate and its count", () => {
    const text = render([
      crate("a", "1.0.0", { license: "MIT", repository: "https://a" }),
      crate("b", "2.0.0", { license: "Apache-2.0" }),
    ]);
    expect(text).toContain("| a | 1.0.0 | MIT | https://a |");
    expect(text).toContain(
      "| b | 2.0.0 | Apache-2.0 | https://crates.io/crates/b |",
    );
    expect(text).toContain("\n2 crates.\n");
    expect(text).toContain("x86_64-unknown-linux-gnu");
  });
});
