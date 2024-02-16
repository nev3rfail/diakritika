use std::collections::{HashMap, VecDeque};
use std::ptr;
use std::str::FromStr;
use std::sync::Arc;
use ini::configparser::ini::Ini;
use winapi::shared::minwindef::BYTE;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{COLOR_WINDOW, CreateWindowExW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, DispatchMessageW, GetMessageW, keybd_event, KEYEVENTF_KEYUP, MapVirtualKeyW, MAPVK_VSC_TO_VK, MSG, RegisterClassW, TranslateMessage, WNDCLASSW, WS_OVERLAPPEDWINDOW};
use crate::hotkeymanager::{HasCharacter, HasShift, HasVirtualKey, HOTKEY_MANAGER_INSTANCE, HotkeyManager, Key, KeyBinding};
use crate::keybindings::{bindings_from_map, CharBindingState, CharKeyBindings, Dump, KeyBindings};
use crate::keymanager::KEY_MANAGER_INSTANCE;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY::{VK_LMENU, VK_LSHIFT, VK_MENU, VK_RMENU, VK_RSHIFT, VK_SHIFT};
use crate::win::{char_to_vk_key_scan, load_preload_keyboard_layouts, ToScanCode, VIRTUAL_KEY};
use crate::win::keyboard::{filter_modifier_keys, KeyAction, KeyStroke, KeyType, press_virtual_keys, release_virtual_keys, send_key_sequence, send_unicode_character};

mod win;
mod keymanager;
mod hotkeymanager;
mod keybindings;





fn main() {
    let mut conf = Ini::new();
    let the_conf = conf.load("bindings.ini").unwrap();
    let bindings = bindings_from_map(the_conf);

    println!("Parsed keybindings: {}", bindings.dump());
    let mut state: CharBindingState = HashMap::new();
    bindings.into_iter().for_each(|(char_to_post, key_bindings)| {
        let char_to_post_clone = char_to_post.clone();
        //state.insert(char_to_post_clone, -1);
        key_bindings.into_iter().for_each(move |binding| {
            // Clone char_to_post to move a copy into the closure

            let char_to_post_clone = char_to_post.clone();


            let the_binding = HOTKEY_MANAGER_INSTANCE.lock_arc().add_magic_binding(binding, Box::new(move |triggered| {
                println!("[ACTIVATION] Triggered {:?} on keypress.", triggered);

                let mut pre_keys: Vec<KeyStroke> =  if triggered.0.triggered {
                    Vec::new()
                } else {
                    println!("[ACTIVATION] Hotkey is not yet activated, releasing pressed keys: {:?}", triggered.1);
                    /*filter_modifier_keys*/(&triggered.1).iter()
                        .map(|&vk| KeyStroke::classic(vk, KeyAction::Release))
                        .collect()
                };
                pre_keys.reverse();
                let char_keystroke = KeyStroke::unicode(char_to_post_clone, KeyAction::Press);
                send_key_sequence(&pre_keys, &[char_keystroke], &[], false);
            }), Box::new(move |triggered| {
                println!("[DEACTIVATION] Triggered {:?} on ňkeyrelease.", triggered);
                let mut post_keys: Vec<KeyStroke> =  if triggered.0.triggered {
                    Vec::new()
                } else {
                    println!("[DEACTIVATION] Hotkey is still activated, releasing pressed keys: {:?}", triggered.1);
                    /*filter_modifier_keys*/(&triggered.1).iter()
                        .map(|&vk| KeyStroke::classic(vk, KeyAction::Press))
                        .collect()
                };
                //post_keys.reverse();
                let char_keystroke = KeyStroke::unicode(char_to_post_clone, KeyAction::Release);
                send_key_sequence(&[], &[char_keystroke], &post_keys, false);
            }), false);
        });
    });



    create_window()
}


fn clone_with_modifier_if_needed(char_to_post: char, bindings: &KeyBindings, modifier: KNOWN_VIRTUAL_KEY) -> KeyBindings {
    let mut created_bingdings = Vec::new();

    for binding in bindings {
        if binding.has_character() && !binding.has_shift() && char_to_post.is_lowercase() {
            let mut modified_binding = binding.clone();
            modified_binding.insert(0, Key::VirtualKey(modifier as u32)); // Or VK_LSHIFT, if preferred
            created_bingdings.push(modified_binding);
        }
    }

    created_bingdings
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