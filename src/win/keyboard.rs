use std::fmt::Formatter;
use std::ptr;
use winapi::um::winuser::{CallNextHookEx, KBDLLHOOKSTRUCT};
use crate::keymanager::KEY_MANAGER_INSTANCE;
use crate::win::{HC_ACTION, KEYBOARD_HOOK, MessageType, ToChar, ToUnicode};
use num_traits::FromPrimitive;
use std::fmt::Debug;

struct KBDStructWrapper(KBDLLHOOKSTRUCT);

impl Debug for KBDStructWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "KBDLLHOOKSTRUCT vk: {}, scan: {}, flags: {}, time: {}, extra: {} | char [{}|{}] | unicode [{:?}|{:?}]", self.0.vkCode, self.0.scanCode, self.0.flags, self.0.time, self.0.dwExtraInfo, self.0.vkCode.to_char(), self.0.vkCode.to_char_localized(), self.0.vkCode.to_unicode(), self.0.vkCode.to_unicode_localized())
    }
}

pub extern "system" fn keyboard_hook_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    let handled = if n_code == HC_ACTION {
        if let Some(ev) = KEYBOARD_HOOK::from_u32(w_param as u32) {
            let kbd_struct = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };
            match ev {
                KEYBOARD_HOOK::WM_KEYDOWN | KEYBOARD_HOOK::WM_SYSKEYDOWN => {
                    println!("press: {:?}", KBDStructWrapper(kbd_struct));
                    let result = &KEY_MANAGER_INSTANCE.write().keydown(kbd_struct.vkCode as _);
                    if *result == true {
                        //println!("IT FUKKEN WORKED?");
                        Some(1)
                    } else {
                        None
                    }
                }
                KEYBOARD_HOOK::WM_KEYUP | KEYBOARD_HOOK::WM_SYSKEYUP => {
                    let result = &KEY_MANAGER_INSTANCE.write().keyup(kbd_struct.vkCode as _);

                    None
                }
                //_ => {}
            }
            //None//Some(1)
        } else {
            None
        }
    } else {
        None
    };

    return match handled {
        None => {
            println!("[KBD] aNo one handled message, redirecting to the next hook :(");
            unsafe { CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param) }
        }
        Some(res) => {
            res
        }
    };
}