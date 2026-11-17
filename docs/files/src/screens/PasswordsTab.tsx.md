# src/screens/PasswordsTab.tsx

The Passwords section: a searchable list of entries in the sidebar and, in the
content area, the selected entry (`EntryView`) or the form to create or edit
one (`EntryForm`). Deleting asks for confirmation. The list uses summaries;
the full entry, with its password, is fetched only when the user opens it. A
`locked` error returns to the unlock screen. Copy requests go through
`copy_entry_field`; their errors are shown in the section.

The list is loaded when the tab opens with `useInitialLoad`.
