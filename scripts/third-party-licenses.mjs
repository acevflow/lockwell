/**
 * Generates THIRD-PARTY-LICENSES.md: the licenses of the third-party software
 * in a Linux build. The Rust crates come from `Cargo.lock` through `cargo`;
 * the system libraries section is fixed text.
 *
 * Usage (from the repository root, with `cargo` installed):
 *   npm run licenses          writes THIRD-PARTY-LICENSES.md
 *   npm run licenses:check    fails if the file is out of date
 *
 * It only reads local files and runs `cargo --locked`; it makes no network
 * requests and adds no dependencies.
 */
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import process from "node:process";

/** Rust target whose dependencies are listed. */
export const TARGET = "x86_64-unknown-linux-gnu";

const MANIFEST = "src-tauri/Cargo.toml";
const OUTPUT = fileURLToPath(
  new URL("../THIRD-PARTY-LICENSES.md", import.meta.url),
);

/**
 * Names and versions in the output of `cargo tree --prefix none`, as
 * `name version` strings. Lines look like `name v1.2.3 (proc-macro) (*)`.
 */
export function parseTree(text) {
  const wanted = new Set();
  for (const line of text.split("\n")) {
    const match = /^(\S+) v(\S+)/.exec(line.trim());
    if (match) {
      wanted.add(`${match[1]} ${match[2]}`);
    }
  }
  return wanted;
}

/**
 * The crates of `metadata` (the JSON of `cargo metadata`) that are in
 * `wanted`, sorted by name and version. The root crate is left out. Wanted
 * entries that metadata does not know (crates of other platforms that
 * `cargo tree` still prints) are ignored.
 */
export function selectCrates(metadata, wanted, rootName = "lockwell") {
  return metadata.packages
    .filter((p) => p.name !== rootName && wanted.has(`${p.name} ${p.version}`))
    .sort(
      (a, b) =>
        a.name.localeCompare(b.name) || a.version.localeCompare(b.version),
    );
}

/** The license a crate declares, with the old `A/B` form written as `A OR B`. */
export function licenseOf(crate) {
  return (crate.license || "see source").replaceAll("/", " OR ");
}

/** Where to find the source of a crate. */
export function sourceOf(crate) {
  return (
    crate.repository ||
    crate.homepage ||
    `https://crates.io/crates/${crate.name}`
  );
}

/** The whole document for `crates`. */
export function render(crates) {
  const rows = crates
    .map(
      (c) => `| ${c.name} | ${c.version} | ${licenseOf(c)} | ${sourceOf(c)} |`,
    )
    .join("\n");
  return `# Third-party licenses (Linux)

lockwell is licensed under the GNU General Public License v3.0 or later (see
[LICENSE](LICENSE)). This file lists the third-party software that a Linux
build of lockwell contains or links to, with the license each one declares.
It is intended to accompany Linux packages, especially those that bundle
system libraries (for example an AppImage).

The Windows and macOS builds use a different set of dependencies; this file
does not cover them.

## 1. System libraries (not Rust crates)

lockwell uses the system webview through Tauri. These libraries are **not**
part of the lockwell source. A package that relies on the ones installed in
the system (such as a \`.deb\`) does not redistribute them. A package that
**bundles** them (such as an AppImage) does, and must meet their license
terms: ship the license texts, offer the corresponding source code, and allow
the library to be replaced by the user.

| Library   | License as published by the project or packagers                            |
| --------- | --------------------------------------------------------------------------- |
| WebKitGTK | BSD-2-Clause and LGPL-2.0-or-later / LGPL-2.1 (varies by packager)           |
| GTK 3     | LGPL-2.1-or-later (not verified against its source; check before bundling)   |

The versions and the exact license files must be taken from the packages that
are actually bundled; check them at packaging time.

## 2. Rust crates

Generated from \`Cargo.lock\` for the \`${TARGET}\` target,
runtime dependencies only (build-time and test-only crates are excluded). The
license is the SPDX expression each crate declares in its \`Cargo.toml\`. The
full text and the copyright notices of each crate are in its source package
and repository. Crates that offer a choice of licenses (\`MIT OR Apache-2.0\`)
are used under either; lockwell uses them under terms compatible with
GPL-3.0-or-later.

Notes:

- Crates under \`MPL-2.0\` keep that license for their own files.
- Crates under \`Apache-2.0\` are compatible with GPL-3.0, but not with GPL-2.0.

| Crate | Version | License | Source |
| ----- | ------- | ------- | ------ |
${rows}

${crates.length} crates.
`;
}

/** Runs `cargo` and returns its output; the lock file is never changed. */
function cargo(args) {
  return execFileSync("cargo", args, {
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
    stdio: ["ignore", "pipe", "inherit"],
  });
}

/** The document for the current `Cargo.lock`. */
export function generate() {
  const tree = cargo([
    "tree",
    "--manifest-path",
    MANIFEST,
    "--target",
    TARGET,
    "-e",
    "normal",
    "--prefix",
    "none",
    "--locked",
  ]);
  const metadata = JSON.parse(
    cargo([
      "metadata",
      "--manifest-path",
      MANIFEST,
      "--format-version",
      "1",
      "--filter-platform",
      TARGET,
      "--locked",
    ]),
  );
  return render(selectCrates(metadata, parseTree(tree)));
}

function main() {
  const document = generate();
  if (process.argv.includes("--check")) {
    if (readFileSync(OUTPUT, "utf8") !== document) {
      console.error(
        "THIRD-PARTY-LICENSES.md is out of date. Run `npm run licenses`.",
      );
      process.exit(1);
    }
    console.log("THIRD-PARTY-LICENSES.md is up to date.");
    return;
  }
  writeFileSync(OUTPUT, document);
  console.log("Wrote THIRD-PARTY-LICENSES.md");
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  main();
}
