//! Copying secrets to the clipboard and clearing them afterwards.
//!
//! A copied value is cleared after [`CLEAR_AFTER`] or when the vault locks,
//! but only if the clipboard still holds it: anything the user copied since
//! then is left alone. To compare without keeping the secret in memory, only
//! its SHA-256 digest is kept.
//!
//! On Windows the text is excluded from the clipboard history (Win+V) and
//! from cloud clipboard sync.

use std::time::Duration;

use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{AppError, AppResult};

/// Time after which a copied secret is cleared from the clipboard.
pub const CLEAR_AFTER: Duration = Duration::from_secs(30);

/// Access to a text clipboard.
pub trait ClipboardBackend: Send {
    /// Puts `text` on the clipboard.
    fn set_text(&mut self, text: &str) -> Result<(), String>;
    /// Reads the clipboard text. Fails if it holds no text.
    fn get_text(&mut self) -> Result<String, String>;
    /// Empties the clipboard.
    fn clear(&mut self) -> Result<(), String>;
}

/// The operating system clipboard.
pub struct SystemClipboard;

impl ClipboardBackend for SystemClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), String> {
        let mut clipboard = open()?;
        let setter = clipboard.set();
        #[cfg(windows)]
        let setter = {
            use arboard::SetExtWindows;
            setter.exclude_from_monitoring()
        };
        setter.text(text).map_err(|error| error.to_string())
    }

    fn get_text(&mut self) -> Result<String, String> {
        open()?.get_text().map_err(|error| error.to_string())
    }

    fn clear(&mut self) -> Result<(), String> {
        open()?.clear().map_err(|error| error.to_string())
    }
}

fn open() -> Result<arboard::Clipboard, String> {
    arboard::Clipboard::new().map_err(|error| error.to_string())
}

/// What was copied last: its digest and a counter that identifies the copy.
struct Copied {
    digest: [u8; 32],
    generation: u64,
}

/// Clipboard that remembers the secret it copied, to clear it later.
pub struct SecureClipboard {
    backend: Box<dyn ClipboardBackend>,
    copied: Option<Copied>,
    generation: u64,
}

impl SecureClipboard {
    /// A clipboard over `backend`.
    pub fn new(backend: Box<dyn ClipboardBackend>) -> SecureClipboard {
        SecureClipboard {
            backend,
            copied: None,
            generation: 0,
        }
    }

    /// Copies `text` and returns the generation to pass to
    /// [`SecureClipboard::clear_if_unchanged`] when the timer expires.
    ///
    /// # Errors
    /// Returns [`AppError::Io`] if the clipboard cannot be written.
    pub fn copy(&mut self, text: &str) -> AppResult<u64> {
        self.backend
            .set_text(text)
            .map_err(|_| AppError::Io("the clipboard is not available".into()))?;
        self.generation += 1;
        self.copied = Some(Copied {
            digest: digest(text),
            generation: self.generation,
        });
        Ok(self.generation)
    }

    /// Clears the clipboard if it still holds the copied secret.
    ///
    /// With `Some(generation)`, only that copy is cleared: a timer from an
    /// older copy does nothing once a newer one exists. With `None` (when
    /// the vault locks), any copied secret is cleared.
    ///
    /// Returns whether the clipboard was cleared.
    pub fn clear_if_unchanged(&mut self, generation: Option<u64>) -> bool {
        let Some(copied) = &self.copied else {
            return false;
        };
        if generation.is_some_and(|expected| expected != copied.generation) {
            return false;
        }
        let expected = copied.digest;
        self.copied = None;
        let current = match self.backend.get_text() {
            Ok(text) => Zeroizing::new(text),
            Err(_) => return false,
        };
        digest(&current) == expected && self.backend.clear().is_ok()
    }
}

fn digest(text: &str) -> [u8; 32] {
    Sha256::digest(text.as_bytes()).into()
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    /// In-memory clipboard shared with the test, to inspect its content.
    #[derive(Clone, Default)]
    pub(crate) struct FakeClipboard {
        pub(crate) content: Arc<Mutex<Option<String>>>,
        pub(crate) fail: bool,
    }

    impl ClipboardBackend for FakeClipboard {
        fn set_text(&mut self, text: &str) -> Result<(), String> {
            if self.fail {
                return Err("unavailable".into());
            }
            *self.content.lock().unwrap() = Some(text.to_owned());
            Ok(())
        }

        fn get_text(&mut self) -> Result<String, String> {
            self.content
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| "empty".into())
        }

        fn clear(&mut self) -> Result<(), String> {
            *self.content.lock().unwrap() = None;
            Ok(())
        }
    }

    fn clipboard() -> (FakeClipboard, SecureClipboard) {
        let fake = FakeClipboard::default();
        (fake.clone(), SecureClipboard::new(Box::new(fake)))
    }

    #[test]
    fn copies_text() {
        let (fake, mut clipboard) = clipboard();
        clipboard.copy("example-secret").unwrap();
        assert_eq!(
            fake.content.lock().unwrap().as_deref(),
            Some("example-secret")
        );
    }

    #[test]
    fn clears_its_own_copy() {
        let (fake, mut clipboard) = clipboard();
        let generation = clipboard.copy("example-secret").unwrap();
        assert!(clipboard.clear_if_unchanged(Some(generation)));
        assert_eq!(*fake.content.lock().unwrap(), None);
    }

    #[test]
    fn leaves_content_the_user_copied_since() {
        let (fake, mut clipboard) = clipboard();
        let generation = clipboard.copy("example-secret").unwrap();
        *fake.content.lock().unwrap() = Some("something else".into());
        assert!(!clipboard.clear_if_unchanged(Some(generation)));
        assert_eq!(
            fake.content.lock().unwrap().as_deref(),
            Some("something else")
        );
    }

    #[test]
    fn an_old_timer_does_not_clear_a_newer_copy() {
        let (fake, mut clipboard) = clipboard();
        let first = clipboard.copy("first").unwrap();
        let second = clipboard.copy("second").unwrap();
        assert!(!clipboard.clear_if_unchanged(Some(first)));
        assert_eq!(fake.content.lock().unwrap().as_deref(), Some("second"));
        assert!(clipboard.clear_if_unchanged(Some(second)));
    }

    #[test]
    fn lock_clears_any_copy() {
        let (fake, mut clipboard) = clipboard();
        clipboard.copy("example-secret").unwrap();
        assert!(clipboard.clear_if_unchanged(None));
        assert_eq!(*fake.content.lock().unwrap(), None);
    }

    #[test]
    fn nothing_to_clear_without_a_copy() {
        let (fake, mut clipboard) = clipboard();
        *fake.content.lock().unwrap() = Some("user text".into());
        assert!(!clipboard.clear_if_unchanged(None));
        assert_eq!(fake.content.lock().unwrap().as_deref(), Some("user text"));
    }

    #[test]
    fn clears_only_once() {
        let (fake, mut clipboard) = clipboard();
        let generation = clipboard.copy("same").unwrap();
        assert!(clipboard.clear_if_unchanged(Some(generation)));
        *fake.content.lock().unwrap() = Some("same".into());
        assert!(!clipboard.clear_if_unchanged(Some(generation)));
    }

    #[test]
    fn non_text_content_is_left_alone() {
        let (fake, mut clipboard) = clipboard();
        clipboard.copy("example-secret").unwrap();
        *fake.content.lock().unwrap() = None;
        assert!(!clipboard.clear_if_unchanged(None));
    }

    #[test]
    fn reports_an_unavailable_clipboard() {
        let fake = FakeClipboard {
            fail: true,
            ..FakeClipboard::default()
        };
        let mut clipboard = SecureClipboard::new(Box::new(fake));
        assert!(matches!(clipboard.copy("x"), Err(AppError::Io(_))));
        assert!(!clipboard.clear_if_unchanged(None));
    }
}
