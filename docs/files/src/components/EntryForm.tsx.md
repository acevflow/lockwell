# src/components/EntryForm.tsx

Form to create or edit an entry: title (required), username, password (with
Show/Hide), website and notes. Field lengths are limited with `ENTRY_LIMITS`.
Errors from saving are shown inside the form, which stays editable. Only the
five editable fields are kept from `initial`, which may be a full `Entry`: the
core rejects unknown fields such as `id` or `createdAt`.
