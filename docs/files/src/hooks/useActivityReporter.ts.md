# src/hooks/useActivityReporter.ts

While the vault is unlocked, reports keyboard, mouse and touch activity to the
core with `touch_activity`, at most once every 15 seconds, so the automatic
lock only happens after real inactivity. A lost report can only make the lock
happen sooner.
