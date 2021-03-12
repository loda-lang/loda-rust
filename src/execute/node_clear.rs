use super::{Node,RegisterIndex,ProgramState};

pub struct NodeClearConstant {
    target: RegisterIndex,
    clear_count: u8,
}

impl NodeClearConstant {
    pub fn new(target: RegisterIndex, clear_count: u8) -> Self {
        Self {
            target: target,
            clear_count: clear_count,
        }
    }
}

impl Node for NodeClearConstant {
    fn shorthand(&self) -> &str {
        "clear constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("clr {},{}", self.target, self.clear_count)
    }

    fn eval(&self, state: &mut ProgramState) {
        state.set_register_range_to_zero(self.target.clone(), self.clear_count);
    }

    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {
        // This operation does not affect the number of registers to be allocated.
        // The default value of an uninitialized register is zero.
        // And accessing a register outside the allocated registers just yields zero.
    }
}
