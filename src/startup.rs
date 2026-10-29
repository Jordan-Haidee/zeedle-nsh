//! Notify the packaged launcher after initial library and UI initialization.
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::PostMessageW,
};
static WINDOW: AtomicIsize = AtomicIsize::new(0);

pub fn set_window(handle: isize) {
    WINDOW.store(handle, Ordering::Release);
}

pub fn ready() {
    let hwnd = HWND(WINDOW.load(Ordering::Acquire) as *mut _);
    if !hwnd.0.is_null() {
        // The launcher's window procedure owns presentation on its UI thread.
        unsafe {
            let _ = PostMessageW(Some(hwnd), 0x8000 + 417, WPARAM(0), LPARAM(0));
        }
    }
}
