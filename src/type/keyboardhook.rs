use crate::r#type::hook::{HookContainer, HookMetadata};
use crate::win::VIRTUAL_KEY;
use anyhow::Error;
use std::any::Any;
use crate::r#type::Dump;

use crate::r#type::hotkeymanager::Key::VirtualKey;
use crate::r#type::hotkeymanager::PressedKeys;
use crate::win::keyboard::KBDStructWrapper;

pub struct KeyManager(PressedKeys, Vec<HookContainer>);

pub enum KeyboardHookMetadata {
    Press {
        key: VIRTUAL_KEY,
        injected: bool,
        pressed_keys: PressedKeys,
        pressed_keys_before_hook: PressedKeys,
        //key_manager: &'a KeyManager
    },
    Release {
        key: VIRTUAL_KEY,
        injected: bool,
        pressed_keys: PressedKeys,
        pressed_keys_before_hook: PressedKeys,
    },
}
impl HookMetadata for KeyboardHookMetadata {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl KeyboardHookMetadata {
    pub fn pressed_keys(&self) -> &PressedKeys {
        match &self {
            KeyboardHookMetadata::Press { pressed_keys, .. } => pressed_keys,
            KeyboardHookMetadata::Release {
                pressed_keys_before_hook,
                ..
            } => pressed_keys_before_hook,
        }
    }

    pub fn key(&self) -> &VIRTUAL_KEY {
        match &self {
            KeyboardHookMetadata::Press { key, .. } => key,
            KeyboardHookMetadata::Release { key, .. } => key,
        }
    }
    pub fn pressing(&self) -> bool {
        match &self {
            KeyboardHookMetadata::Press { .. } => true,
            KeyboardHookMetadata::Release { .. } => false,
        }
    }

    pub fn releasing(&self) -> bool {
        match &self {
            KeyboardHookMetadata::Press { .. } => false,
            KeyboardHookMetadata::Release { .. } => true,
        }
    }

    pub fn injected(&self) -> bool {
        match &self {
            KeyboardHookMetadata::Press { injected, .. } => injected == &true,
            KeyboardHookMetadata::Release { injected, .. } => injected == &true,
        }
    }
}

impl KeyManager {
    pub(crate) fn with_storage(storage: PressedKeys) -> Self {
        Self(storage, Vec::new())
    }

    pub fn keydown(&mut self, key: VIRTUAL_KEY, injected: bool, raw: KBDStructWrapper) -> bool {
        let old_pressed = self.0.clone();
        let existed = self.0.insert(key);
        if existed {
            log::debug!(target: "KeyboardHook", "Pressing  key: {:width$?}. Keys pressed: {:?} | {:?}", VirtualKey(key), self.dump().dump(), raw, width=15)
        }
        //if existed {
        let mut result = false;
        for (i, item) in self.1.iter().enumerate() {
            result = item
                .trigger(&KeyboardHookMetadata::Press {
                    key,
                    injected,
                    pressed_keys: self.0.clone(),
                    pressed_keys_before_hook: old_pressed.clone(),
                })
                .unwrap_or_else(|e| {
                    log::error!("Error processing hook #{}: {:?}", i, e);
                    false
                });
            log::trace!(
                "Hooking keydown of {:?} resulted in {}",
                VirtualKey(key),
                result
            );
            if result {
                break;
            }
        }
        result
        /*} else {
            false
        }*/
    }

    pub(crate) fn dump(&self) -> &PressedKeys {
        &(self.0)
    }

    pub fn keyup(&mut self, key: VIRTUAL_KEY, injected: bool, raw: KBDStructWrapper) -> bool {
        let old_pressed = self.0.clone();
        let existed = self.0.remove(&key);
        if existed {
            log::debug!(target: "KeyboardHook", "Releasing key: {:width$?}. Keys pressed: {:?} | {:?}", VirtualKey(key), self.dump().dump(), raw, width=15)
        }
        let mut result = false;
        for (i, item) in self.1.iter().enumerate() {
            result = item
                .trigger(&KeyboardHookMetadata::Release {
                    key,
                    injected,
                    pressed_keys: self.0.clone(),
                    pressed_keys_before_hook: old_pressed.clone(),
                })
                .unwrap_or_else(|e| {
                    log::error!("Error processing hook #{}: {:?}", i, e);
                    false
                });
            log::trace!(
                "Hooking keyup of {:?} resulted in {}",
                VirtualKey(key),
                result
            );
            if result {
                break;
            }
        }
        result
        /*} else {
            false
        }*/
    }

    pub fn add_hook<F, T>(&mut self, callback: F, arg: T)
    //{
    //fn new<F, T>(callback: F, arg: T) -> Self
    where
        F: Fn(&dyn HookMetadata, &T) -> Result<bool, Error> + 'static + Send + Sync,
        T: 'static + Send + Sync,
    {
        self.1.push(HookContainer::new(callback, arg));
    }
}
