
use std::env;
use std::str::FromStr;

use ini::configparser::ini::Ini;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use crate::keybindings::bindings_from_map;
use crate::r#static::HOTKEY_MANAGER_INSTANCE;

use crate::r#type::Dump;

use crate::win::keyboard::{send_key_sequence, KeyAction, KeyStroke};
use crate::win::window::create_window;

mod keybindings;
mod r#static;
mod r#type;
mod win;

fn main() {
    SimpleLogger::new().init().expect("Can't load logger.");

    let args: Vec<String> = env::args().collect();

    let level = match args.len() {
        0|1 => LevelFilter::Error,
        2 => LevelFilter::from_str(&args[1]).unwrap_or(LevelFilter::Error),
        _ => LevelFilter::from_str(&args[1]).unwrap_or(LevelFilter::Trace),
    };
    println!("Verbosity: {}", level);
    log::set_max_level(level);

    let mut conf = Ini::new();
    let the_conf = conf.load("bindings.ini").expect("Can't open keybindings");
    let bindings = bindings_from_map(the_conf);

    log::info!("Parsed keybindings:\n{}", bindings.dump());
    bindings.into_iter().for_each(|(char_to_post, key_bindings)| {
        key_bindings.into_iter().for_each(move |binding| {
            let char_to_post_clone = char_to_post.clone();

            let _the_binding = HOTKEY_MANAGER_INSTANCE.lock_arc().add_magic_binding(binding, Box::new(move |triggered| {
                let target= "[ACTIVATION]";
                log::debug!(target: target, "Triggered {:?} on keypress.", triggered);

                let mut pre_keys: Vec<KeyStroke> =  if triggered.0.triggered {
                    Vec::new()
                } else {
                    log::debug!(target: target,"Hotkey is not yet activated, releasing pressed keys: {:?}", triggered.1);
                    /*filter_modifier_keys*/(&triggered.1).iter()
                        .map(|&vk| KeyStroke::classic(vk, KeyAction::Release))
                        .collect()
                };
                pre_keys.reverse();
                let char_keystroke = KeyStroke::unicode(char_to_post_clone, KeyAction::Press);
                send_key_sequence(&pre_keys, &[char_keystroke], &[]);
            }), Box::new(move |triggered| {
                let target= "[DEACTIVATION]";
                log::debug!(target: target, "Triggered {:?} on keyrelease.", triggered);
                let post_keys: Vec<KeyStroke> =  if triggered.0.triggered {
                    Vec::new()
                } else {
                    log::debug!(target: target, "Hotkey is still activated, releasing pressed keys: {:?}", triggered.1);
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
