# src/test/setup.ts

Vitest setup: registers the Testing Library DOM matchers, replaces
`@tauri-apps/api/core` and `@tauri-apps/api/event` with mocks (see
`tauriMock.ts`), and cleans up the DOM
and the mocks after each test.
