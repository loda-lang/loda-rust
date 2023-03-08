use super::{AssertFunction, AssertFunctionMode, DebugFunction, ProductFunction, SortFunction, SumFunction, UnofficialFunctionRegistry};
use std::sync::Arc;

pub fn register_common_functions(registry: &UnofficialFunctionRegistry) {
    macro_rules! register_function {
        ($create_instance:expr) => {
            registry.register(Arc::new(Box::new($create_instance)));
        }
    }

    // Developer functions
    register_function!(DebugFunction::new(1));

    // Common functions
    {
        let id: u32 = 1000;
        for input_count in 2..9u8 {
            register_function!(SumFunction::new(id, input_count));
        }
    }
    {
        let id: u32 = 1010;
        for input_count in 2..9u8 {
            register_function!(ProductFunction::new(id, input_count));
        }
    }
    {
        let id: u32 = 1020;
        for inout_count in 2..9u8 {
            register_function!(SortFunction::new(id, inout_count));
        }
    }

    // Assert functions
    {
        register_function!(AssertFunction::new(1030, AssertFunctionMode::Equal));
        register_function!(AssertFunction::new(1031, AssertFunctionMode::Different));
        register_function!(AssertFunction::new(1032, AssertFunctionMode::LessThan));
        register_function!(AssertFunction::new(1033, AssertFunctionMode::LessThanOrEqual));
        register_function!(AssertFunction::new(1034, AssertFunctionMode::GreaterThan));
        register_function!(AssertFunction::new(1035, AssertFunctionMode::GreaterThanOrEqual));
    }
}
