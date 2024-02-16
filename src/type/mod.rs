pub (crate) mod hook;
pub (crate) mod keyboardhook;
pub (crate)mod hotkeymanager;

pub trait Dump {
    fn dump(&self) -> String;
}