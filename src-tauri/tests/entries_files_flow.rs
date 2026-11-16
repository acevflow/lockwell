//! Integration tests of entries and files: storage, persistence between
//! instances and encryption at rest.

mod common;

use std::fs;

use common::{PASSWORD, Workspace, any_file_contains, entry, names_in};
use lockwell_lib::error::AppError;

#[test]
fn entries_persist_across_instances() {
    let workspace = Workspace::new();
    let vault = workspace.created_vault();
    let unlocked = vault.unlocked().unwrap();
    let kept = unlocked.create_entry(&entry("kept")).unwrap();
    let edited = unlocked.create_entry(&entry("before")).unwrap();
    let deleted = unlocked.create_entry(&entry("deleted")).unwrap();
    unlocked.update_entry(&edited, &entry("after")).unwrap();
    unlocked.delete_entry(&deleted).unwrap();
    drop(vault);

    let mut reopened = workspace.vault();
    reopened.unlock(PASSWORD).unwrap();
    let unlocked = reopened.unlocked().unwrap();
    let titles: Vec<_> = unlocked
        .list_entries()
        .unwrap()
        .into_iter()
        .map(|summary| summary.title)
        .collect();
    assert_eq!(titles, ["after title", "kept title"]);
    assert_eq!(unlocked.get_entry(&kept).unwrap().notes, "kept notes");
    assert_eq!(
        unlocked.get_entry(&deleted).unwrap_err(),
        AppError::NotFound("entry")
    );
}

#[test]
fn files_persist_across_instances_and_export_intact() {
    let workspace = Workspace::new();
    let content: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
    let source = workspace.user_file("report.bin", &content);
    let vault = workspace.created_vault();
    let summary = vault.unlocked().unwrap().import_file(&source).unwrap();
    assert_eq!(summary.name, "report.bin");
    assert_eq!(summary.size, content.len() as u64);
    drop(vault);

    let mut reopened = workspace.vault();
    reopened.unlock(PASSWORD).unwrap();
    let unlocked = reopened.unlocked().unwrap();
    assert_eq!(
        unlocked.list_files().unwrap(),
        std::slice::from_ref(&summary)
    );
    let exported = workspace.outside().join("copy.bin");
    unlocked.export_file(&summary.id, &exported, false).unwrap();
    assert_eq!(fs::read(&exported).unwrap(), content);

    unlocked.delete_file(&summary.id).unwrap();
    assert!(unlocked.list_files().unwrap().is_empty());
    assert!(names_in(&workspace.vault_dir().join("files")).is_empty());
}

#[test]
fn nothing_is_stored_in_clear_text() {
    let workspace = Workspace::new();
    let source = workspace.user_file("marker-file-name.txt", b"marker file content");
    let mut vault = workspace.created_vault();
    {
        let unlocked = vault.unlocked().unwrap();
        unlocked.create_entry(&entry("marker")).unwrap();
        unlocked.import_file(&source).unwrap();
    }
    vault.lock();

    for needle in [
        "marker title",
        "marker-password",
        "marker-user@example.test",
        "marker.example.test",
        "marker notes",
        "marker-file-name",
        "marker file content",
        PASSWORD,
    ] {
        assert!(
            !any_file_contains(&workspace.vault_dir(), needle.as_bytes()),
            "{needle} is visible on disk"
        );
    }
}

#[test]
fn the_vault_folder_is_never_used_for_user_files() {
    let workspace = Workspace::new();
    let vault = workspace.created_vault();
    let unlocked = vault.unlocked().unwrap();
    let database = workspace.vault_dir().join("vault.db");
    assert!(matches!(
        unlocked.import_file(&database),
        Err(AppError::InvalidInput(_))
    ));
    let source = workspace.user_file("a.txt", b"x");
    let id = unlocked.import_file(&source).unwrap().id;
    assert!(matches!(
        unlocked.export_file(&id, &database, true),
        Err(AppError::InvalidInput(_))
    ));
    assert!(matches!(
        unlocked.create_backup(&workspace.vault_dir().join("b.lwbackup")),
        Err(AppError::InvalidInput(_))
    ));
}
