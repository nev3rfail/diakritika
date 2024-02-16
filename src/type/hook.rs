


use std::any::Any;
use anyhow::Error;
use std::ops::Fn;

pub trait Hook: Any + Send + Sync {
    fn call(&self, extra: &dyn HookMetadata) -> Result<bool, anyhow::Error>;
}

impl<F, T> Hook for (F, T)
    where
        F: Fn(&dyn HookMetadata, &T) -> Result<bool, anyhow::Error> + 'static + Send + Sync,
        T: 'static + Send + Sync,
{
    fn call(&self, extra: &dyn HookMetadata) -> Result<bool, anyhow::Error> {
        (self.0)(extra, &self.1)
    }
}

pub struct HookContainer {
    hook: Box<dyn Hook>,
}
impl HookContainer {
    pub fn new<F, T>(hook: F, arg: T) -> Self
        where
            F: Fn(&dyn HookMetadata, &T) -> Result<bool, anyhow::Error> + 'static + Send + Sync,
            T: 'static + Send + Sync,
    {
        HookContainer {
            hook: Box::new((hook, arg)),
        }
    }

    pub fn trigger(&self, extra: &dyn HookMetadata) -> Result<bool, Error> {
        self.hook.call(extra)
    }
}
pub trait HookMetadata: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}
