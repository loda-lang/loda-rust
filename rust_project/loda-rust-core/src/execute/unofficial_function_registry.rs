use super::UnofficialFunction;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
struct RegistryInner<T = Box<dyn UnofficialFunction>> {
    plugin_vec: Vec<Arc<T>>,
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
            plugin_vec: vec!(),
        };
        let instance = UnofficialFunctionRegistry { 
            inner: Arc::new(RwLock::new(inner)) 
        };
        instance
    }

    pub fn register(&self, plugin: Arc<Box<dyn UnofficialFunction>>) {
        self.inner.write().unwrap().plugin_vec.push(plugin);
    }

    pub fn lookup(&self, inputs: u8, outputs: u8, function_id: u64) -> Option<Arc<Box<dyn UnofficialFunction>>> {
        let plugin_vec = self.inner.read().unwrap().plugin_vec.clone();
        for plugin in plugin_vec {
            let plugin_clone: Arc<Box<dyn UnofficialFunction>> = plugin.clone();
            return Some(plugin_clone);
        }
        None
    }

    pub fn execute(&self) -> anyhow::Result<String> {
        let mut execute_output: Option<String> = None;
        let plugin_vec = self.inner.read().unwrap().plugin_vec.clone();
        for plugin in plugin_vec {
            let result = plugin.execute();
            match result {
                Ok(value) => {
                    execute_output = Some(value);
                },
                Err(error) => {
                    return Err(anyhow::anyhow!("execute failed. error: {:?}", error));
                }
            }
        }
        let value: String = match execute_output {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("plugin didn't return anything"));
            }
        };
        Ok(value)
    }
}
