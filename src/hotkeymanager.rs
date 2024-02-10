
use std::collections::{HashMap, HashSet, VecDeque};
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender, SendError};
use std::thread;
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use crate::keymanager::{KEY_MANAGER_INSTANCE, KeyManager};
use crate::win::{ToScanCode, ToUnicode, VIRTUAL_KEY};

/*pub static KEY_MANAGER_INSTANCE: Lazy<RwLock<KeyManager>> = Lazy::new(|| {
    Arc::new(parking_lot::Mutex::new(HotkeyManager::new()));
});
*/
pub static HOTKEY_MANAGER_INSTANCE: Lazy<Arc<parking_lot::Mutex<HotkeyManager>>> = Lazy::new(|| {
    let hotkey_manager = Arc::new(parking_lot::Mutex::new(HotkeyManager::new()));

    KEY_MANAGER_INSTANCE.write().add_hook(|key_manager, hotkey_manager| {
        let pressed_keys = key_manager.dump();
        Ok(hotkey_manager.lock_arc().check_and_trigger(&pressed_keys))
    }, hotkey_manager.clone());

    hotkey_manager
});


pub enum Key {
    VK(VIRTUAL_KEY),
    Char(String),
    Scancode(u32)
}
pub type KeyBinding = Vec<Key>; // Now a Vec to preserve order
type Callback = Box<dyn Fn() + Send + Sync>;

enum BindingAction {
    Callback(Callback),
    Channel(Sender<()>),
    Magic(Sender<()>),
}

struct HotkeyBinding {
    keys: KeyBinding,
    action: BindingAction,
    ordered: bool,
}

pub struct HotkeyManager {
    //bindings: VecDeque<HotkeyBinding>,
    bindings_by_length: HashMap<usize, VecDeque<HotkeyBinding>>
}
pub(crate) trait Bindable {
    fn to_virtual_key(self) -> u32;
}

impl Bindable for u32 {
    fn to_virtual_key(self) -> u32 {
        self
    }
}

impl HotkeyManager {
    pub fn new() -> Self {
        HotkeyManager {
            //bindings: VecDeque::new(),
            bindings_by_length: HashMap::new()
        }
    }



    fn _add_binding(&mut self, keys: KeyBinding, action: BindingAction, ordered: bool) {
            let binding_length = keys.len();
            let binding = HotkeyBinding { keys, action, ordered };

            // Insert the new binding into the appropriate VecDeque based on its length
            // If there's no entry for this length yet, create a new VecDeque
            let bindings_for_length = self.bindings_by_length.entry(binding_length).or_insert_with(VecDeque::new);
            bindings_for_length.push_back(binding);
        }

    pub(crate) fn add_binding(&mut self, keys: KeyBinding, callback: Callback, ordered: bool)  {
        let action = BindingAction::Callback(callback);
        self._add_binding(keys, action, ordered)
    }

    pub(crate) fn add_magic_binding(&mut self, keys: KeyBinding, callback: Callback, ordered: bool) {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            for _ in rx {
                println!("WOW! something arrived!");
                callback();
            }
        });
        self._add_binding(keys, BindingAction::Magic(tx), ordered)
    }

    pub fn add_channel_binding(&mut self, keys: KeyBinding) -> Receiver<()> {
        let (tx, rx) = mpsc::channel();
        self._add_binding(keys, BindingAction::Channel(tx), false);
        rx
    }


    pub(crate) fn check_and_trigger(&self, pressed_keys: &HashSet<VIRTUAL_KEY>) -> bool {
        let pressed_count = pressed_keys.len();
        let mut scancode_cache: HashMap<u32, u32> = HashMap::new(); // Cache for VK to scancode conversions
        let mut char_cache: HashMap<u32, Option<String>> = HashMap::new(); // Cache for VK to character conversions

        // Retrieve the list of bindings that match the number of pressed keys
        if let Some(bindings) = self.bindings_by_length.get(&pressed_count) {
            for binding in bindings {
                if self.is_triggered(pressed_keys, &binding, &mut scancode_cache, &mut char_cache) {

                }
                match &binding.action {
                    BindingAction::Callback(cb) => cb(),
                    BindingAction::Channel(tx) | BindingAction::Magic(tx) => {
                        let sent = tx.send(());
                        match sent {
                            Ok(ok) => {}
                            Err(error) => {
                                println!("BROKEN PIPE: {:?}", error);
                            }
                        }
                    }
                }
                return true;
            }
        }

        false
    }

    fn is_triggered(&self, pressed_keys: &HashSet<u32>, binding: &HotkeyBinding, scancode_cache: &mut HashMap<u32, u32>, char_cache: &mut HashMap<u32, Option<String>>) -> bool {
        // Similar implementation as before, but use the caches for conversions
        // Example for character bindings:
        for key in &binding.keys {
            match key {
                Key::VK(vk) => {
                    if !pressed_keys.contains(vk) {
                        return false;
                    }
                },
                Key::Char(expected_str) => {
                    let pressed_str = pressed_keys.iter().find_map(|&vk| {
                        // Clone the String to ensure the returned value is owned and not a reference
                        char_cache.entry(vk).or_insert_with(|| vk.to_unicode_localized()).clone()
                    });
                    // Compare Option<&String> with &Option<String> using map and as_deref
                    if pressed_str.as_deref() != Some(expected_str) {
                        return false
                    }
                },
                Key::Scancode(expected_sc) => {
                    // Find the pressed scan code by converting each virtual key code using the cache
                    let pressed_sc_match = pressed_keys.iter().any(|&vk| {
                        let pressed_sc = scancode_cache.entry(vk).or_insert_with(|| vk.to_code());
                        pressed_sc == expected_sc
                    });
                    if !pressed_sc_match {
                        return false;
                    }
                },

            }
        }

        true
    }

/*    fn is_triggered(&self, pressed_keys: &HashSet<VIRTUAL_KEY>, binding: &HotkeyBinding) -> bool {
        let result = if binding.ordered {
            binding.keys.iter().all(|f| {
                pressed_keys.contains(&f.vk)
            })
        } else {
            binding.keys.iter().all(|key| pressed_keys.contains(&key.vk))
        };
        result
    }*/
}
