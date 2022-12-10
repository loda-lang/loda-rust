use super::{DebugFunction, ProductFunction, SortFunction, SumFunction, UnofficialFunctionRegistry};
use std::sync::Arc;

pub fn register_common_functions(registry: &UnofficialFunctionRegistry) {
    // Developer functions
    {
        let id: u32 = 1;
        registry.register(Arc::new(Box::new(DebugFunction::new(id))));
    }

    // Common functions
    {
        let id: u32 = 1000;
        for input_count in 2..9u8 {
            registry.register(Arc::new(Box::new(SumFunction::new(id, input_count))));
        }
    }
    {
        let id: u32 = 1010;
        for input_count in 2..9u8 {
            registry.register(Arc::new(Box::new(ProductFunction::new(id, input_count))));
        }
    }
    {
        let id: u32 = 1020;
        for inout_count in 2..9u8 {
            registry.register(Arc::new(Box::new(SortFunction::new(id, inout_count))));
        }
    }
}
