use std::collections::HashMap;

use ini::configparser::ini::Ini;

use crate::keybindings::bindings_from_map;
use crate::r#static::HOTKEY_MANAGER_INSTANCE;
use crate::r#type::hotkeymanager::CharBindingState;
use crate::r#type::Dump;

use crate::win::keyboard::{send_key_sequence, KeyAction, KeyStroke};
use crate::win::window::create_window;

mod keybindings;
mod r#static;
mod r#type;
mod win;

fn main() {
    let mut conf = Ini::new();
    let the_conf = conf.load("bindings.ini").unwrap();
    let bindings = bindings_from_map(the_conf);

    println!("Parsed keybindings: {}", bindings.dump());
    let _state: CharBindingState = HashMap::new();
    bindings.into_iter().for_each(|(char_to_post, key_bindings)| {
        let _char_to_post_clone = char_to_post.clone();
        //state.insert(char_to_post_clone, -1);
        key_bindings.into_iter().for_each(move |binding| {
            // Clone char_to_post to move a copy into the closure

            let char_to_post_clone = char_to_post.clone();


            let _the_binding = HOTKEY_MANAGER_INSTANCE.lock_arc().add_magic_binding(binding, Box::new(move |triggered| {
                println!("[ACTIVATION] Triggered {:?} on keypress.", triggered);

                let mut pre_keys: Vec<KeyStroke> =  if triggered.0.triggered {
                    Vec::new()
                } else {
                    println!("[ACTIVATION] Hotkey is not yet activated, releasing pressed keys: {:?}", triggered.1);
                    /*filter_modifier_keys*/(&triggered.1).iter()
                        .map(|&vk| KeyStroke::classic(vk, KeyAction::Release))
                        .collect()
                };
                pre_keys.reverse();
                let char_keystroke = KeyStroke::unicode(char_to_post_clone, KeyAction::Press);
                send_key_sequence(&pre_keys, &[char_keystroke], &[]);
            }), Box::new(move |triggered| {
                println!("[DEACTIVATION] Triggered {:?} on keyrelease.", triggered);
                let post_keys: Vec<KeyStroke> =  if triggered.0.triggered {
                    Vec::new()
                } else {
                    println!("[DEACTIVATION] Hotkey is still activated, releasing pressed keys: {:?}", triggered.1);
                    /*filter_modifier_keys*/(&triggered.1).iter()
                        .map(|&vk| KeyStroke::classic(vk, KeyAction::Press))
                        .collect()
                };
                //post_keys.reverse();
                let char_keystroke = KeyStroke::unicode(char_to_post_clone, KeyAction::Release);
                send_key_sequence(&[], &[char_keystroke], &post_keys);
            }), false);
        });
    });

    create_window()
}
