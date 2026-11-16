//! Integration tests of encrypted backups: restoring on another computer
//! and refusing damaged backups without touching the current vault.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{PASSWORD, Workspace, entry, names_in};
use lockwell_lib::error::AppError;

/// A workspace with one entry and one file, and a backup of it.
fn backed_up() -> (Workspace, PathBuf) {
    let workspace = Workspace::new();
    let source = workspace.user_file("doc.txt", b"backed up content");
    let vault = workspace.created_vault();
    let unlocked = vault.unlocked().unwrap();
    unlocked.create_entry(&entry("backed")).unwrap();
    unlocked.import_file(&source).unwrap();
    let backup = workspace.outside().join("vault.lwbackup");
    unlocked.create_backup(&backup).unwrap();
    (workspace, backup)
}

fn titles(vault: &lockwell_lib::vault::Vault) -> Vec<String> {
    vault
        .unlocked()
        .unwrap()
        .list_entries()
        .unwrap()
        .into_iter()
        .map(|summary| summary.title)
        .collect()
}

#[test]
fn a_backup_restores_on_another_computer() {
    let (_original, backup) = backed_up();
    let computer = Workspace::new();
    let mut vault = computer.vault();

    vault.restore_backup(&backup, PASSWORD).unwrap();
    assert!(!vault.is_unlocked());
    vault.unlock(PASSWORD).unwrap();

    let unlocked = vault.unlocked().unwrap();
    let entries = unlocked.list_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        unlocked.get_entry(&entries[0].id).unwrap().password,
        "backed-password"
    );
    let files = unlocked.list_files().unwrap();
    let exported = computer.outside().join("doc.txt");
    unlocked
        .export_file(&files[0].id, &exported, false)
        .unwrap();
    assert_eq!(fs::read(exported).unwrap(), b"backed up content");
}

#[test]
fn restoring_replaces_newer_changes() {
    let (workspace, backup) = backed_up();
    let mut vault = workspace.vault();
    vault.unlock(PASSWORD).unwrap();
    vault
        .unlocked()
        .unwrap()
        .create_entry(&entry("later"))
        .unwrap();

    vault.restore_backup(&backup, PASSWORD).unwrap();
    vault.unlock(PASSWORD).unwrap();
    assert_eq!(titles(&vault), ["backed title"]);
}

#[test]
fn damaged_backups_leave_the_vault_untouched() {
    let (workspace, backup) = backed_up();
    let bytes = fs::read(&backup).unwrap();
    let damaged = workspace.outside().join("damaged.lwbackup");
    let middle = bytes.len() / 2;
    let mut flipped = bytes.clone();
    flipped[middle] ^= 0x10;
    let cases = [bytes[..middle].to_vec(), flipped, Vec::new()];

    let mut vault = workspace.vault();
    vault.unlock(PASSWORD).unwrap();
    let parent = workspace.vault_dir().parent().unwrap().to_path_buf();
    let before = names_in(&parent);
    for case in cases {
        fs::write(&damaged, &case).unwrap();
        assert!(matches!(
            vault.restore_backup(&damaged, PASSWORD),
            Err(AppError::Corrupt(_))
        ));
        assert!(vault.is_unlocked());
        assert_eq!(titles(&vault), ["backed title"]);
        assert_eq!(names_in(&parent), before);
    }
}

#[test]
fn a_wrong_password_does_not_open_a_backup() {
    let (_original, backup) = backed_up();
    let computer = Workspace::new();
    let mut vault = computer.vault();
    assert_eq!(
        vault
            .restore_backup(&backup, "not the synthetic password")
            .unwrap_err(),
        AppError::WrongPasswordOrCorrupt
    );
    assert!(!vault.exists());
}
