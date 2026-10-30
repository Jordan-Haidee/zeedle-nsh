//! Keep the native window's content area at or above the Slint minimum.

pub(crate) fn set_minimum_size(width: f32, height: f32) {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return;
    }

    #[cfg(windows)]
    windows::set_minimum_size(width, height);
    #[cfg(target_os = "macos")]
    macos::set_minimum_size(width, height);
    #[cfg(target_os = "linux")]
    linux::set_minimum_size(width, height);
}

pub(crate) fn configure(width: f32, height: f32, min_width: f32, min_height: f32) {
    if ![width, height, min_width, min_height]
        .into_iter()
        .all(|value| value.is_finite() && value > 0.0)
    {
        return;
    }

    #[cfg(windows)]
    windows::configure(width, height, min_width, min_height);
    #[cfg(target_os = "macos")]
    macos::configure(width, height, min_width, min_height);
    #[cfg(target_os = "linux")]
    linux::configure(width, height, min_width, min_height);
}

#[cfg(windows)]
mod windows {
    use std::sync::{
        Mutex, OnceLock,
        atomic::{AtomicIsize, AtomicU32, Ordering},
    };

    use windows::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        System::Threading::GetCurrentProcessId,
        UI::{
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{
                CallWindowProcW, DefWindowProcW, EnumWindows, GWLP_WNDPROC, GetClassNameW,
                GetClientRect, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId,
                MINMAXINFO, PostMessageW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER,
                SetWindowLongPtrW, SetWindowPos, WM_APP, WM_GETMINMAXINFO, WNDPROC,
            },
        },
    };
    use windows::core::BOOL;

    static WINDOW: AtomicIsize = AtomicIsize::new(0);
    static PREVIOUS_PROC: AtomicIsize = AtomicIsize::new(0);
    static MINIMUM_WIDTH: AtomicU32 = AtomicU32::new(850);
    static MINIMUM_HEIGHT: AtomicU32 = AtomicU32::new(500);
    static INITIAL_WIDTH: AtomicU32 = AtomicU32::new(900);
    static INITIAL_HEIGHT: AtomicU32 = AtomicU32::new(510);
    static INSTALL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    const CONFIGURE_WINDOW: u32 = WM_APP + 418;

    struct WindowSearch {
        process_id: u32,
        window: HWND,
    }

    pub(super) fn set_minimum_size(min_width: f32, min_height: f32) {
        MINIMUM_WIDTH.store(min_width.ceil() as u32, Ordering::Release);
        MINIMUM_HEIGHT.store(min_height.ceil() as u32, Ordering::Release);
        let _ = attach_window();
    }

    pub(super) fn configure(width: f32, height: f32, min_width: f32, min_height: f32) {
        INITIAL_WIDTH.store(width.ceil() as u32, Ordering::Release);
        INITIAL_HEIGHT.store(height.ceil() as u32, Ordering::Release);
        set_minimum_size(min_width, min_height);
        let Some(hwnd) = current_window() else {
            return;
        };
        // SAFETY: hwnd is the live top-level window we subclassed.
        unsafe {
            let _ = PostMessageW(Some(hwnd), CONFIGURE_WINDOW, WPARAM(0), LPARAM(0));
        }
    }

    fn current_window() -> Option<HWND> {
        let hwnd = WINDOW.load(Ordering::Acquire);
        (hwnd != 0).then_some(HWND(hwnd as *mut _))
    }

    fn attach_window() -> Option<HWND> {
        if WINDOW.load(Ordering::Acquire) != 0 {
            return current_window();
        }
        let _guard = INSTALL_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        if WINDOW.load(Ordering::Acquire) != 0 {
            return current_window();
        }

        let mut search = WindowSearch {
            // SAFETY: this only reads the calling process identifier.
            process_id: unsafe { GetCurrentProcessId() },
            window: HWND(std::ptr::null_mut()),
        };
        // SAFETY: the callback only reads and writes the stack context for the duration of EnumWindows.
        let _ = unsafe {
            EnumWindows(
                Some(find_app_window),
                LPARAM((&mut search as *mut WindowSearch) as isize),
            )
        };
        if search.window.0.is_null() {
            return None;
        }

        // SAFETY: this is the existing WndProc for the app's NSH window.
        let previous = unsafe { GetWindowLongPtrW(search.window, GWLP_WNDPROC) };
        if previous == 0 {
            return None;
        }
        PREVIOUS_PROC.store(previous, Ordering::Release);
        // SAFETY: the forwarding WndProc preserves NSH's existing handler and only adjusts one message.
        unsafe {
            SetWindowLongPtrW(
                search.window,
                GWLP_WNDPROC,
                minimum_size_proc as *const () as isize,
            );
        }
        WINDOW.store(search.window.0 as isize, Ordering::Release);
        Some(search.window)
    }

    unsafe extern "system" fn find_app_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: EnumWindows passes the pointer supplied above unchanged.
        let search = unsafe { &mut *(lparam.0 as *mut WindowSearch) };
        let mut owner = 0;
        // SAFETY: hwnd is supplied by EnumWindows.
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut owner)) };
        if owner != search.process_id {
            return BOOL(1);
        }

        let mut class_name = [0u16; 64];
        // SAFETY: class_name is writable for its full declared length.
        let class_length = unsafe { GetClassNameW(hwnd, &mut class_name) };
        if class_length <= 0
            || String::from_utf16_lossy(&class_name[..class_length as usize]) != "NshWindow"
        {
            return BOOL(1);
        }
        search.window = hwnd;
        BOOL(0)
    }

    unsafe extern "system" fn minimum_size_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == CONFIGURE_WINDOW {
            let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
            let (width, height) = outer_size(
                hwnd,
                INITIAL_WIDTH.load(Ordering::Acquire),
                INITIAL_HEIGHT.load(Ordering::Acquire),
                dpi,
            );
            // SAFETY: this message is handled on the thread that owns hwnd.
            let _ = unsafe {
                SetWindowPos(
                    hwnd,
                    None,
                    0,
                    0,
                    width,
                    height,
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                )
            };
            return LRESULT(0);
        }

        let previous = PREVIOUS_PROC.load(Ordering::Acquire);
        if previous == 0 {
            // SAFETY: this is the platform window's registered window procedure.
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        }
        // SAFETY: previous is the WndProc read from this HWND before installation.
        let previous: WNDPROC = unsafe { std::mem::transmute(previous) };
        // SAFETY: this forwards every event to the original NSH window procedure.
        let result = unsafe { CallWindowProcW(previous, hwnd, message, wparam, lparam) };

        if message == WM_GETMINMAXINFO {
            // SAFETY: Windows supplies a MINMAXINFO pointer for WM_GETMINMAXINFO.
            let info = unsafe { &mut *(lparam.0 as *mut MINMAXINFO) };
            // The NSH window receives a 96-DPI content size; Windows expects the
            // tracking size of the full decorated window in device pixels.
            let (width, height) = outer_size(
                hwnd,
                MINIMUM_WIDTH.load(Ordering::Acquire),
                MINIMUM_HEIGHT.load(Ordering::Acquire),
                unsafe { GetDpiForWindow(hwnd) }.max(96),
            );
            info.ptMinTrackSize = POINT {
                x: width,
                y: height,
            };
        }
        result
    }

    fn outer_size(hwnd: HWND, width: u32, height: u32, dpi: u32) -> (i32, i32) {
        let scale = dpi as f32 / 96.0;
        let mut outer = RECT::default();
        let mut client = RECT::default();
        // SAFETY: these queries read the current frame extents of hwnd.
        if unsafe { GetWindowRect(hwnd, &mut outer) }.is_ok()
            && unsafe { GetClientRect(hwnd, &mut client) }.is_ok()
        {
            let frame_width = (outer.right - outer.left) - (client.right - client.left);
            let frame_height = (outer.bottom - outer.top) - (client.bottom - client.top);
            return (
                (width as f32 * scale).round() as i32 + frame_width,
                (height as f32 * scale).round() as i32 + frame_height,
            );
        }
        (
            (width as f32 * scale).round() as i32,
            (height as f32 * scale).round() as i32,
        )
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use dispatch2::DispatchQueue;
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    use objc2_foundation::NSSize;

    pub(super) fn set_minimum_size(min_width: f32, min_height: f32) {
        DispatchQueue::main().exec_async(move || {
            let Some(main_thread) = MainThreadMarker::new() else {
                return;
            };
            let application = NSApplication::sharedApplication(main_thread);
            for window in application.windows().iter() {
                if window.title().to_string() == "Zeedle" {
                    window.setContentMinSize(NSSize::new(min_width as f64, min_height as f64));
                }
            }
        });
    }

    pub(super) fn configure(width: f32, height: f32, min_width: f32, min_height: f32) {
        DispatchQueue::main().exec_async(move || {
            let Some(main_thread) = MainThreadMarker::new() else {
                return;
            };
            let application = NSApplication::sharedApplication(main_thread);
            for window in application.windows().iter() {
                if window.title().to_string() == "Zeedle" {
                    window.setContentMinSize(NSSize::new(min_width as f64, min_height as f64));
                    window.setContentSize(NSSize::new(width as f64, height as f64));
                }
            }
        });
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use gtk4::prelude::*;
    use nsh_platform_linux::gtk4;

    pub(super) fn set_minimum_size(min_width: f32, min_height: f32) {
        gtk4::glib::MainContext::default().invoke(move || {
            for widget in gtk4::Window::list_toplevels() {
                if let Ok(window) = widget.downcast::<gtk4::Window>()
                    && window.title().as_deref() == Some("Zeedle")
                {
                    window.set_size_request(min_width.ceil() as i32, min_height.ceil() as i32);
                }
            }
        });
    }

    pub(super) fn configure(width: f32, height: f32, min_width: f32, min_height: f32) {
        gtk4::glib::MainContext::default().invoke(move || {
            for widget in gtk4::Window::list_toplevels() {
                if let Ok(window) = widget.downcast::<gtk4::Window>()
                    && window.title().as_deref() == Some("Zeedle")
                {
                    window.set_size_request(min_width.ceil() as i32, min_height.ceil() as i32);
                    window.set_default_size(width.ceil() as i32, height.ceil() as i32);
                }
            }
        });
    }
}
