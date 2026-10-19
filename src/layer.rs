use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows_sys::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, GetWindowLongW, SendMessageTimeoutW,
    SetParent, SetWindowLongW, GWL_EXSTYLE, SMTO_NORMAL, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT,
};

fn encode_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

struct WorkerWInfo {
    workerw: HWND,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut class_name = [0u16; 256];
    let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
    if len > 0 {
        let name = String::from_utf16_lossy(&class_name[..len as usize]);
        if name == "WorkerW" {
            // Find SHELLDLL_DefView
            let def_view = FindWindowExW(
                hwnd,
                0,
                encode_wide("SHELLDLL_DefView").as_ptr(),
                std::ptr::null(),
            );
            if def_view != 0 {
                // The correct WorkerW is the next sibling of the one with SHELLDLL_DefView
                let target =
                    FindWindowExW(0, hwnd, encode_wide("WorkerW").as_ptr(), std::ptr::null());
                if target != 0 {
                    let info = &mut *(lparam as *mut WorkerWInfo);
                    info.workerw = target;
                    return 0; // stop enumerating
                }
            }
        }
    }
    1
}

pub fn attach_to_desktop(hwnd: isize) {
    let hwnd = hwnd as HWND;
    unsafe {
        // Hide from Alt-Tab and taskbar
        let mut ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        ex_style &= !WS_EX_APPWINDOW as i32;
        ex_style |= WS_EX_TOOLWINDOW as i32 | WS_EX_TRANSPARENT as i32;
        SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);

        let progman = FindWindowW(encode_wide("Progman").as_ptr(), std::ptr::null());

        // Windows 11 modern taskbar check vs classic Progman
        // Send 0x052C to spawn WorkerW
        SendMessageTimeoutW(
            progman,
            0x052C,
            0,
            0,
            SMTO_NORMAL,
            1000,
            std::ptr::null_mut(),
        );

        let mut info = WorkerWInfo { workerw: 0 };
        EnumWindows(Some(enum_windows_proc), &mut info as *mut _ as LPARAM);

        if info.workerw != 0 {
            crate::log::info("Attached to WorkerW (Windows 10/11 classic layout)");
            SetParent(hwnd, info.workerw);
        } else {
            crate::log::info("Attached to Progman (Fallback)");
            SetParent(hwnd, progman);
        }
    }
}

pub fn set_ghost_mode(hwnd: isize, ghost: bool) {
    let hwnd = hwnd as HWND;
    unsafe {
        let mut ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if ghost {
            ex_style |= WS_EX_TRANSPARENT as i32;
        } else {
            ex_style &= !WS_EX_TRANSPARENT as i32;
        }
        SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);
    }
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct MARGINS {
    pub cxLeftWidth: i32,
    pub cxRightWidth: i32,
    pub cyTopHeight: i32,
    pub cyBottomHeight: i32,
}

pub fn enable_drop_shadow(hwnd: isize) {
    let hwnd = hwnd as HWND;
    unsafe {
        // The classic trick to enable native drop shadows on borderless windows
        // is to extend the frame into the client area by 1 pixel.
        let margins = MARGINS {
            cxLeftWidth: 0,
            cxRightWidth: 1,
            cyTopHeight: 0,
            cyBottomHeight: 0,
        };
        DwmExtendFrameIntoClientArea(hwnd, std::mem::transmute(&margins));
    }
}
