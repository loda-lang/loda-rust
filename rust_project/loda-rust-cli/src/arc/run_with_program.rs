use super::{Image, ImagePair, ImageToNumber, ImageUnicodeFormatting, Model, NumberToImage, register_arc_functions, StackStrings, Prediction, Grid};
use loda_rust_core::execute::{ProgramId, ProgramState};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::unofficial_function::{UnofficialFunctionRegistry, register_common_functions};
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use anyhow::Context;
use num_bigint::{BigInt, BigUint, ToBigInt};
use num_traits::Signed;
use std::path::PathBuf;
use std::fmt;

pub struct RunWithProgramResult {
    message_items: Vec::<String>,
    count_train_correct: usize,
    count_test_correct: usize,
    predictions: Vec<Prediction>,
}

impl RunWithProgramResult {
    pub fn messages(&self) -> String {
        self.message_items.join("\n")
    }

    pub fn count_train_correct(&self) -> usize {
        self.count_train_correct
    }

    pub fn count_test_correct(&self) -> usize {
        self.count_test_correct
    }

    pub fn predictions(&self) -> &Vec<Prediction> {
        &self.predictions
    }
}

impl fmt::Debug for RunWithProgramResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RunWithProgramResult count_train_correct: {} count_test_correct: {}\n message {}", self.count_train_correct, self.count_test_correct, self.messages())
    }
}

pub struct RunWithProgram {
    verify_test_output: bool,
    model: Model,
    train_pairs: Vec<ImagePair>,
    test_pairs: Vec<ImagePair>,
}

impl RunWithProgram {
    pub fn new(model: Model, verify_test_output: bool) -> anyhow::Result<Self> {
        let train_pairs: Vec<ImagePair> = model.images_train()?;
        let test_pairs: Vec<ImagePair> = model.images_test()?;
        Ok(Self {
            verify_test_output,
            model,
            train_pairs,
            test_pairs,
        })
    }

    pub fn create_dependency_manager() -> DependencyManager {
        let registry = UnofficialFunctionRegistry::new();
        register_common_functions(&registry);
        register_arc_functions(&registry);
        let dm = DependencyManager::new(
            DependencyManagerFileSystemMode::Virtual,
            PathBuf::from("non-existing-dir"),
            registry,
        );
        dm
    }

    pub const SIMPLE_PROGRAM_PRE: &'static str = r#"
    ; process "train"+"test" vectors
    mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
    mov $81,100 ; address of vector[0].input
    mov $82,102 ; address of vector[0].computed_output
    lps $80
        mov $0,$$81 ; load vector[x].input image
        ; before: do stuff to the image
        "#;

    pub const SIMPLE_PROGRAM_POST: &'static str = r#"
        ; after: do stuff to the image
        mov $$82,$0 ; save vector[x].computed_output image

        ; next iteration
        add $81,10 ; jump to address of next input image
        add $82,10 ; jump to address of next computed_output image
    lpe
    "#;

    pub fn convert_simple_to_full<S: AsRef<str>>(simple_program: S) -> String {
        format!("{}\n{}\n{}", Self::SIMPLE_PROGRAM_PRE, simple_program.as_ref(), Self::SIMPLE_PROGRAM_POST)
    }

    pub fn run_simple<S: AsRef<str>>(&self, simple_program: S) -> anyhow::Result<RunWithProgramResult> {
        let program = Self::convert_simple_to_full(simple_program);
        self.run_advanced(program)
    }

    pub fn run_advanced<S: AsRef<str>>(&self, program: S) -> anyhow::Result<RunWithProgramResult> {
        let program_str: &str = program.as_ref();

        let mut dm = Self::create_dependency_manager();
        let program_runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, program_str)
            .map_err(|e| anyhow::anyhow!("couldn't parse program string. error: {:?}", e))?;

        self.run_program_runner(&program_runner)
    }

    pub fn run_program_runner(&self, program_runner: &ProgramRunner) -> anyhow::Result<RunWithProgramResult> {
        // self.print_full_state();

        let mut cache = ProgramCache::new();

        // Blank state
        let step_count_limit: u64 = 900;
        let mut state = ProgramState::new(
            RunMode::Silent, 
            step_count_limit, 
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
        );

        self.initial_memory_layout(&mut state)?;

        // Invoke the actual run() function
        program_runner.program().run(&mut state, &mut cache).context("run_result error in program.run")?;

        self.process_output(&state)
    }

    #[allow(dead_code)]
    fn print_full_state(&self) {
        println!("model: {:?}", self.model.id());
        for (index, pair) in self.train_pairs.iter().enumerate() {
            let input = format!("input\n{}", pair.input.to_unicode_string());
            let output = format!("output\n{}", pair.output.to_unicode_string());
            let s: String = StackStrings::hstack(vec![input, output], " | ");
            println!("model: {:?} train#{}\n{}", self.model.id(), index, s);
        }
    }
        
    /// Prepare the starting state of the program
    ///
    /// Variable number of items in the `train` vector and `test` vector.
    /// 
    /// Memory layout:
    /// 
    /// ```
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
    /// ```
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

    fn process_output(&self, state: &ProgramState) -> anyhow::Result<RunWithProgramResult> {
        let pretty_print = false;

        let number_of_images: usize = self.train_pairs.len() + self.test_pairs.len();
        let computed_images: Vec<Image> = state.computed_images(number_of_images)?;

        let mut message_items = Vec::<String>::new();

        // Compare computed images with train[x].output
        let mut count_train_correct: usize = 0;
        for (index, pair) in self.train_pairs.iter().enumerate() {
            let computed_image: &Image = &computed_images[index];
            
            let expected_image: Image = pair.output.clone();
            if *computed_image == expected_image {
                if pretty_print {
                    println!("model: {:?} train#{} correct {}", self.model.id(), index, computed_image.to_unicode_string());
                }
                count_train_correct += 1;
                continue;
            }
            let s = format!("train. The computed output, doesn't match train[{}].output.\nExpected {:?}\nActual {:?}", index, expected_image, computed_image);
            message_items.push(s);

            if *computed_image == pair.input {
                // println!("train#{} incorrect. same as input", index);
                continue;
            }
            let same_size = computed_image.width() == expected_image.width() && computed_image.height() == expected_image.height();
            if !same_size {
                // println!("train#{} incorrect. computed {}x{} expected {}x{}", index, computed_image.width(), computed_image.height(), expected_image.width(), expected_image.height());
                continue;
            }
            // if pretty_print {
                // let computed_output_pretty = format!("computed output {}", computed_image.to_unicode_string());
                // let expected_output_pretty = format!("expected output {}", expected_image.to_unicode_string());
                // let comparison_pretty: String = StackStrings::hstack(vec![computed_output_pretty, expected_output_pretty], " | ");
                // println!("model: {:?} train#{} incorrect.\n{}", self.model.id(), index, comparison_pretty);

                // HtmlLog::compare_images(vec![computed_image.clone(), expected_image.clone()]);
            // }
        }
        let count_train: usize = self.train_pairs.len();

        let mut predictions = Vec::<Prediction>::new();

        // Compare computed images with test[x].output
        let mut count_test_correct: usize = 0;
        for (index, pair) in self.test_pairs.iter().enumerate() {
            let computed_image: &Image = &computed_images[count_train + index];
            
            if self.verify_test_output {
                let expected_image: Image = pair.output.clone();
                if *computed_image != expected_image {
                    let s = format!("test. The computed output, doesn't match test[{}].output.\nExpected {:?}\nActual {:?}", index, expected_image, computed_image);
                    message_items.push(s);
                    continue;
                }
            }

            if pretty_print {
                println!("model: {:?} test#{} correct {}", self.model.id(), index, computed_image.to_unicode_string());
            }

            let grid: Grid = Self::image_to_grid(&computed_image);
            let prediction = Prediction {
                prediction_id: index as u8,
                output: grid,
            };
            // println!("prediction: {:?}", prediction);
            predictions.push(prediction);

            count_test_correct += 1;
        }

        // println!("predictions: {:?}", predictions);
        
        let result = RunWithProgramResult { 
            message_items,
            count_train_correct,
            count_test_correct,
            predictions,
        };

        Ok(result)
    }

    fn image_to_grid(image: &Image) -> Grid {
        let mut grid = Grid::new();
        for y in 0..image.height() {
            let mut row = Vec::<u8>::new();
            for x in 0..image.width() {
                let pixel_value: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                row.push(pixel_value);
            }
            grid.push(row);
        }
        grid
    }
}

trait ComputedImages {
    fn computed_images(&self, number_of_images: usize) -> anyhow::Result<Vec<Image>>;
}

impl ComputedImages for ProgramState {
    /// Extract images from `ProgramState`
    /// 
    /// Variable number of items in the `train` vector and `test` vector.
    /// 
    /// The first image is at the address `102`. Add `10` to get to the following images.
    /// 
    /// Memory layout:
    /// 
    /// ```
    /// $102 = train[0] computed_output image
    /// $112 = train[1] computed_output image
    /// $122 = train[2] computed_output image
    /// $132 = test[0] computed_output image
    /// $142 = test[1] computed_output image
    /// ```
    fn computed_images(&self, number_of_images: usize) -> anyhow::Result<Vec<Image>> {
        let mut images = Vec::<Image>::with_capacity(number_of_images);
        for index in 0..number_of_images {
            let address: u64 = (index as u64) * 10 + 102;
            let computed_int: BigInt = self.get_u64(address).clone();
            if computed_int.is_negative() {
                return Err(anyhow::anyhow!("computed_images. output[{}]. Expected non-negative number, but got {:?}", address, computed_int));
            }
            let computed_uint: BigUint = computed_int.to_biguint()
                .ok_or_else(|| anyhow::anyhow!("computed_images. output[{}] computed_int.to_biguint return None", address))?;
            let computed_image: Image = computed_uint.to_image()
                .map_err(|e| anyhow::anyhow!("computed_images. output[{}] computed_uint.to_image. error: {:?}", address, e))?;
            if computed_image.is_empty() {
                // Verify that the output image is 1x1 or bigger.
                // Reject a "cheating" program. These are programs that copy from the expected_output to the actual_output.
                // For the test_pairs the expected_output is set to the empty image 0x0.
                // If the output has the size of 0x0, it seems like it has been "cheating".
                return Err(anyhow::anyhow!("computed_images. output[{}]. Expected an image bigger than 0x0, but image was empty", address));
            }
            images.push(computed_image);
        }
        Ok(images)
    }
}
