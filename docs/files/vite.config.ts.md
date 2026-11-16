# vite.config.ts

Configuration for Vite and Vitest.

- Uses the React plugin.
- Serves the interface on the fixed port 1420 that Tauri expects, and does not
  watch `src-tauri/`.
- Runs the tests in a `jsdom` environment with `src/test/setup.ts` as the setup
  file.
- Measures interface test coverage with the V8 provider
  (`npm run test:coverage`) and writes text, HTML and LCOV reports to
  `coverage/frontend/`.
  Tests, test helpers, the entry point and type-only files are excluded.
