use crate::r#type::hotkeymanager::HotkeyManager;
use crate::r#type::keyboardhook::{KeyManager, KeyboardHookMetadata};
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY::{VK_CONTROL, VK_LWIN, VK_MENU, VK_SHIFT};
use indexmap::IndexSet;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;

pub const CONST_VK_SHIFT: u32 = VK_SHIFT as u32;
pub const CONST_VK_MENU: u32 = VK_MENU as u32;
pub const CONST_VK_CONTROL: u32 = VK_CONTROL as u32;
pub const CONST_VK_WIN: u32 = VK_LWIN as u32;

pub static KEY_MANAGER_INSTANCE: Lazy<RwLock<KeyManager>> =
    Lazy::new(|| RwLock::new(KeyManager::with_storage(IndexSet::with_capacity(20))));

pub static HOTKEY_MANAGER_INSTANCE: Lazy<Arc<parking_lot::Mutex<HotkeyManager>>> =
    Lazy::new(|| {
        let hotkey_manager = Arc::new(parking_lot::Mutex::new(HotkeyManager::new()));

        KEY_MANAGER_INSTANCE.write().add_hook(
            |metadata, hotkey_manager| {
                let s = metadata
                    .as_any()
                    .downcast_ref::<KeyboardHookMetadata>()
                    .expect("Failed to downcast metadata as keyboard hook.");
                Ok(hotkey_manager.lock_arc().check_and_trigger(s))
            },
            hotkey_manager.clone(),
        );

        hotkey_manager
    });
/// This list is used for blocking specific scancodes from entering the Window message queue
///
/// Windows for some odd legacy reason generates an additional keystroke when AltGr is pressed.
/// This leads to bad hotkey interpretation in keystroke processor and in general to some weird side
/// effects because of the way how we treat keybindings with alt pressed.
///
pub const SCANCODE_BLACKLIST: [u32;2] = [0x21d, 0xE038];