use std::ptr;
use winapi::shared::windef::HWND;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{COLOR_WINDOW, CreateWindowExW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, DefWindowProcW, DeregisterShellHookWindow, DispatchMessageW, GetMessageW, MSG, RegisterClassW, SetWindowsHookExW, TranslateMessage, WNDCLASSW, WS_OVERLAPPEDWINDOW};
use crate::win::{MessageType};
use crate::win::keyboard::keyboard_hook_proc;
pub const WH_KEYBOARD_LL: WINDOWS_HOOK_ID = WINDOWS_HOOK_ID(13i32);
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
impl Default for WINDOWS_HOOK_ID {
    fn default() -> Self {
        Self(0)
    }
}
/*impl ::windows_core::TypeKind for WINDOWS_HOOK_ID {
    type TypeKind = ::windows_core::CopyType;
}*/
impl ::core::fmt::Debug for WINDOWS_HOOK_ID {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_tuple("WINDOWS_HOOK_ID").field(&self.0).finish()
    }
}

pub(crate) unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w_param: usize, l_param: isize) -> isize {
    let handled = match MessageType::from_u32(msg) {
        Some(common) => {
            match common {
                MessageType::WM_CREATE => {
                    println!("registering???");
                    /////////////
                    //////////// keyboard
                    ////////////
                    let hook = SetWindowsHookExW(
                        WH_KEYBOARD_LL.0,
                        Some(keyboard_hook_proc),
                        GetModuleHandleW(ptr::null()),
                        0,
                    );
                    if hook.is_null() {
                        println!("FAILED TO INSTALL KB HOOK: {:?}", GetLastError());
                    }
                    Some(0)
                },
                MessageType::WM_NCCREATE => {
                    Some(1)
                },
                MessageType::WM_QUIT => {
                    let r = DeregisterShellHookWindow(hwnd);
                    println!("HOOK RdeEGISTERED?? {}", r);
                    Some(0)
                }
                MessageType::WM_DESTROY => {
                    winapi::um::winuser::PostQuitMessage(0);
                    Some(0)
                }
            }
        }
        None => {
            None
        }
    };

    return match handled {
        None => {
            println!("No one handled message, redirecting to the next hook :(");
            DefWindowProcW(hwnd, msg, w_param, l_param)
        }
        Some(res) => {
            res
        }
    };
}


