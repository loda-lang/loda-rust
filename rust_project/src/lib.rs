use wasm_bindgen::prelude::*;

// mod config;
// mod control;
// mod execute;
// mod mine;
// mod parser;
// mod oeis;
// mod util;
// use control::*;


#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
  return a + b;
}

pub mod console {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(message: &str);
    }
}

#[wasm_bindgen]
pub fn console_log_from_wasm() {
    console::log("This console.log is from wasm!");
}

#[wasm_bindgen]
pub fn myjsfunc_from_wasm() {
    console::log(&format!("myjsfunc_from_wasm: {:?}", 42));

    eval_loda_program();
}

fn eval_loda_program() {
    // let program_id: u64 = 40;
    // let number_of_terms: u64 = 20;
    // let mode = SubcommandEvaluateMode::PrintTerms;
    // subcommand_evaluate(program_id, number_of_terms, mode);
}
