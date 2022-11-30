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
    use std::sync::{Arc, RwLock};

    struct MyContext;
    
    trait MyPluginLegacy {
        fn plugin_name(&self) -> &'static str;
        fn execute(&mut self, context: &MyContext) -> Result<String, Box<dyn Error>>;
    }
    
    type MyPluginItemLegacy = Rc<RefCell<dyn MyPluginLegacy>>;

    struct HelloWorldPluginLegacy;

    impl MyPluginLegacy for HelloWorldPluginLegacy {
        fn plugin_name(&self) -> &'static str {
            "HelloWorldPluginLegacy"
        }
    
        fn execute(&mut self, _context: &MyContext) -> Result<String, Box<dyn Error>> {
            debug!("execute");
            Ok("executed".to_string())
        }
    }
    
    #[test]
    fn test_10000_function_plugin_singlethreaded() -> anyhow::Result<()> {
        let mut plugin_vec: Vec<MyPluginItemLegacy> = vec!();
        let the_plugin = Rc::new(RefCell::new(HelloWorldPluginLegacy {}));
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

    trait MyPlugin2: Send + Sync {
        fn name(&self) -> &'static str;
        fn inputs(&self) -> u8;
        fn outputs(&self) -> u8;
        fn execute(&self) -> Result<String, Box<dyn Error>>;
    }

    struct HelloWorldPlugin2;

    impl MyPlugin2 for HelloWorldPlugin2 {
        fn name(&self) -> &'static str {
            "HelloWorldPlugin2"
        }

        fn inputs(&self) -> u8 {
            1
        }
    
        fn outputs(&self) -> u8 {
            1
        }
    
        fn execute(&self) -> Result<String, Box<dyn Error>> {
            debug!("execute");
            Ok("executed".to_string())
        }
    }

    #[derive(Debug)]
    struct RegistryInner<T = Box<dyn MyPlugin2>> {
        plugin_vec: Vec<Arc<T>>,
    }

    impl<T> RegistryInner<T> {
    }

    struct Registry {
        inner: RwLock<RegistryInner>,
    }

    impl Registry {
        fn new() -> Arc<Registry> {
            let inner = RegistryInner {
                plugin_vec: vec!(),
            };
            let instance = Registry { 
                inner: RwLock::new(inner) 
            };
            Arc::new(instance)
        }

        fn add_plugin(&self, plugin: Arc<Box<dyn MyPlugin2>>) {
            self.inner.write().unwrap().plugin_vec.push(plugin);
        }

        fn execute(&self) -> anyhow::Result<String> {
            let mut execute_output: Option<String> = None;
            let plugin_vec = self.inner.read().unwrap().plugin_vec.clone();
            for plugin in plugin_vec {
                let result = plugin.execute();
                match result {
                    Ok(value) => {
                        execute_output = Some(value);
                    },
                    Err(error) => {
                        return Err(anyhow::anyhow!("execute failed. error: {:?}", error));
                    }
                }
            }
            let value: String = match execute_output {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("plugin didn't return anything"));
                }
            };
            Ok(value)
        }
    }

    #[test]
    fn test_20000_function_plugin_multithreaded_immutable() -> anyhow::Result<()> {
        let registry = Registry::new();
        let the_plugin = HelloWorldPlugin2 {};
        registry.add_plugin(Arc::new(Box::new(the_plugin)));
        let execute_output: String = registry.execute()?;
        assert_eq!(execute_output, "executed2");
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
