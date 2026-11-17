//! Fixtures shared by the integration tests.
//!
//! Every test works in its own temporary folder, with small Argon2id
//! parameters so it runs quickly, and with synthetic data only.

// Each test crate uses a different subset of these helpers.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use lockwell_lib::crypto::kdf::KdfParams;
use lockwell_lib::security::clipboard::ClipboardBackend;
use lockwell_lib::vault::Vault;
use lockwell_lib::vault::entries::EntryInput;

/// Master password of the test vaults.
pub const PASSWORD: &str = "synthetic master password";
/// Another valid master password.
pub const NEW_PASSWORD: &str = "another synthetic password";

/// Argon2id parameters small enough for fast tests.
pub const FAST_KDF: KdfParams = KdfParams {
    memory_kib: 64,
    iterations: 1,
    parallelism: 1,
};

/// A temporary workspace: a folder for the vault and one for user files.
pub struct Workspace {
    root: tempfile::TempDir,
}

impl Workspace {
    /// A new workspace with an empty `outside` folder.
    pub fn new() -> Workspace {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("outside")).unwrap();
        Workspace { root }
    }

    /// The folder of the vault (created by the vault itself).
    pub fn vault_dir(&self) -> PathBuf {
        self.root.path().join("vault")
    }

    /// A folder outside the vault for imported and exported files.
    pub fn outside(&self) -> PathBuf {
        self.root.path().join("outside")
    }

    /// A vault over this workspace, not created yet.
    pub fn vault(&self) -> Vault {
        Vault::with_kdf(self.vault_dir(), FAST_KDF)
    }

    /// A new vault, created and unlocked.
    pub fn created_vault(&self) -> Vault {
        let mut vault = self.vault();
        vault.create(PASSWORD).unwrap();
        vault
    }

    /// Writes a user file outside the vault and returns its path.
    pub fn user_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.outside().join(name);
        fs::write(&path, content).unwrap();
        path
    }
}

/// An entry whose fields are all marked with `marker`.
pub fn entry(marker: &str) -> EntryInput {
    EntryInput {
        title: format!("{marker} title"),
        username: format!("{marker}-user@example.test"),
        password: format!("{marker}-password"),
        url: format!("https://{marker}.example.test"),
        notes: format!("{marker} notes"),
    }
}

/// Every file under `dir`, recursively.
pub fn files_under(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap()
        .map(|item| item.unwrap().path())
        .flat_map(|path| {
            if path.is_dir() {
                files_under(&path)
            } else {
                vec![path]
            }
        })
        .collect()
}

/// Whether any file under `dir` contains `needle`.
pub fn any_file_contains(dir: &Path, needle: &[u8]) -> bool {
    files_under(dir).iter().any(|path| {
        let bytes = fs::read(path).unwrap();
        bytes.windows(needle.len()).any(|window| window == needle)
    })
}

/// Names of the items in `dir`, sorted.
pub fn names_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|item| item.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

/// In-memory clipboard shared with the test.
#[derive(Clone, Default)]
pub struct MemoryClipboard {
    content: Arc<Mutex<Option<String>>>,
}

impl MemoryClipboard {
    /// The current clipboard text, if any.
    pub fn text(&self) -> Option<String> {
        self.content.lock().unwrap().clone()
    }

    /// Replaces the content, as another program copying text would.
    pub fn set(&self, text: &str) {
        *self.content.lock().unwrap() = Some(text.to_owned());
    }
}

impl ClipboardBackend for MemoryClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), String> {
        self.set(text);
        Ok(())
    }

    fn get_text(&mut self) -> Result<String, String> {
        self.text().ok_or_else(|| "empty".into())
    }

    fn clear(&mut self) -> Result<(), String> {
        *self.content.lock().unwrap() = None;
        Ok(())
    }
}
