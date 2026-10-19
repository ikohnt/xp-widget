use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, WAIT_OBJECT_0,
};
use windows_sys::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForMultipleObjects, EVENT_MODIFY_STATE,
    INFINITE,
};

use crate::events::{self, AppEvent};

const MUTEX_NAME: &str = "xp_widget_mutex";
const RELOAD_NAME: &str = "xp_widget_reload";
const QUIT_NAME: &str = "xp_widget_quit";
const SHOW_NAME: &str = "xp_widget_show";
const ADD_NAME: &str = "xp_widget_add";

fn encode_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub struct IpcServer {
    mutex_handle: HANDLE,
    reload_handle: HANDLE,
    quit_handle: HANDLE,
    show_handle: HANDLE,
    add_handle: HANDLE,
}

impl IpcServer {
    /// Returns `None` if another instance already owns the named mutex.
    pub fn new() -> Option<Self> {
        unsafe {
            let mutex_name = encode_wide(MUTEX_NAME);
            let mutex = CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr());
            if mutex == 0 || GetLastError() == ERROR_ALREADY_EXISTS {
                if mutex != 0 {
                    CloseHandle(mutex);
                }
                return None;
            }

            let create = |name: &str| {
                let wide = encode_wide(name);
                CreateEventW(std::ptr::null(), 0, 0, wide.as_ptr())
            };

            Some(Self {
                mutex_handle: mutex,
                reload_handle: create(RELOAD_NAME),
                quit_handle: create(QUIT_NAME),
                show_handle: create(SHOW_NAME),
                add_handle: create(ADD_NAME),
            })
        }
    }

    pub fn start_listener(self) {
        std::thread::spawn(move || {
            let handles = [
                self.reload_handle,
                self.quit_handle,
                self.show_handle,
                self.add_handle,
            ];
            loop {
                let wait_res = unsafe {
                    WaitForMultipleObjects(handles.len() as u32, handles.as_ptr(), 0, INFINITE)
                };
                match wait_res {
                    WAIT_OBJECT_0 => events::send(AppEvent::Reload),
                    x if x == WAIT_OBJECT_0 + 1 => {
                        events::send(AppEvent::Quit);
                        break; // Exit thread
                    }
                    x if x == WAIT_OBJECT_0 + 2 => events::send(AppEvent::ShowAllWidgets),
                    x if x == WAIT_OBJECT_0 + 3 => events::send(AppEvent::AddWidget),
                    _ => break, // Error or unexpected
                }
            }
            unsafe {
                for h in handles {
                    CloseHandle(h);
                }
                CloseHandle(self.mutex_handle);
            }
        });
    }
}

/// Signals a named event owned by the running instance. Returns false if no
/// instance is running.
fn signal(name: &str) -> bool {
    unsafe {
        let wide = encode_wide(name);
        let handle = OpenEventW(EVENT_MODIFY_STATE, 0, wide.as_ptr());
        if handle != 0 {
            SetEvent(handle);
            CloseHandle(handle);
            true
        } else {
            false
        }
    }
}

pub fn send_reload() -> bool {
    signal(RELOAD_NAME)
}

pub fn send_quit() -> bool {
    signal(QUIT_NAME)
}

/// Asks the running instance to open its settings window.
pub fn send_show() -> bool {
    signal(SHOW_NAME)
}

pub fn send_add() -> bool {
    signal(ADD_NAME)
}
