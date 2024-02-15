use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use crate::hotkeymanager::{Key, KeyBinding};
use crate::{modify_bindings};
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY::{VK_LMENU, VK_LSHIFT, VK_MENU, VK_RMENU, VK_RSHIFT, VK_SHIFT};

/// Parse binding that looks like lshift+alt+b+0x18
fn parse_binding(binding_str: &str) -> KeyBinding {
    let parts: Vec<&str> = binding_str.split('+').collect();
    let mut binding = Vec::new();

    for part in parts.into_iter() {
        match part {
            _ if part.starts_with("0x") => {
                // Scancode binding
                binding.push(Key::Scancode(u32::from_str_radix(&part[2..], 16).expect(&*format!("Failed to parse scancode from {part}"))));
            },
            _ if part.chars().count() > 1 => {
                // VK binding
                binding.push(Key::VirtualKey(KNOWN_VIRTUAL_KEY::from_human(part).expect(&*format!("From human failed for {part}")).into()));
            },
            _ => {
                // Character binding, including uppercase letters
                binding.push(Key::Character(part.to_owned()));
            },
        }
    }

    binding
}

pub type KeyBindings = Vec<KeyBinding>;

pub type CharKeyBindings = HashMap<char, KeyBindings>;

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

/// extends bindings:
/// if binding contains char binding and does not contain shift binding,
/// then add shift binding with an upper version of the target character.
///
/// then it gets all the bindings and searches for win, alt, shift and adds lwin rwin, lalt ralt, lshift rshift and appends them to the bindings
pub(crate) fn bindings_from_map(the_conf: HashMap<String, HashMap<String, Option<String>>>) -> CharKeyBindings {
    let mut bindings = HashMap::<char, Vec<KeyBinding>>::new();
    for (section, prop) in the_conf.iter() {
        if let char_to_post = section {
            for (key, value) in prop.iter() {
                //println!("{char_to_post} | {key} | {:?}", value);
                let char_to_post: char = char_to_post.parse().expect(&*format!("Can't parse {char_to_post}"));
                bindings.entry(char_to_post).or_insert_with(Vec::new);
            }
        }
    }
    for (section, prop) in the_conf.iter() {
        if let char_to_post = section {
            for (key, value) in prop.iter() {
                //println!("{char_to_post} | {key} | {:?}", value);
                let char_to_post: char = char_to_post.parse().expect(&*format!("Can't parse {char_to_post}"));

                let mut binding = parse_binding(key);
                //let binding_towork = binding.copy();
                //println!("{:?}",binding);
                if value.is_none() {
                    // Loop through KeyBinding and modify based on VK_WIN presence
                    for i in 0..binding.len() {
                        if let Key::VirtualKey(vk) = binding[i] {
                            if vk == VK_SHIFT.into() { // Assuming VK_WIN is a constant for the Windows key
                                // Copy the binding and replace VK_WIN with VK_LWIN in the copy
                                let mut with_lwin = binding.clone();
                                with_lwin[i] = Key::VirtualKey(VK_LSHIFT.into()); // Assuming VK_LWIN is a constant for the left Windows key

                                // Modify the original binding to replace VK_WIN with VK_RWIN
                                binding[i] = Key::VirtualKey(VK_RSHIFT.into()); // Assuming VK_RWIN is a constant for the right Windows key
                                bindings.entry(char_to_post).and_modify(|f|{
                                    f.push(with_lwin);
                                });
                                break; // Assuming only one VK_WIN per binding; remove if there can be multiple
                            }

                            if vk == VK_MENU.into() { // Assuming VK_WIN is a constant for the Windows key
                                // Copy the binding and replace VK_WIN with VK_LWIN in the copy
                                let mut with_lwin = binding.clone();
                                with_lwin[i] = Key::VirtualKey(VK_LMENU.into()); // Assuming VK_LWIN is a constant for the left Windows key

                                // Modify the original binding to replace VK_WIN with VK_RWIN
                                binding[i] = Key::VirtualKey(VK_RMENU.into()); // Assuming VK_RWIN is a constant for the right Windows key
                                bindings.entry(char_to_post).and_modify(|f|{
                                    f.push(with_lwin);
                                });
                                break; // Assuming only one VK_WIN per binding; remove if there can be multiple
                            }
                        }
                    }

                    bindings.entry(char_to_post).and_modify(|c|{
                        c.push(binding);
                    });

                }
            }
        }
    }

    modify_bindings(&mut bindings);

    return bindings;
}

