use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use indexmap::IndexSet;

use crate::r#type::Dump;
use crate::r#type::hotkeymanager::Key::VirtualKey;
use crate::r#type::keyboardhook::KeyboardHookMetadata;
use crate::win::{is_meta_or_alt, ToScanCode, ToUnicode, VIRTUAL_KEY};
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY::{VK_LSHIFT, VK_RSHIFT, VK_SHIFT};

pub type KeyBindings = Vec<KeyBinding>;
pub type CharKeyBindings = BTreeMap<BindingChar, KeyBindings>;

pub type BindingChar = char;
pub type CharBindingState<'a> = HashMap<&'a BindingChar, i32>;

impl Dump for CharKeyBindings {
    fn dump(&self) -> String {
        self.iter()
            .map(|(char, binding)| format!("{}:\n{}\n", char, binding.dump()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Dump for KeyBindings {
    fn dump(&self) -> String {
        self.iter()
            .map(|binding| binding.dump())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone)]
pub enum Key {
    VirtualKey(VIRTUAL_KEY),
    Character(String),
    Scancode(u32),
}

impl Debug for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Key::VirtualKey(k) => {
                    match KNOWN_VIRTUAL_KEY::try_from(*k) {
                        Ok(k) => format!("{:?}", k),
                        Err(e) => {
                            let code = e.into_inner();

                            if let Some(char) = &code.to_unicode_localized() {
                                format!("'{}'", char)
                            } else {
                                format!("VKey({})", &code)
                            }

                        },
                    }
                }
                Key::Character(char) => format!("'{}'", char),
                Key::Scancode(code) => format!("0x{:x}", code),
            }
        )
    }
}

pub type KeyBinding = Vec<Key>; // Now a Vec to preserve order

impl Dump for KeyBinding {
    fn dump(&self) -> String {
        return String::from(format!("{:?}", self));
    }
}

type Callback = Box<dyn Fn(TriggeredHotkey) + Send + Sync>;
type ChannelSender = Sender<TriggeredHotkey>;
type ChannelReceiver = Receiver<TriggeredHotkey>;

pub trait HasCharacter {
    fn has_character(&self) -> bool;

    fn has_character_value(&self, exact: String) -> bool;
}

pub trait HasShift {
    fn has_shift(&self) -> bool;
}

impl HasShift for KeyBinding {
    fn has_shift(&self) -> bool {
        self.iter().any(|key| matches!(key, Key::VirtualKey(vk) if vk == &VK_SHIFT.into() || vk == &VK_LSHIFT.into() || vk == &VK_RSHIFT.into()))
    }
}

pub trait HasVirtualKey {
    fn has_virtual_key(&self) -> bool;

    fn has_virtual_key_value(&self, exact: VIRTUAL_KEY) -> bool;
}

impl HasCharacter for KeyBinding {
    fn has_character(&self) -> bool {
        self.iter().any(|key| matches!(key, Key::Character(_)))
    }
    fn has_character_value(&self, exact: String) -> bool {
        self.iter().any(|key| {
            if let Key::Character(value) = key {
                value == &exact
            } else {
                false
            }
        })
    }
}

impl HasVirtualKey for KeyBinding {
    fn has_virtual_key(&self) -> bool {
        self.iter().any(|key| matches!(key, Key::VirtualKey(_)))
    }

    fn has_virtual_key_value(&self, exact: VIRTUAL_KEY) -> bool {
        self.iter().any(|key| {
            if let Key::VirtualKey(vk) = key {
                *vk == exact
            } else {
                false
            }
        })
    }
}

#[derive(Clone, Debug)]
enum BindingAction {
    //Callback(Callback),
    Channel(ChannelSender),
    Magic(ChannelSender),
}

#[derive(Clone, Debug)]
pub struct HotkeyBinding {
    keys: KeyBinding,
    on_press: BindingAction,
    on_release: BindingAction,
    ordered: bool,
    pub triggered: bool,
}

impl HotkeyBinding {
    fn is_triggered(&self) -> bool {
        self.triggered
    }
}

#[derive(Clone, Debug)]
pub struct TriggeredHotkey(pub HotkeyBinding, pub PressedKeys);

impl HotkeyBinding {
    fn execute_binding_actions(
        &mut self,
        metadata: &KeyboardHookMetadata,
        pressed_keys: &PressedKeys,
    ) -> bool {
        match metadata {
            KeyboardHookMetadata::Press { .. } => {
                self.on_press.execute_action(self, pressed_keys);
                if !self.triggered {
                    self.triggered = true
                }
                true
            }
            KeyboardHookMetadata::Release { .. } => {
                self.on_release.execute_action(self, pressed_keys);
                if self.triggered {
                    self.triggered = false
                }
                true
            }
        }
    }

    fn should_trigger(
        &self,
        pressed_keys: &IndexSet<VIRTUAL_KEY>,
        scancode_cache: &mut HashMap<u32, u32>,
        char_cache: &mut HashMap<u32, Option<String>>,
    ) -> bool {
        log::trace!("{} = {}", self.keys.dump(), pressed_keys.dump());
        for key in &self.keys {
            match key {
                Key::VirtualKey(vk) => {
                    if !pressed_keys.contains(vk) {
                        log::trace!(" == false");
                        return false;
                    }
                }
                Key::Character(expected_str) => {
                    let pressed_str = pressed_keys.iter().find_map(|&vk| {
                        // Clone the String to ensure the returned value is owned and not a reference
                        char_cache
                            .entry(vk)
                            .or_insert_with(|| vk.to_unicode_localized())
                            .clone()
                    });
                    // Compare Option<&String> with &Option<String> using map and as_deref
                    //log::trace!("Checking if {:?} in {:?}", expected_str, pressed_str);
                    if pressed_str.as_deref() != Some(expected_str) {
                        log::trace!(" == false");
                        return false;
                    }
                }
                Key::Scancode(expected_sc) => {
                    // Find the pressed scan code by converting each virtual key code using the cache
                    let pressed_sc_match = pressed_keys.iter().any(|&vk| {
                        let pressed_sc = scancode_cache.entry(vk).or_insert_with(|| vk.to_code());
                        pressed_sc == expected_sc
                    });
                    if !pressed_sc_match {
                        log::trace!(" == false");
                        return false;
                    }
                }
            }
        }
        log::trace!(" == true");
        true
    }
}

impl BindingAction {
    fn execute_action(&self, binding: &HotkeyBinding, pressed_keys: &PressedKeys) {
        match self {
            //BindingAction::Callback(cb) => cb(TriggeredHotkey(binding.clone(), pressed_keys.clone())),
            BindingAction::Channel(tx) | BindingAction::Magic(tx) => {
                if let Err(error) = tx.send(TriggeredHotkey(binding.clone(), pressed_keys.clone()))
                {
                    log::trace!("BROKEN PIPE: {:?}", error);
                }
            }
        }
    }
}

pub struct HotkeyManager {
    //bindings: VecDeque<HotkeyBinding>,
    bindings_by_length: HashMap<usize, VecDeque<HotkeyBinding>>,
}

pub(crate) trait Bindable {
    fn to_virtual_key(self) -> u32;
}

impl Bindable for u32 {
    fn to_virtual_key(self) -> u32 {
        self
    }
}

pub type PressedKeys = IndexSet<VIRTUAL_KEY>;

impl Dump for PressedKeys {
    fn dump(&self) -> String {
        self.iter()
            .map(|item| Key::VirtualKey(*item))
            .collect::<Vec<_>>()
            .dump()
    }
}

impl HotkeyManager {
    pub fn new() -> Self {
        HotkeyManager {
            //bindings: VecDeque::new(),
            bindings_by_length: HashMap::new(),
        }
    }

    fn _add_binding(
        &mut self,
        keys: KeyBinding,
        on_press: BindingAction,
        on_release: BindingAction,
        ordered: bool,
    ) -> &HotkeyBinding {
        let binding_length = keys.len();
        let binding = HotkeyBinding {
            keys,
            on_press,
            on_release,
            ordered,
            triggered: false,
        };

        let bindings_for_length = self.bindings_by_length.entry(binding_length).or_default();
        bindings_for_length.push_back(binding);

        return bindings_for_length
            .back()
            .expect("Can't add binding to the pile");
    }

    /*pub(crate) fn add_binding(&mut self, keys: KeyBinding, on_press: Callback, on_release:Callback, ordered: bool) -> &HotkeyBinding  {
        self._add_binding(keys, BindingAction::Callback(on_press),  BindingAction::Callback(on_release), ordered)
    }*/

    pub(crate) fn add_magic_binding(
        &mut self,
        keys: KeyBinding,
        on_press: Callback,
        on_release: Callback,
        ordered: bool,
    ) -> &HotkeyBinding {
        let (on_press_tx, on_press_rx): (ChannelSender, ChannelReceiver) = mpsc::channel();

        let (on_release_tx, on_release_rx): (ChannelSender, ChannelReceiver) = mpsc::channel();

        std::thread::Builder::new()
            .name("hotkey::press".to_string())
            .spawn(move || {
                for data in on_press_rx {
                    log::trace!(target: "MagicBinding", "[keypress] received new data");
                    on_press(data);
                }
            }).unwrap_or_else(|e|panic!("Thread hotkey::release failed. {:?}", e));
        std::thread::Builder::new()
            .name("hotkey::release".to_string())
            .spawn(move || {
                for data in on_release_rx {
                    log::trace!(target: "MagicBinding", "[keyrelease] received new data");
                    on_release(data);
                }
            }).unwrap_or_else(|e|panic!("Thread hotkey::release failed. {:?}", e));
        self._add_binding(
            keys,
            BindingAction::Magic(on_press_tx),
            BindingAction::Magic(on_release_tx),
            ordered,
        )
    }

    pub fn add_channel_binding(&mut self, keys: KeyBinding) -> (ChannelReceiver, ChannelReceiver) {
        let (on_press_tx, on_press_rx) = mpsc::channel();
        let (on_release_tx, on_release_rx) = mpsc::channel();
        self._add_binding(
            keys,
            BindingAction::Channel(on_press_tx),
            BindingAction::Channel(on_release_tx),
            false,
        );
        (on_press_rx, on_release_rx)
    }

    pub(crate) fn check_and_trigger(&mut self, metadata: &KeyboardHookMetadata) -> bool {
        let key = *metadata.key();
        if is_meta_or_alt(key) {
            let activated = self
                .bindings_by_length
                .values()
                .flat_map(|bindings| bindings.iter()) // Iterate over all bindings in all VecDeques
                .filter(|binding| binding.is_triggered()) // Keep only those bindings that are triggered
                .cloned() // Clone the triggered bindings to create a Vec<HotkeyBinding>
                .collect::<Vec<_>>(); // Collect the filtered and cloned bindings into a Vec
            if !activated
                .iter()
                .filter(|f| f.keys.has_virtual_key_value(key))
                .collect::<Vec<_>>()
                .is_empty()
            {
                if metadata.pressing() {
                    log::debug!(target: "HotkeyManager",
                        "We think that we are sill pressing a binding, so ignoring {:?}",
                        VirtualKey(key.clone())
                    );
                    return true;
                } else if metadata.releasing() && metadata.injected() {
                    log::debug!(target: "HotkeyManager",
                        "Ignoring injected releases {:?}",
                        VirtualKey(key.clone())
                    );
                    return true;
                }
            }
        }

        let pressed_keys = metadata.pressed_keys();
        let pressed_count = pressed_keys.len();
        let mut scancode_cache: HashMap<u32, u32> = HashMap::new();
        let mut char_cache: HashMap<u32, Option<String>> = HashMap::new();

        if let Some(bindings) = self.bindings_by_length.get_mut(&pressed_count) {
            for binding in bindings.iter_mut() {
                if binding.should_trigger(pressed_keys, &mut scancode_cache, &mut char_cache) {
                    log::trace!(target: "HotkeyManager", "Got a match, firing callback {:?}", binding);
                    return binding.execute_binding_actions(metadata, pressed_keys);
                }
            }
        }

        false
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
