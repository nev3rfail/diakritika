
use std::any::Any;
use std::collections::{HashMap, HashSet};

use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;
use once_cell::sync::{Lazy, OnceCell};

use parking_lot::RwLock;

pub static KEY_MANAGER_INSTANCE: Lazy<RwLock<KeyManager>> = Lazy::new(|| {
    RwLock::new(KeyManager::with_storage(HashSet::with_capacity(20)))
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
pub struct KeyManager(HashSet<VIRTUAL_KEY>, Vec<CallbackContainer>);


use std::ops::Fn;
use anyhow::Error;
use winapi::um::winuser::{MapVirtualKeyW, MAPVK_VK_TO_VSC};
use crate::win::{ToUnicode, VIRTUAL_KEY};

trait CallbackWithArg: Any + Send + Sync {
    fn call(&self, extra: &KeyManager) -> Result<bool, anyhow::Error>;
}

// Implement the trait for a specific combination of closure and argument
impl<F, T> CallbackWithArg for (F, T)
    where
        F: Fn(&KeyManager, &T) -> Result<bool, anyhow::Error> + 'static + Send + Sync,
        T: 'static + Send + Sync,
{
    fn call(&self, extra: &KeyManager) -> Result<bool, anyhow::Error> {
        (self.0)(extra, &self.1)
    }
}

struct CallbackContainer {
    callback_with_arg: Box<dyn CallbackWithArg>,
}

impl CallbackContainer {
    fn new<F, T>(callback: F, arg: T) -> Self
        where
            F: Fn(&KeyManager, &T) -> Result<bool, anyhow::Error> + 'static + Send + Sync,
            T: 'static + Send + Sync,
    {
        CallbackContainer {
            callback_with_arg: Box::new((callback, arg)),
        }
    }

    fn trigger(&self, extra: &KeyManager) -> Result<bool, Error> {
        self.callback_with_arg.call(extra)
    }
}
enum ControlFlow {
    KeepGoing,
    BreakLocal,
    BreakGlobal
}
impl KeyManager {
    pub(crate) fn with_storage(storage: HashSet<VIRTUAL_KEY>) -> Self {
        Self(storage, Vec::new())
    }

    pub fn keydown(&mut self, key: VIRTUAL_KEY) -> bool {
        return if self.0.insert(key) {
            let mut i = 0;
            let mut result = false;
            for item in self.1.iter() {
                result = item.trigger(self).unwrap_or_else(|e| {
                    println!("Error processing hook #{}: {:?}", i, e);
                    false
                });
                i+=1;
                if result == true {
                    break
                }
            }
            result
        } else {
            false
        }
    }

    pub(crate) fn dump(&self) -> &HashSet<VIRTUAL_KEY> {
        &(self.0)
    }


    pub fn keyup(&mut self, key: VIRTUAL_KEY) -> bool {
        //self.2.remove(&scan);
        if let true = self.0.remove(&key) {
            println!("NEW KEY RELEASED! 0{:x}", key);
            true
        } else {
            return false
        }
    }

    pub fn add_hook<F, T>(&mut self, callback: F, arg: T) //{
    //fn new<F, T>(callback: F, arg: T) -> Self
        where
            F: Fn(&KeyManager, &T) -> Result<bool, Error>+ 'static + Send + Sync,
            T: 'static + Send + Sync,
    {
        self.1.push(CallbackContainer {
            callback_with_arg: Box::new((callback, arg)),
        });
    }
}
