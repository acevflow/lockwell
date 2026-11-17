//! Commands to create and restore encrypted backups. The core shows the
//! native save and open dialogs.

use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use zeroize::Zeroizing;

use super::dialog::{run_blocking, to_path};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::vault::backup::BACKUP_EXTENSION;
use crate::vault::now_millis;
use crate::vault::password_policy::MAX_MASTER_PASSWORD_CHARS;

/// Asks where to save a backup and writes it. Returns `false` if the user
/// cancelled.
#[tauri::command]
pub async fn create_backup<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> AppResult<bool> {
    state.with_vault(|vault| vault.unlocked().map(|_| ()))?;
    let file_name = backup_file_name(now_millis());
    let chosen = run_blocking(move || {
        app.dialog()
            .file()
            .set_title("Save an encrypted backup")
            .set_file_name(file_name)
            .add_filter("lockwell backup", &[BACKUP_EXTENSION])
            .blocking_save_file()
    })
    .await?;
    let Some(destination) = chosen else {
        return Ok(false);
    };
    let destination = to_path(destination)?;
    state
        .with_vault_blocking(move |vault| {
            vault.unlocked()?.create_backup(&destination).map(|()| true)
        })
        .await
}

/// Asks for a backup file and replaces the vault with it. `password` is the
/// master password the vault had when the backup was made. The vault is left
/// locked. Returns `false` if the user cancelled.
#[tauri::command]
pub async fn restore_backup<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    password: String,
) -> AppResult<bool> {
    let password = Zeroizing::new(password);
    if password.is_empty() || password.chars().count() > MAX_MASTER_PASSWORD_CHARS {
        return Err(AppError::invalid("enter the master password of the backup"));
    }
    let chosen = run_blocking(move || {
        app.dialog()
            .file()
            .set_title("Choose a backup to restore")
            .add_filter("lockwell backup", &[BACKUP_EXTENSION])
            .blocking_pick_file()
    })
    .await?;
    let Some(source) = chosen else {
        return Ok(false);
    };
    let source = to_path(source)?;
    state
        .with_vault_blocking(move |vault| vault.restore_backup(&source, &password))
        .await?;
    // Also clears a secret left on the clipboard by the replaced vault.
    state.lock();
    Ok(true)
}

const MILLIS_PER_DAY: i64 = 86_400_000;

/// Default backup file name for the date of `millis`, such as
/// `lockwell-backup-2026-11-16.lwbackup`.
fn backup_file_name(millis: i64) -> String {
    let (year, month, day) = civil_date(millis.div_euclid(MILLIS_PER_DAY));
    format!("lockwell-backup-{year:04}-{month:02}-{day:02}.{BACKUP_EXTENSION}")
}

/// Converts days since 1970-01-01 to a (year, month, day) UTC date, using
/// Howard Hinnant's `civil_from_days` algorithm.
fn civil_date(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    // Both values are small and positive by construction.
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_known_dates() {
        assert_eq!(civil_date(0), (1970, 1, 1));
        assert_eq!(civil_date(59), (1970, 3, 1));
        assert_eq!(civil_date(10_957), (2000, 1, 1));
        assert_eq!(civil_date(11_016), (2000, 2, 29));
        assert_eq!(civil_date(20_773), (2026, 11, 16));
        assert_eq!(civil_date(-1), (1969, 12, 31));
    }

    #[test]
    fn names_backups_by_date() {
        // 2026-11-16 12:00 UTC.
        assert_eq!(
            backup_file_name(1_794_830_400_000),
            "lockwell-backup-2026-11-16.lwbackup"
        );
        assert_eq!(backup_file_name(0), "lockwell-backup-1970-01-01.lwbackup");
    }
}
