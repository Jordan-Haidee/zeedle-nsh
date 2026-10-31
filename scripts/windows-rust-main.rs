#![cfg_attr(windows, windows_subsystem = "windows")]

use nsh_platform_windows::windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Dwm::{DWMWA_CLOAK, DwmFlush, DwmSetWindowAttribute},
    Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow, UpdateWindow,
    },
    System::Ole::{OleInitialize, OleUninitialize},
    UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
    UI::WindowsAndMessaging::{
        ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_LOADFROMFILE, LoadImageW, SendMessageW, WM_SETICON,
        *,
    },
};
use nsh_platform_windows::windows::core::PCWSTR;
use nsh_platform_windows::{AppOptions, AppView, AssetBase, Limits, Root};
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicIsize, Ordering};

static ORIGINAL: AtomicIsize = AtomicIsize::new(0);
const SHOW_READY: u32 = WM_APP + 417;

fn main() -> std::process::ExitCode {
    std::thread::Builder::new()
        .name("zeedle-ui".into())
        .stack_size(8 << 20)
        .spawn(run_ui)
        .ok()
        .and_then(|thread| thread.join().ok())
        .unwrap_or(std::process::ExitCode::FAILURE)
}

fn run_ui() -> std::process::ExitCode {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let _ = OleInitialize(None);
    }
    let result = run_window();
    unsafe {
        OleUninitialize();
    }
    if let Err(error) = result {
        nsh_platform_windows::fatal_error(&error);
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

fn run_window() -> Result<(), String> {
    let dir = std::env::current_exe()
        .map_err(|error| error.to_string())?
        .parent()
        .ok_or("No executable directory")?
        .join("nsh");
    nsh_platform_windows::set_ui_sets(vec![
        nsh_base::registry(),
        app_nsh_app_zeedle_nsh::registry(),
    ]);
    let view = AppView::new(
        None,
        AppOptions {
            title: "Zeedle".into(),
            app_id: "dev.zeedle.music-player".into(),
            size: (900, 510),
            asset_base: Some(AssetBase::Directory(dir.join("assets"))),
            limits: Limits::standalone(),
            ..Default::default()
        },
    )
    .map_err(|error| error.to_string())?;
    let hwnd = view.hwnd();
    unsafe {
        app_nsh_app_zeedle_nsh::set_startup_window(hwnd.0 as isize);
        let icon_path = dir.join("assets/assets/cover.ico");
        if icon_path.is_file() {
            let icon_path: Vec<u16> = icon_path.as_os_str().encode_wide().chain(Some(0)).collect();
            if let Ok(icon) = LoadImageW(
                None,
                PCWSTR(icon_path.as_ptr()),
                IMAGE_ICON,
                256,
                256,
                LR_LOADFROMFILE,
            ) {
                let _ = SendMessageW(
                    hwnd,
                    WM_SETICON,
                    Some(WPARAM(ICON_BIG as usize)),
                    Some(LPARAM(icon.0 as isize)),
                );
                let _ = SendMessageW(
                    hwnd,
                    WM_SETICON,
                    Some(WPARAM(ICON_SMALL as usize)),
                    Some(LPARAM(icon.0 as isize)),
                );
            }
        }
        ORIGINAL.store(
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, startup_proc as *const () as isize),
            Ordering::Release,
        );
    }
    let root = Root::of::<app_nsh_app_zeedle_nsh::ui::App>().tokens(
        nsh_base::Theme::from_json(include_str!("../../../theme.json"))
            .expect("nsh.json: theme")
            .into_tokens(),
    );
    view.load(root.into());

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    view.stop();
    Ok(())
}

unsafe extern "system" fn startup_proc(
    hwnd: HWND,
    message: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    unsafe {
        if message == SHOW_READY {
            let hidden: i32 = 1;
            let _ = DwmSetWindowAttribute(hwnd, DWMWA_CLOAK, &hidden as *const _ as _, 4);
            let mut monitor = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok()
                && GetMonitorInfoW(
                    MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST),
                    &mut monitor,
                )
                .as_bool()
            {
                let work = monitor.rcWork;
                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    work.left + (work.right - work.left - width) / 2,
                    work.top + (work.bottom - work.top - height) / 2,
                    width,
                    height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            let _ = UpdateWindow(hwnd);
            let _ = DwmFlush();
            let hidden: i32 = 0;
            let _ = DwmSetWindowAttribute(hwnd, DWMWA_CLOAK, &hidden as *const _ as _, 4);
            return LRESULT(0);
        }
        let previous: WNDPROC = std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
        CallWindowProcW(previous, hwnd, message, wp, lp)
    }
}
