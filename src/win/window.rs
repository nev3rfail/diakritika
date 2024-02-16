use crate::win::keyboard::keyboard_hook_proc;
use crate::win::MessageType;
use std::ptr;
use winapi::shared::windef::HWND;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    SetWindowsHookExW, TranslateMessage, COLOR_WINDOW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW,
    CW_USEDEFAULT, MSG, WH_KEYBOARD_LL, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

use num_traits::FromPrimitive;
pub(crate) unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    let handled = match MessageType::from_u32(msg) {
        Some(common) => {
            match common {
                MessageType::WM_CREATE => {
                    println!("registering???");
                    /////////////
                    //////////// keyboard
                    ////////////
                    let hook = SetWindowsHookExW(
                        WH_KEYBOARD_LL,
                        Some(keyboard_hook_proc),
                        GetModuleHandleW(ptr::null()),
                        0,
                    );
                    if hook.is_null() {
                        println!("FAILED TO INSTALL KB HOOK: {:?}", GetLastError());
                    }
                    Some(0)
                }
                MessageType::WM_NCCREATE => Some(1),
                MessageType::WM_QUIT => Some(0),
                MessageType::WM_DESTROY => {
                    winapi::um::winuser::PostQuitMessage(0);
                    Some(0)
                }
            }
        }
        None => None,
    };

    match handled {
        None => {
            //println!("No one handled message, redirecting to the next hook :(");
            DefWindowProcW(hwnd, msg, w_param, l_param)
        }
        Some(res) => res,
    }
}

pub fn create_window() {
    // Register window class
    unsafe {
        let class_name = "MyWindowClass\0";
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(crate::win::window::wnd_proc);
        wc.hInstance = GetModuleHandleW(ptr::null());
        wc.lpszClassName = class_name.as_ptr() as _;
        wc.hbrBackground = (COLOR_WINDOW + 1) as _;
        wc.style = CS_HREDRAW | CS_VREDRAW | CS_OWNDC;

        RegisterClassW(&wc);

        // Create window
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr() as _,
            "My Window\0".as_ptr() as _,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            500,
            400,
            ptr::null_mut(),
            ptr::null_mut(),
            GetModuleHandleW(ptr::null()),
            ptr::null_mut(),
        );

        if hwnd.is_null() {
            println!("HWND is null, dying");
            return;
        }
        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
