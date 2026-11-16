# src/components/ConfirmDialog.tsx

Modal confirmation (`role="alertdialog"`) for destructive actions. Cancel has
the initial focus and Escape cancels, so a stray Enter never confirms. While
`busy`, the buttons are disabled and Escape is ignored.
