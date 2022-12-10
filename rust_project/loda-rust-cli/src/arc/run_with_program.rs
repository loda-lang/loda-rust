use super::{Image, ImagePair, ImageToNumber, Model, NumberToImage, register_arc_functions};
use loda_rust_core::execute::{ProgramId, ProgramState};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::unofficial_function::UnofficialFunctionRegistry;
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use anyhow::Context;
use num_bigint::{BigInt, BigUint, ToBigInt};
use num_traits::Signed;
use std::path::PathBuf;

pub struct RunWithProgram {
    train_pairs: Vec<ImagePair>,
    test_pairs: Vec<ImagePair>,
}

impl RunWithProgram {
    pub fn new(model: Model) -> anyhow::Result<Self> {
        let train_pairs: Vec<ImagePair> = model.images_train()?;
        let test_pairs: Vec<ImagePair> = model.images_test()?;
        Ok(Self {
            train_pairs,
            test_pairs,
        })
    }

    pub fn create_dependency_manager() -> DependencyManager {
        let registry = UnofficialFunctionRegistry::new();
        register_arc_functions(&registry);
        let dm = DependencyManager::new(
            DependencyManagerFileSystemMode::Virtual,
            PathBuf::from("non-existing-dir"),
            registry,
        );
        dm
    }

    pub fn run<S: AsRef<str>>(&self, program: S) -> anyhow::Result<()> {
        let program_str: &str = program.as_ref();

        let mut dm = Self::create_dependency_manager();
        let program_runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, program_str).expect("ProgramRunner");
        let mut cache = ProgramCache::new();

        // Blank state
        let step_count_limit: u64 = 1000;
        let mut state = ProgramState::new(
            RunMode::Silent, 
            step_count_limit, 
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
        );

        self.initial_memory_layout(&mut state)?;

        // Invoke the actual run() function
        program_runner.program().run(&mut state, &mut cache).context("run_result error in program.run")?;

        self.process_output(&state)?;

        Ok(())
    }
        
    /// Prepare starting state of the program
    ///
    /// MEMORY LAYOUT
    /// $97 = length of "train" vector
    /// $98 = length of "test" vector
    /// $99 = length of "train"+"test" vectors
    /// ---
    /// $100 = train[0] input
    /// $101 = train[0] expected_output
    /// $102 = train[0] computed_output
    /// $103..109 is reserved for train[0] extra data
    /// ---
    /// $110 = train[1] input
    /// $111 = train[1] expected_output
    /// $112 = train[1] computed_output
    /// $113..119 is reserved for train[1] extra data
    /// ---
    /// $120 = train[2] input
    /// $121 = train[2] expected_output
    /// $122 = train[2] computed_output
    /// $123..129 is reserved for train[2] extra data
    /// ---
    /// $130 = train[3] input
    /// $131 = train[3] expected_output
    /// $132 = train[3] computed_output
    /// $133..139 is reserved for train[3] extra data
    /// ---
    /// $140 = test[0] input
    /// $141 = test[0] expected_output <---- this is not provided, it's up to the program to compute it.
    /// $142 = test[0] computed_output
    /// $143..149 is reserved for test[0] extra data
    fn initial_memory_layout(&self, state: &mut ProgramState) -> anyhow::Result<()> {
        let count_train: usize = self.train_pairs.len();
        let count_test: usize = self.test_pairs.len();
        let count_all: usize = count_train + count_test;
        // memory[97] = length of "train" vector
        let count_train_bigint: BigInt = count_train.to_bigint().expect("count_train.to_bigint");
        state.set_u64(97, count_train_bigint).context("set_u64 count_train_bigint")?;
        // memory[98] = length of "test" vector
        let count_test_bigint: BigInt = count_test.to_bigint().expect("count_test.to_bigint");
        state.set_u64(98, count_test_bigint).context("set_u64 count_test_bigint")?;
        // memory[99] = length of "train"+"test" vectors
        let count_all_bigint: BigInt = count_all.to_bigint().expect("count_all.to_bigint");
        state.set_u64(99, count_all_bigint).context("set_u64 count_all_bigint")?;
        // memory[x*10+100] = train[x].input
        for (index, pair) in self.train_pairs.iter().enumerate() {
            let image_number_uint: BigUint = pair.input.to_number().expect("pair.input image to number");
            let image_number_int: BigInt = image_number_uint.to_bigint().expect("pair.input BigUint to BigInt");
            state.set_u64((index * 10 + 100) as u64, image_number_int).context("pair.input, set_u64")?;
        }
        // memory[x*10+101] = train[x].output
        for (index, pair) in self.train_pairs.iter().enumerate() {
            let image_number_uint: BigUint = pair.output.to_number().expect("pair.output image to number");
            let image_number_int: BigInt = image_number_uint.to_bigint().expect("pair.output BigUint to BigInt");
            state.set_u64((index * 10 + 101) as u64, image_number_int).context("pair.output, set_u64")?;
        }
        // memory[(count_train + x)*10+100] = test[x].input
        for (index, pair) in self.test_pairs.iter().enumerate() {
            let image_number_uint: BigUint = pair.input.to_number().expect("pair.input image to number");
            let image_number_int: BigInt = image_number_uint.to_bigint().expect("pair.input BigUint to BigInt");
            let set_index: usize = (count_train + index) * 10 + 100;
            state.set_u64(set_index as u64, image_number_int).context("pair.input, set_u64")?;
        }
        Ok(())
    }

    fn process_output(&self, state: &ProgramState) -> anyhow::Result<()> {

        // Output vector
        let output_count: u8 = 1;
        let mut output_vec = Vec::<BigInt>::with_capacity(output_count as usize);
        for index in 0..output_count {
            let value: BigInt = state.get_u64(index as u64).clone();
            output_vec.push(value);
        }

        // Output
        if output_vec.len() != 1 {
            return Err(anyhow::anyhow!("output_vec. Expected 1 output value, but got {:?}", output_vec.len()));
        }
        let output0_int: &BigInt = &output_vec[0];

        let expected_output: BigInt = 3.to_bigint().unwrap();
        assert_eq!(*output0_int, expected_output);

        // Compare computed images with train[x].output
        for (index, pair) in self.train_pairs.iter().enumerate() {
            let address: u64 = (index as u64) * 10 + 102;
            let computed_int: BigInt = state.get_u64(address).clone();
            if computed_int.is_negative() {
                return Err(anyhow::anyhow!("output[{}]. Expected non-negative number, but got {:?}", address, computed_int));
            }
            let computed_uint: BigUint = computed_int.to_biguint().expect("output biguint");
            let computed_image: Image = computed_uint.to_image().expect("output uint to image");
            
            let expected_image: Image = pair.output.clone();
            assert_eq!(computed_image, expected_image, "output[{}]. The computed output, doesn't match train[{}].output", address, index);
        }
        let count_train: usize = self.train_pairs.len();

        // Compare computed images with test[x].output
        for (index, pair) in self.test_pairs.iter().enumerate() {
            let address: u64 = ((index + count_train) as u64) * 10 + 102;
            let computed_int: BigInt = state.get_u64(address).clone();
            if computed_int.is_negative() {
                return Err(anyhow::anyhow!("output[{}]. Expected non-negative number, but got {:?}", address, computed_int));
            }
            let computed_uint: BigUint = computed_int.to_biguint().expect("output biguint");
            let computed_image: Image = computed_uint.to_image().expect("output uint to image");
            
            let expected_image: Image = pair.output.clone();
            assert_eq!(computed_image, expected_image, "output[{}]. The computed output, doesn't match test[{}].output", address, index);
        }
        
        Ok(())
    }
}
