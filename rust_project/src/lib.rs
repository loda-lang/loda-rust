use wasm_bindgen::prelude::*;

use log::{Log,Metadata,Record,LevelFilter};

#[macro_use]
extern crate log;

// mod config;
// mod control;
// mod execute;
// mod mine;
// mod parser;
// mod oeis;
// mod util;
// use control::*;

#[derive(Clone)]
struct MyCustomLog {
}

impl Log for MyCustomLog {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level_string = record.level().to_string();

        let target = if !record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };

        let message = format!("{:<5} [{}] {}", level_string, target, record.args());
        console::log(&message);
    }

    fn flush(&self) {
    }
}

impl MyCustomLog {
    fn new() -> Self {
        Self {}
    }

    fn init(&mut self) -> Result<(), log::SetLoggerError> {
        log::set_max_level(LevelFilter::Trace);
        log::set_boxed_logger(Box::new(self.clone()))
    }
}

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
    MyCustomLog::new().init().unwrap();

    trace!("I'm trace");
    debug!("I'm debug");
    error!("I'm error");
    info!("I'm info");
    warn!("I'm warn");

    // let program_id: u64 = 40;
    // let number_of_terms: u64 = 20;
    // let mode = SubcommandEvaluateMode::PrintTerms;
    // subcommand_evaluate(program_id, number_of_terms, mode);
}
