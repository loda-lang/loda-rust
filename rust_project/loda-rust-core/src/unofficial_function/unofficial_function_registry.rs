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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unofficial_function::SumFunction;
    use num_bigint::{BigInt, ToBigInt};
    use std::sync::Arc;

    #[test]
    fn test_10000_lookup() {
        let registry = UnofficialFunctionRegistry::new();
        let plugin = SumFunction::new(1234, 2);
        registry.register(Arc::new(Box::new(plugin)));
        let key = UnofficialFunctionId::InputOutput { 
            id: 1234, 
            inputs: 2, 
            outputs: 1 
        };

        // Act
        let unofficial_function: Arc<Box<dyn UnofficialFunction>> = registry.lookup(key).expect("unofficial_function");

        // Assert
        let input_vec: Vec<BigInt> = vec![1000.to_bigint().unwrap(), 1.to_bigint().unwrap()];
        let output_vec: Vec<BigInt> = unofficial_function.run(input_vec).expect("output");
        let expected_output_vec: Vec<BigInt> = vec![1001.to_bigint().unwrap()];
        assert_eq!(output_vec, expected_output_vec);
    }

    #[test]
    fn test_20000_clone() {
        // Arrange
        let registry_original = UnofficialFunctionRegistry::new();
        let registry: UnofficialFunctionRegistry = registry_original.clone();
        let plugin = SumFunction::new(1234, 3);
        registry.register(Arc::new(Box::new(plugin)));
        let key = UnofficialFunctionId::InputOutput { 
            id: 1234, 
            inputs: 3, 
            outputs: 1 
        };
        
        // Act
        // Check that the original registry contains the item
        let unofficial_function: Arc<Box<dyn UnofficialFunction>> = registry_original.lookup(key).expect("unofficial_function");

        // Assert
        let input_vec: Vec<BigInt> = vec![100.to_bigint().unwrap(), 1.to_bigint().unwrap(), 10.to_bigint().unwrap()];
        let output_vec: Vec<BigInt> = unofficial_function.run(input_vec).expect("output");
        let expected_output_vec: Vec<BigInt> = vec![111.to_bigint().unwrap()];
        assert_eq!(output_vec, expected_output_vec);
    }
}
