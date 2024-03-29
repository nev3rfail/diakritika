pub(crate) mod keyboard;
pub(crate) mod keyboard_vk;
pub(crate) mod window;

use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY::{VK_LMENU, VK_RMENU, VK_RWIN};
use crate::win::MapType::MAPVK_VK_TO_CHAR;
use num_derive::FromPrimitive;
use std::ffi::{c_int, OsString};
use std::os::windows::prelude::OsStringExt;
use std::ptr;
use std::ptr::{null, null_mut};
use winapi::shared::minwindef::{DWORD, HKL};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::FormatMessageW;
use winapi::um::winnt::{LPWSTR, WCHAR};
use winapi::um::winuser::{
    GetForegroundWindow, GetKeyboardLayout, GetKeyboardState, GetWindowThreadProcessId,
    LoadKeyboardLayoutW, MapVirtualKeyExW, MapVirtualKeyW, ToUnicode, ToUnicodeEx, VkKeyScanW,
    MAPVK_VK_TO_VSC, VK_LWIN,
};
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(PartialEq, FromPrimitive, Debug)]
enum KEYBOARD_HOOK {
    WM_KEYDOWN = 256u32,
    WM_KEYUP = 257,
    WM_SYSKEYDOWN = 260,
    WM_SYSKEYUP = 261,
}
pub const HC_ACTION: c_int = 0;
#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(PartialEq, FromPrimitive)]
enum MessageType {
    WM_NCCREATE = 129,
    WM_QUIT = 18,
    WM_CREATE = 1,
    WM_DESTROY = 2,
}

#[allow(non_camel_case_types)]
pub type VIRTUAL_KEY = u32;

pub trait ToChar {
    fn to_char(&self) -> char;

    fn to_char_localized(&self) -> char;
}

pub trait ToUnicode {
    fn to_unicode(&self) -> Option<String>;

    fn to_unicode_localized(&self) -> Option<String>;
}

pub trait ToScanCode {
    fn to_code(&self) -> u32;
}

#[allow(non_camel_case_types)]
#[repr(u32)]
enum MapType {
    MAPVK_VK_TO_VSC = 0, // 	The uCode parameter is a virtual-key code and is translated into a scan code. If it is a virtual-key code that does not distinguish between left- and right-hand keys, the left-hand scan code is returned. If there is no translation, the function returns 0.
    MAPVK_VSC_TO_VK = 1, // 	The uCode parameter is a scan code and is translated into a virtual-key code that does not distinguish between left- and right-hand keys. If there is no translation, the function returns 0.
    MAPVK_VK_TO_CHAR = 2, //	The uCode parameter is a virtual-key code and is translated into an unshifted character value in the low order word of the return value. Dead keys (diacritics) are indicated by setting the top bit of the return value. If there is no translation, the function returns 0. See Remarks.
    MAPVK_VSC_TO_VK_EX = 3, // 	The uCode parameter is a scan code and is translated into a virtual-key code that distinguishes between left- and right-hand keys. If there is no translation, the function returns 0.
    MAPVK_VK_TO_VSC_EX = 4,
}

impl ToChar for VIRTUAL_KEY {
    fn to_char(&self) -> char {
        to_char(*self, null_mut())
    }

    fn to_char_localized(&self) -> char {
        to_char(*self, get_foreground_window_keyboard_layout())
    }
}

pub fn is_meta_or_alt(key: VIRTUAL_KEY) -> bool {
    key == VK_LMENU as u32
        || key == VK_RMENU as u32
        || key == VK_LWIN as u32
        || key == VK_RWIN as u32
}

pub fn char_to_vk_key_scan(ch: char) -> (u8, u8) {
    // Returns (virtual key code, shift state)
    unsafe {
        let vk = VkKeyScanW(ch as WCHAR) as u16;
        let vk_code = (vk & 0xFF) as u8; // Low-order byte contains the virtual key code
        let shift_state = ((vk >> 8) & 0xFF) as u8; // High-order byte contains the shift state
        (vk_code, shift_state)
    }
}

impl ToUnicode for VIRTUAL_KEY {
    fn to_unicode(&self) -> Option<String> {
        to_unicode(*self, null_mut())
    }

    fn to_unicode_localized(&self) -> Option<String> {
        to_unicode(*self, get_foreground_window_keyboard_layout())
    }
}

impl ToScanCode for VIRTUAL_KEY {
    fn to_code(&self) -> u32 {
        unsafe { MapVirtualKeyW(*self, MAPVK_VK_TO_VSC) }
    }
}

fn to_char(key: VIRTUAL_KEY, locale: HKL) -> char {
    let char = unsafe { MapVirtualKeyExW(key, MAPVK_VK_TO_CHAR as u32, locale) };
    char::try_from(char)
        .unwrap_or_else(|e| panic!("Failed to extract char from {} [{}]: {}", key, char, e))
}

fn to_unicode(key: VIRTUAL_KEY, locale: HKL) -> Option<String> {
    //let buf = [0u16;2];
    let mut key_state = [0u8; 256]; // Array to hold the state of each key

    // Populate key_state with the current state of each key
    let the_key_state = unsafe {
        if GetKeyboardState(key_state.as_mut_ptr()) == 0 {
            log::trace!("Failed to get keyboard state: {}", get_last_error_message());
            null()
        } else {
            key_state.as_ptr()
        }
    };

    let mut buffer: [u16; 5] = [0; 5]; // Buffer to receive the translated character
    let buffer_size = buffer.len() as c_int;

    let result = unsafe {
        ToUnicodeEx(
            key,
            MapVirtualKeyW(key, MAPVK_VK_TO_VSC),
            the_key_state,
            buffer.as_mut_ptr(),
            buffer_size,
            0, // Flags set to 0 for default behavior
            locale,
        )
    };

    if result > 0 {
        // Successfully translated the key press into a Unicode character
        let translated_chars = &buffer[..result as usize];
        // Handle or display the translated characters as needed
        let s = String::from_utf16(translated_chars);
        //log::trace!("Translated characters: {:?}", translated_chars);

        Some(s.expect("Can't convert translated_chars to String"))
    } else {
        None
    }
}

fn get_last_error_message() -> String {
    unsafe {
        let error_code = GetLastError();

        let mut buffer: LPWSTR = null_mut();
        let buffer_size = FormatMessageW(
            winapi::um::winbase::FORMAT_MESSAGE_ALLOCATE_BUFFER
                | winapi::um::winbase::FORMAT_MESSAGE_FROM_SYSTEM
                | winapi::um::winbase::FORMAT_MESSAGE_IGNORE_INSERTS,
            ptr::null_mut(),
            error_code,
            winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_NEUTRAL,
                winapi::um::winnt::SUBLANG_DEFAULT,
            ) as u32,
            (&mut buffer as *mut _ as LPWSTR) as LPWSTR,
            0,
            null_mut(),
        );

        let message = if buffer_size > 0 {
            let message =
                OsString::from_wide(std::slice::from_raw_parts(buffer, buffer_size as usize));
            winapi::um::winbase::LocalFree(buffer as *mut winapi::ctypes::c_void);
            message.to_string_lossy().into_owned()
        } else {
            "Failed to retrieve error message.".to_owned()
        };
        format!("[{}]: {}", error_code, message)
    }
}

fn get_foreground_window_keyboard_layout() -> HKL {
    unsafe {
        let hwnd = GetForegroundWindow(); // Get handle to the foreground window
        let mut process_id: DWORD = 0;
        let thread_id = GetWindowThreadProcessId(hwnd, &mut process_id as *mut DWORD); // Get thread ID
        let layout = GetKeyboardLayout(thread_id); // Get the keyboard layout for the thread
        if layout.is_null() {
            log::trace!("{}", get_last_error_message());
        }
        layout
    }
}

pub(crate) fn load_preload_keyboard_layouts() {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let preload_key_result = hklm.open_subkey("Keyboard Layout\\Preload");

    if preload_key_result.is_err() {
        log::trace!("Failed to open registry key for keyboard layout preload.");
        return;
    }

    let preload_key = preload_key_result.unwrap();

    for i in 1.. {
        let value_name = format!("{}", i);
        let layout_id_result: Result<String, _> = preload_key.get_value(&value_name);

        match layout_id_result {
            Ok(layout_id) => {
                // Convert the layout ID to the format expected by LoadKeyboardLayout (e.g., "00000409")
                let layout_id_wide = format!("{:08}", layout_id)
                    .encode_utf16()
                    .chain(Some(0))
                    .collect::<Vec<u16>>();

                unsafe {
                    let _hkl = LoadKeyboardLayoutW(layout_id_wide.as_ptr(), 0);
                    // Note: You might want to check if hkl is null (0) which indicates failure to load the layout.
                }
            }
            Err(_) => break, // No more preload entries
        }
    }
}
