# src/screens/SettingsTab.test.tsx

Tests for the Settings section: successful change with every field cleared,
local validation, the busy state, wrong current password, other errors that
never show the typed passwords, the locked case, and backup errors (a locked
vault calls `onLocked`, other errors are shown).
