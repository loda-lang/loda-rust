use super::{RegisterIndex, RegisterValue, RunMode};

pub struct ProgramState {
    register_vec: Vec<RegisterValue>,
    eval_count: u64,
    run_mode: RunMode,
}

impl ProgramState {
    pub fn new(register_count: u8, run_mode: RunMode) -> Self {
        let mut register_vec: Vec<RegisterValue> = vec!();
        for _ in 0..register_count {
            register_vec.push(RegisterValue::zero());
        }
        Self {
            register_vec: register_vec,
            eval_count: 0,
            run_mode: run_mode,
        }
    }

    pub fn increment_eval_count(&mut self) {
        self.eval_count += 1;
    }

    pub fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    pub fn get_register_value(&self, register_index: RegisterIndex) -> RegisterValue {
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            panic!("get_register_value. index is outside the number of registers.");
        }
        return self.register_vec[index].clone();
    }

    pub fn set_register_value(&mut self, register_index: RegisterIndex, register_value: RegisterValue) {
        let index = register_index.0 as usize;
        if index >= self.register_vec.len() {
            panic!("set_register_value. index is outside the number of registers.");
        }
        self.register_vec[index] = register_value;
    }

    // Make the internal state human readable
    pub fn register_vec_to_string(&self) -> String {
        let strings: Vec<String> = self.register_vec.iter().map(|register_value| {
            register_value.0.to_string()
        }).collect();
        let strings_joined: String = strings.join(",");
        format!("[{}]", strings_joined)
    }
}
