use anyhow::Result;

// Контекст, который ядро передает плагину
pub struct PluginContext<'a> {
    pub buffer: &'a mut dyn BufferTrait,
    pub ui: &'a mut dyn UiTrait,
    pub lsp: &'a dyn LspClientTrait,
}

// Основной трейт плагина
pub trait IrmPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    // Хуки событий
    fn on_key(&mut self, ctx: &mut PluginContext, key: crossterm::event::KeyEvent) -> bool {
        false // вернул true = событие обработано, не передавать дальше
    }
    
    fn on_save(&mut self, ctx: &mut PluginContext) -> Result<()> { Ok(()) }
    fn on_tick(&mut self, ctx: &mut PluginContext) -> Result<()> { Ok(()) }
}

// Макрос для экспорта плагина из .so
#[macro_export]
macro_rules! register_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _irm_plugin_create() -> *mut dyn $crate::IrmPlugin {
            Box::into_raw(Box::new(<$plugin_type>::default()))
        }
    };
}
