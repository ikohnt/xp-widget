//! Watches config.toml and notifies the UI thread when it changes.
//!
//! Uses FindFirstChangeNotificationW, so the thread sleeps in the kernel
//! until something in the config folder changes (no polling). The folder
//! also contains widget.log, so we compare config.toml's modification time
//! before sending anything.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use windows_sys::Win32::Foundation::{INVALID_HANDLE_VALUE, WAIT_OBJECT_0};
use windows_sys::Win32::Storage::FileSystem::{
    FindFirstChangeNotificationW, FindNextChangeNotification, FILE_NOTIFY_CHANGE_FILE_NAME,
    FILE_NOTIFY_CHANGE_LAST_WRITE,
};
use windows_sys::Win32::System::Threading::{WaitForSingleObject, INFINITE};

use crate::events::{self, AppEvent};

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

pub fn start(config_path: PathBuf) {
    let Some(dir) = config_path.parent().map(Path::to_path_buf) else {
        return;
    };
    std::thread::spawn(move || unsafe {
        let wide: Vec<u16> = OsStr::new(&dir)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let handle = FindFirstChangeNotificationW(
            wide.as_ptr(),
            0,
            FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_FILE_NAME,
        );
        if handle == INVALID_HANDLE_VALUE {
            crate::log::error(
                "Could not watch the config folder; use --reload after editing config.toml",
            );
            return;
        }

        let mut last = modified(&config_path);
        loop {
            if WaitForSingleObject(handle, INFINITE) != WAIT_OBJECT_0 {
                break;
            }
            // Editors often save in several steps; let them finish.
            std::thread::sleep(Duration::from_millis(250));
            let now = modified(&config_path);
            if now != last {
                last = now;
                events::send(AppEvent::ConfigFileChanged);
            }
            if FindNextChangeNotification(handle) == 0 {
                break;
            }
        }
    });
}
