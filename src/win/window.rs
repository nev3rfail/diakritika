use crate::win::keyboard::keyboard_hook_proc;
use crate::win::MessageType;
use std::ptr;
use winapi::shared::windef::HWND;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    DefWindowProcW, SetWindowsHookExW, WH_KEYBOARD_LL,
};

use num_traits::FromPrimitive;
#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct WINDOWS_HOOK_ID(pub i32);
impl Copy for WINDOWS_HOOK_ID {}
impl Clone for WINDOWS_HOOK_ID {
    fn clone(&self) -> Self {
        *self
    }
}

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
