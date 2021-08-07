use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use log::{Log,Metadata,Record,LevelFilter};

#[macro_use]
extern crate log;

mod config;
mod control;
mod execute;
mod parser;
mod oeis;
mod util;

use std::path::PathBuf;
use control::DependencyManager;
use execute::{NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, RegisterValue, RunMode};
use execute::NodeRegisterLimit;
use execute::node_binomial::NodeBinomialLimit;
use execute::node_power::NodePowerLimit;


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
pub fn setup_log() {
    console::log("This console.log is from wasm!");

    MyCustomLog::new().init().unwrap();

    trace!("I'm trace");
    debug!("I'm debug");
    error!("I'm error");
    info!("I'm info");
    warn!("I'm warn");
}

#[wasm_bindgen]
pub async fn fetch_from_repo() -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = "https://raw.githubusercontent.com/ckrause/loda/master/programs/oeis/000/A000045.asm";

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    let text_result: Result<js_sys::Promise, JsValue> = resp.text();
    let text_jspromise: js_sys::Promise = match text_result {
        Ok(jspromise) => jspromise,
        Err(err) => {
            error!("Unable to obtain text() from response");
            return Err(err)
        }
    };
    // Convert this javascript `Promise` into a rust `Future`.
    let text_jsvalue: JsValue = wasm_bindgen_futures::JsFuture::from(text_jspromise).await?;
    
    let response_text: String = match text_jsvalue.as_string() {
        Some(value) => value,
        None => {
            error!("Unable to obtain convert JsValue to Rust String");
            let err = JsValue::from_str("Unable to obtain convert JsValue to Rust String");
            return Err(err);
        }
    };

    debug!("response: {:?}", response_text);
    eval_loda_program(&response_text);

    Ok(JsValue::from("success"))
}

#[wasm_bindgen]
pub fn myjsfunc_from_wasm() {
    eval_loda_program_mock();
}

fn eval_loda_program(source_code: &String) {
    let mut dm = DependencyManager::new(
        PathBuf::from("non-existing-dir"),
    );
    let runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, source_code).unwrap();
    runner.my_print_terms(10);
}

fn eval_loda_program_mock() {
    const PROGRAM: &str = r#"        
    mov $1,2
    pow $1,$0
    "#;
    let mut dm = DependencyManager::new(
        PathBuf::from("non-existing-dir"),
    );
    let source_code: String = PROGRAM.to_string();
    let runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, &source_code).unwrap();
    runner.my_print_terms(10);
}

impl ProgramRunner {
    fn my_print_terms(&self, count: u64) {
        if count >= 0x7fff_ffff_ffff_ffff {
            error!("Value is too high. Cannot be converted to 64bit signed integer.");
            return;
        }
        if count < 1 {
            error!("Expected number of terms to be 1 or greater.");
            return;
        }
        let mut cache = ProgramCache::new();
        let step_count_limit: u64 = 10000000;
        let mut step_count: u64 = 0;
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let result_run = self.run(
                &input, 
                RunMode::Verbose, 
                &mut step_count, 
                step_count_limit,
                NodeRegisterLimit::Unlimited,
                NodeBinomialLimit::Unlimited,
                NodeLoopLimit::Unlimited,
                NodePowerLimit::Unlimited,
                &mut cache
            );
            let output: RegisterValue = match result_run {
                Ok(value) => value,
                Err(error) => {
                    error!("Failure while computing term {}, error: {:?}", index, error);
                    return;
                }
            };
            if index == 0 {
                info!("{}", output.0);
                continue;
            }
            info!(",{}", output.0);
        }
        info!("\n");
        debug!("steps: {}", step_count);
        debug!("cache: {}", cache.hit_miss_info());
    }
}
