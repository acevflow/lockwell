# src/components/EntryView.tsx

Read-only view of an entry. The password is masked until the user clicks
Show. The username and password have Copy buttons; copying goes through the
core (`copy_entry_field`) and a message says the clipboard will be cleared in
30 seconds. Offers Edit and Delete.
