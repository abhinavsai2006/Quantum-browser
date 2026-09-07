//! Qualium Quantum Browser v5 — Pure Native Win32 Wizard GUI
//! Ultra-premium dark design, zero WinForms, zero .NET runtime dependencies.
//! Direct Win32 implementation with crisp typography, high-DPI awareness,
//! dynamic disk space metrics, and responsive interactive cards.
//!
//! This module is Windows-only and must not be compiled on Linux or macOS.
#![cfg(windows)]

#![allow(non_snake_case, static_mut_refs, dead_code)]

use crate::engine::{InstallEngine, InstallOptions, PayloadMetrics};
use crate::uninstaller_engine::UninstallerEngine;
use crate::win32;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

pub mod ffi {
    pub type HWND = *mut std::ffi::c_void;
    pub type HDC = *mut std::ffi::c_void;
    pub type HBRUSH = *mut std::ffi::c_void;
    pub type HFONT = *mut std::ffi::c_void;
    pub type HMENU = *mut std::ffi::c_void;
    pub type HINSTANCE = *mut std::ffi::c_void;
    pub type HICON = *mut std::ffi::c_void;
    pub type HCURSOR = *mut std::ffi::c_void;
    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type UINT = u32;
    pub type WPARAM = usize;
    pub type LPARAM = isize;
    pub type LRESULT = isize;
    pub type LPCWSTR = *const u16;
    pub type LPWSTR = *mut u16;

    pub type WNDPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

    #[repr(C)]
    pub struct WNDCLASSEXW {
        pub cbSize: UINT,
        pub style: UINT,
        pub lpfnWndProc: WNDPROC,
        pub cbClsExtra: i32,
        pub cbWndExtra: i32,
        pub hInstance: HINSTANCE,
        pub hIcon: HICON,
        pub hCursor: HCURSOR,
        pub hbrBackground: HBRUSH,
        pub lpszMenuName: LPCWSTR,
        pub lpszClassName: LPCWSTR,
        pub hIconSm: HICON,
    }

    #[repr(C)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    pub struct MSG {
        pub hwnd: HWND,
        pub message: UINT,
        pub wParam: WPARAM,
        pub lParam: LPARAM,
        pub time: DWORD,
        pub pt: POINT,
    }

    #[repr(C)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    pub struct PAINTSTRUCT {
        pub hdc: HDC,
        pub fErase: BOOL,
        pub rcPaint: RECT,
        pub fRestore: BOOL,
        pub fIncUpdate: BOOL,
        pub rgbReserved: [u8; 32],
    }

    #[repr(C)]
    pub struct INITCOMMONCONTROLSEX {
        pub dwSize: DWORD,
        pub dwICC: DWORD,
    }

    #[repr(C)]
    pub struct BROWSEINFOW {
        pub hwndOwner: HWND,
        pub pidlRoot: *const std::ffi::c_void,
        pub pszDisplayName: LPWSTR,
        pub lpszTitle: LPCWSTR,
        pub ulFlags: DWORD,
        pub lpfn: *const std::ffi::c_void,
        pub lParam: LPARAM,
        pub iImage: i32,
    }

    pub const WS_OVERLAPPEDWINDOW: DWORD = 0x00CF0000;
    pub const WS_VISIBLE: DWORD = 0x10000000;
    pub const WS_CHILD: DWORD = 0x40000000;
    pub const WS_BORDER: DWORD = 0x00800000;
    pub const WS_TABSTOP: DWORD = 0x00010000;
    pub const WS_VSCROLL: DWORD = 0x00200000;
    pub const ES_MULTILINE: DWORD = 0x0004;
    pub const ES_AUTOVSCROLL: DWORD = 0x0040;
    pub const ES_READONLY: DWORD = 0x0800;
    pub const BS_PUSHBUTTON: DWORD = 0x0000;
    pub const BS_AUTOCHECKBOX: DWORD = 0x0003;
    pub const BS_AUTORADIOBUTTON: DWORD = 0x0009;
    pub const SS_LEFT: DWORD = 0x0000;

    pub const WM_CREATE: UINT = 0x0001;
    pub const WM_DESTROY: UINT = 0x0002;
    pub const WM_PAINT: UINT = 0x000F;
    pub const WM_COMMAND: UINT = 0x0111;
    pub const WM_CTLCOLORSTATIC: UINT = 0x0138;
    pub const WM_CTLCOLOREDIT: UINT = 0x0133;
    pub const WM_CTLCOLORBTN: UINT = 0x0135;
    pub const WM_SETFONT: UINT = 0x0030;
    pub const WM_SETICON: UINT = 0x0080;
    pub const WM_USER: UINT = 0x0400;

    pub const BM_GETCHECK: UINT = 0x00F0;
    pub const BM_SETCHECK: UINT = 0x00F1;
    pub const BST_UNCHECKED: WPARAM = 0;
    pub const BST_CHECKED: WPARAM = 1;

    pub const PBM_SETRANGE32: UINT = WM_USER + 6;
    pub const PBM_SETPOS: UINT = WM_USER + 2;

    pub const SW_SHOW: i32 = 5;
    pub const ICON_SMALL: WPARAM = 0;
    pub const ICON_BIG: WPARAM = 1;
    pub const IMAGE_ICON: UINT = 1;
    pub const LR_LOADFROMFILE: UINT = 0x00000010;
    pub const LR_DEFAULTSIZE: UINT = 0x00000040;

    pub const BIF_RETURNONLYFSDIRS: DWORD = 0x00000001;
    pub const BIF_NEWDIALOGSTYLE: DWORD = 0x00000040;

    #[link(name = "user32")]
    extern "system" {
        pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
        pub fn CreateWindowExW(
            dwExStyle: DWORD,
            lpClassName: LPCWSTR,
            lpWindowName: LPCWSTR,
            dwStyle: DWORD,
            X: i32,
            Y: i32,
            nWidth: i32,
            nHeight: i32,
            hWndParent: HWND,
            hMenu: HMENU,
            hInstance: HINSTANCE,
            lpParam: *mut std::ffi::c_void,
        ) -> HWND;
        pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
        pub fn UpdateWindow(hWnd: HWND) -> BOOL;
        pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
        pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
        pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
        pub fn PostQuitMessage(nExitCode: i32);
        pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
        pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
        pub fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
        pub fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL;
        pub fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: i32) -> i32;
        pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
        pub fn DestroyWindow(hWnd: HWND) -> BOOL;
        pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
        pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
        pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
        pub fn AdjustWindowRectEx(lpRect: *mut RECT, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD) -> BOOL;
        pub fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
        pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
        pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
        pub fn GetSystemMetrics(nIndex: i32) -> i32;
        pub fn SetProcessDpiAwarenessContext(value: isize) -> BOOL;
        pub fn LoadImageW(
            hInst: HINSTANCE,
            name: LPCWSTR,
            type_: UINT,
            cx: i32,
            cy: i32,
            fuLoad: UINT,
        ) -> *mut std::ffi::c_void;
    }

    #[link(name = "gdi32")]
    extern "system" {
        pub fn CreateSolidBrush(color: DWORD) -> HBRUSH;
        pub fn DeleteObject(ho: *mut std::ffi::c_void) -> BOOL;
        pub fn CreateFontW(
            cHeight: i32,
            cWidth: i32,
            cEscapement: i32,
            cOrientation: i32,
            cWeight: i32,
            bItalic: DWORD,
            bUnderline: DWORD,
            bStrikeOut: DWORD,
            iCharSet: DWORD,
            iOutPrecision: DWORD,
            iClipPrecision: DWORD,
            iQuality: DWORD,
            iPitchAndFamily: DWORD,
            pszFaceName: LPCWSTR,
        ) -> HFONT;
        pub fn SelectObject(hdc: HDC, h: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        pub fn SetTextColor(hdc: HDC, color: DWORD) -> DWORD;
        pub fn SetBkColor(hdc: HDC, color: DWORD) -> DWORD;
        pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
        pub fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HINSTANCE;
    }

    #[link(name = "comctl32")]
    extern "system" {
        pub fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> BOOL;
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn SHBrowseForFolderW(lpbi: *mut BROWSEINFOW) -> *mut std::ffi::c_void;
        pub fn SHGetPathFromIDListW(pidl: *const std::ffi::c_void, pszPath: LPWSTR) -> BOOL;
    }

    #[link(name = "ole32")]
    extern "system" {
        pub fn CoInitializeEx(pvReserved: *mut std::ffi::c_void, dwCoInit: DWORD) -> i32;
        pub fn CoUninitialize();
        pub fn CoTaskMemFree(pv: *mut std::ffi::c_void);
    }
}

pub fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

// Custom Windows Messages for Worker Thread -> UI Synchronization
const WM_INSTALL_PROGRESS: u32 = ffi::WM_USER + 101;
const WM_INSTALL_DONE: u32 = ffi::WM_USER + 102;
const WM_INSTALL_ERROR: u32 = ffi::WM_USER + 103;

const WM_UNINSTALL_PROGRESS: u32 = ffi::WM_USER + 201;
const WM_UNINSTALL_DONE: u32 = ffi::WM_USER + 202;
const WM_UNINSTALL_ERROR: u32 = ffi::WM_USER + 203;

// Button Control IDs
const ID_BTN_CANCEL: usize = 1001;
const ID_BTN_BACK: usize = 1002;
const ID_BTN_NEXT: usize = 1003;
const ID_BTN_BROWSE: usize = 1004;
const ID_BTN_RETRY: usize = 1005;

// Checkboxes and Radios
const ID_CHK_DESKTOP: usize = 1010;
const ID_CHK_STARTMENU: usize = 1011;
const ID_CHK_LAUNCH: usize = 1012;
const ID_CHK_STARTWIN: usize = 1013;
const ID_CHK_LICENSE: usize = 1014;
const ID_LBL_DESKTOP_SUB: usize = 1015;
const ID_LBL_STARTMENU_SUB: usize = 1016;
const ID_LBL_LAUNCH_SUB: usize = 1017;
const ID_LBL_STARTWIN_SUB: usize = 1018;

const ID_RAD_KEEP: usize = 1020;
const ID_RAD_PURGE: usize = 1021;

// Colors (COLORREF: 0x00BBGGRR)
const COLOR_BG_MAIN: u32 = 0x002A170F;     // Slate 900 (RGB 15, 23, 42)
const COLOR_BG_HEADER: u32 = 0x003B291E;   // Slate 800 (RGB 30, 41, 59)
const COLOR_BG_CARD: u32 = 0x00332219;     // Dark Slate Card (RGB 25, 34, 51)
const COLOR_BG_INPUT: u32 = 0x00170F02;    // Slate 950 (RGB 2, 6, 23)
const COLOR_TEXT_WHITE: u32 = 0x00FCFAF8;  // Slate 50 (RGB 248, 250, 252)
const COLOR_TEXT_MUTED: u32 = 0x00B8A394;  // Slate 400 (RGB 148, 163, 184)
const COLOR_TEXT_CYAN: u32 = 0x00F8BD38;   // Sky 400 (RGB 56, 189, 248)
const COLOR_ALERT_RED: u32 = 0x004444EF;   // Red 500 (RGB 239, 68, 68)
const COLOR_ALERT_YELLOW: u32 = 0x0024BFFB;// Amber 400 (RGB 251, 191, 36)
const COLOR_ALERT_GREEN: u32 = 0x005EC522; // Green 500 (RGB 34, 197, 94)

fn set_window_icon(hwnd: ffi::HWND) {
    let candidates = [
        PathBuf::from(r"C:\Users\mndab\AppData\Local\Programs\Qaulium\resources\qualium.ico"),
        PathBuf::from(r"C:\Users\mndab\AppData\Local\Programs\Qualium\resources\qualium.ico"),
        PathBuf::from(r"E:\Qaulium AI\Broswer\qualium.ico"),
        PathBuf::from(r"E:\Qaulium AI\Broswer\dist\qualium.ico"),
    ];
    for cand in &candidates {
        if cand.exists() {
            let wide = to_wide_null(&cand.to_string_lossy());
            unsafe {
                let hicon_big = ffi::LoadImageW(
                    std::ptr::null_mut(),
                    wide.as_ptr(),
                    ffi::IMAGE_ICON,
                    32, 32,
                    ffi::LR_LOADFROMFILE,
                );
                let hicon_sm = ffi::LoadImageW(
                    std::ptr::null_mut(),
                    wide.as_ptr(),
                    ffi::IMAGE_ICON,
                    16, 16,
                    ffi::LR_LOADFROMFILE,
                );
                if !hicon_big.is_null() {
                    ffi::SendMessageW(hwnd, ffi::WM_SETICON, ffi::ICON_BIG, hicon_big as isize);
                }
                if !hicon_sm.is_null() {
                    ffi::SendMessageW(hwnd, ffi::WM_SETICON, ffi::ICON_SMALL, hicon_sm as isize);
                }
            }
            break;
        }
    }
}

// ==============================================================================
// 1. INSTALLER WIZARD IMPLEMENTATION
// ==============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerPage {
    Welcome = 1,
    License = 2,
    Location = 3,
    Options = 4,
    Ready = 5,
    Installing = 6,
    Verification = 7,
    Complete = 8,
}

struct InstallerState {
    page: InstallerPage,
    dest_dir: PathBuf,
    req_bytes: u64,
    total_files: usize,
    avail_bytes: u64,
    create_desktop: bool,
    create_startmenu: bool,
    launch_after: bool,
    start_with_win: bool,
    license_accepted: bool,
    engine: Arc<InstallEngine>,
    hwnd: ffi::HWND,
    // Header & Description Controls
    hwnd_title: ffi::HWND,
    hwnd_subtitle: ffi::HWND,
    hwnd_desc: ffi::HWND,
    // Welcome Page Box
    hwnd_welcome_box: ffi::HWND,
    // License Page
    hwnd_license_edit: ffi::HWND,
    hwnd_chk_license: ffi::HWND,
    // Location Page
    hwnd_path_edit: ffi::HWND,
    hwnd_btn_browse: ffi::HWND,
    hwnd_lbl_space: ffi::HWND,
    // Options Page (Checkboxes & Subtitles)
    hwnd_chk_desktop: ffi::HWND,
    hwnd_lbl_desktop_sub: ffi::HWND,
    hwnd_chk_startmenu: ffi::HWND,
    hwnd_lbl_startmenu_sub: ffi::HWND,
    hwnd_chk_launch: ffi::HWND,
    hwnd_lbl_launch_sub: ffi::HWND,
    hwnd_chk_startwin: ffi::HWND,
    hwnd_lbl_startwin_sub: ffi::HWND,
    // Ready & Complete Boxes
    hwnd_ready_box: ffi::HWND,
    hwnd_complete_box: ffi::HWND,
    // Progress
    hwnd_progress: ffi::HWND,
    hwnd_prog_text: ffi::HWND,
    // Buttons
    hwnd_btn_cancel: ffi::HWND,
    hwnd_btn_back: ffi::HWND,
    hwnd_btn_next: ffi::HWND,
    // Fonts & Brushes
    brush_bg: ffi::HBRUSH,
    brush_header: ffi::HBRUSH,
    brush_card: ffi::HBRUSH,
    brush_input: ffi::HBRUSH,
    font_title: ffi::HFONT,
    font_subtitle: ffi::HFONT,
    font_desc: ffi::HFONT,
    font_body: ffi::HFONT,
    font_bold: ffi::HFONT,
    font_sub: ffi::HFONT,
}

static mut G_INSTALLER_STATE: Option<Box<InstallerState>> = None;

pub fn run_installer_gui(engine: InstallEngine, default_dest: PathBuf) -> anyhow::Result<()> {
    unsafe {
        let _ = ffi::SetProcessDpiAwarenessContext(-4);
        let icce = ffi::INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<ffi::INITCOMMONCONTROLSEX>() as u32,
            dwICC: 0x00000020 | 0x00004000,
        };
        ffi::InitCommonControlsEx(&icce);
        ffi::CoInitializeEx(std::ptr::null_mut(), 0x0);

        let metrics = engine.inspect_payload().unwrap_or(PayloadMetrics {
            total_file_count: 298,
            total_uncompressed_bytes: 258 * 1024 * 1024,
        });

        let (free_bytes, _) = win32::get_disk_free_space(&default_dest).unwrap_or((100 * 1024 * 1024 * 1024, 0));

        let hinstance = ffi::GetModuleHandleW(std::ptr::null());
        let class_name = to_wide_null("QualiumInstallerClass");

        let brush_bg = ffi::CreateSolidBrush(COLOR_BG_MAIN);
        let brush_header = ffi::CreateSolidBrush(COLOR_BG_HEADER);
        let brush_card = ffi::CreateSolidBrush(COLOR_BG_CARD);
        let brush_input = ffi::CreateSolidBrush(COLOR_BG_INPUT);

        let font_title = create_font("Segoe UI", 24, 700);
        let font_subtitle = create_font("Segoe UI", 16, 400);
        let font_desc = create_font("Segoe UI", 16, 600);
        let font_body = create_font("Segoe UI", 15, 400);
        let font_bold = create_font("Segoe UI", 15, 600);
        let font_sub = create_font("Segoe UI", 13, 400);

        let wnd_class = ffi::WNDCLASSEXW {
            cbSize: std::mem::size_of::<ffi::WNDCLASSEXW>() as u32,
            style: 0x0003,
            lpfnWndProc: installer_wndproc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: brush_bg,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        ffi::RegisterClassExW(&wnd_class);

        let screen_w = ffi::GetSystemMetrics(0);
        let screen_h = ffi::GetSystemMetrics(1);
        let mut rect = ffi::RECT { left: 0, top: 0, right: 920, bottom: 640 };
        ffi::AdjustWindowRectEx(&mut rect, 0x00CA0000 | ffi::WS_VISIBLE, 0, 0);
        let win_w = rect.right - rect.left;
        let win_h = rect.bottom - rect.top;
        let pos_x = (screen_w - win_w) / 2;
        let pos_y = (screen_h - win_h) / 2;

        let title = to_wide_null("Qaulium Quantum Browser v5.0.0 Setup");
        let hwnd = ffi::CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            0x00CA0000 | ffi::WS_VISIBLE,
            pos_x,
            pos_y,
            win_w,
            win_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        );

        set_window_icon(hwnd);

        let state = Box::new(InstallerState {
            page: InstallerPage::Welcome,
            dest_dir: default_dest,
            req_bytes: metrics.total_uncompressed_bytes,
            total_files: metrics.total_file_count,
            avail_bytes: free_bytes,
            create_desktop: true,
            create_startmenu: true,
            launch_after: true,
            start_with_win: false,
            license_accepted: true,
            engine: Arc::new(engine),
            hwnd,
            hwnd_title: std::ptr::null_mut(),
            hwnd_subtitle: std::ptr::null_mut(),
            hwnd_desc: std::ptr::null_mut(),
            hwnd_welcome_box: std::ptr::null_mut(),
            hwnd_license_edit: std::ptr::null_mut(),
            hwnd_chk_license: std::ptr::null_mut(),
            hwnd_path_edit: std::ptr::null_mut(),
            hwnd_btn_browse: std::ptr::null_mut(),
            hwnd_lbl_space: std::ptr::null_mut(),
            hwnd_chk_desktop: std::ptr::null_mut(),
            hwnd_lbl_desktop_sub: std::ptr::null_mut(),
            hwnd_chk_startmenu: std::ptr::null_mut(),
            hwnd_lbl_startmenu_sub: std::ptr::null_mut(),
            hwnd_chk_launch: std::ptr::null_mut(),
            hwnd_lbl_launch_sub: std::ptr::null_mut(),
            hwnd_chk_startwin: std::ptr::null_mut(),
            hwnd_lbl_startwin_sub: std::ptr::null_mut(),
            hwnd_ready_box: std::ptr::null_mut(),
            hwnd_complete_box: std::ptr::null_mut(),
            hwnd_progress: std::ptr::null_mut(),
            hwnd_prog_text: std::ptr::null_mut(),
            hwnd_btn_cancel: std::ptr::null_mut(),
            hwnd_btn_back: std::ptr::null_mut(),
            hwnd_btn_next: std::ptr::null_mut(),
            brush_bg,
            brush_header,
            brush_card,
            brush_input,
            font_title,
            font_subtitle,
            font_desc,
            font_body,
            font_bold,
            font_sub,
        });

        G_INSTALLER_STATE = Some(state);

        create_installer_controls(hwnd);
        update_installer_page();

        ffi::ShowWindow(hwnd, ffi::SW_SHOW);
        ffi::UpdateWindow(hwnd);

        let mut msg: ffi::MSG = std::mem::zeroed();
        while ffi::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            ffi::TranslateMessage(&msg);
            ffi::DispatchMessageW(&msg);
        }

        ffi::CoUninitialize();
    }

    Ok(())
}

unsafe fn create_installer_controls(hwnd: ffi::HWND) {
    if let Some(state) = G_INSTALLER_STATE.as_mut() {
        let hinstance = ffi::GetModuleHandleW(std::ptr::null());
        let static_class = to_wide_null("STATIC");
        let btn_class = to_wide_null("BUTTON");
        let edit_class = to_wide_null("EDIT");
        let prog_class = to_wide_null("msctls_progress32");

        let mut rc: ffi::RECT = std::mem::zeroed();
        ffi::GetClientRect(hwnd, &mut rc);
        let client_w = (rc.right - rc.left).max(900);
        let client_h = (rc.bottom - rc.top).max(620);
        let content_w = client_w - 70;
        let bar_y = client_h - 58;

        // Header Title
        state.hwnd_title = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("QAULIUM QUANTUM BROWSER").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | 0x00000080, // SS_NOPREFIX
            35, 18, content_w, 30, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_title, ffi::WM_SETFONT, state.font_title as usize, 1);

        // Header Subtitle (SS_NOPREFIX prevents & from showing as underscore)
        state.hwnd_subtitle = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("Welcome to Setup").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | 0x00000080, // SS_NOPREFIX
            35, 50, content_w, 24, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_subtitle, ffi::WM_SETFONT, state.font_subtitle as usize, 1);

        // Page Description (Static label at Y=105, H=45)
        state.hwnd_desc = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | 0x00000080, // SS_NOPREFIX
            35, 105, content_w, 45, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_desc, ffi::WM_SETFONT, state.font_desc as usize, 1);

        // Welcome Box (Highlights Card)
        state.hwnd_welcome_box = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::ES_MULTILINE | ffi::ES_READONLY,
            35, 155, content_w, 370, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_welcome_box, ffi::WM_SETFONT, state.font_body as usize, 1);

        // License Multiline Edit Box
        state.hwnd_license_edit = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::ES_MULTILINE | ffi::ES_READONLY | ffi::ES_AUTOVSCROLL | ffi::WS_VSCROLL,
            35, 155, content_w, 335, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_license_edit, ffi::WM_SETFONT, state.font_body as usize, 1);

        state.hwnd_chk_license = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("I accept the terms of the License Agreement and Privacy Disclosures").as_ptr(),
            ffi::WS_CHILD | ffi::BS_AUTOCHECKBOX | ffi::WS_TABSTOP,
            35, 502, content_w, 28, hwnd, ID_CHK_LICENSE as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_chk_license, ffi::WM_SETFONT, state.font_bold as usize, 1);
        ffi::SendMessageW(state.hwnd_chk_license, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);

        // Location Page
        let edit_w = content_w - 150;
        state.hwnd_path_edit = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null(&state.dest_dir.to_string_lossy()).as_ptr(),
            ffi::WS_CHILD | ffi::WS_BORDER | ffi::WS_TABSTOP,
            35, 160, edit_w, 34, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_path_edit, ffi::WM_SETFONT, state.font_body as usize, 1);

        state.hwnd_btn_browse = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Browse...").as_ptr(),
            ffi::WS_CHILD | ffi::WS_TABSTOP,
            35 + edit_w + 15, 159, 135, 36, hwnd, ID_BTN_BROWSE as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_btn_browse, ffi::WM_SETFONT, state.font_bold as usize, 1);

        state.hwnd_lbl_space = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::ES_MULTILINE | ffi::ES_READONLY,
            35, 215, content_w, 100, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_space, ffi::WM_SETFONT, state.font_body as usize, 1);

        // Options Page Checkboxes & Subtitles (SS_NOTIFY makes clicking description toggle checkbox)
        state.hwnd_chk_desktop = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Create a Desktop Shortcut (Qaulium Quantum Browser.lnk)").as_ptr(),
            ffi::WS_CHILD | ffi::BS_AUTOCHECKBOX | ffi::WS_TABSTOP,
            40, 150, content_w - 10, 26, hwnd, ID_CHK_DESKTOP as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_chk_desktop, ffi::WM_SETFONT, state.font_bold as usize, 1);
        ffi::SendMessageW(state.hwnd_chk_desktop, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);

        state.hwnd_lbl_desktop_sub = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("Places a quick launch shortcut on your active Windows Desktop screen.").as_ptr(),
            ffi::WS_CHILD | 0x00000100, // SS_NOTIFY
            65, 178, content_w - 35, 20, hwnd, ID_LBL_DESKTOP_SUB as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_desktop_sub, ffi::WM_SETFONT, state.font_sub as usize, 1);

        state.hwnd_chk_startmenu = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Add Shortcuts to Windows Start Menu").as_ptr(),
            ffi::WS_CHILD | ffi::BS_AUTOCHECKBOX | ffi::WS_TABSTOP,
            40, 215, content_w - 10, 26, hwnd, ID_CHK_STARTMENU as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_chk_startmenu, ffi::WM_SETFONT, state.font_bold as usize, 1);
        ffi::SendMessageW(state.hwnd_chk_startmenu, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);

        state.hwnd_lbl_startmenu_sub = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("Adds Qaulium Quantum Browser and Uninstaller under Start Menu > Programs > Qaulium.").as_ptr(),
            ffi::WS_CHILD | 0x00000100, // SS_NOTIFY
            65, 243, content_w - 35, 20, hwnd, ID_LBL_STARTMENU_SUB as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_startmenu_sub, ffi::WM_SETFONT, state.font_sub as usize, 1);

        state.hwnd_chk_launch = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Launch Qaulium Quantum Browser after installation").as_ptr(),
            ffi::WS_CHILD | ffi::BS_AUTOCHECKBOX | ffi::WS_TABSTOP,
            40, 280, content_w - 10, 26, hwnd, ID_CHK_LAUNCH as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_chk_launch, ffi::WM_SETFONT, state.font_bold as usize, 1);
        ffi::SendMessageW(state.hwnd_chk_launch, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);

        state.hwnd_lbl_launch_sub = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("Automatically launches the browser immediately upon clicking Finish.").as_ptr(),
            ffi::WS_CHILD | 0x00000100, // SS_NOTIFY
            65, 308, content_w - 35, 20, hwnd, ID_LBL_LAUNCH_SUB as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_launch_sub, ffi::WM_SETFONT, state.font_sub as usize, 1);

        state.hwnd_chk_startwin = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Start with Windows (Optional)").as_ptr(),
            ffi::WS_CHILD | ffi::BS_AUTOCHECKBOX | ffi::WS_TABSTOP,
            40, 345, content_w - 10, 26, hwnd, ID_CHK_STARTWIN as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_chk_startwin, ffi::WM_SETFONT, state.font_body as usize, 1);
        ffi::SendMessageW(state.hwnd_chk_startwin, ffi::BM_SETCHECK, ffi::BST_UNCHECKED, 0);

        state.hwnd_lbl_startwin_sub = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("Launches the Qaulium background privacy daemon when Windows starts (Default: Off).").as_ptr(),
            ffi::WS_CHILD | 0x00000100, // SS_NOTIFY
            65, 373, content_w - 35, 20, hwnd, ID_LBL_STARTWIN_SUB as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_startwin_sub, ffi::WM_SETFONT, state.font_sub as usize, 1);

        // Ready Page Box
        state.hwnd_ready_box = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::ES_MULTILINE | ffi::ES_READONLY,
            35, 155, content_w, 370, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_ready_box, ffi::WM_SETFONT, state.font_body as usize, 1);

        // Complete Page Box
        state.hwnd_complete_box = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::ES_MULTILINE | ffi::ES_READONLY,
            35, 155, content_w, 370, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_complete_box, ffi::WM_SETFONT, state.font_body as usize, 1);

        // Progress Bar
        state.hwnd_progress = ffi::CreateWindowExW(
            0, prog_class.as_ptr(), std::ptr::null(),
            ffi::WS_CHILD | ffi::WS_BORDER,
            35, 210, content_w, 32, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_progress, ffi::PBM_SETRANGE32, 0, 1000);

        state.hwnd_prog_text = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("Preparing installation...").as_ptr(),
            ffi::WS_CHILD,
            35, 255, content_w, 95, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_prog_text, ffi::WM_SETFONT, state.font_body as usize, 1);

        // Bottom Action Buttons (Next button is 190px wide to fit 'Agree & Continue >' comfortably)
        state.hwnd_btn_cancel = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Cancel").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | ffi::WS_TABSTOP,
            35, bar_y, 120, 38, hwnd, ID_BTN_CANCEL as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_btn_cancel, ffi::WM_SETFONT, state.font_bold as usize, 1);

        state.hwnd_btn_back = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("< Back").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | ffi::WS_TABSTOP,
            client_w - 360, bar_y, 120, 38, hwnd, ID_BTN_BACK as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_btn_back, ffi::WM_SETFONT, state.font_bold as usize, 1);

        state.hwnd_btn_next = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Next >").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | ffi::WS_TABSTOP,
            client_w - 225, bar_y, 190, 38, hwnd, ID_BTN_NEXT as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_btn_next, ffi::WM_SETFONT, state.font_bold as usize, 1);
    }
}

unsafe fn update_installer_page() {
    if let Some(state) = G_INSTALLER_STATE.as_mut() {
        let hide = |w: ffi::HWND| if !w.is_null() { ffi::ShowWindow(w, 0); };
        let show = |w: ffi::HWND| if !w.is_null() { ffi::ShowWindow(w, 5); };

        hide(state.hwnd_welcome_box);
        hide(state.hwnd_license_edit);
        hide(state.hwnd_chk_license);
        hide(state.hwnd_path_edit);
        hide(state.hwnd_btn_browse);
        hide(state.hwnd_lbl_space);
        hide(state.hwnd_chk_desktop);
        hide(state.hwnd_lbl_desktop_sub);
        hide(state.hwnd_chk_startmenu);
        hide(state.hwnd_lbl_startmenu_sub);
        hide(state.hwnd_chk_launch);
        hide(state.hwnd_lbl_launch_sub);
        hide(state.hwnd_chk_startwin);
        hide(state.hwnd_lbl_startwin_sub);
        hide(state.hwnd_ready_box);
        hide(state.hwnd_complete_box);
        hide(state.hwnd_progress);
        hide(state.hwnd_prog_text);

        ffi::EnableWindow(state.hwnd_btn_back, 1);
        ffi::EnableWindow(state.hwnd_btn_next, 1);
        ffi::EnableWindow(state.hwnd_btn_cancel, 1);

        match state.page {
            InstallerPage::Welcome => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 1 of 8: Welcome to Setup").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Welcome to the Qaulium Quantum Browser v5.0.0 Installation Wizard.").as_ptr());
                
                let welcome_txt = "QAULIUM QUANTUM BROWSER v5.0.0 (x64 Native)\r\n\r\n\
                    • Ultimate Privacy Architecture\r\n\
                      Zero corporate telemetry, zero keystroke logging, and complete hardware-level session isolation.\r\n\r\n\
                    • Native Gecko ESR 140 Engine\r\n\
                      Authentic Gecko rendering layout combined with the high-performance Necko network stack.\r\n\r\n\
                    • Unified In-Tab Navigation (qualium://)\r\n\
                      Bookmarks, Downloads, Settings, Passwords, and Extensions render inside normal browser tabs — no popup modals.\r\n\r\n\
                    • Quantum Cryptographic Vault\r\n\
                      Client-side AES-256-GCM and X25519 cryptographic key management stored securely in your profile.\r\n\r\n\
                    Click 'Next >' to review license notices and choose installation options.";
                ffi::SetWindowTextW(state.hwnd_welcome_box, to_wide_null(welcome_txt).as_ptr());
                show(state.hwnd_welcome_box);

                ffi::EnableWindow(state.hwnd_btn_back, 0);
                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Next >").as_ptr());
            }
            InstallerPage::License => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 2 of 8: License Agreement & Notices").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Please read the following license agreement and privacy disclosures:").as_ptr());

                let lic_txt = "MOZILLA PUBLIC LICENSE Version 2.0 & QAULIUM QUANTUM BROWSER TERMS\r\n\r\n\
                    1. Definitions\r\n\
                    1.1. \"Contributor\" means each individual or legal entity that creates, contributes to the creation of, or owns Covered Software.\r\n\
                    1.2. \"Covered Software\" means Source Code Form to which the initial Contributor has attached the notice in Exhibit A, the Executable Form of such Source Code Form, and Modifications of such Source Code Form.\r\n\r\n\
                    2. License Grants and Conditions\r\n\
                    Each Contributor hereby grants You a world-wide, royalty-free, non-exclusive license to use, reproduce, make available, modify, display, perform, distribute, and otherwise exploit its Contributions.\r\n\r\n\
                    3. Qaulium Privacy & Security Assurance\r\n\
                    Qaulium Quantum Browser guarantees that no telemetry data, browsing history, keystrokes, or search requests are transmitted to Qaulium or external third parties without explicit user consent. All cryptographic vault data remains strictly local on your device.\r\n\r\n\
                    Portions of this software are based on Mozilla Gecko and Firefox technology under MPL 2.0.";
                ffi::SetWindowTextW(state.hwnd_license_edit, to_wide_null(lic_txt).as_ptr());
                show(state.hwnd_license_edit);
                show(state.hwnd_chk_license);

                ffi::SendMessageW(state.hwnd_chk_license, ffi::BM_SETCHECK, if state.license_accepted { ffi::BST_CHECKED } else { ffi::BST_UNCHECKED }, 0);
                ffi::EnableWindow(state.hwnd_btn_next, if state.license_accepted { 1 } else { 0 });
                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Agree && Continue >").as_ptr());
            }
            InstallerPage::Location => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 3 of 8: Choose Install Location").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Setup will install Qaulium Quantum Browser into the destination directory below:").as_ptr());

                show(state.hwnd_path_edit);
                show(state.hwnd_btn_browse);
                show(state.hwnd_lbl_space);

                let req_mb = (state.req_bytes as f64 / (1024.0 * 1024.0)).ceil();
                if let Some((avail, _)) = win32::get_disk_free_space(&state.dest_dir) {
                    state.avail_bytes = avail;
                }
                let avail_gb = state.avail_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

                let space_info = if state.avail_bytes < state.req_bytes + 50 * 1024 * 1024 {
                    ffi::EnableWindow(state.hwnd_btn_next, 0);
                    format!("Space Required: {:.1} MB\r\nSpace Available: {:.2} GB\r\n\r\n[!] Insufficient disk space on selected drive! Please select another destination.", req_mb, avail_gb)
                } else {
                    format!("Space Required: {:.1} MB\r\nSpace Available: {:.2} GB\r\n\r\nStatus: Destination drive has sufficient verified free disk space.", req_mb, avail_gb)
                };
                ffi::SetWindowTextW(state.hwnd_lbl_space, to_wide_null(&space_info).as_ptr());

                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Next >").as_ptr());
            }
            InstallerPage::Options => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 4 of 8: Installation Options").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Select shortcuts and startup preferences for Qaulium Quantum Browser:").as_ptr());

                ffi::SendMessageW(state.hwnd_chk_desktop, ffi::BM_SETCHECK, if state.create_desktop { ffi::BST_CHECKED } else { ffi::BST_UNCHECKED }, 0);
                ffi::SendMessageW(state.hwnd_chk_startmenu, ffi::BM_SETCHECK, if state.create_startmenu { ffi::BST_CHECKED } else { ffi::BST_UNCHECKED }, 0);
                ffi::SendMessageW(state.hwnd_chk_launch, ffi::BM_SETCHECK, if state.launch_after { ffi::BST_CHECKED } else { ffi::BST_UNCHECKED }, 0);
                ffi::SendMessageW(state.hwnd_chk_startwin, ffi::BM_SETCHECK, if state.start_with_win { ffi::BST_CHECKED } else { ffi::BST_UNCHECKED }, 0);

                show(state.hwnd_chk_desktop);
                show(state.hwnd_lbl_desktop_sub);
                show(state.hwnd_chk_startmenu);
                show(state.hwnd_lbl_startmenu_sub);
                show(state.hwnd_chk_launch);
                show(state.hwnd_lbl_launch_sub);
                show(state.hwnd_chk_startwin);
                show(state.hwnd_lbl_startwin_sub);

                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Next >").as_ptr());
            }
            InstallerPage::Ready => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 5 of 8: Ready to Install").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Setup is now ready to begin installing Qaulium Quantum Browser on your computer.").as_ptr());

                let req_mb = (state.req_bytes as f64 / (1024.0 * 1024.0)).ceil();
                let avail_gb = state.avail_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let ready_summary = format!(
                    "INSTALLATION SPECIFICATION:\r\n\r\n\
                    • Product Name: Qaulium Quantum Browser v5.0.0 (x64 Native)\r\n\
                    • Install Folder: {}\r\n\
                    • Required Space: {:.1} MB\r\n\
                    • Available Space: {:.2} GB\r\n\
                    • Desktop Shortcut: {}\r\n\
                    • Start Menu Shortcut: {}\r\n\
                    • Launch After Install: {}\r\n\
                    • Start With Windows: {}\r\n\r\n\
                    COMPONENTS TO BE INSTALLED:\r\n\
                    1. QauliumQuantumBrowser.exe (Primary Desktop Browser)\r\n\
                    2. qualium-daemon.exe (Background Privacy & Crypto Daemon)\r\n\
                    3. Gecko ESR 140 Engine Runtime & Necko Stack ({} files)\r\n\
                    4. Qaulium Chrome Assets & In-Tab Schemes (qualium://)\r\n\
                    5. QauliumUninstall.exe (Independent Windows Uninstaller)\r\n\r\n\
                    Click 'Install' to start copying files.",
                    state.dest_dir.display(),
                    req_mb,
                    avail_gb,
                    if state.create_desktop { "Yes (Qaulium Quantum Browser.lnk)" } else { "No" },
                    if state.create_startmenu { "Yes (Programs\\Qaulium)" } else { "No" },
                    if state.launch_after { "Yes" } else { "No" },
                    if state.start_with_win { "Yes (Startup Run Key)" } else { "No" },
                    state.total_files
                );
                ffi::SetWindowTextW(state.hwnd_ready_box, to_wide_null(&ready_summary).as_ptr());
                show(state.hwnd_ready_box);

                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Install").as_ptr());
            }
            InstallerPage::Installing => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 6 of 8: Installing Qaulium Quantum Browser...").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Copying and verifying components into the installation directory:").as_ptr());

                show(state.hwnd_progress);
                show(state.hwnd_prog_text);

                ffi::EnableWindow(state.hwnd_btn_back, 0);
                ffi::EnableWindow(state.hwnd_btn_next, 0);
                ffi::EnableWindow(state.hwnd_btn_cancel, 0);

                start_install_worker(state.hwnd);
            }
            InstallerPage::Verification => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 7 of 8: Verifying Installation Integrity").as_ptr());
                ffi::SendMessageW(state.hwnd_progress, ffi::PBM_SETPOS, 1000, 0);
                ffi::SetWindowTextW(state.hwnd_prog_text, to_wide_null("Verifying binary checksums, registry registrations, and desktop shortcuts...").as_ptr());
            }
            InstallerPage::Complete => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 8 of 8: Installation Complete!").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Qaulium Quantum Browser v5.0.0 has been successfully installed on your computer.").as_ptr());

                let done_txt = format!(
                    "INSTALLATION COMPLETE!\r\n\r\n\
                    • Installed Path:\r\n  {}\r\n\r\n\
                    • Authoritative Inventory Generated:\r\n  {}\\install-manifest.json\r\n\r\n\
                    • Shortcuts Configured:\r\n  - Desktop Shortcut: {}\r\n  - Start Menu: {}\r\n  - Start with Windows: {}\r\n  - Launch on Finish: {}\r\n\r\n\
                    • Windows Registration:\r\n  - Registered in Windows Installed Apps under Qaulium Quantum Browser.\r\n\r\n\
                    Click 'Finish' to exit Setup.",
                    state.dest_dir.display(),
                    state.dest_dir.display(),
                    if state.create_desktop { "Qaulium Quantum Browser.lnk (Desktop)" } else { "Disabled" },
                    if state.create_startmenu { "Programs\\Qaulium" } else { "Disabled" },
                    if state.start_with_win { "Enabled (HKCU Run)" } else { "Disabled" },
                    if state.launch_after { "Enabled (Browser will launch)" } else { "Disabled" }
                );
                ffi::SetWindowTextW(state.hwnd_complete_box, to_wide_null(&done_txt).as_ptr());
                show(state.hwnd_complete_box);

                hide(state.hwnd_progress);
                hide(state.hwnd_prog_text);
                hide(state.hwnd_btn_cancel);
                ffi::EnableWindow(state.hwnd_btn_back, 0);
                ffi::EnableWindow(state.hwnd_btn_next, 1);
                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Finish").as_ptr());
            }
        }

        let mut rc: ffi::RECT = std::mem::zeroed();
        ffi::GetClientRect(state.hwnd, &mut rc);
        ffi::InvalidateRect(state.hwnd, &rc, 1);
    }
}

fn start_install_worker(hwnd: ffi::HWND) {
    let engine = unsafe {
        if let Some(state) = G_INSTALLER_STATE.as_ref() {
            state.engine.clone()
        } else {
            return;
        }
    };

    let options = unsafe {
        if let Some(state) = G_INSTALLER_STATE.as_ref() {
            InstallOptions {
                install_dir: state.dest_dir.clone(),
                create_desktop_shortcut: state.create_desktop,
                create_start_menu_shortcut: state.create_startmenu,
                launch_after_install: state.launch_after,
                start_with_windows: state.start_with_win,
            }
        } else {
            return;
        }
    };

    let hwnd_val = hwnd as usize;
    thread::spawn(move || {
        let hwnd = hwnd_val as ffi::HWND;
        let res = engine.install(&options, |cur_f, tot_f, cur_b, tot_b, path, comp| {
            let pct = if tot_b > 0 { (cur_b as f64 / tot_b as f64 * 1000.0) as usize } else { 0 };
            let mb_cur = cur_b as f64 / (1024.0 * 1024.0);
            let mb_tot = tot_b as f64 / (1024.0 * 1024.0);
            let pct_100 = if tot_b > 0 { (cur_b as f64 / tot_b as f64 * 100.0) as usize } else { 0 };
            let msg = format!(
                "Installing Qaulium Quantum Browser\r\nComponent: {}\r\nFiles: {} / {} | {:.1} MB / {:.1} MB ({}%)\r\nExtracting: {}",
                comp, cur_f, tot_f, mb_cur, mb_tot, pct_100, path
            );
            let boxed = Box::into_raw(Box::new(msg)) as isize;
            unsafe {
                ffi::PostMessageW(hwnd, WM_INSTALL_PROGRESS, pct, boxed);
            }
        });

        match res {
            Ok(_) => unsafe {
                ffi::PostMessageW(hwnd, WM_INSTALL_DONE, 0, 0);
            },
            Err(e) => {
                let err_msg = e.to_string();
                let boxed = Box::into_raw(Box::new(err_msg)) as isize;
                unsafe {
                    ffi::PostMessageW(hwnd, WM_INSTALL_ERROR, 0, boxed);
                }
            }
        }
    });
}

unsafe extern "system" fn installer_wndproc(
    hwnd: ffi::HWND,
    msg: ffi::UINT,
    wparam: ffi::WPARAM,
    lparam: ffi::LPARAM,
) -> ffi::LRESULT {
    match msg {
        ffi::WM_PAINT => {
            let mut ps: ffi::PAINTSTRUCT = std::mem::zeroed();
            let hdc = ffi::BeginPaint(hwnd, &mut ps);

            if let Some(state) = G_INSTALLER_STATE.as_ref() {
                let mut rc_client: ffi::RECT = std::mem::zeroed();
                ffi::GetClientRect(hwnd, &mut rc_client);
                let w = rc_client.right - rc_client.left;
                let h = rc_client.bottom - rc_client.top;

                // Header Banner
                let rect_header = ffi::RECT { left: 0, top: 0, right: w, bottom: 85 };
                ffi::FillRect(hdc, &rect_header, state.brush_header);

                let rect_line1 = ffi::RECT { left: 0, top: 85, right: w, bottom: 86 };
                let brush_line = ffi::CreateSolidBrush(0x00503525);
                ffi::FillRect(hdc, &rect_line1, brush_line);

                let bar_line_y = h - 68;
                let rect_line2 = ffi::RECT { left: 0, top: bar_line_y, right: w, bottom: bar_line_y + 1 };
                ffi::FillRect(hdc, &rect_line2, brush_line);

                ffi::DeleteObject(brush_line as *mut std::ffi::c_void);
            }

            ffi::EndPaint(hwnd, &ps);
            0
        }
        ffi::WM_CTLCOLORSTATIC => {
            let hdc = wparam as ffi::HDC;
            ffi::SetBkMode(hdc, 1);
            if let Some(state) = G_INSTALLER_STATE.as_ref() {
                let ctl_hwnd = lparam as ffi::HWND;
                if ctl_hwnd == state.hwnd_title {
                    ffi::SetTextColor(hdc, COLOR_TEXT_CYAN);
                    return state.brush_header as ffi::LRESULT;
                } else if ctl_hwnd == state.hwnd_subtitle {
                    ffi::SetTextColor(hdc, COLOR_TEXT_MUTED);
                    return state.brush_header as ffi::LRESULT;
                } else if ctl_hwnd == state.hwnd_desc {
                    ffi::SetTextColor(hdc, COLOR_TEXT_WHITE);
                    return state.brush_bg as ffi::LRESULT;
                } else if ctl_hwnd == state.hwnd_lbl_desktop_sub
                    || ctl_hwnd == state.hwnd_lbl_startmenu_sub
                    || ctl_hwnd == state.hwnd_lbl_launch_sub
                    || ctl_hwnd == state.hwnd_lbl_startwin_sub
                {
                    ffi::SetTextColor(hdc, COLOR_TEXT_MUTED);
                    return state.brush_bg as ffi::LRESULT;
                } else {
                    ffi::SetTextColor(hdc, COLOR_TEXT_WHITE);
                    return state.brush_bg as ffi::LRESULT;
                }
            }
            0
        }
        ffi::WM_CTLCOLOREDIT => {
            let hdc = wparam as ffi::HDC;
            ffi::SetTextColor(hdc, COLOR_TEXT_WHITE);
            ffi::SetBkColor(hdc, COLOR_BG_CARD);
            if let Some(state) = G_INSTALLER_STATE.as_ref() {
                return state.brush_card as ffi::LRESULT;
            }
            0
        }
        ffi::WM_CTLCOLORBTN => {
            let hdc = wparam as ffi::HDC;
            ffi::SetBkMode(hdc, 1);
            ffi::SetTextColor(hdc, COLOR_TEXT_WHITE);
            if let Some(state) = G_INSTALLER_STATE.as_ref() {
                return state.brush_bg as ffi::LRESULT;
            }
            0
        }
        ffi::WM_COMMAND => {
            let cmd_id = (wparam & 0xFFFF) as usize;
            match cmd_id {
                ID_BTN_CANCEL => {
                    ffi::PostQuitMessage(0);
                }
                ID_BTN_BACK => {
                    if let Some(state) = G_INSTALLER_STATE.as_mut() {
                        state.page = match state.page {
                            InstallerPage::License => InstallerPage::Welcome,
                            InstallerPage::Location => InstallerPage::License,
                            InstallerPage::Options => InstallerPage::Location,
                            InstallerPage::Ready => InstallerPage::Options,
                            _ => state.page,
                        };
                        update_installer_page();
                    }
                }
                ID_CHK_LICENSE => {
                    if let Some(state) = G_INSTALLER_STATE.as_mut() {
                        let checked = ffi::SendMessageW(state.hwnd_chk_license, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                        state.license_accepted = checked;
                        ffi::EnableWindow(state.hwnd_btn_next, if checked { 1 } else { 0 });
                    }
                }
                ID_CHK_DESKTOP | ID_LBL_DESKTOP_SUB => {
                    if let Some(state) = G_INSTALLER_STATE.as_mut() {
                        if cmd_id == ID_LBL_DESKTOP_SUB {
                            let cur = ffi::SendMessageW(state.hwnd_chk_desktop, ffi::BM_GETCHECK, 0, 0);
                            let next = if cur == ffi::BST_CHECKED as isize { ffi::BST_UNCHECKED } else { ffi::BST_CHECKED };
                            ffi::SendMessageW(state.hwnd_chk_desktop, ffi::BM_SETCHECK, next, 0);
                        }
                        state.create_desktop = ffi::SendMessageW(state.hwnd_chk_desktop, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                    }
                }
                ID_CHK_STARTMENU | ID_LBL_STARTMENU_SUB => {
                    if let Some(state) = G_INSTALLER_STATE.as_mut() {
                        if cmd_id == ID_LBL_STARTMENU_SUB {
                            let cur = ffi::SendMessageW(state.hwnd_chk_startmenu, ffi::BM_GETCHECK, 0, 0);
                            let next = if cur == ffi::BST_CHECKED as isize { ffi::BST_UNCHECKED } else { ffi::BST_CHECKED };
                            ffi::SendMessageW(state.hwnd_chk_startmenu, ffi::BM_SETCHECK, next, 0);
                        }
                        state.create_startmenu = ffi::SendMessageW(state.hwnd_chk_startmenu, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                    }
                }
                ID_CHK_LAUNCH | ID_LBL_LAUNCH_SUB => {
                    if let Some(state) = G_INSTALLER_STATE.as_mut() {
                        if cmd_id == ID_LBL_LAUNCH_SUB {
                            let cur = ffi::SendMessageW(state.hwnd_chk_launch, ffi::BM_GETCHECK, 0, 0);
                            let next = if cur == ffi::BST_CHECKED as isize { ffi::BST_UNCHECKED } else { ffi::BST_CHECKED };
                            ffi::SendMessageW(state.hwnd_chk_launch, ffi::BM_SETCHECK, next, 0);
                        }
                        state.launch_after = ffi::SendMessageW(state.hwnd_chk_launch, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                    }
                }
                ID_CHK_STARTWIN | ID_LBL_STARTWIN_SUB => {
                    if let Some(state) = G_INSTALLER_STATE.as_mut() {
                        if cmd_id == ID_LBL_STARTWIN_SUB {
                            let cur = ffi::SendMessageW(state.hwnd_chk_startwin, ffi::BM_GETCHECK, 0, 0);
                            let next = if cur == ffi::BST_CHECKED as isize { ffi::BST_UNCHECKED } else { ffi::BST_CHECKED };
                            ffi::SendMessageW(state.hwnd_chk_startwin, ffi::BM_SETCHECK, next, 0);
                        }
                        state.start_with_win = ffi::SendMessageW(state.hwnd_chk_startwin, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                    }
                }
                ID_BTN_NEXT => {
                    if let Some(state) = G_INSTALLER_STATE.as_mut() {
                        match state.page {
                            InstallerPage::Welcome => {
                                state.page = InstallerPage::License;
                                update_installer_page();
                            }
                            InstallerPage::License => {
                                state.page = InstallerPage::Location;
                                update_installer_page();
                            }
                            InstallerPage::Location => {
                                let mut buf = [0u16; 1024];
                                let len = ffi::GetWindowTextW(state.hwnd_path_edit, buf.as_mut_ptr(), 1024);
                                if len > 0 {
                                    let s = String::from_utf16_lossy(&buf[..len as usize]);
                                    state.dest_dir = PathBuf::from(s.trim());
                                }
                                state.page = InstallerPage::Options;
                                update_installer_page();
                            }
                            InstallerPage::Options => {
                                state.create_desktop = ffi::SendMessageW(state.hwnd_chk_desktop, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                                state.create_startmenu = ffi::SendMessageW(state.hwnd_chk_startmenu, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                                state.launch_after = ffi::SendMessageW(state.hwnd_chk_launch, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                                state.start_with_win = ffi::SendMessageW(state.hwnd_chk_startwin, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                                state.page = InstallerPage::Ready;
                                update_installer_page();
                            }
                            InstallerPage::Ready => {
                                state.page = InstallerPage::Installing;
                                update_installer_page();
                            }
                            InstallerPage::Complete => {
                                if state.launch_after {
                                    let exe1 = state.dest_dir.join("QauliumQuantumBrowser.exe");
                                    let exe2 = state.dest_dir.join("QualiumQuantumBrowser.exe");
                                    let target = if exe1.exists() { exe1 } else { exe2 };
                                    if target.exists() {
                                        let _ = std::process::Command::new(&target)
                                            .current_dir(&state.dest_dir)
                                            .spawn();
                                    }
                                }
                                ffi::PostQuitMessage(0);
                            }
                            _ => {}
                        }
                    }
                }
                ID_BTN_BROWSE => {
                    if let Some(folder) = browse_for_folder(hwnd, "Select Qaulium Destination Folder") {
                        if let Some(state) = G_INSTALLER_STATE.as_mut() {
                            state.dest_dir = folder;
                            ffi::SetWindowTextW(state.hwnd_path_edit, to_wide_null(&state.dest_dir.to_string_lossy()).as_ptr());
                            update_installer_page();
                        }
                    }
                }
                _ => {}
            }
            0
        }
        WM_INSTALL_PROGRESS => {
            let pct = wparam as isize;
            let msg_ptr = lparam as *mut String;
            if !msg_ptr.is_null() {
                let msg = Box::from_raw(msg_ptr);
                if let Some(state) = G_INSTALLER_STATE.as_ref() {
                    ffi::SendMessageW(state.hwnd_progress, ffi::PBM_SETPOS, pct as usize, 0);
                    ffi::SetWindowTextW(state.hwnd_prog_text, to_wide_null(&msg).as_ptr());
                }
            }
            0
        }
        WM_INSTALL_DONE => {
            if let Some(state) = G_INSTALLER_STATE.as_mut() {
                state.page = InstallerPage::Complete;
                update_installer_page();
            }
            0
        }
        WM_INSTALL_ERROR => {
            let err_ptr = lparam as *mut String;
            if !err_ptr.is_null() {
                let err_msg = Box::from_raw(err_ptr);
                if let Some(state) = G_INSTALLER_STATE.as_mut() {
                    let err_display = format!("Installation Failed:\r\n\r\n{}\r\n\r\nPlease check permissions and disk space.", err_msg);
                    ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("An error occurred during installation:").as_ptr());
                    ffi::SetWindowTextW(state.hwnd_welcome_box, to_wide_null(&err_display).as_ptr());
                    ffi::ShowWindow(state.hwnd_welcome_box, 5);
                    ffi::ShowWindow(state.hwnd_progress, 0);
                    ffi::ShowWindow(state.hwnd_prog_text, 0);
                    ffi::EnableWindow(state.hwnd_btn_cancel, 1);
                }
            }
            0
        }
        ffi::WM_DESTROY => {
            ffi::PostQuitMessage(0);
            0
        }
        _ => ffi::DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ==============================================================================
// 2. UNINSTALLER WIZARD IMPLEMENTATION
// ==============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallerPage {
    ProcessCheck = 1,
    Options = 2,
    Ready = 3,
    Removing = 4,
    Complete = 5,
}

struct UninstallerState {
    page: UninstallerPage,
    install_dir: PathBuf,
    keep_user_data: bool,
    running_procs: Vec<String>,
    engine: Arc<UninstallerEngine>,
    hwnd: ffi::HWND,
    // Header & Description
    hwnd_title: ffi::HWND,
    hwnd_subtitle: ffi::HWND,
    hwnd_desc: ffi::HWND,
    // Welcome / Info Box
    hwnd_info_box: ffi::HWND,
    hwnd_lbl_warning: ffi::HWND,
    hwnd_btn_retry: ffi::HWND,
    hwnd_lbl_clean: ffi::HWND,
    // Options Page (Radios & Subtitles)
    hwnd_rad_keep: ffi::HWND,
    hwnd_lbl_keep_sub: ffi::HWND,
    hwnd_rad_purge: ffi::HWND,
    hwnd_lbl_purge_sub: ffi::HWND,
    hwnd_lbl_safety_notice: ffi::HWND,
    // Ready & Complete Boxes
    hwnd_ready_box: ffi::HWND,
    hwnd_complete_box: ffi::HWND,
    // Progress
    hwnd_progress: ffi::HWND,
    hwnd_prog_text: ffi::HWND,
    // Buttons
    hwnd_btn_cancel: ffi::HWND,
    hwnd_btn_back: ffi::HWND,
    hwnd_btn_next: ffi::HWND,
    // Fonts & Brushes
    brush_bg: ffi::HBRUSH,
    brush_header: ffi::HBRUSH,
    brush_card: ffi::HBRUSH,
    brush_input: ffi::HBRUSH,
    font_title: ffi::HFONT,
    font_subtitle: ffi::HFONT,
    font_desc: ffi::HFONT,
    font_body: ffi::HFONT,
    font_bold: ffi::HFONT,
    font_sub: ffi::HFONT,
}

static mut G_UNINSTALLER_STATE: Option<Box<UninstallerState>> = None;

pub fn run_uninstaller_gui(engine: UninstallerEngine, install_dir: PathBuf) -> anyhow::Result<()> {
    unsafe {
        let _ = ffi::SetProcessDpiAwarenessContext(-4);
        let icce = ffi::INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<ffi::INITCOMMONCONTROLSEX>() as u32,
            dwICC: 0x00000020 | 0x00004000,
        };
        ffi::InitCommonControlsEx(&icce);

        let running = win32::get_running_qualium_processes();

        let hinstance = ffi::GetModuleHandleW(std::ptr::null());
        let class_name = to_wide_null("QualiumUninstallerClass");

        let brush_bg = ffi::CreateSolidBrush(COLOR_BG_MAIN);
        let brush_header = ffi::CreateSolidBrush(COLOR_BG_HEADER);
        let brush_card = ffi::CreateSolidBrush(COLOR_BG_CARD);
        let brush_input = ffi::CreateSolidBrush(COLOR_BG_INPUT);

        let font_title = create_font("Segoe UI", 24, 700);
        let font_subtitle = create_font("Segoe UI", 16, 400);
        let font_desc = create_font("Segoe UI", 16, 600);
        let font_body = create_font("Segoe UI", 15, 400);
        let font_bold = create_font("Segoe UI", 15, 600);
        let font_sub = create_font("Segoe UI", 13, 400);

        let wnd_class = ffi::WNDCLASSEXW {
            cbSize: std::mem::size_of::<ffi::WNDCLASSEXW>() as u32,
            style: 0x0003,
            lpfnWndProc: uninstaller_wndproc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: brush_bg,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        ffi::RegisterClassExW(&wnd_class);

        let screen_w = ffi::GetSystemMetrics(0);
        let screen_h = ffi::GetSystemMetrics(1);
        let mut rect = ffi::RECT { left: 0, top: 0, right: 920, bottom: 640 };
        ffi::AdjustWindowRectEx(&mut rect, 0x00CA0000 | ffi::WS_VISIBLE, 0, 0);
        let win_w = rect.right - rect.left;
        let win_h = rect.bottom - rect.top;
        let pos_x = (screen_w - win_w) / 2;
        let pos_y = (screen_h - win_h) / 2;

        let title = to_wide_null("Qaulium Quantum Browser — Uninstaller");
        let hwnd = ffi::CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            0x00CA0000 | ffi::WS_VISIBLE,
            pos_x,
            pos_y,
            win_w,
            win_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        );

        set_window_icon(hwnd);

        let state = Box::new(UninstallerState {
            page: UninstallerPage::ProcessCheck,
            install_dir,
            keep_user_data: true,
            running_procs: running,
            engine: Arc::new(engine),
            hwnd,
            hwnd_title: std::ptr::null_mut(),
            hwnd_subtitle: std::ptr::null_mut(),
            hwnd_desc: std::ptr::null_mut(),
            hwnd_info_box: std::ptr::null_mut(),
            hwnd_lbl_warning: std::ptr::null_mut(),
            hwnd_btn_retry: std::ptr::null_mut(),
            hwnd_lbl_clean: std::ptr::null_mut(),
            hwnd_rad_keep: std::ptr::null_mut(),
            hwnd_lbl_keep_sub: std::ptr::null_mut(),
            hwnd_rad_purge: std::ptr::null_mut(),
            hwnd_lbl_purge_sub: std::ptr::null_mut(),
            hwnd_lbl_safety_notice: std::ptr::null_mut(),
            hwnd_ready_box: std::ptr::null_mut(),
            hwnd_complete_box: std::ptr::null_mut(),
            hwnd_progress: std::ptr::null_mut(),
            hwnd_prog_text: std::ptr::null_mut(),
            hwnd_btn_cancel: std::ptr::null_mut(),
            hwnd_btn_back: std::ptr::null_mut(),
            hwnd_btn_next: std::ptr::null_mut(),
            brush_bg,
            brush_header,
            brush_card,
            brush_input,
            font_title,
            font_subtitle,
            font_desc,
            font_body,
            font_bold,
            font_sub,
        });

        G_UNINSTALLER_STATE = Some(state);

        create_uninstaller_controls(hwnd);
        update_uninstaller_page();

        ffi::ShowWindow(hwnd, ffi::SW_SHOW);
        ffi::UpdateWindow(hwnd);

        let mut msg: ffi::MSG = std::mem::zeroed();
        while ffi::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            ffi::TranslateMessage(&msg);
            ffi::DispatchMessageW(&msg);
        }
    }

    Ok(())
}

unsafe fn create_uninstaller_controls(hwnd: ffi::HWND) {
    if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
        let hinstance = ffi::GetModuleHandleW(std::ptr::null());
        let static_class = to_wide_null("STATIC");
        let btn_class = to_wide_null("BUTTON");
        let edit_class = to_wide_null("EDIT");
        let prog_class = to_wide_null("msctls_progress32");

        let mut rc: ffi::RECT = std::mem::zeroed();
        ffi::GetClientRect(hwnd, &mut rc);
        let client_w = (rc.right - rc.left).max(900);
        let client_h = (rc.bottom - rc.top).max(620);
        let content_w = client_w - 70;
        let bar_y = client_h - 58;

        // Header Title
        state.hwnd_title = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("QAULIUM QUANTUM BROWSER").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE,
            35, 18, content_w, 30, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_title, ffi::WM_SETFONT, state.font_title as usize, 1);

        // Header Subtitle
        state.hwnd_subtitle = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("Uninstallation Wizard").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE,
            35, 50, content_w, 24, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_subtitle, ffi::WM_SETFONT, state.font_subtitle as usize, 1);

        // Page Description (Static label at Y=105, H=45)
        state.hwnd_desc = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE,
            35, 105, content_w, 45, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_desc, ffi::WM_SETFONT, state.font_desc as usize, 1);

        // Welcome / Process Info Box
        state.hwnd_info_box = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::ES_MULTILINE | ffi::ES_READONLY,
            35, 160, content_w, 180, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_info_box, ffi::WM_SETFONT, state.font_body as usize, 1);

        // Running Process Warning & Retry
        state.hwnd_lbl_warning = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD,
            35, 360, content_w, 45, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_warning, ffi::WM_SETFONT, state.font_bold as usize, 1);

        state.hwnd_btn_retry = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Retry Detection").as_ptr(),
            ffi::WS_CHILD | ffi::WS_TABSTOP,
            35, 415, 170, 36, hwnd, ID_BTN_RETRY as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_btn_retry, ffi::WM_SETFONT, state.font_bold as usize, 1);

        state.hwnd_lbl_clean = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("[✓] No active Qaulium browser processes detected. Safe to continue.").as_ptr(),
            ffi::WS_CHILD,
            35, 360, content_w, 30, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_clean, ffi::WM_SETFONT, state.font_bold as usize, 1);

        // Options Page: Radio Button 1 (Keep)
        state.hwnd_rad_keep = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Keep personal data (Recommended)").as_ptr(),
            ffi::WS_CHILD | ffi::BS_AUTORADIOBUTTON | ffi::WS_TABSTOP,
            40, 165, content_w - 10, 26, hwnd, ID_RAD_KEEP as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_rad_keep, ffi::WM_SETFONT, state.font_bold as usize, 1);
        ffi::SendMessageW(state.hwnd_rad_keep, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);

        state.hwnd_lbl_keep_sub = ffi::CreateWindowExW(
            0, static_class.as_ptr(),
            to_wide_null("Preserves your bookmarks, history, passwords, and cryptographic vault in %LOCALAPPDATA%\\Qaulium so they remain available if you reinstall.").as_ptr(),
            ffi::WS_CHILD,
            65, 195, content_w - 35, 40, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_keep_sub, ffi::WM_SETFONT, state.font_sub as usize, 1);

        // Options Page: Radio Button 2 (Purge)
        state.hwnd_rad_purge = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Remove all Qaulium personal data").as_ptr(),
            ffi::WS_CHILD | ffi::BS_AUTORADIOBUTTON | ffi::WS_TABSTOP,
            40, 250, content_w - 10, 26, hwnd, ID_RAD_PURGE as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_rad_purge, ffi::WM_SETFONT, state.font_bold as usize, 1);

        state.hwnd_lbl_purge_sub = ffi::CreateWindowExW(
            0, static_class.as_ptr(),
            to_wide_null("Completely deletes all application files and personal profile data. Personal Documents, Desktop files, and Downloads are NEVER touched.").as_ptr(),
            ffi::WS_CHILD,
            65, 280, content_w - 35, 40, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_purge_sub, ffi::WM_SETFONT, state.font_sub as usize, 1);

        state.hwnd_lbl_safety_notice = ffi::CreateWindowExW(
            0, static_class.as_ptr(),
            to_wide_null("Safety Guarantee: Windows user documents, Desktop personal files, and the Downloads folder are NEVER deleted under any option.").as_ptr(),
            ffi::WS_CHILD,
            40, 345, content_w - 10, 30, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_lbl_safety_notice, ffi::WM_SETFONT, state.font_sub as usize, 1);

        // Ready Page Box
        state.hwnd_ready_box = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::ES_MULTILINE | ffi::ES_READONLY,
            35, 160, content_w, 310, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_ready_box, ffi::WM_SETFONT, state.font_body as usize, 1);

        // Complete Page Box
        state.hwnd_complete_box = ffi::CreateWindowExW(
            0, edit_class.as_ptr(), to_wide_null("").as_ptr(),
            ffi::WS_CHILD | ffi::ES_MULTILINE | ffi::ES_READONLY,
            35, 160, content_w, 260, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_complete_box, ffi::WM_SETFONT, state.font_body as usize, 1);

        // Progress Bar
        state.hwnd_progress = ffi::CreateWindowExW(
            0, prog_class.as_ptr(), std::ptr::null(),
            ffi::WS_CHILD | ffi::WS_BORDER,
            35, 220, content_w, 32, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_progress, ffi::PBM_SETRANGE32, 0, 100);

        state.hwnd_prog_text = ffi::CreateWindowExW(
            0, static_class.as_ptr(), to_wide_null("Preparing removal...").as_ptr(),
            ffi::WS_CHILD,
            35, 265, content_w, 50, hwnd, std::ptr::null_mut(), hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_prog_text, ffi::WM_SETFONT, state.font_body as usize, 1);

        // Action Buttons
        state.hwnd_btn_cancel = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Cancel").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | ffi::WS_TABSTOP,
            35, bar_y, 120, 38, hwnd, ID_BTN_CANCEL as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_btn_cancel, ffi::WM_SETFONT, state.font_bold as usize, 1);

        state.hwnd_btn_back = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("< Back").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | ffi::WS_TABSTOP,
            client_w - 325, bar_y, 135, 38, hwnd, ID_BTN_BACK as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_btn_back, ffi::WM_SETFONT, state.font_bold as usize, 1);

        state.hwnd_btn_next = ffi::CreateWindowExW(
            0, btn_class.as_ptr(), to_wide_null("Next >").as_ptr(),
            ffi::WS_CHILD | ffi::WS_VISIBLE | ffi::WS_TABSTOP,
            client_w - 175, bar_y, 145, 38, hwnd, ID_BTN_NEXT as ffi::HMENU, hinstance, std::ptr::null_mut()
        );
        ffi::SendMessageW(state.hwnd_btn_next, ffi::WM_SETFONT, state.font_bold as usize, 1);
    }
}

unsafe fn update_uninstaller_page() {
    if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
        let hide = |w: ffi::HWND| if !w.is_null() { ffi::ShowWindow(w, 0); };
        let show = |w: ffi::HWND| if !w.is_null() { ffi::ShowWindow(w, 5); };

        hide(state.hwnd_info_box);
        hide(state.hwnd_lbl_warning);
        hide(state.hwnd_btn_retry);
        hide(state.hwnd_lbl_clean);
        hide(state.hwnd_rad_keep);
        hide(state.hwnd_lbl_keep_sub);
        hide(state.hwnd_rad_purge);
        hide(state.hwnd_lbl_purge_sub);
        hide(state.hwnd_lbl_safety_notice);
        hide(state.hwnd_ready_box);
        hide(state.hwnd_complete_box);
        hide(state.hwnd_progress);
        hide(state.hwnd_prog_text);

        ffi::EnableWindow(state.hwnd_btn_back, 1);
        ffi::EnableWindow(state.hwnd_btn_next, 1);
        ffi::EnableWindow(state.hwnd_btn_cancel, 1);

        match state.page {
            UninstallerPage::ProcessCheck => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 1 of 5: Process Detection & Confirmation").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("This wizard will uninstall Qaulium Quantum Browser v5.0.0 from your system.").as_ptr());

                let welcome_msg = format!(
                    "INSTALLED APPLICATION DETAILS:\r\n\r\n\
                    • Installation Directory:\r\n  {}\r\n\r\n\
                    • Components to be Removed:\r\n\
                      - Primary Browser Executable (QauliumQuantumBrowser.exe)\r\n\
                      - Background Network & Privacy Daemon (qaulium-daemon.exe)\r\n\
                      - Bundled Gecko ESR 140 Runtime & Necko Engine\r\n\
                      - Desktop & Start Menu Shortcuts\r\n\
                      - Windows Installed Apps Registry Registration\r\n\r\n\
                    Before proceeding, setup verifies whether any Qaulium processes are currently open.",
                    state.install_dir.display()
                );
                ffi::SetWindowTextW(state.hwnd_info_box, to_wide_null(&welcome_msg).as_ptr());
                show(state.hwnd_info_box);

                state.running_procs = win32::get_running_qualium_processes();
                if !state.running_procs.is_empty() {
                    let warn = format!("[!] Qaulium is currently running ({})!\r\nPlease close all browser windows and click 'Retry Detection' before continuing.", state.running_procs.join(", "));
                    ffi::SetWindowTextW(state.hwnd_lbl_warning, to_wide_null(&warn).as_ptr());
                    show(state.hwnd_lbl_warning);
                    show(state.hwnd_btn_retry);
                    ffi::EnableWindow(state.hwnd_btn_next, 0);
                } else {
                    show(state.hwnd_lbl_clean);
                    ffi::EnableWindow(state.hwnd_btn_next, 1);
                }

                ffi::EnableWindow(state.hwnd_btn_back, 0);
                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Next >").as_ptr());
            }
            UninstallerPage::Options => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 2 of 5: Personal User Data Options").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Please choose how you want the uninstaller to handle your personal browsing profile:").as_ptr());

                show(state.hwnd_rad_keep);
                show(state.hwnd_lbl_keep_sub);
                show(state.hwnd_rad_purge);
                show(state.hwnd_lbl_purge_sub);
                show(state.hwnd_lbl_safety_notice);

                // Reflect current radio button state
                if state.keep_user_data {
                    ffi::SendMessageW(state.hwnd_rad_keep, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);
                    ffi::SendMessageW(state.hwnd_rad_purge, ffi::BM_SETCHECK, ffi::BST_UNCHECKED, 0);
                } else {
                    ffi::SendMessageW(state.hwnd_rad_purge, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);
                    ffi::SendMessageW(state.hwnd_rad_keep, ffi::BM_SETCHECK, ffi::BST_UNCHECKED, 0);
                }

                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Next >").as_ptr());
            }
            UninstallerPage::Ready => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 3 of 5: Ready to Remove").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("The uninstaller has all required information and is ready to remove Qaulium Quantum Browser.").as_ptr());

                let data_choice_str = if state.keep_user_data {
                    "PRESERVE USER DATA (Recommended)\r\n    Retains your bookmarks, settings, and vault in %LOCALAPPDATA%\\Qaulium."
                } else {
                    "PURGE ALL USER DATA\r\n    Permanently removes %LOCALAPPDATA%\\Qaulium alongside application files."
                };

                let ready_summary = format!(
                    "UNINSTALLATION SPECIFICATION:\r\n\r\n\
                    • Application Directory to Delete:\r\n  {}\r\n\r\n\
                    • Shortcuts to Remove:\r\n  - Desktop: Qaulium Quantum Browser.lnk\r\n  - Start Menu: Programs\\Qaulium\r\n\r\n\
                    • Windows Registry Entry to Delete:\r\n  HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\QauliumQuantumBrowser\r\n\r\n\
                    • User Profile Data Action:\r\n  {}\r\n\r\n\
                    Click 'Uninstall' to begin removing the product.",
                    state.install_dir.display(),
                    data_choice_str
                );
                ffi::SetWindowTextW(state.hwnd_ready_box, to_wide_null(&ready_summary).as_ptr());
                show(state.hwnd_ready_box);

                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Uninstall").as_ptr());
            }
            UninstallerPage::Removing => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 4 of 5: Removing Application Files...").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Deleting application components, shortcuts, and registry registrations:").as_ptr());

                show(state.hwnd_progress);
                show(state.hwnd_prog_text);

                ffi::EnableWindow(state.hwnd_btn_back, 0);
                ffi::EnableWindow(state.hwnd_btn_next, 0);
                ffi::EnableWindow(state.hwnd_btn_cancel, 0);

                start_uninstall_worker(state.hwnd);
            }
            UninstallerPage::Complete => {
                ffi::SetWindowTextW(state.hwnd_subtitle, to_wide_null("Step 5 of 5: Uninstallation Complete").as_ptr());
                ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Qaulium Quantum Browser v5.0.0 has been successfully removed from your computer.").as_ptr());

                let done_msg = format!(
                    "UNINSTALLATION SUCCESSFUL\r\n\r\n\
                    • Application files removed from: {}\r\n\
                    • Desktop and Start Menu shortcuts removed.\r\n\
                    • Windows Installed Apps registration cleaned.\r\n\
                    • User Data Status: {}\r\n\r\n\
                    The uninstaller will now clean up remaining uninstall artifacts and exit.\r\n\
                    Click 'Close' to finish.",
                    state.install_dir.display(),
                    if state.keep_user_data { "Preserved in %LOCALAPPDATA%\\Qaulium" } else { "Completely Purged" }
                );
                ffi::SetWindowTextW(state.hwnd_complete_box, to_wide_null(&done_msg).as_ptr());
                show(state.hwnd_complete_box);

                hide(state.hwnd_progress);
                hide(state.hwnd_prog_text);
                hide(state.hwnd_btn_cancel);

                ffi::EnableWindow(state.hwnd_btn_back, 0);
                ffi::EnableWindow(state.hwnd_btn_next, 1);
                ffi::SetWindowTextW(state.hwnd_btn_next, to_wide_null("Close").as_ptr());
            }
        }

        let mut rc: ffi::RECT = std::mem::zeroed();
        ffi::GetClientRect(state.hwnd, &mut rc);
        ffi::InvalidateRect(state.hwnd, &rc, 1);
    }
}

fn start_uninstall_worker(hwnd: ffi::HWND) {
    let engine = unsafe {
        if let Some(state) = G_UNINSTALLER_STATE.as_ref() {
            state.engine.clone()
        } else {
            return;
        }
    };

    let keep_data = unsafe {
        if let Some(state) = G_UNINSTALLER_STATE.as_ref() {
            state.keep_user_data
        } else {
            true
        }
    };

    let hwnd_val = hwnd as usize;
    thread::spawn(move || {
        let hwnd = hwnd_val as ffi::HWND;
        let res = engine.execute_uninstall(keep_data, |cur, tot, op, item| {
            let pct = if tot > 0 { (cur as f64 / tot as f64 * 100.0) as usize } else { 0 };
            let msg = format!("{}: {}", op, item);
            let boxed = Box::into_raw(Box::new(msg)) as isize;
            unsafe {
                ffi::PostMessageW(hwnd, WM_UNINSTALL_PROGRESS, pct, boxed);
            }
        });

        match res {
            Ok(_) => unsafe {
                ffi::PostMessageW(hwnd, WM_UNINSTALL_DONE, 0, 0);
            },
            Err(e) => {
                let err_msg = e.to_string();
                let boxed = Box::into_raw(Box::new(err_msg)) as isize;
                unsafe {
                    ffi::PostMessageW(hwnd, WM_UNINSTALL_ERROR, 0, boxed);
                }
            }
        }
    });
}

unsafe extern "system" fn uninstaller_wndproc(
    hwnd: ffi::HWND,
    msg: ffi::UINT,
    wparam: ffi::WPARAM,
    lparam: ffi::LPARAM,
) -> ffi::LRESULT {
    match msg {
        ffi::WM_PAINT => {
            let mut ps: ffi::PAINTSTRUCT = std::mem::zeroed();
            let hdc = ffi::BeginPaint(hwnd, &mut ps);

            if let Some(state) = G_UNINSTALLER_STATE.as_ref() {
                let mut rc_client: ffi::RECT = std::mem::zeroed();
                ffi::GetClientRect(hwnd, &mut rc_client);
                let w = rc_client.right - rc_client.left;
                let h = rc_client.bottom - rc_client.top;

                let rect_header = ffi::RECT { left: 0, top: 0, right: w, bottom: 85 };
                ffi::FillRect(hdc, &rect_header, state.brush_header);

                let rect_line1 = ffi::RECT { left: 0, top: 85, right: w, bottom: 86 };
                let brush_line = ffi::CreateSolidBrush(0x00503525);
                ffi::FillRect(hdc, &rect_line1, brush_line);

                let bar_line_y = h - 68;
                let rect_line2 = ffi::RECT { left: 0, top: bar_line_y, right: w, bottom: bar_line_y + 1 };
                ffi::FillRect(hdc, &rect_line2, brush_line);

                ffi::DeleteObject(brush_line as *mut std::ffi::c_void);
            }

            ffi::EndPaint(hwnd, &ps);
            0
        }
        ffi::WM_CTLCOLORSTATIC => {
            let hdc = wparam as ffi::HDC;
            ffi::SetBkMode(hdc, 1);
            if let Some(state) = G_UNINSTALLER_STATE.as_ref() {
                let ctl_hwnd = lparam as ffi::HWND;
                if ctl_hwnd == state.hwnd_title {
                    ffi::SetTextColor(hdc, COLOR_ALERT_RED);
                    return state.brush_header as ffi::LRESULT;
                } else if ctl_hwnd == state.hwnd_subtitle {
                    ffi::SetTextColor(hdc, COLOR_TEXT_MUTED);
                    return state.brush_header as ffi::LRESULT;
                } else if ctl_hwnd == state.hwnd_lbl_warning {
                    ffi::SetTextColor(hdc, COLOR_ALERT_YELLOW);
                    return state.brush_bg as ffi::LRESULT;
                } else if ctl_hwnd == state.hwnd_lbl_clean {
                    ffi::SetTextColor(hdc, COLOR_ALERT_GREEN);
                    return state.brush_bg as ffi::LRESULT;
                } else if ctl_hwnd == state.hwnd_lbl_keep_sub
                    || ctl_hwnd == state.hwnd_lbl_purge_sub
                    || ctl_hwnd == state.hwnd_lbl_safety_notice
                {
                    ffi::SetTextColor(hdc, COLOR_TEXT_MUTED);
                    return state.brush_bg as ffi::LRESULT;
                } else {
                    ffi::SetTextColor(hdc, COLOR_TEXT_WHITE);
                    return state.brush_bg as ffi::LRESULT;
                }
            }
            0
        }
        ffi::WM_CTLCOLOREDIT => {
            let hdc = wparam as ffi::HDC;
            ffi::SetTextColor(hdc, COLOR_TEXT_WHITE);
            ffi::SetBkColor(hdc, COLOR_BG_CARD);
            if let Some(state) = G_UNINSTALLER_STATE.as_ref() {
                return state.brush_card as ffi::LRESULT;
            }
            0
        }
        ffi::WM_CTLCOLORBTN => {
            let hdc = wparam as ffi::HDC;
            ffi::SetBkMode(hdc, 1);
            ffi::SetTextColor(hdc, COLOR_TEXT_WHITE);
            if let Some(state) = G_UNINSTALLER_STATE.as_ref() {
                return state.brush_bg as ffi::LRESULT;
            }
            0
        }
        ffi::WM_COMMAND => {
            let cmd_id = (wparam & 0xFFFF) as usize;
            match cmd_id {
                ID_BTN_CANCEL => {
                    ffi::PostQuitMessage(0);
                }
                ID_BTN_RETRY => {
                    if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
                        state.running_procs = win32::get_running_qualium_processes();
                        update_uninstaller_page();
                    }
                }
                ID_RAD_KEEP => {
                    if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
                        state.keep_user_data = true;
                        ffi::SendMessageW(state.hwnd_rad_keep, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);
                        ffi::SendMessageW(state.hwnd_rad_purge, ffi::BM_SETCHECK, ffi::BST_UNCHECKED, 0);
                    }
                }
                ID_RAD_PURGE => {
                    if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
                        state.keep_user_data = false;
                        ffi::SendMessageW(state.hwnd_rad_purge, ffi::BM_SETCHECK, ffi::BST_CHECKED, 0);
                        ffi::SendMessageW(state.hwnd_rad_keep, ffi::BM_SETCHECK, ffi::BST_UNCHECKED, 0);
                    }
                }
                ID_BTN_BACK => {
                    if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
                        state.page = match state.page {
                            UninstallerPage::Options => UninstallerPage::ProcessCheck,
                            UninstallerPage::Ready => UninstallerPage::Options,
                            _ => state.page,
                        };
                        update_uninstaller_page();
                    }
                }
                ID_BTN_NEXT => {
                    if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
                        match state.page {
                            UninstallerPage::ProcessCheck => {
                                state.page = UninstallerPage::Options;
                                update_uninstaller_page();
                            }
                            UninstallerPage::Options => {
                                state.keep_user_data = ffi::SendMessageW(state.hwnd_rad_keep, ffi::BM_GETCHECK, 0, 0) == ffi::BST_CHECKED as isize;
                                state.page = UninstallerPage::Ready;
                                update_uninstaller_page();
                            }
                            UninstallerPage::Ready => {
                                state.page = UninstallerPage::Removing;
                                update_uninstaller_page();
                            }
                            UninstallerPage::Complete => {
                                ffi::PostQuitMessage(0);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            0
        }
        WM_UNINSTALL_PROGRESS => {
            let pct = wparam as isize;
            let msg_ptr = lparam as *mut String;
            if !msg_ptr.is_null() {
                let msg = Box::from_raw(msg_ptr);
                if let Some(state) = G_UNINSTALLER_STATE.as_ref() {
                    ffi::SendMessageW(state.hwnd_progress, ffi::PBM_SETPOS, pct as usize, 0);
                    ffi::SetWindowTextW(state.hwnd_prog_text, to_wide_null(&msg).as_ptr());
                }
            }
            0
        }
        WM_UNINSTALL_DONE => {
            if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
                state.page = UninstallerPage::Complete;
                update_uninstaller_page();
            }
            0
        }
        WM_UNINSTALL_ERROR => {
            let err_ptr = lparam as *mut String;
            if !err_ptr.is_null() {
                let err_msg = Box::from_raw(err_ptr);
                if let Some(state) = G_UNINSTALLER_STATE.as_mut() {
                    let err_display = format!("Uninstallation Encountered Issues:\r\n\r\n{}\r\n\r\nSome files may have been locked by running processes.", err_msg);
                    ffi::SetWindowTextW(state.hwnd_desc, to_wide_null("Uninstallation error:").as_ptr());
                    ffi::SetWindowTextW(state.hwnd_info_box, to_wide_null(&err_display).as_ptr());
                    ffi::ShowWindow(state.hwnd_info_box, 5);
                    ffi::ShowWindow(state.hwnd_progress, 0);
                    ffi::ShowWindow(state.hwnd_prog_text, 0);
                    ffi::EnableWindow(state.hwnd_btn_cancel, 1);
                }
            }
            0
        }
        ffi::WM_DESTROY => {
            ffi::PostQuitMessage(0);
            0
        }
        _ => ffi::DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn create_font(name: &str, size_pt: i32, weight: i32) -> ffi::HFONT {
    let wide = to_wide_null(name);
    ffi::CreateFontW(
        -size_pt, 0, 0, 0, weight, 0, 0, 0,
        1,
        0, 0, 5,
        0, wide.as_ptr()
    )
}

fn browse_for_folder(hwnd_owner: ffi::HWND, title: &str) -> Option<PathBuf> {
    unsafe {
        let mut display_name = [0u16; 260];
        let title_wide = to_wide_null(title);
        let mut bi = ffi::BROWSEINFOW {
            hwndOwner: hwnd_owner,
            pidlRoot: std::ptr::null(),
            pszDisplayName: display_name.as_mut_ptr(),
            lpszTitle: title_wide.as_ptr(),
            ulFlags: ffi::BIF_RETURNONLYFSDIRS | ffi::BIF_NEWDIALOGSTYLE,
            lpfn: std::ptr::null(),
            lParam: 0,
            iImage: 0,
        };

        let pidl = ffi::SHBrowseForFolderW(&mut bi);
        if !pidl.is_null() {
            let mut path_buf = [0u16; 260];
            if ffi::SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr()) != 0 {
                ffi::CoTaskMemFree(pidl);
                let len = path_buf.iter().position(|&c| c == 0).unwrap_or(260);
                let s = String::from_utf16_lossy(&path_buf[..len]);
                return Some(PathBuf::from(s));
            }
            ffi::CoTaskMemFree(pidl);
        }
    }
    None
}
