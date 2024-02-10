use std::ptr;
use std::str::FromStr;
use std::sync::Arc;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{COLOR_WINDOW, CreateWindowExW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, DispatchMessageW, GetMessageW, MSG, RegisterClassW, TranslateMessage, WNDCLASSW, WS_OVERLAPPEDWINDOW};
use crate::hotkeymanager::{HotkeyManager, Key, KeyBinding};
use crate::keymanager::KEY_MANAGER_INSTANCE;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY;
use crate::win::load_preload_keyboard_layouts;

mod win;
mod keymanager;
mod hotkeymanager;

/// Parse binding that looks like lshift+alt+b+0x18
fn parse_binding(binding_str: &str) -> KeyBinding {
    let parts: Vec<&str> = binding_str.split('+').collect();
    parts.iter().map(|&part| {
        if part.starts_with("0x") {
            Key::Scancode(u32::from_str_radix(&part[2..], 16).unwrap())
        } else if part.len() > 1 {
            // VK binding, to be handled later
            Key::VK(KNOWN_VIRTUAL_KEY::from_human(part).expect("From human failed").into()) // Placeholder
        } else {
            Key::Char(part.to_owned())
        }
    }).collect()
}


fn main() {
    //println!("Hello, world!");
    create_window()
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
            CW_USEDEFAULT, CW_USEDEFAULT, 500, 400,
            ptr::null_mut(),
            ptr::null_mut(),
            GetModuleHandleW(ptr::null()),
            ptr::null_mut(),
        );

        /*        let hwnd2 = CreateWindowExW(
                    0,
                    class_name.as_ptr() as _,
                    "My notification Window\0".as_ptr() as _,
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT, CW_USEDEFAULT, 500, 400,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    GetModuleHandleW(ptr::null()),
                    ptr::null_mut(),
                );*/


        //ShowWindow(hwnd, SW_SHOWNORMAL);
        if hwnd.is_null() {
            println!("HWND is nulasdasdфывфывфывфывasdasdasdфывasdфывasdфывasdфывl, dying");
            return;
        }
        /* if hwnd2.is_null() {
             println!("HWND2 is null, dying");
             return;
         }*/

        //load_preload_keyboard_layouts();
        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}