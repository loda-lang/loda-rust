use super::{Node,RegisterIndex,RegisterValue,ProgramState};

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
        let value = RegisterValue::zero();
        let mut register_index_inner: u16 = self.target.0 as u16;
        for _ in 0..self.clear_count {
            if register_index_inner >= 256 {
                panic!("attempting to clear a register that is out of bounds");
            }
            let register_index = RegisterIndex(register_index_inner as u8);
            state.set_register_value(register_index, value.clone());
            register_index_inner += 1;
        }
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        let mut register_index_inner: u16 = self.target.0 as u16;
        for _ in 0..self.clear_count {
            if register_index_inner >= 256 {
                panic!("attempting to clear a register that is out of bounds");
            }
            let register_index = RegisterIndex(register_index_inner as u8);
            register_vec.push(register_index);
            register_index_inner += 1;
        }
    }
}
