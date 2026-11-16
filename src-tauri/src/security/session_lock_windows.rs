//! Detects when Windows is locked (Win+L, lock screen, user switch).
//!
//! A hidden message-only window registers for session notifications with
//! `WTSRegisterSessionNotification` and calls a callback on every
//! `WTS_SESSION_LOCK` event. It runs on its own thread with its own message
//! loop for the lifetime of the app.

use std::sync::OnceLock;
use std::sync::mpsc;
use std::thread;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::RemoteDesktop::{
    NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, HWND_MESSAGE, MSG,
    RegisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_WTSSESSION_CHANGE, WNDCLASSW,
    WTS_SESSION_LOCK,
};
use windows::core::w;

type Callback = Box<dyn Fn() + Send + Sync>;

/// The callback for lock events. A window procedure cannot capture state,
/// so it is stored once for the process.
static ON_SESSION_LOCK: OnceLock<Callback> = OnceLock::new();

/// Starts watching for Windows session locks and calls `on_lock` for each.
///
/// # Errors
/// Returns a description of the failure if the watcher was already started
/// or the notification window cannot be created or registered.
pub fn watch(on_lock: impl Fn() + Send + Sync + 'static) -> Result<(), String> {
    ON_SESSION_LOCK
        .set(Box::new(on_lock))
        .map_err(|_| "the session lock watcher is already running".to_string())?;

    let (ready, started) = mpsc::channel();
    thread::Builder::new()
        .name("session-lock-watcher".into())
        .spawn(move || match create_notification_window() {
            Ok(_window) => {
                let _ = ready.send(Ok(()));
                run_message_loop();
            }
            Err(error) => {
                let _ = ready.send(Err(error));
            }
        })
        .map_err(|error| error.to_string())?;

    started
        .recv()
        .map_err(|_| "the session lock watcher stopped unexpectedly".to_string())?
}

fn create_notification_window() -> Result<HWND, String> {
    // SAFETY: plain Win32 calls with valid, static arguments; the window
    // class name and procedure live for the whole program.
    unsafe {
        let instance = GetModuleHandleW(None).map_err(|error| error.to_string())?;
        let class_name = w!("lockwell-session-lock-watcher");
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_procedure),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err("cannot register the notification window class".into());
        }
        let window = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("lockwell"),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance.into()),
            None,
        )
        .map_err(|error| error.to_string())?;
        WTSRegisterSessionNotification(window, NOTIFY_FOR_THIS_SESSION)
            .map_err(|error| error.to_string())?;
        Ok(window)
    }
}

fn run_message_loop() {
    let mut message = MSG::default();
    // SAFETY: `message` is a valid MSG owned by this thread; GetMessageW
    // returns 0 on WM_QUIT and -1 on error, both of which end the loop.
    unsafe {
        while GetMessageW(&mut message, None, 0, 0).0 > 0 {
            DispatchMessageW(&message);
        }
    }
}

extern "system" fn window_procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if is_session_lock(message, wparam) {
        if let Some(on_lock) = ON_SESSION_LOCK.get() {
            on_lock();
        }
        return LRESULT(0);
    }
    // SAFETY: forwards the unmodified message to the default procedure.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

/// Whether a window message reports that the session was locked.
fn is_session_lock(message: u32, wparam: WPARAM) -> bool {
    message == WM_WTSSESSION_CHANGE && wparam.0 == WTS_SESSION_LOCK as usize
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::WindowsAndMessaging::{WM_CLOSE, WTS_SESSION_UNLOCK};

    use super::*;

    #[test]
    fn recognizes_only_session_lock_messages() {
        assert!(is_session_lock(
            WM_WTSSESSION_CHANGE,
            WPARAM(WTS_SESSION_LOCK as usize)
        ));
        assert!(!is_session_lock(
            WM_WTSSESSION_CHANGE,
            WPARAM(WTS_SESSION_UNLOCK as usize)
        ));
        assert!(!is_session_lock(
            WM_CLOSE,
            WPARAM(WTS_SESSION_LOCK as usize)
        ));
    }
}
