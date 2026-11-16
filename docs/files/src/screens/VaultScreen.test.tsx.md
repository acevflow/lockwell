# src/screens/VaultScreen.test.tsx

Tests for the unlocked vault screen: it opens on the passwords tab, switches
tabs with the right ARIA state, locks with the Lock button, reports a lock by
the core with its reason and a locked error from a tab without one, and
reports user activity to the core.
