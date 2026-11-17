# src/hooks/hooks.test.tsx

Tests for the hooks, with a fixed fake clock instead of the real one:
`useActivityReporter` (every activity event, one report per interval, again
after the interval, no reports after unmount, failed reports ignored),
`useApiError` (messages, the `locked` error calling `onLocked`, clearing),
`useInitialLoad` (result, failure, ignored after unmount) and
`useVaultLockedEvent` (the lock reason, unlistening on unmount, also when
unmounted before listening starts, and logging when it cannot listen).
