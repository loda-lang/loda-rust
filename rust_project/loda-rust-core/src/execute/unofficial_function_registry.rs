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
