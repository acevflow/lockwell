//! Integration tests of the vault life cycle through the public API,
//! reopening the vault from disk with new instances.

mod common;

use common::{NEW_PASSWORD, PASSWORD, Workspace, entry};
use lockwell_lib::error::AppError;
use lockwell_lib::vault::VaultStatus;

#[test]
fn a_new_vault_can_be_reopened_from_disk() {
    let workspace = Workspace::new();
    let created = workspace.created_vault();
    let id = created
        .unlocked()
        .unwrap()
        .create_entry(&entry("kept"))
        .unwrap();
    drop(created);

    let mut reopened = workspace.vault();
    assert_eq!(
        reopened.status(),
        VaultStatus {
            exists: true,
            unlocked: false
        }
    );
    reopened.unlock(PASSWORD).unwrap();
    let stored = reopened.unlocked().unwrap().get_entry(&id).unwrap();
    assert_eq!(stored.title, "kept title");
    assert_eq!(stored.password, "kept-password");
}

/// Create, lock, unlock, change the master password and reopen from disk:
/// the old password stops working and the entry is still there.
#[test]
fn the_full_life_cycle_keeps_the_data() {
    let workspace = Workspace::new();
    let mut vault = workspace.created_vault();
    let id = vault
        .unlocked()
        .unwrap()
        .create_entry(&entry("cycle"))
        .unwrap();

    vault.lock();
    vault.unlock(PASSWORD).unwrap();
    vault
        .change_master_password(PASSWORD, NEW_PASSWORD)
        .unwrap();
    vault.lock();
    drop(vault);

    let mut reopened = workspace.vault();
    assert_eq!(
        reopened.unlock(PASSWORD).unwrap_err(),
        AppError::WrongPasswordOrCorrupt
    );
    reopened.unlock(NEW_PASSWORD).unwrap();
    let stored: Vec<_> = reopened
        .unlocked()
        .unwrap()
        .list_entries()
        .unwrap()
        .into_iter()
        .map(|summary| (summary.id, summary.title))
        .collect();
    assert_eq!(stored, [(id, "cycle title".to_owned())]);
}

#[test]
fn a_second_vault_cannot_be_created_over_the_first() {
    let workspace = Workspace::new();
    drop(workspace.created_vault());
    let mut second = workspace.vault();
    assert_eq!(second.create(PASSWORD).unwrap_err(), AppError::VaultExists);
    assert!(!second.is_unlocked());
}

/// Unlocking where no vault exists reports it and does not create the folder.
#[test]
fn opening_a_missing_vault_is_reported() {
    let workspace = Workspace::new();
    let mut vault = workspace.vault();
    assert_eq!(vault.unlock(PASSWORD).unwrap_err(), AppError::VaultNotFound);
    assert!(!workspace.vault_dir().exists());
}

#[test]
fn locked_vaults_refuse_every_operation() {
    let workspace = Workspace::new();
    let mut vault = workspace.created_vault();
    vault.lock();
    assert_eq!(vault.unlocked().err(), Some(AppError::Locked));
    assert_eq!(
        vault
            .change_master_password(PASSWORD, NEW_PASSWORD)
            .unwrap_err(),
        AppError::Locked
    );
}
