# Code file index

One document per code file. Each document describes what the file does and
how it fits in the project.

## Interface (`src/`)

| File                                        | Description                         |
| ------------------------------------------- | ----------------------------------- |
| [`src/main.tsx`](src/main.tsx.md)           | Mounts the React app into the page. |
| [`src/App.tsx`](src/App.tsx.md)             | Root component of the interface.    |
| [`src/App.test.tsx`](src/App.test.tsx.md)   | Smoke test for the root component.  |
| [`src/test/setup.ts`](src/test/setup.ts.md) | Vitest setup for Testing Library.   |
| [`src/vite-env.d.ts`](src/vite-env.d.ts.md) | Vite client type declarations.      |

## Core (`src-tauri/`)

| File                                                | Description                            |
| --------------------------------------------------- | -------------------------------------- |
| [`src-tauri/src/main.rs`](src-tauri/src/main.rs.md) | Desktop binary entry point.            |
| [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs.md)   | Builds and runs the Tauri application. |
| [`src-tauri/build.rs`](src-tauri/build.rs.md)       | Tauri build script.                    |

## Tooling

| File                                      | Description                    |
| ----------------------------------------- | ------------------------------ |
| [`vite.config.ts`](vite.config.ts.md)     | Vite and Vitest configuration. |
| [`eslint.config.js`](eslint.config.js.md) | ESLint configuration.          |
