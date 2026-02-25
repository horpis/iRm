use libloading::{Library, Symbol};
use crate::core::plugin_api::IrmPlugin;

pub struct LoadedPlugin {
    _lib: Library, // Библиотека должна жить, пока живет плагин
    pub instance: Box<dyn IrmPlugin>,
}

impl LoadedPlugin {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        unsafe {
            let lib = Library::new(path)?;
            // Ищем функцию-конструктор, экспортированную макросом
            let constructor: Symbol<fn() -> *mut dyn IrmPlugin> = lib.get(b"_irm_plugin_create")?;
            let raw_ptr = constructor();
            let instance = Box::from_raw(raw_ptr);
            
            Ok(Self { _lib: lib, instance })
        }
    }
}
