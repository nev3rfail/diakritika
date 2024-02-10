
use std::any::Any;
use std::collections::{HashMap, HashSet};

use std::hash::{Hash, Hasher};
use once_cell::sync::{Lazy, OnceCell};

use parking_lot::RwLock;

pub static KEY_MANAGER_INSTANCE: Lazy<RwLock<KeyManager>> = Lazy::new(|| {
    RwLock::new(KeyManager::with_storage(HashSet::with_capacity(20)))
});


enum KeyboardKey {
    VK(VIRTUAL_KEY),
    Code(u32),
}

struct PressedKey(
);
struct KeyPressed(
    KeyboardKey, KeyboardKey
);
pub struct KeyManager(HashSet<VIRTUAL_KEY>, Vec<CallbackContainer>);


use std::ops::Fn;
use anyhow::Error;
use crate::win::VIRTUAL_KEY;

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
