# src/vite-env.d.ts

## Purpose

Type declaration file that pulls in the types of Vite's client environment.

## Place in the architecture

The file contains one line, `/// <reference types="vite/client" />`. It makes TypeScript
understand what Vite adds to the browser environment: `import.meta.env`, and imports of
assets such as the stylesheet that [`main.tsx`](main.tsx.md) loads with `import
"./styles.css"`. TypeScript includes it automatically through the `src` folder in
`tsconfig.json`.

## Public API

None. A declaration file produces no code. It only adds types to the project.

## Internal structure

A single triple-slash reference directive to the `vite/client` types.

## Dependencies

- Internal: none.
- External: `vite` (its `client` type declarations).

## Usage example

Nothing to call. Its effect is visible when the type check passes:

```bash
npm run typecheck
```

Without this file the type check fails. Compiling the interface with this one file left
out (TypeScript 6.0.3, with a temporary configuration outside the repository) gave:

```text
src/main.tsx(5,8): error TS2882: Cannot find module or type declarations for side-effect import of './styles.css'.
```

The file is also excluded from the coverage report in `vite.config.ts`.

## Edge cases, invariants and error handling

- It must stay inside the folders that `tsconfig.json` includes. Moving it elsewhere makes
  the compiler lose the Vite types.
- It does nothing at run time, and it does not appear in the built app.

## Security

None. It affects only type checking.

## Related files and tests

- Related: [`main.tsx`](main.tsx.md), `tsconfig.json`, `vite.config.ts`.
- Tests: none. `npm run typecheck` is the check.
