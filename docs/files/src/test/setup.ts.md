# src/test/setup.ts

Vitest setup: registers the Testing Library DOM matchers, replaces
`@tauri-apps/api/core` with a mock (see `tauriMock.ts`), and cleans up the DOM
and the mocks after each test.
