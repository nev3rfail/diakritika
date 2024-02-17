use crate::r#static::KEY_MANAGER_INSTANCE;
use crate::r#type::hotkeymanager::Key::VirtualKey;
use crate::r#type::hotkeymanager::PressedKeys;
use crate::win::{ToChar, ToScanCode, ToUnicode, HC_ACTION, KEYBOARD_HOOK, VIRTUAL_KEY};
use num_traits::FromPrimitive;
use std::fmt::Debug;
use std::fmt::Formatter;

use std::ptr;
use winapi::shared::minwindef::{BYTE, DWORD, UINT};
use winapi::um::winuser::{
    keybd_event, CallNextHookEx, SendInput, INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT, KEYBDINPUT,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, LLKHF_INJECTED,
};

use crate::win::keyboard::KeyAction::Press;
use crate::win::keyboard::KeyType::Classic;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY::{
    VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
};

pub const KEYSTROKE_MARKER: usize = 0x666;

struct KBDStructWrapper(KBDLLHOOKSTRUCT);

impl KBDStructWrapper {
    fn is_injected(&self) -> bool {
        self.0.flags & LLKHF_INJECTED != 0
    }
}

fn test_flag(what: DWORD, flag: DWORD) -> bool {
    what & flag != 0
}

impl Debug for KBDStructWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "KBDLLHOOKSTRUCT[{}] vk: {} | {:?}, scan: 0x{:x} ({}), flags: {}, time: {}, extra: {} | char [{}|{}] | unicode [{:?}|{:?}]", if self.is_injected(){ "injected" } else { "raw" },self.0.vkCode, KNOWN_VIRTUAL_KEY::try_from(self.0.vkCode), self.0.scanCode, self.0.scanCode, self.0.flags, self.0.time, self.0.dwExtraInfo, match self.0.vkCode.to_char() {
            '\x1b' => "ESC".to_string(),
            char => char.to_string()
        }, match self.0.vkCode.to_char_localized() {
            '\x1b' => "ESC".to_string(),
            char => char.to_string()
        }, self.0.vkCode.to_unicode(), self.0.vkCode.to_unicode_localized())
    }
}

pub extern "system" fn keyboard_hook_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    let handled = if n_code == HC_ACTION {
        if let Some(ev) = KEYBOARD_HOOK::from_u32(w_param as u32) {
            let kbd_struct = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };

            /// We ignore all of the keystrokes that were produced by us
            if kbd_struct.dwExtraInfo == KEYSTROKE_MARKER {
                //if kbd_struct.vkCode == VK_PACKET as u32 {
                log::debug!(target: "keyboard_hook_proc",
                    "key_ignored: {:?}: {:?}",
                    ev,
                    KBDStructWrapper(kbd_struct)
                );
                None
            } else {
                match ev {
                    KEYBOARD_HOOK::WM_KEYDOWN | KEYBOARD_HOOK::WM_SYSKEYDOWN => {
                        log::debug!(target: "keyboard_hook_proc",
                            "key_press: {:?}: {:?}",
                            ev,
                            KBDStructWrapper(kbd_struct)
                        );
                        let result = &KEY_MANAGER_INSTANCE.write().keydown(
                            kbd_struct.vkCode as _,
                            kbd_struct.flags & LLKHF_INJECTED != 0,
                        );
                        if *result {
                            Some(1)
                        } else {
                            None
                        }
                    }
                    KEYBOARD_HOOK::WM_KEYUP | KEYBOARD_HOOK::WM_SYSKEYUP => {
                        log::trace!(target: "keyboard_hook_proc",
                            "key_release: {:?}: {:?}",
                            ev,
                            KBDStructWrapper(kbd_struct)
                        );
                        let result = &KEY_MANAGER_INSTANCE.write().keyup(
                            kbd_struct.vkCode as _,
                            kbd_struct.flags & LLKHF_INJECTED != 0,
                        );
                        if *result {
                            Some(1)
                        } else {
                            None
                        }
                    }
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    match handled {
        None => unsafe { CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param) },
        Some(res) => res,
    }
}

pub fn virtual_keys<'a, T: AsRef<[KeyStroke]> + IntoIterator<Item = &'a KeyStroke> + Clone>(
    keys: T,
) {
    for stroke in keys {
        log::trace!("{:?}", stroke);
        if stroke.key_type == Classic {
            unsafe {
                keybd_event(
                    stroke.virtual_key as BYTE,
                    stroke.scancode as BYTE,
                    if stroke.action == KeyAction::Release {
                        KEYEVENTF_KEYUP
                    } else {
                        0
                    },
                    KEYSTROKE_MARKER,
                );
            }
        } else {
            send_keystrokes(&[*stroke]);
        }
    }
}

pub fn filter_modifier_keys(vk_list: &PressedKeys) -> Vec<VIRTUAL_KEY> {
    // Define a list of modifier keys
    let modifiers: Vec<VIRTUAL_KEY> = vec![
        VK_SHIFT as u32,  // VK_SHIFT
        VK_LSHIFT as u32, // VK_LSHIFT
        VK_RSHIFT as u32, // VK_RSHIFT
        VK_MENU as u32,   // VK_MENU (Alt)
        VK_LMENU as u32,  // VK_LMENU (LAlt)
        VK_RMENU as u32,  // VK_RMENU (RAlt)
        VK_LWIN as u32,   // VK_LWIN
        VK_RWIN as u32,   // VK_RWIN
    ];

    // Filter the input list to include only the modifier keys
    vk_list
        .iter()
        .cloned()
        .filter(|vk| modifiers.contains(vk))
        .collect()
}

#[derive(Clone, Copy)]
pub(crate) struct KeyStroke {
    key_type: KeyType,
    virtual_key: u32,
    scancode: u32,
    action: KeyAction,
}

impl Debug for KeyStroke {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {:?} {:?} key 0x{:x}",
            if self.action == Press {
                "Pressing"
            } else {
                "Releasing"
            },
            self.key_type,
            VirtualKey(self.virtual_key),
            self.scancode
        )
    }
}

impl KeyStroke {
    pub fn classic(virtual_key: VIRTUAL_KEY, action: KeyAction) -> Self {
        Self {
            key_type: KeyType::Classic,
            virtual_key,
            scancode: virtual_key.to_code(),
            action,
        }
    }

    pub fn unicode(char: char, action: KeyAction) -> Self {
        Self {
            key_type: KeyType::Unicode,
            virtual_key: 0,
            scancode: char as u32,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyType {
    Unicode,
    Classic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyAction {
    Press,
    Release,
}

impl KeyStroke {
    fn clone_as_press(self) -> KeyStroke {
        let mut pew = self.clone();
        pew.action = KeyAction::Press;
        return pew;
    }

    pub(crate) fn clone_as_release(self) -> KeyStroke {
        let mut pew = self;
        pew.action = KeyAction::Release;
        pew
    }
}

fn send_keystrokes<'a, T: AsRef<[KeyStroke]> + IntoIterator<Item = &'a KeyStroke> + Clone>(
    keys: T,
) {
    let iter_count = keys.clone().into_iter().count();
    if iter_count > 0 {
        unsafe {
            let mut inputs = keys
                .into_iter()
                .map(|&key| create_input(key))
                .collect::<Vec<_>>();
            SendInput(
                inputs.len() as UINT,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }
}

pub fn send_key_sequence(pre_keys: &[KeyStroke], the_char: &[KeyStroke], post_keys: &[KeyStroke]) {
    let mut inputs = Vec::new();
    inputs.is_empty();
    inputs.extend(pre_keys);
    inputs.extend(the_char);
    inputs.extend(post_keys);
    virtual_keys(&inputs);
}

fn create_input(stroke: KeyStroke) -> INPUT {
    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: unsafe { std::mem::zeroed() },
    };

    unsafe {
        let ki = match stroke.key_type {
            KeyType::Unicode => {
                KEYBDINPUT {
                    wVk: 0,                        // Virtual-key code is not needed for Unicode input
                    wScan: stroke.scancode as u16, // Unicode character code
                    dwFlags: KEYEVENTF_UNICODE
                        | if stroke.action == KeyAction::Release {
                            KEYEVENTF_KEYUP
                        } else {
                            0
                        },
                    time: 0,
                    dwExtraInfo: KEYSTROKE_MARKER,
                }
            }
            KeyType::Classic => {
                KEYBDINPUT {
                    wVk: 0,                        // Virtual-key code is not needed for scancode input
                    wScan: stroke.scancode as u16, // Scancode
                    dwFlags: KEYEVENTF_SCANCODE
                        | if stroke.action == KeyAction::Release {
                            KEYEVENTF_KEYUP
                        } else {
                            0
                        },
                    time: 0,
                    dwExtraInfo: KEYSTROKE_MARKER,
                }
            }
        };

        *input.u.ki_mut() = ki;
    }

    input
}
