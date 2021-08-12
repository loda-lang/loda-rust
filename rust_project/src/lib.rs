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
use std::rc::Rc;
use core::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use control::DependencyManager;
use execute::{NodeLoopLimit, ProgramCache, ProgramId, ProgramRunner, RegisterValue, RunMode};
use execute::NodeRegisterLimit;
use execute::node_binomial::NodeBinomialLimit;
use execute::node_power::NodePowerLimit;
use parser::{ParsedProgram, ParseProgramError, parse_program, create_program, CreatedProgram, CreateProgramError};


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

// Construct url for a program id (eg A112088), like the following
// https://raw.githubusercontent.com/ckrause/loda/master/programs/oeis/112/A112088.asm
fn url_from_program_id(program_id: u64) -> String {
    let dir_index: u64 = program_id / 1000;
    let dir_index_string: String = format!("{:0>3}", dir_index);
    let filename_string: String = format!("A{:0>6}.asm", program_id);
    let baseurl = "https://raw.githubusercontent.com/ckrause/loda/master/programs/oeis";
    format!("{}/{}/{}", baseurl, dir_index_string, filename_string)
}

pub fn get_element_by_id(element_id: &str) -> Option<web_sys::Element> {
    web_sys::window()?.document()?.get_element_by_id(element_id)
}

#[wasm_bindgen]
pub async fn run_program_id(js_program_id: i32) -> Result<JsValue, JsValue> {
    if js_program_id < 0 {
        let err = JsValue::from_str("Expected program_id to be positive, but it's negative");
        return Err(err);
    }
    let execute_program_id: u64 = js_program_id as u64;

    let output_div: web_sys::Element = match get_element_by_id("output-inner") {
        Some(value) => value,
        None => {
            let err = JsValue::from_str("No #output-inner div found");
            return Err(err);
        }
    };

    if let Some(node) = output_div.dyn_ref::<web_sys::Node>() {
        let val = "Downloading";
        node.set_text_content(Some(&val));
    }

    let window = web_sys::window().unwrap();

    let mut pending_program_ids: Vec<u64> = vec!(execute_program_id);
    let mut already_fetched_program_ids = HashSet::<u64>::new();
    let mut virtual_filesystem: HashMap<u64, String> = HashMap::new();

    loop {
        let program_id: u64 = match pending_program_ids.pop() {
            Some(value) => value,
            None => {
                debug!("all programs have been fetched");
                break;
            }
        };
        if already_fetched_program_ids.contains(&program_id) {
            debug!("skip program that have already been fetched. {:?}", program_id);
            continue;
        }

        let url = url_from_program_id(program_id);

        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);
        let request = Request::new_with_str_and_init(&url, &opts)?;
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
        
        let parsed_program: ParsedProgram = match parse_program(&response_text) {
            Ok(value) => value,
            Err(error) => {
                error!("Unable to parse program: {:?}", error);
                let err = JsValue::from_str("Unable to parse program");
                return Err(err);
            }
        };
    
        let dependencies: Vec<u64> = parsed_program.direct_dependencies();
        debug!("program: {:?} has these dependencies: {:?}", program_id, dependencies);
        pending_program_ids.extend(dependencies);
        already_fetched_program_ids.insert(program_id);
        virtual_filesystem.insert(program_id, response_text);
    }

    let mut dm = DependencyManager::new(
        PathBuf::from("non-existing-dir"),
    );
    for (program_id, file_content) in virtual_filesystem {
        dm.virtual_filesystem_insert_file(program_id, file_content);
    }
    let runner: Rc::<ProgramRunner> = dm.load(execute_program_id).unwrap();
    execute_program(runner, 10, &output_div).await?;

    Ok(JsValue::from("success"))
}

#[wasm_bindgen]
pub async fn run_source_code(root_source_code: String) -> Result<JsValue, JsValue> {
    let root_parsed_program: ParsedProgram = match parse_program(&root_source_code) {
        Ok(value) => value,
        Err(error) => {
            error!("Unable to parse program: {:?}", error);
            let err = JsValue::from_str("Unable to parse program");
            return Err(err);
        }
    };

    let root_dependencies: Vec<u64> = root_parsed_program.direct_dependencies();
    debug!("the root program has these dependencies: {:?}", root_dependencies);

    let output_div: web_sys::Element = match get_element_by_id("output-inner") {
        Some(value) => value,
        None => {
            let err = JsValue::from_str("No #output-inner div found");
            return Err(err);
        }
    };

    if let Some(node) = output_div.dyn_ref::<web_sys::Node>() {
        let val = "Downloading";
        node.set_text_content(Some(&val));
    }

    let window = web_sys::window().unwrap();

    let mut pending_program_ids: Vec<u64> = vec!();
    let mut already_fetched_program_ids = HashSet::<u64>::new();
    let mut virtual_filesystem: HashMap<u64, String> = HashMap::new();

    pending_program_ids.extend(root_dependencies);

    loop {
        let program_id: u64 = match pending_program_ids.pop() {
            Some(value) => value,
            None => {
                debug!("all programs have been fetched");
                break;
            }
        };
        if already_fetched_program_ids.contains(&program_id) {
            debug!("skip program that have already been fetched. {:?}", program_id);
            continue;
        }

        let url = url_from_program_id(program_id);

        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);
        let request = Request::new_with_str_and_init(&url, &opts)?;
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
        
        let parsed_program: ParsedProgram = match parse_program(&response_text) {
            Ok(value) => value,
            Err(error) => {
                error!("Unable to parse program: {:?}", error);
                let err = JsValue::from_str("Unable to parse program");
                return Err(err);
            }
        };
    
        let dependencies: Vec<u64> = parsed_program.direct_dependencies();
        debug!("program: {:?} has these dependencies: {:?}", program_id, dependencies);
        pending_program_ids.extend(dependencies);
        already_fetched_program_ids.insert(program_id);
        virtual_filesystem.insert(program_id, response_text);
    }

    let mut dm = DependencyManager::new(
        PathBuf::from("non-existing-dir"),
    );
    for (program_id, file_content) in virtual_filesystem {
        dm.virtual_filesystem_insert_file(program_id, file_content);
    }
    let runner1: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, &root_source_code).unwrap();

    let runner: Rc::<ProgramRunner> = Rc::new(runner1);
    execute_program(runner, 10, &output_div).await?;

    Ok(JsValue::from("success"))
}

async fn execute_program(runner: Rc::<ProgramRunner>, count: u64, output_div: &web_sys::Element) -> Result<JsValue, JsValue> {
    if count >= 0x7fff_ffff_ffff_ffff {
        let err = JsValue::from_str("Value is too high. Cannot be converted to 64bit signed integer.");
        return Err(err);
    }
    if count < 1 {
        let err = JsValue::from_str("Expected number of terms to be 1 or greater.");
        return Err(err);
    }
    if let Some(node) = output_div.dyn_ref::<web_sys::Node>() {
        let val = "Executing";
        node.set_text_content(Some(&val));
    }
    let mut cache = ProgramCache::new();
    let step_count_limit: u64 = 10000000;
    let mut step_count: u64 = 0;
    for index in 0..(count as i64) {
        let input = RegisterValue::from_i64(index);
        let result_run = runner.run(
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
                let s = format!("Failure while computing term {}, error: {:?}", index, error);
                let err = JsValue::from_str(&s);
                return Err(err);
            }
        };
        let term_string: String = match index {
            0 => format!("{}", output.0),
            _ => format!(", {}", output.0)
        };
        if let Some(node) = output_div.dyn_ref::<web_sys::Node>() {
            let val = web_sys::window().unwrap().document().unwrap().create_element("span")?;
            val.set_text_content(Some(&term_string));
            if index == 0 {
                // remove all child elements
                node.set_text_content(None);
            }
            node.append_child(&val)?;
        }
    }
    debug!("steps: {}", step_count);
    debug!("cache: {}", cache.hit_miss_info());

    Ok(JsValue::from("success"))
}

#[wasm_bindgen]
pub fn perform_selfcheck() {
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
    info!("Selfcheck success");
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
        debug!("steps: {}", step_count);
        debug!("cache: {}", cache.hit_miss_info());
    }
}

struct WebDependencyManagerInner {
    count: i32,
    dependency_manager: DependencyManager,
    cache: ProgramCache,
    step_count: u64,
    outputted_byte_count: u64,
    program_runner: Rc::<ProgramRunner>,
}

impl WebDependencyManagerInner {
    fn create() -> WebDependencyManagerInner {
        let mut dm = DependencyManager::new(
            PathBuf::from("non-existing-dir"),
        );
        let cache = ProgramCache::new();

        let dummy_program = "mov $1,$0".to_string();
        let dummy_runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, &dummy_program).unwrap();
        let program_runner: Rc::<ProgramRunner> = Rc::new(dummy_runner);

        WebDependencyManagerInner {
            count: 0,
            dependency_manager: dm,
            cache: cache,
            step_count: 0,
            outputted_byte_count: 0,
            program_runner: program_runner,
        }
    }

    async fn run_source_code(&mut self, root_source_code: String) -> Result<JsValue, JsValue> {
        debug!("WebDependencyManagerInner.run_source_code() count: {:?} root_source_code: {:?}", self.count, root_source_code);
        let root_parsed_program: ParsedProgram = match parse_program(&root_source_code) {
            Ok(value) => value,
            Err(error) => {
                error!("Unable to parse program: {:?}", error);
                let err = JsValue::from_str("Unable to parse program");
                return Err(err);
            }
        };

        let root_dependencies: Vec<u64> = root_parsed_program.direct_dependencies();
        debug!("the root program has these dependencies: {:?}", root_dependencies);

        let output_div: web_sys::Element = match get_element_by_id("output-inner") {
            Some(value) => value,
            None => {
                let err = JsValue::from_str("No #output-inner div found");
                return Err(err);
            }
        };

        if let Some(node) = output_div.dyn_ref::<web_sys::Node>() {
            let val = "Downloading";
            node.set_text_content(Some(&val));
        }

        let window = web_sys::window().unwrap();

        let mut pending_program_ids: Vec<u64> = vec!();
        let mut already_fetched_program_ids = HashSet::<u64>::new();
        let mut virtual_filesystem: HashMap<u64, String> = HashMap::new();

        pending_program_ids.extend(root_dependencies);

        loop {
            let program_id: u64 = match pending_program_ids.pop() {
                Some(value) => value,
                None => {
                    debug!("all programs have been fetched");
                    break;
                }
            };
            if already_fetched_program_ids.contains(&program_id) {
                debug!("skip program that have already been fetched. {:?}", program_id);
                continue;
            }
            if self.dependency_manager.contains(program_id) {
                debug!("skip program that have already been parsed. {:?}", program_id);
                already_fetched_program_ids.insert(program_id);
                continue;
            }

            let url = url_from_program_id(program_id);

            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);
            let request = Request::new_with_str_and_init(&url, &opts)?;
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
            
            let parsed_program: ParsedProgram = match parse_program(&response_text) {
                Ok(value) => value,
                Err(error) => {
                    error!("Unable to parse program: {:?}", error);
                    let err = JsValue::from_str("Unable to parse program");
                    return Err(err);
                }
            };
        
            let dependencies: Vec<u64> = parsed_program.direct_dependencies();
            debug!("program: {:?} has these dependencies: {:?}", program_id, dependencies);
            pending_program_ids.extend(dependencies);
            already_fetched_program_ids.insert(program_id);
            virtual_filesystem.insert(program_id, response_text);
        }

        for (program_id, file_content) in virtual_filesystem {
            self.dependency_manager.virtual_filesystem_insert_file(program_id, file_content);
        }
        let runner1: ProgramRunner = self.dependency_manager.parse(ProgramId::ProgramWithoutId, &root_source_code).unwrap();
        let runner: Rc::<ProgramRunner> = Rc::new(runner1);
        self.program_runner = runner;

        self.step_count = 0;
        self.outputted_byte_count = 0;

        Ok(JsValue::from_str("success"))
    }

    async fn execute_current_program(&mut self, js_index: i32) -> Result<JsValue, JsValue> {
        // debug!("WebDependencyManagerInner.execute_current_program() js_index: {:?}", js_index);

        let output_div: web_sys::Element = match get_element_by_id("output-inner") {
            Some(value) => value,
            None => {
                let err = JsValue::from_str("No #output-inner div found");
                return Err(err);
            }
        };
        if js_index < 0 {
            let err = JsValue::from_str("Expecting non-negative index");
            return Err(err);
        }

        let output_byte_count_limit: u64 = 2000;
        let step_count_limit: u64 = 1000000000;
        let index: i64 = js_index as i64;

        let input = RegisterValue::from_i64(index);
        let result_run = self.program_runner.run(
            &input, 
            RunMode::Verbose, 
            &mut self.step_count, 
            step_count_limit,
            NodeRegisterLimit::Unlimited,
            NodeBinomialLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            NodePowerLimit::Unlimited,
            &mut self.cache
        );
        let output: RegisterValue = match result_run {
            Ok(value) => value,
            Err(error) => {
                error!("Failure while computing term {}, error: {:?}", index, error);
                let s = format!("Failure while computing term {}, error: {:?}", index, error);
                let err = JsValue::from_str(&s);
                return Err(err);
            }
        };
        let term_string: String = output.0.to_str_radix(10);
        self.outputted_byte_count += term_string.len() as u64;
        if self.outputted_byte_count > output_byte_count_limit {
            error!("Failure while computing term {}, the amount of output exceeded the limit {} bytes", index, output_byte_count_limit);
            let s = format!("Stop - output exceeded the limit of {} bytes", output_byte_count_limit);
            let err = JsValue::from_str(&s);
            return Err(err);
        }
        if let Some(node) = output_div.dyn_ref::<web_sys::Node>() {
            let val0 = web_sys::window().unwrap().document().unwrap().create_element("span")?;
            val0.set_class_name("separator");
            val0.set_text_content(Some(","));
            let val1 = web_sys::window().unwrap().document().unwrap().create_element("span")?;
            val1.set_class_name("term");
            val1.set_text_content(Some(&term_string));
            if index == 0 {
                // remove all child elements
                node.set_text_content(None);
                node.append_child(&val1)?;
            } else {
                node.append_child(&val0)?;
                node.append_child(&val1)?;
            }
        }

        Ok(JsValue::from_str("success"))
    }

    fn print_stats(&self) {
        debug!("steps: {}", self.step_count);
        debug!("cache: {}", self.cache.hit_miss_info());    
    }
}

#[wasm_bindgen]
pub struct WebDependencyManager {
    inner: Rc<RefCell<WebDependencyManagerInner>>,
}

#[wasm_bindgen]
impl WebDependencyManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WebDependencyManager, JsValue> {
        debug!("WebDependencyManager.new");
        let inner0 = WebDependencyManagerInner::create();
        let inner1 = Rc::new(RefCell::new(inner0));
        Ok(Self { 
            inner: inner1,
        })
    }
        
    pub fn increment(&mut self) {
        debug!("WebDependencyManager.increment");
        self.inner.borrow_mut().count += 1;
    }

    pub fn clone(&self) -> WebDependencyManager {
        WebDependencyManager {
            inner: self.inner.clone(),
        }
    }

    pub async fn run_source_code(self, root_source_code: String) -> Result<JsValue, JsValue> {
        self.inner.borrow_mut()
            .run_source_code(root_source_code).await
    }

    pub async fn execute_current_program(self, js_index: i32) -> Result<JsValue, JsValue> {
        self.inner.borrow_mut()
            .execute_current_program(js_index).await
    }

    pub fn print_stats(self) {
        self.inner.borrow_mut()
            .print_stats();
    }
}
