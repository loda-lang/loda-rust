use super::arc_json_model;
use super::arc_work_model;
use super::{Image, ImageToNumber, NumberToImage, register_arc_functions, Prediction, HtmlLog, ImageToHTML};
use loda_rust_core::execute::{ProgramId, ProgramState};
use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::unofficial_function::{UnofficialFunctionRegistry, register_common_functions};
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use anyhow::Context;
use num_bigint::{BigInt, BigUint, ToBigInt};
use num_traits::{Signed, One};
use std::path::PathBuf;
use std::fmt;

pub struct RunWithProgramResult {
    message_items: Vec::<String>,
    count_train_correct: usize,
    count_train_incorrect: usize,
    count_test_correct: usize,
    count_test_empty: usize,
    predictions: Vec<Prediction>,
}

impl RunWithProgramResult {
    pub fn messages(&self) -> String {
        self.message_items.join("\n")
    }

    pub fn count_train_correct(&self) -> usize {
        self.count_train_correct
    }

    pub fn count_train_incorrect(&self) -> usize {
        self.count_train_incorrect
    }

    pub fn count_test_correct(&self) -> usize {
        self.count_test_correct
    }

    pub fn count_test_empty(&self) -> usize {
        self.count_test_empty
    }

    pub fn predictions(&self) -> &Vec<Prediction> {
        &self.predictions
    }
}

impl fmt::Debug for RunWithProgramResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RunWithProgramResult count_train_correct: {} count_train_incorrect: {} count_test_correct: {}\n message {}", self.count_train_correct, self.count_train_incorrect, self.count_test_correct, self.messages())
    }
}

pub struct SolutionSimpleData {
    pub index: usize,
    pub image: Image,
}

pub type SolutionSimple = fn(SolutionSimpleData) -> anyhow::Result<Image>;

pub trait AnalyzeAndSolve {
    fn analyze(&mut self, task: &arc_work_model::Task) -> anyhow::Result<()>;
    fn solve(&self, data: &SolutionSimpleData) -> anyhow::Result<Image>;
}

pub struct RunWithProgram {
    verify_test_output: bool,
    task: arc_work_model::Task,
}

impl RunWithProgram {
    pub fn new(task: arc_work_model::Task, verify_test_output: bool) -> Self {
        Self {
            verify_test_output,
            task,
        }
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
        add $81,100 ; jump to address of next input image
        add $82,100 ; jump to address of next computed_output image
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

    #[allow(dead_code)]
    pub fn run_solution(&self, callback: SolutionSimple) -> anyhow::Result<RunWithProgramResult> {
        let mut computed_images = Vec::<Image>::new();
        for (index, pair) in self.task.pairs.iter().enumerate() {
            let data = SolutionSimpleData {
                index,
                image: pair.input.image.clone(),
            };
            let computed_image: Image = callback(data)?;
            computed_images.push(computed_image);
        }
        self.process_computed_images(computed_images)
    }

    #[allow(dead_code)]
    pub fn run_analyze_and_solve(
        &self,
        analyze_and_solve: &mut dyn AnalyzeAndSolve,
    ) -> anyhow::Result<RunWithProgramResult> {
        analyze_and_solve.analyze(&self.task)?;
        let mut computed_images = Vec::<Image>::new();
        for (index, pair) in self.task.pairs.iter().enumerate() {
            let data = SolutionSimpleData {
                index,
                image: pair.input.image.clone(),
            };
            let computed_image: Image = analyze_and_solve.solve(&data)?;
            computed_images.push(computed_image);
        }
        self.process_computed_images(computed_images)
    }

    pub fn run_program_runner(&self, program_runner: &ProgramRunner) -> anyhow::Result<RunWithProgramResult> {
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

        // Currently the ProgramState is recreated over and over.
        // Optimization. The ProgramState can be computed once.

        // Invoke the actual run() function
        program_runner.program().run(&mut state, &mut cache).context("run_result error in program.run")?;

        let number_of_images: usize = self.task.pairs.len();
        let computed_images: Vec<Image> = state.computed_images(number_of_images)?;
        self.process_computed_images(computed_images)
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
    /// $103..199 is reserved for train[0] extra data
    /// ---
    /// $200 = train[1] input
    /// $201 = train[1] expected_output
    /// $202 = train[1] computed_output
    /// $203..299 is reserved for train[1] extra data
    /// ---
    /// $300 = train[2] input
    /// $301 = train[2] expected_output
    /// $302 = train[2] computed_output
    /// $303..399 is reserved for train[2] extra data
    /// ---
    /// $400 = train[3] input
    /// $401 = train[3] expected_output
    /// $402 = train[3] computed_output
    /// $403..499 is reserved for train[3] extra data
    /// ---
    /// $500 = test[0] input
    /// $501 = test[0] expected_output <---- this is not provided, it's up to the program to compute it.
    /// $502 = test[0] computed_output
    /// $503..599 is reserved for test[0] extra data
    /// ```
    fn initial_memory_layout(&self, state: &mut ProgramState) -> anyhow::Result<()> {

        // Traverse the `Train` pairs
        let mut count_train: usize = 0;
        for pair in &self.task.pairs {
            if pair.pair_type != arc_work_model::PairType::Train {
                continue;
            }

            let index: usize = count_train;
            // memory[x*100+100] = train[x].input
            {
                let image_number_uint: BigUint = pair.input.image.to_number().context("pair.input image to number")?;
                let image_number_int: BigInt = image_number_uint.to_bigint().context("pair.input BigUint to BigInt")?;
                state.set_u64((index * 100 + 100) as u64, image_number_int).context("pair.input, set_u64")?;
            }

            // memory[x*100+101] = train[x].output
            {
                let image_number_uint: BigUint = pair.output.image.to_number().context("pair.output image to number")?;
                let image_number_int: BigInt = image_number_uint.to_bigint().context("pair.output BigUint to BigInt")?;
                state.set_u64((index * 100 + 101) as u64, image_number_int).context("pair.output, set_u64")?;
            }

            // Ideas for data to make available to the program.
            // output_size
            // output_palette
            // substitutions, replace this color with that color
            // substitutions, replace this image with that image
            // remove trim color
            // remove grid color

            count_train += 1;
        }

        // Traverse the `Test` pairs
        let mut count_test: usize = 0;
        for pair in &self.task.pairs {
            if pair.pair_type != arc_work_model::PairType::Test {
                continue;
            }

            let index: usize = count_train + count_test;
            // memory[(count_train + x)*100+100] = test[x].input
            {
                let image_number_uint: BigUint = pair.input.image.to_number().context("pair.input image to number")?;
                let image_number_int: BigInt = image_number_uint.to_bigint().context("pair.input BigUint to BigInt")?;
                state.set_u64((index * 100 + 100) as u64, image_number_int).context("pair.input, set_u64")?;
            }

            // The program is never supposed to read from the the test[x].output register.
            // memory[(count_train + x)*100+101] is where the program is supposed to write its predicted output.
            // Use `-1` as placeholder so it's easy to spot when the image is missing.
            {
                let value: BigInt = -BigInt::one();
                state.set_u64((index * 100 + 101) as u64, value).context("pair.output, set_u64")?;
            }

            count_test += 1;
        }

        let count_all: usize = count_train + count_test;

        // memory[97] = length of "train" vector
        let count_train_bigint: BigInt = count_train.to_bigint().context("count_train.to_bigint")?;
        state.set_u64(97, count_train_bigint).context("set_u64 count_train_bigint")?;

        // memory[98] = length of "test" vector
        let count_test_bigint: BigInt = count_test.to_bigint().context("count_test.to_bigint")?;
        state.set_u64(98, count_test_bigint).context("set_u64 count_test_bigint")?;

        // memory[99] = length of "train"+"test" vectors
        let count_all_bigint: BigInt = count_all.to_bigint().context("count_all.to_bigint")?;
        state.set_u64(99, count_all_bigint).context("set_u64 count_all_bigint")?;

        Ok(())
    }

    fn process_computed_images(&self, computed_images: Vec<Image>) -> anyhow::Result<RunWithProgramResult> {
        let pretty_print = false;

        let mut status_texts = Vec::<&str>::new();

        let mut message_items = Vec::<String>::new();

        // Traverse the `Train` pairs
        // Compare computed images with train[x].output
        let mut count_train_correct: usize = 0;
        let mut count_train_incorrect: usize = 0;
        let mut count_train: usize = 0;
        for pair in &self.task.pairs {
            if pair.pair_type != arc_work_model::PairType::Train {
                continue;
            }

            let index: usize = count_train;
            count_train += 1;

            let computed_image: &Image = &computed_images[index];
            
            let expected_image: Image = pair.output.image.clone();
            if *computed_image == expected_image {
                count_train_correct += 1;
                if pretty_print {
                    status_texts.push("OK");
                }
                continue;
            }
            count_train_incorrect += 1;
            let s = format!("train. The computed output, doesn't match train[{}].output.\nExpected {:?}\nActual {:?}", index, expected_image, computed_image);
            message_items.push(s);
            if pretty_print {
                status_texts.push("Incorrect");
            }
        }

        let mut predictions = Vec::<Prediction>::new();

        // Traverse the `Test` pairs
        // Compare computed images with test[x].output
        let mut count_test_correct: usize = 0;
        let mut count_test_empty: usize = 0;
        let mut count_test: usize = 0;
        for pair in &self.task.pairs {
            if pair.pair_type != arc_work_model::PairType::Test {
                continue;
            }

            let index: usize = count_test;
            count_test += 1;

            let computed_image: &Image = &computed_images[count_train + index];
            if computed_image.is_empty() {
                count_test_empty += 1;
                if pretty_print {
                    status_texts.push("Empty");
                }
                continue;
            }
            if self.verify_test_output {
                let expected_image: Image = pair.output.test_image.clone();
                if *computed_image != expected_image {
                    let s = format!("test. The computed output, doesn't match test[{}].output.\nExpected {:?}\nActual {:?}", index, expected_image, computed_image);
                    message_items.push(s);
                    if pretty_print {
                        status_texts.push("Incorrect");
                    }
                    continue;
                }
                if pretty_print {
                    status_texts.push("OK");
                }
            } else {
                if pretty_print {
                    status_texts.push("Unverified");
                }
            }

            let grid: arc_json_model::Grid = Self::image_to_grid(&computed_image);
            let prediction = Prediction {
                prediction_id: index as u8,
                output: grid,
            };
            predictions.push(prediction);

            count_test_correct += 1;
        }

        if pretty_print {
            self.inspect_computed_images(&computed_images, &status_texts);
        }

        let result = RunWithProgramResult { 
            message_items,
            count_train_correct,
            count_train_incorrect,
            count_test_correct,
            count_test_empty,
            predictions,
        };

        Ok(result)
    }

    fn image_to_grid(image: &Image) -> arc_json_model::Grid {
        let mut grid = arc_json_model::Grid::new();
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

    fn inspect_computed_images(&self, computed_images: &Vec<Image>, status_texts: &Vec<&str>) {

        // Table row with input and row with expected output
        let mut row_input: String = "<tr>".to_string();
        let mut row_output: String = "<tr>".to_string();

        // Traverse the `Train` pairs
        for pair in &self.task.pairs {
            if pair.pair_type != arc_work_model::PairType::Train {
                continue;
            }
            {
                row_input += "<td>";
                row_input += &pair.input.image.to_html();
                row_input += "</td>";
            }
            {
                row_output += "<td>";
                row_output += &pair.output.image.to_html();
                row_output += "</td>";
            }
        }

        // Traverse the `Test` pairs
        for pair in &self.task.pairs {
            if pair.pair_type != arc_work_model::PairType::Test {
                continue;
            }
            {
                row_input += "<td>";
                row_input += &pair.input.image.to_html();
                row_input += "</td>";
            }
            {
                row_output += "<td>";
                row_output += &pair.output.test_image.to_html();
                row_output += "</td>";
            }
        }
        row_input += "<td>Input</td></tr>";
        row_output += "<td>Output</td></tr>";

        // Table row with computed output
        let mut row_predicted: String = "<tr>".to_string();
        for computed_image in computed_images {
            row_predicted += "<td>";
            row_predicted += &computed_image.to_html();
            row_predicted += "</td>";
        }
        row_predicted += "<td>Predicted</td></tr>";

        // Table row with status text
        let mut row_status: String = "<tr>".to_string();
        for text in status_texts {
            row_status += &format!("<td>{}</td>", text);
        }
        row_status += "</tr>";

        let html = format!("<h2>{}</h2><table>{}{}{}{}</table>", self.task.id, row_input, row_output, row_predicted, row_status);
        HtmlLog::html(html);
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
    /// The first image is at the address `102`. Add `100` to get to the following images.
    /// 
    /// Memory layout:
    /// 
    /// ```
    /// $102 = train[0] computed_output image
    /// $202 = train[1] computed_output image
    /// $302 = train[2] computed_output image
    /// $402 = test[0] computed_output image
    /// $502 = test[1] computed_output image
    /// ```
    fn computed_images(&self, number_of_images: usize) -> anyhow::Result<Vec<Image>> {
        let mut images = Vec::<Image>::with_capacity(number_of_images);
        for index in 0..number_of_images {
            let address: u64 = (index as u64) * 100 + 102;
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
