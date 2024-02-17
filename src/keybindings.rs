use std::collections::{BTreeMap, HashMap};

use crate::r#static;
use std::iter::once_with;

use crate::r#type::hotkeymanager::{
    CharKeyBindings, HasCharacter, HasShift, Key, KeyBinding, KeyBindings,
};

use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY;
use crate::win::keyboard_vk::KNOWN_VIRTUAL_KEY::{
    VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_SHIFT,
};

/// Parse binding that looks like lshift+alt+b+0x18
fn parse_binding(binding_str: &str) -> KeyBinding {
    binding_str
        .split('+')
        .map(|part| match part {
            part if part.starts_with("0x") => Key::Scancode(
                u32::from_str_radix(&part[2..], 16)
                    .unwrap_or_else(|_| panic!("Failed to parse scancode from {part}")),
            ),
            part if part.chars().count() > 1 => Key::VirtualKey(
                KNOWN_VIRTUAL_KEY::from_human(part)
                    .unwrap_or_else(|_| panic!("From human failed for {part}"))
                    .into(),
            ),
            part => Key::Character(part.to_owned()),
        })
        .collect()
}

/// Read bindings from map. If map value is empty, then
pub(crate) fn bindings_from_map(
    the_conf: HashMap<String, HashMap<String, Option<String>>>,
) -> CharKeyBindings {
    let mut bindings: CharKeyBindings = BTreeMap::new();

    for (section, prop) in the_conf.iter() {
        let char_to_post: char = section.parse().expect(&format!("Can't parse {section}"));

        prop.iter().for_each(|(key, value)| {
            let binding = parse_binding(key);
            let capitalize = value.is_none();
            let ex = expand_modifiers(&binding);
            let upper = if capitalize {
                let cap = clone_with_modifier_if_needed(char_to_post, &ex, VK_SHIFT);
                if !cap.is_empty() {
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
                    upper
                        .expect("Failed to get upper variants")
                        .iter()
                        .for_each(|item| {
                            let ex = expand_modifiers(item);
                            new.extend(ex)
                        });
                    new
                })
                .next()
                .expect("OnceWith broke");
                bindings
                    .entry(
                        char_to_post
                            .to_uppercase()
                            .next()
                            .expect(&*format!("Failed to get uppercase from {}", &char_to_post)),
                    )
                    .or_default()
                    .extend(upper);
            }
        });
    }

    bindings
}

fn expand_modifiers(binding: &KeyBinding) -> Vec<KeyBinding> {
    let mut expanded_bindings: Vec<KeyBinding> = vec![binding.clone()]; // Start with the original binding

    for (i, key) in binding.iter().enumerate() {
        if let Key::VirtualKey(vk) = key {
            let (left, right) = match vk {
                &r#static::CONST_VK_SHIFT => (VK_LSHIFT as u32, VK_RSHIFT as u32),
                &r#static::CONST_VK_MENU => (VK_LMENU as u32, VK_RMENU as u32),
                &r#static::CONST_VK_CONTROL => (VK_LCONTROL as u32, VK_RCONTROL as u32),
                //VK_WIN => (VK_LWIN as u32, VK_RWIN as u32),
                _ => continue,
            };

            // Create variations for each expanded binding and add them to the list
            expanded_bindings = expanded_bindings
                .iter()
                .flat_map(|current_binding| {
                    let mut with_left = current_binding.clone();
                    with_left[i] = Key::VirtualKey(left);

                    let mut with_right = current_binding.clone();
                    with_right[i] = Key::VirtualKey(right);

                    vec![with_left, with_right]
                })
                .collect();
        }
    }

    expanded_bindings
}

fn clone_with_modifier_if_needed(
    char_to_post: char,
    bindings: &KeyBindings,
    modifier: KNOWN_VIRTUAL_KEY,
) -> KeyBindings {
    let mut created_bingdings = Vec::new();

    for binding in bindings {
        if binding.has_character() && !binding.has_shift() && char_to_post.is_lowercase() {
            let mut modified_binding = binding.clone();
            modified_binding.insert(0, Key::VirtualKey(modifier as u32)); // Or VK_LSHIFT, if preferred
            created_bingdings.push(modified_binding);
        }
    }

    created_bingdings
}
