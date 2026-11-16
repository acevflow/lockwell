# src-tauri/src/security/autolock.rs

Inactivity tracking for the automatic lock.

- `IDLE_TIMEOUT` is 5 minutes; `CHECK_INTERVAL` is 5 seconds.
- `ActivityTracker` stores the time of the last activity. `touch()` records
  activity (never moving the deadline backwards) and `is_idle()` tells whether
  the timeout has passed.

Time is passed in by the caller, so the tests check the deadline without
waiting.
