use std::collections::{HashMap, VecDeque};
use std::ptr;
use std::str::FromStr;
use std::sync::Arc;
use ini::configparser::ini::Ini;
use winapi::shared::minwindef::BYTE;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{COLOR_WINDOW, CreateWindowExW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, DispatchMessageW, GetMessageW, keybd_event, KEYEVENTF_KEYUP, MapVirtualKeyW, MSG, RegisterClassW, TranslateMessage, WNDCLASSW, WS_OVERLAPPEDWINDOW};
use crate::hotkeymanager::{HasCharacter, HasShift, HasVirtualKey, HOTKEY_MANAGER_INSTANCE, HotkeyManager, Key, KeyBinding};
use crate::keybindings::{bindings_from_map, Dump};
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
    //send_unicode_character('є');
    //return;
    //println!("Hello, world!");
    let mut conf = Ini::new();
    let the_conf = conf.load("bindings.ini").unwrap();
    let bindings = bindings_from_map(the_conf);

    println!("Parsed keybindings: {}", bindings.dump());

    bindings.into_iter().for_each(|(char_to_post, key_bindings)| {
        key_bindings.into_iter().for_each(move |binding| {
            // Clone char_to_post to move a copy into the closure
            let char_to_post_clone = char_to_post.clone();

            HOTKEY_MANAGER_INSTANCE.lock_arc().add_magic_binding(binding, Box::new(move |keys| {
                let pre_keys: Vec<KeyStroke> = filter_modifier_keys(&keys).iter()
                    .map(|&vk| KeyStroke { key_type: KeyType::ScanCode, action: KeyAction::Release, scancode: vk.to_code() })
                    .collect();

                /*let post_keys: Vec<KeyStroke> = filter_modifier_keys(&keys).iter()
                    .map(|&vk| KeyStroke { key_type: KeyType::ScanCode, action: KeyAction::Press, scancode: vk.to_code() })
                    .collect();*/

                /*release_virtual_keys(keys.clone());
                send_unicode_character(char_to_post_clone);
                press_virtual_keys(keys);*/
                println!("releasing");
                send_key_sequence(&pre_keys, char_to_post_clone, &[]);
                println!("released");

            }), false);
        });
    });



    create_window()
}
fn modify_bindings(bindings: &mut HashMap<char, Vec<KeyBinding>>) {
    let mut additions = Vec::new();

    for (&char_to_post, key_bindings) in bindings.iter() {
        for binding in key_bindings {
            if binding.has_character() && !binding.has_shift() && char_to_post.is_lowercase() {
                // Create a copy of the binding and prepend a VK_SHIFT
                let mut modified_binding = binding.clone();
                modified_binding.insert(0, Key::VirtualKey(VK_SHIFT.into())); // Or VK_LSHIFT, if preferred

                // Convert char_to_post to uppercase and prepare it for addition to the map
                let uppercase_char = char_to_post.to_uppercase().next().unwrap(); // Safe unwrap, uppercase always returns at least one char
                additions.push((uppercase_char, modified_binding));
            }
        }
    }

    // Add the new bindings to the map
    for (uppercase_char, new_binding) in additions {
        bindings.entry(uppercase_char).or_insert_with(Vec::new).push(new_binding);
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