use super::{UnofficialFunction, UnofficialFunctionId};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

#[derive(Debug)]
struct RegistryInner<T = Box<dyn UnofficialFunction>> {
    plugin_dict: HashMap<UnofficialFunctionId,Arc<T>>,
}

impl<T> RegistryInner<T> {
}

#[derive(Clone)]
pub struct UnofficialFunctionRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

impl UnofficialFunctionRegistry {
    pub fn new() -> UnofficialFunctionRegistry {
        let inner = RegistryInner {
            plugin_dict: HashMap::new(),
        };
        let instance = UnofficialFunctionRegistry { 
            inner: Arc::new(RwLock::new(inner)) 
        };
        instance
    }

    pub fn register(&self, plugin: Arc<Box<dyn UnofficialFunction>>) {
        let key: UnofficialFunctionId = plugin.id();
        let mut inner = self.inner.write().expect("UnofficialFunctionRegistry.register() RwLock poisoned");
        match inner.plugin_dict.insert(key, plugin) {
            Some(_) => {
                error!("UnofficialFunctionRegistry.register({:?}) overwriting existing value", key);
            },
            None => {}
        }
    }

    pub fn lookup(&self, key: UnofficialFunctionId) -> Option<Arc<Box<dyn UnofficialFunction>>> {
        let inner = self.inner.read().expect("UnofficialFunctionRegistry.lookup() RwLock poisoned");
        if let Some(value) = inner.plugin_dict.get(&key) {
            return Some(value.clone());
        }
        None
    }
}
