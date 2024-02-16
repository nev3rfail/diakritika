
use std::any::Any;
use std::collections::{HashMap, HashSet};

use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;
use once_cell::sync::{Lazy, OnceCell};

use parking_lot::RwLock;

pub static KEY_MANAGER_INSTANCE: Lazy<RwLock<KeyManager>> = Lazy::new(|| {
    RwLock::new(KeyManager::with_storage(IndexSet::with_capacity(20)))
});


#[derive(Eq, Hash, PartialEq, Debug)]
struct KeyPressed {
    scan_code: u32,
    vk: VIRTUAL_KEY,
    unicode: Option<String>,
    localized_unicode: Option<String>
}



impl KeyPressed {
    pub fn from_code(vk: VIRTUAL_KEY) -> Self {
        Self::from_scan_and_code(vk, unsafe { MapVirtualKeyW(vk, MAPVK_VK_TO_VSC as u32) })
    }

    pub fn from_scan_and_code(vk: VIRTUAL_KEY, scan: u32) -> Self {
        Self {
            scan_code: scan,
            vk,
            unicode: vk.to_unicode(),
            localized_unicode: vk.to_unicode_localized()
        }
    }
}
pub struct KeyManager(PressedKeys, Vec<HookContainer>);


use std::ops::Fn;
use anyhow::Error;
use indexmap::IndexSet;
use winapi::um::winuser::{MapVirtualKeyW, MAPVK_VK_TO_VSC};
use crate::hotkeymanager::Key::VirtualKey;
use crate::hotkeymanager::PressedKeys;
use crate::win::{ToUnicode, VIRTUAL_KEY};



trait Hook: Any + Send + Sync {
    fn call(&self, extra: &dyn HookMetadata) -> Result<bool, anyhow::Error>;
}


impl<F, T> Hook for (F, T)
    where
        F: Fn(&dyn HookMetadata, &T) -> Result<bool, anyhow::Error> + 'static + Send + Sync,
        T: 'static + Send + Sync,
{
    fn call(&self, extra: &dyn HookMetadata) -> Result<bool, anyhow::Error> {
        (self.0)(extra, &self.1)
    }
}

struct HookContainer {
    hook: Box<dyn Hook>,
}
impl HookContainer {
    fn new<F, T>(hook: F, arg: T) -> Self
        where
            F: Fn(&dyn HookMetadata, &T) -> Result<bool, anyhow::Error> + 'static + Send + Sync,
            T: 'static + Send + Sync,
    {
        HookContainer {
            hook: Box::new((hook, arg)),
        }
    }

    fn trigger(&self, extra: &dyn HookMetadata) -> Result<bool, Error> {
        self.hook.call(extra)
    }
}
pub trait HookMetadata: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

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
            KeyboardHookMetadata::Press {  pressed_keys, ..} => {
                pressed_keys
            }
            KeyboardHookMetadata::Release { pressed_keys_before_hook , .. } => {
                pressed_keys_before_hook
            }
        }
    }

    pub fn key(&self) -> &VIRTUAL_KEY {
        match &self {
            KeyboardHookMetadata::Press { key, ..} => {
                key
            }
            KeyboardHookMetadata::Release { key, .. } => {
                key
            }
        }
    }
    pub fn pressing(&self) -> bool {
        match &self {
            KeyboardHookMetadata::Press { .. } => {
                true
            }
            KeyboardHookMetadata::Release { .. } => {
                false
            }
        }
    }

    pub fn releasing(&self) -> bool {
        match &self {
            KeyboardHookMetadata::Press { .. } => {
                false
            }
            KeyboardHookMetadata::Release {  .. } => {
                true
            }
        }
    }

    pub fn injected(&self) -> bool {
        match &self {
            KeyboardHookMetadata::Press { injected, .. } => {
                injected == &true
            }
            KeyboardHookMetadata::Release { injected, .. } => {
                injected == &true
            }
        }
    }

}


enum ControlFlow {
    KeepGoing,
    BreakLocal,
    BreakGlobal
}
impl KeyManager {
    pub(crate) fn with_storage(storage: PressedKeys) -> Self {
        Self(storage, Vec::new())
    }

    pub fn keydown(&mut self, key: VIRTUAL_KEY, injected: bool) -> bool {
        let old_pressed = self.0.clone();
        let existed = self.0.insert(key);
        //if existed {
            let mut result = false;
            for (i, item) in self.1.iter().enumerate() {
                result = item.trigger(&KeyboardHookMetadata::Press {
                    key,
                    injected,
                    pressed_keys: self.0.clone(),
                    pressed_keys_before_hook: old_pressed.clone()
                }).unwrap_or_else(|e| {
                    println!("Error processing hookď #{}: {:?}", i, e);
                    false
                });
                println!("Hooking keydown of {:?} resulted in {}", VirtualKey(key), result);
                if result == true {
                    break
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

    pub fn keyup(&mut self, key: VIRTUAL_KEY, injected: bool) -> bool {
        let old_pressed = self.0.clone();
        let existed = self.0.remove(&key);
        //if existed {
            let mut result = false;
            for (i, item) in self.1.iter().enumerate() {
                result = item.trigger(&KeyboardHookMetadata::Release {
                    key,
                    injected,
                    pressed_keys: self.0.clone(),
                    pressed_keys_before_hook: old_pressed.clone()
                }).unwrap_or_else(|e| {
                    println!("Error processing hook #{}: {:?}", i, e);
                    false
                });
                println!("Hooking keyup of {:?} resulted in {}", VirtualKey(key), result);
                if result == true {
                    break
                }
            }
            result
        /*} else {
            false
        }*/
    }

    pub fn add_hook<F, T>(&mut self, callback: F, arg: T) //{
    //fn new<F, T>(callback: F, arg: T) -> Self
        where
            F: Fn(&dyn HookMetadata, &T) -> Result<bool, Error>+ 'static + Send + Sync,
            T: 'static + Send + Sync,
    {
        self.1.push(HookContainer {
            hook: Box::new((callback, arg)),
        });
    }
}
