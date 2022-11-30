#[cfg(test)]
mod tests {
    use loda_rust_core::execute::ProgramId;
    use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
    use loda_rust_core::execute::NodeRegisterLimit;
    use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
    use crate::config::Config;
    use num_bigint::BigInt;
    use num_bigint::ToBigInt;
    use std::path::PathBuf;
    use std::rc::Rc;
    use core::cell::RefCell;
    use std::error::Error;
    
    struct MyContext;
    
    trait MyPlugin {
        fn plugin_name(&self) -> &'static str;
        fn execute(&mut self, context: &MyContext) -> Result<String, Box<dyn Error>>;
        fn human_readable_summary(&self) -> String;
    
        fn format_summary(&self) -> String {
            let name: &str = self.plugin_name();
            let summary: String = self.human_readable_summary();
            format!("\n{}\n{}\n", name.trim(), summary.trim())
        }
    }
    
    type MyPluginItem = Rc<RefCell<dyn MyPlugin>>;

    struct HelloWorldPlugin;

    impl MyPlugin for HelloWorldPlugin {
        fn plugin_name(&self) -> &'static str {
            "HelloWorldPlugin"
        }
    
        fn execute(&mut self, _context: &MyContext) -> Result<String, Box<dyn Error>> {
            debug!("execute");
            Ok("executed".to_string())
        }
    
        fn human_readable_summary(&self) -> String {
            format!("hello world")
        }
    }
    
    #[test]
    fn test_10000_function_plugin_singlethreaded() -> anyhow::Result<()> {
        let mut plugin_vec: Vec<MyPluginItem> = vec!();
        let the_plugin = Rc::new(RefCell::new(HelloWorldPlugin {}));
        plugin_vec.push(the_plugin);

        let context = MyContext {};
        let mut execute_output: Option<String> = None;
        for plugin in plugin_vec.iter() {
            let result = plugin.borrow_mut().execute(&context);
            match result {
                Ok(value) => {
                    execute_output = Some(value);
                },
                Err(error) => {
                    return Err(anyhow::anyhow!("execute failed. error: {:?}", error));
                }
            }
        }
        assert_eq!(execute_output, Some("executed".to_string()));
        Ok(())
    }

    // #[test]
    fn test_30000_simple() -> anyhow::Result<()> {
        let program_content: &str = "
        f11 $0,1
        ";

        let config = Config::load();
        let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            loda_programs_oeis_dir,
        );
        let result_parse = dm.parse(ProgramId::ProgramWithoutId, &program_content.to_string());

        let program_runner: ProgramRunner = result_parse.expect("ProgramRunner");

        let step_count_limit: u64 = 1000000000;
        let mut cache = ProgramCache::new();
        let mut step_count: u64 = 0;

        let input_original: i32 = 132;
        let input_raw_int: BigInt = match input_original.to_bigint() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Integrity error. Couldn't convert to BigInt."));
            }
        };
        let input = RegisterValue(input_raw_int);
        
        let result_run = program_runner.run(
            &input, 
            RunMode::Silent, 
            &mut step_count, 
            step_count_limit,
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            &mut cache
        );
        let output: RegisterValue = match result_run {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("run failed for input {:?}, error: {:?}", input_original, error));
            }
        };

        let output_i64: i64 = match output.try_to_i64() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("output value {} is out of range i64 when computing term for input {}", output, input_original));
            }
        };
        if output_i64 < 0 || output_i64 > 255 {
            return Err(anyhow::anyhow!("output value {} is out of range [0..255] when computing term for input {}", output, input_original));
        }
        let output: u8 = output_i64 as u8;

        assert_eq!(output, 42);

        Ok(())
    }
}
