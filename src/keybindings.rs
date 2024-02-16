use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};
use std::iter::once_with;
use std::str::Chars;
use num_traits::ToPrimitive;
use crate::hotkeymanager::{Key, KeyBinding};
use crate::{clone_with_modifier_if_needed};
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY::{VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT};


/// Parse binding that looks like lshift+alt+b+0x18
fn parse_binding(binding_str: &str) -> KeyBinding {
    binding_str.split('+').map(|part| match part {
        part if part.starts_with("0x") => Key::Scancode(u32::from_str_radix(&part[2..], 16)
            .unwrap_or_else(|_| panic!("Failed to parse scancode from {part}"))),
        part if part.chars().count() > 1 => Key::VirtualKey(KNOWN_VIRTUAL_KEY::from_human(part)
            .unwrap_or_else(|_| panic!("From human failed for {part}")).into()),
        part => Key::Character(part.to_owned()),
    }).collect()
}
pub type KeyBindings = Vec<KeyBinding>;
pub type CharKeyBindings = BTreeMap<BindingChar, KeyBindings>;

pub type BindingChar = char;
pub type CharBindingState<'a> = HashMap<&'a BindingChar, i32>;

pub trait Dump {
    fn dump(&self) -> String;
}

impl Dump for CharKeyBindings {
    fn dump(&self) -> String {
        self.iter()
            .map(|(char, binding)| format!("{}:\n{}\n",char, binding.dump()))
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


/// Read bindings from map. If map value is empty, then
pub(crate) fn bindings_from_map<'a>(the_conf: HashMap<String, HashMap<String, Option<String>>>) -> CharKeyBindings {
    let mut bindings: CharKeyBindings = BTreeMap::new();

    for (section, prop) in the_conf.iter() {
        let char_to_post: char = section.parse()
            .expect(&format!("Can't parse {section}"));

        prop.iter().for_each(|(key, value)| {
            let mut binding = parse_binding(key);
            let capitalize = value.is_none();
                let ex = expand_modifiers(&mut binding);
                let upper =  if capitalize {
                    let cap = clone_with_modifier_if_needed(char_to_post, &ex, VK_SHIFT);
                    if cap.len() > 0 {
                        Some(cap)
                    } else {
                        None
                    }
                } else {
                    None
                };
                bindings.entry(char_to_post).or_default().extend(ex);
                if upper.is_some() {
                    let upper = once_with(|| {
                        let mut new = Vec::new();
                        upper.unwrap().iter().for_each(|item| {
                            let ex = expand_modifiers(item);
                            new.extend(ex)
                        });
                        new
                    }).next().expect("OMG WTF");
                    bindings.entry(char_to_post.to_uppercase().next().unwrap()).or_default().extend(upper);
                }
        });
    }

    bindings
}

// Extract modifier application logic to a separate function for clarity
/*fn apply_modifiers(binding: &mut KeyBinding) {
    let mut bindings: KeyBindings = Vec::new();
    let modifiers = [(VK_SHIFT, VK_LSHIFT, VK_RSHIFT), (VK_MENU, VK_LMENU, VK_RMENU)];

    for (vk, vk_left, vk_right) in modifiers.iter().cloned() {
        if let Some(pos) = binding.iter().position(|k| matches!(k, Key::VirtualKey(vk_code) if vk_code == &(vk.to_u32().unwrap()))) {
            let with_left = replace_modifier(binding.clone(), pos, vk_left);
            let with_right = replace_modifier(binding.clone(), pos, vk_right);

            bindings.extend(with_left);
            bindings.extend(with_right);
            return; // Assuming only one modifier of each type per binding
        }
    }
}
*/



const CONST_VK_SHIFT: u32 = VK_SHIFT as u32;
const CONST_VK_MENU: u32 = VK_MENU as u32;
const CONST_VK_CONTROL: u32 = VK_CONTROL as u32;
const CONST_VK_WIN: u32 = VK_LWIN as u32;



fn expand_modifiers(binding: &KeyBinding) -> Vec<KeyBinding> {
    let mut expanded_bindings: Vec<KeyBinding> = vec![binding.clone()]; // Start with the original binding

    for (i, key) in binding.iter().enumerate() {
        if let Key::VirtualKey(vk) = key {
            let (left, right) = match (vk as &u32) {
                &CONST_VK_SHIFT => (VK_LSHIFT as u32, VK_RSHIFT as u32),
                &CONST_VK_MENU => (VK_LMENU as u32, VK_RMENU as u32),
                &CONST_VK_CONTROL => (VK_LCONTROL as u32, VK_RCONTROL as u32),
                //VK_WIN => (VK_LWIN as u32, VK_RWIN as u32),
                _ => continue,
            };

            // Create variations for each expanded binding and add them to the list
            expanded_bindings = expanded_bindings.iter().flat_map(|current_binding| {
                let mut with_left = current_binding.clone();
                with_left[i] = Key::VirtualKey(left);

                let mut with_right = current_binding.clone();
                with_right[i] = Key::VirtualKey(right);

                vec![with_left, with_right]
            }).collect();
        }
    }

    expanded_bindings
}

// Function to replace a modifier key with its left or right equivalent
fn replace_modifier(mut binding: KeyBinding, pos: usize, modifier: KNOWN_VIRTUAL_KEY) -> KeyBinding {
    binding[pos] = Key::VirtualKey(modifier.into());
    binding
}

/*fn sort_char_key_bindings(bindings: &CharKeyBindings) -> BTreeMap<char, Vec<KeyBinding>> {
    let mut sorted_keys: Vec<_> = bindings.keys().collect();
    sorted_keys.sort_by_key(|&k| k.to_uppercase().next().unwrap_or(*k));

    let mut sorted_bindings = BTreeMap::new();
    for &key in sorted_keys.iter() {
        if let Some(value) = bindings.get(&key) {
            sorted_bindings.insert(*key, value.clone());
        }
    }

    sorted_bindings
}*/