use super::{ProductFunction, SortFunction, SumFunction, UnofficialFunctionRegistry};
use std::sync::Arc;

pub fn register_common_functions(registry: &UnofficialFunctionRegistry) {
    {
        let id: u32 = 1;
        for input_count in 2..9u8 {
            registry.register(Arc::new(Box::new(SumFunction::new(id, input_count))));
        }
    }
    {
        let id: u32 = 2;
        for input_count in 2..9u8 {
            registry.register(Arc::new(Box::new(ProductFunction::new(id, input_count))));
        }
    }
    {
        let id: u32 = 3;
        for input_count in 2..9u8 {
            registry.register(Arc::new(Box::new(SortFunction::new(id, input_count))));
        }
    }
}
