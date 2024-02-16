pub(crate) mod hook;
pub(crate) mod hotkeymanager;
pub(crate) mod keyboardhook;

pub trait Dump {
    fn dump(&self) -> String;
}
