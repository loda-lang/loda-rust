use super::arc_json_model;
use super::arc_work_model;
use super::{Image, ImageSize, ImageToNumber, NumberToImage, register_arc_functions, Prediction, HtmlLog, ImageToHTML};
use super::{ImageRotate, ImageSymmetry, Color};
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

enum MemoryLayoutItem {
    InputImage = 0,
    ExpectedOutputImage = 1,
    ComputedOutputImage = 2,

    /// When the `PredictedOutputWidth` is available it's a value in the range `[0..255]`.
    /// 
    /// When it's not available then the value is `-1`.
    PredictedOutputWidth = 3,

    /// When the `PredictedOutputHeight` is available it's a value in the range `[0..255]`.
    /// 
    /// When it's not available then the value is `-1`.
    PredictedOutputHeight = 4,
    
    /// The output image seems to a copy of the input image.
    /// 
    /// The changes are isolated to pixels with a particular color.
    /// It's unclear how many of the pixels that changes state.
    /// It may be just a few pixels that changes state, or it may be all of the pixels.
    /// 
    /// When the `OutputImageIsInputImageWithChangesLimitedToPixelsWithColor` is available it's a color value in the range `[0..255]`.
    /// 
    /// When it's not available then the value is `-1`.
    OutputImageIsInputImageWithChangesLimitedToPixelsWithColor = 5,

    /// Some ARC tasks is about repairing damaged pixels.
    /// 
    /// In a highly symmetric pattern, it's sometimes possible to determine what pixels are damaged.
    /// If 3 out of 4 pixels agree on a color, that the damage color seems to be the same, then it's probably a damaged pixel.
    /// This is a mask with the damaged pixels, where a pixel value of `1=damaged` and `0=good`.
    /// 
    /// Drawback: It cannot detect damages in rotational symmetries.
    ///
    /// When it's not available then the value is `-1`.
    RepairMask = 6,

    /// Some ARC tasks is about repairing damaged pixels.
    /// 
    /// In a highly symmetric pattern, it's sometimes possible to determine what pixels are damaged.
    /// If 3 out of 4 pixels agree on a color, that the damage pixel gets repaired.
    /// 
    /// Drawback: It cannot fix rotational symmetries.
    ///
    /// When it's not available then the value is `-1`.
    RepairedImage = 7,

    /// Some ARC tasks contains a grid structure.
    /// 
    /// This is a fuzzy guess about what the grid pattern may be like.
    /// 
    /// - Drawback: It cannot detect a grid uneven spacing.
    /// - Drawback: It cannot detect a grid without a grid line, the grid line must be 1px or wider.
    /// - Drawback: It cannot detect a horizontal stack of cells.
    /// - Drawback: It cannot detect a vertical stack of cells.
    ///
    /// When it's not available then the value is `-1`.
    GridMask = 8,

    /// Some ARC tasks contains a grid structure.
    /// 
    /// This is a fuzzy guess about what color is used for the grid lines.
    ///
    /// When it's not available then the value is `-1`.
    GridColor = 9,

    /// Clusters of pixels that makes up objects.
    /// - The value `0` indicates that it's not an object.
    /// - The values `[1..255]` are object id's.
    /// 
    /// This is a fuzzy guess about where the objects are located.
    /// In a grid the objects are the cells. The top-left cell is assigned `object id=1`.
    /// The next cell is assigned value `object id=2`. Until reaching the bottom-right cell.
    /// The grid itself is assigned the value `0`, so that it's not considered an object.
    ///
    /// When it's not available then the value is `-1`.
    EnumeratedObjects = 10,
    
    /// The image that appear when applying substitution rules.
    /// 
    /// This is only available when the output-size is the same as the input-size.
    /// And when a pattern between 1x1 and 4x4 can be replaced in the input to get to the output.
    ///
    /// When it's not available then the value is `-1`.
    SubstitutionRuleApplied = 11,

    /// When there is a predicted output size, and there is a single color in the predicted palette
    /// 
    /// Then this contains an image with the size and the color
    ///
    /// When it's not available then the value is `-1`.
    PredictedSingleColorImage = 12,

    /// The training pairs agree on removing a single color,
    /// or each training pair has its own color that is being removed.
    RemovalColor = 13,

    /// The input image's most popular color across all the training pairs.
    InputMostPopularColor = 14,

    // Ideas for more
    // Number of cells horizontal
    // Number of cells vertical
    // Cell width
    // Cell height
    // horizontal_periodicity, vertical_periodicity
    // Background color in input
    // Predicted background color in output
    // Background color in input and output
    // Primary color
    // Secondary color
    // Primary object mask
    // Child object mask
    // Cell mask
}

pub struct RunWithProgramResult {
    message_items: Vec::<String>,
    count_train_correct: usize,
    count_train_incorrect: usize,
    count_test_correct: usize,
    count_test_empty: usize,
    predictions: Vec<Prediction>,
    all_train_pairs_are_correct: bool,
    all_test_pairs_are_correct: bool,
}

impl RunWithProgramResult {
    pub fn messages(&self) -> String {
        self.message_items.join("\n")
    }

    pub fn count_train_correct(&self) -> usize {
        self.count_train_correct
    }

    #[allow(dead_code)]
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

    pub fn all_train_pairs_are_correct(&self) -> bool {
        self.all_train_pairs_are_correct
    }

    pub fn all_test_pairs_are_correct(&self) -> bool {
        self.all_test_pairs_are_correct
    }

    pub fn all_train_pairs_and_test_pairs_are_correct(&self) -> bool {
        self.all_train_pairs_are_correct && self.all_test_pairs_are_correct
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
    fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
        Ok(())
    }

    fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image>;
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
            let computed_image: Image = analyze_and_solve.solve(&data, &self.task)?;
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
    /// $103 = train[0] PredictedOutputWidth
    /// $104 = train[0] PredictedOutputHeight
    /// $105..199 is reserved for train[0] extra data
    /// ---
    /// $200 = train[1] input
    /// $201 = train[1] expected_output
    /// $202 = train[1] computed_output
    /// $203 = train[1] PredictedOutputWidth
    /// $204 = train[1] PredictedOutputHeight
    /// $205..299 is reserved for train[1] extra data
    /// ---
    /// $300 = train[2] input
    /// $301 = train[2] expected_output
    /// $302 = train[2] computed_output
    /// $303 = train[2] PredictedOutputWidth
    /// $304 = train[2] PredictedOutputHeight
    /// $305..399 is reserved for train[2] extra data
    /// ---
    /// $400 = train[3] input
    /// $401 = train[3] expected_output
    /// $402 = train[3] computed_output
    /// $403 = train[3] PredictedOutputWidth
    /// $404 = train[3] PredictedOutputHeight
    /// $405..499 is reserved for train[3] extra data
    /// ---
    /// $500 = test[0] input
    /// $501 = test[0] expected_output <---- this is not provided, it's up to the program to compute it.
    /// $502 = test[0] computed_output
    /// $503 = test[0] PredictedOutputWidth
    /// $504 = test[0] PredictedOutputHeight
    /// $505..599 is reserved for test[0] extra data
    /// ```
    fn initial_memory_layout(&self, state: &mut ProgramState) -> anyhow::Result<()> {

        // Traverse the `Train` pairs
        let mut count_train: usize = 0;
        for pair in &self.task.pairs {
            if pair.pair_type != arc_work_model::PairType::Train {
                continue;
            }

            let index: usize = count_train;
            let address: u64 = (index * 100 + 100) as u64;
            // memory[x*100+100] = train[x].input
            {
                let image_number_uint: BigUint = pair.input.image.to_number().context("pair.input image to number")?;
                let image_number_int: BigInt = image_number_uint.to_bigint().context("pair.input BigUint to BigInt")?;
                state.set_u64(address + MemoryLayoutItem::InputImage as u64, image_number_int).context("pair.input, set_u64")?;
            }

            // memory[x*100+101] = train[x].output
            {
                let image_number_uint: BigUint = pair.output.image.to_number().context("pair.output image to number")?;
                let image_number_int: BigInt = image_number_uint.to_bigint().context("pair.output BigUint to BigInt")?;
                state.set_u64(address + MemoryLayoutItem::ExpectedOutputImage as u64, image_number_int).context("pair.output, set_u64")?;
            }

            // memory[x*100+102] = train[x].computed output
            {
                let value: BigInt = -BigInt::one();
                state.set_u64(address + MemoryLayoutItem::ComputedOutputImage as u64, value).context("pair.ComputedOutputImage, set_u64")?;
            }

            // memory[x*100+103] = train[x].predicted output width
            // memory[x*100+104] = train[x].predicted output height
            {
                let width: i16;
                let height: i16;
                if let Some(size) = pair.predicted_output_size() {
                    width = size.width as i16;
                    height = size.height as i16;
                } else {
                    width = -1;
                    height = -1;
                }
                if let Some(value) = width.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::PredictedOutputWidth as u64, value).context("pair.PredictedOutputWidth, set_u64")?;
                }
                if let Some(value) = height.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::PredictedOutputHeight as u64, value).context("pair.PredictedOutputHeight, set_u64")?;
                }
            }

            // memory[x*100+105] = train[x].predicted_output_image_is_input_image_with_changes_limited_to_pixels_with_color
            {
                let the_color: i16;
                if let Some(color) = pair.predicted_output_image_is_input_image_with_changes_limited_to_pixels_with_color() {
                    the_color = color as i16;
                } else {
                    the_color = -1;
                }
                if let Some(value) = the_color.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor as u64, value).context("pair.OutputImageIsInputImageWithChangesLimitedToPixelsWithColor, set_u64")?;
                }
            }

            // memory[x*100+106] = train[x].repair_mask
            {
                if let Some(image) = &pair.input.repair_mask {
                    let image_number_uint: BigUint = image.to_number().context("repair_mask image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("repair_mask BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::RepairMask as u64, image_number_int).context("repair_mask, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::RepairMask as u64, value).context("repair_mask, set_u64 with -1")?;
                    }
                }
            }

            // memory[x*100+107] = train[x].repaired_image
            {
                if let Some(image) = &pair.input.repaired_image {
                    let image_number_uint: BigUint = image.to_number().context("repaired_image image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("repaired_image BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::RepairedImage as u64, image_number_int).context("repaired_image, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::RepairedImage as u64, value).context("repaired_image, set_u64 with -1")?;
                    }
                }
            }

            // memory[x*100+108] = train[x].grid_pattern.line_mask
            {
                if let Some(pattern) = &pair.input.grid_pattern {
                    let image: &Image = &pattern.line_mask;
                    let image_number_uint: BigUint = image.to_number().context("line_mask image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("line_mask BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::GridMask as u64, image_number_int).context("line_mask, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::GridMask as u64, value).context("line_mask, set_u64 with -1")?;
                    }
                }
            }

            // memory[x*100+109] = train[x].grid_pattern.color
            {
                let the_color: i16;
                if let Some(pattern) = &pair.input.grid_pattern {
                    the_color = pattern.color as i16;
                } else {
                    the_color = -1;
                }
                if let Some(value) = the_color.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::GridColor as u64, value).context("pair.GridColor, set_u64")?;
                }
            }

            // memory[x*100+110] = train[x].enumerated_objects
            {
                if let Some(image) = &pair.input.enumerated_objects {
                    let image_number_uint: BigUint = image.to_number().context("enumerated_objects image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("enumerated_objects BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::EnumeratedObjects as u64, image_number_int).context("enumerated_objects, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::EnumeratedObjects as u64, value).context("enumerated_objects, set_u64 with -1")?;
                    }
                }
            }
            
            // memory[x*100+111] = train[x].substitution_rule_applied
            {
                if let Some(image) = &pair.input.substitution_rule_applied {
                    let image_number_uint: BigUint = image.to_number().context("substitution_rule_applied image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("substitution_rule_applied BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::SubstitutionRuleApplied as u64, image_number_int).context("substitution_rule_applied, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::SubstitutionRuleApplied as u64, value).context("substitution_rule_applied, set_u64 with -1")?;
                    }
                }
            }
            
            // memory[x*100+112] = train[x].predicted_single_color_image
            {
                if let Some(image) = &pair.input.predicted_single_color_image {
                    let image_number_uint: BigUint = image.to_number().context("predicted_single_color_image image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("predicted_single_color_image BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::PredictedSingleColorImage as u64, image_number_int).context("predicted_single_color_image, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::PredictedSingleColorImage as u64, value).context("predicted_single_color_image, set_u64 with -1")?;
                    }
                }
            }
            
            // memory[x*100+113] = train[x].removal_color
            {
                let the_color: i16;
                if let Some(color) = pair.input.removal_color {
                    the_color = color as i16;
                } else {
                    the_color = -1;
                }
                if let Some(value) = the_color.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::RemovalColor as u64, value).context("pair.RemovalColor, set_u64")?;
                }
            }
            
            // memory[x*100+114] = train[x].most_popular_intersection_color
            {
                let the_color: i16;
                if let Some(color) = pair.input.most_popular_intersection_color {
                    the_color = color as i16;
                } else {
                    the_color = -1;
                }
                if let Some(value) = the_color.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::InputMostPopularColor as u64, value).context("pair.RemovalColor, set_u64")?;
                }
            }

            // Ideas for data to make available to the program.
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
            let address: u64 = (index * 100 + 100) as u64;
            // memory[(count_train + x)*100+100] = test[x].input
            {
                let image_number_uint: BigUint = pair.input.image.to_number().context("pair.input image to number")?;
                let image_number_int: BigInt = image_number_uint.to_bigint().context("pair.input BigUint to BigInt")?;
                state.set_u64(address + MemoryLayoutItem::InputImage as u64, image_number_int).context("pair.input, set_u64")?;
            }

            // The program is never supposed to read from the the test[x].output register.
            // memory[(count_train + x)*100+101] is where the program is supposed to write its predicted output.
            // Use `-1` as placeholder so it's easy to spot when the image is missing.
            {
                let value: BigInt = -BigInt::one();
                state.set_u64(address + MemoryLayoutItem::ExpectedOutputImage as u64, value).context("pair.output, set_u64")?;
            }

            // memory[x*100+102] = test[x].computed output
            {
                let value: BigInt = -BigInt::one();
                state.set_u64(address + MemoryLayoutItem::ComputedOutputImage as u64, value).context("pair.ComputedOutputImage, set_u64")?;
            }

            // memory[x*100+103] = test[x].predicted output width
            // memory[x*100+104] = test[x].predicted output height
            {
                let width: i16;
                let height: i16;
                if let Some(size) = pair.predicted_output_size() {
                    width = size.width as i16;
                    height = size.height as i16;
                } else {
                    width = -1;
                    height = -1;
                }
                if let Some(value) = width.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::PredictedOutputWidth as u64, value).context("pair.PredictedOutputWidth, set_u64")?;
                }
                if let Some(value) = height.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::PredictedOutputHeight as u64, value).context("pair.PredictedOutputHeight, set_u64")?;
                }
            }

            // memory[x*100+105] = test[x].predicted_output_image_is_input_image_with_changes_limited_to_pixels_with_color
            {
                let the_color: i16;
                if let Some(color) = pair.predicted_output_image_is_input_image_with_changes_limited_to_pixels_with_color() {
                    the_color = color as i16;
                } else {
                    the_color = -1;
                }
                if let Some(value) = the_color.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor as u64, value).context("pair.OutputImageIsInputImageWithChangesLimitedToPixelsWithColor, set_u64")?;
                }
            }

            // memory[x*100+106] = test[x].repair_mask
            {
                if let Some(image) = &pair.input.repair_mask {
                    let image_number_uint: BigUint = image.to_number().context("repair_mask image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("repair_mask BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::RepairMask as u64, image_number_int).context("repair_mask, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::RepairMask as u64, value).context("repair_mask, set_u64 with -1")?;
                    }
                }
            }

            // memory[x*100+107] = test[x].repaired_image
            {
                if let Some(image) = &pair.input.repaired_image {
                    let image_number_uint: BigUint = image.to_number().context("repaired_image image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("repaired_image BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::RepairedImage as u64, image_number_int).context("repaired_image, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::RepairedImage as u64, value).context("repaired_image, set_u64 with -1")?;
                    }
                }
            }

            // memory[x*100+108] = test[x].grid_pattern.line_mask
            {
                if let Some(pattern) = &pair.input.grid_pattern {
                    let image: &Image = &pattern.line_mask;
                    let image_number_uint: BigUint = image.to_number().context("line_mask image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("line_mask BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::GridMask as u64, image_number_int).context("line_mask, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::GridMask as u64, value).context("line_mask, set_u64 with -1")?;
                    }
                }
            }

            // memory[x*100+109] = test[x].grid_pattern.color
            {
                let the_color: i16;
                if let Some(pattern) = &pair.input.grid_pattern {
                    the_color = pattern.color as i16;
                } else {
                    the_color = -1;
                }
                if let Some(value) = the_color.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::GridColor as u64, value).context("pair.GridColor, set_u64")?;
                }
            }

            // memory[x*100+110] = test[x].enumerated_objects
            {
                if let Some(image) = &pair.input.enumerated_objects {
                    let image_number_uint: BigUint = image.to_number().context("enumerated_objects image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("enumerated_objects BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::EnumeratedObjects as u64, image_number_int).context("enumerated_objects, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::EnumeratedObjects as u64, value).context("enumerated_objects, set_u64 with -1")?;
                    }
                }
            }
            
            // memory[x*100+111] = test[x].substitution_rule_applied
            {
                if let Some(image) = &pair.input.substitution_rule_applied {
                    let image_number_uint: BigUint = image.to_number().context("substitution_rule_applied image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("substitution_rule_applied BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::SubstitutionRuleApplied as u64, image_number_int).context("substitution_rule_applied, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::SubstitutionRuleApplied as u64, value).context("substitution_rule_applied, set_u64 with -1")?;
                    }
                }
            }
            
            // memory[x*100+112] = test[x].predicted_single_color_image
            {
                if let Some(image) = &pair.input.predicted_single_color_image {
                    let image_number_uint: BigUint = image.to_number().context("predicted_single_color_image image to number")?;
                    let image_number_int: BigInt = image_number_uint.to_bigint().context("predicted_single_color_image BigUint to BigInt")?;
                    state.set_u64(address + MemoryLayoutItem::PredictedSingleColorImage as u64, image_number_int).context("predicted_single_color_image, set_u64")?;
                } else {
                    if let Some(value) = (-1i16).to_bigint() {
                        state.set_u64(address + MemoryLayoutItem::PredictedSingleColorImage as u64, value).context("predicted_single_color_image, set_u64 with -1")?;
                    }
                }
            }
            
            // memory[x*100+113] = test[x].removal_color
            {
                let the_color: i16;
                if let Some(color) = pair.input.removal_color {
                    the_color = color as i16;
                } else {
                    the_color = -1;
                }
                if let Some(value) = the_color.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::RemovalColor as u64, value).context("pair.RemovalColor, set_u64")?;
                }
            }
            
            // memory[x*100+114] = test[x].most_popular_intersection_color
            {
                let the_color: i16;
                if let Some(color) = pair.input.most_popular_intersection_color {
                    the_color = color as i16;
                } else {
                    the_color = -1;
                }
                if let Some(value) = the_color.to_bigint() {
                    state.set_u64(address + MemoryLayoutItem::InputMostPopularColor as u64, value).context("pair.RemovalColor, set_u64")?;
                }
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

    /// It's ok for the `train` pairs to contain `Color::CannotCompute`.
    /// 
    /// It's illegal for the `test` pairs to contain `Color::CannotCompute`.
    fn preserve_output_for_traindata(&self, computed_images: &mut Vec<Image>) -> anyhow::Result<()> {
        // Future idea
        // together with the computed_images, also store a `rejection reason`,
        // so it's possible to debug why an image was rejected.
        // make a struct that contains both the pair index, if it's a train or test.
        // in case there is a problem then nothing gets printed to the console,
        // however here it's especially important to print useful info to the console.

        let count_train: usize = self.task.count_train();

        // Reject the solution if `Color::CannotCompute` is detected in the `test` pairs.
        {
            let mut count_test: usize = 0;
            for pair in &self.task.pairs {
                if pair.pair_type != arc_work_model::PairType::Test {
                    continue;
                }
                let index: usize = count_train + count_test;
                count_test += 1;
                let computed_image: &Image = &computed_images[index];
                let size: ImageSize = computed_image.size();
                let mut count_invalid_colors: u16 = 0;
                for y in 0..size.height {
                    for x in 0..size.width {
                        let xx = x as i32;
                        let yy = y as i32;
                        let computed_pixel: u8 = computed_image.get(xx, yy).unwrap_or(255);
                        if computed_pixel == Color::CannotCompute as u8 {
                            count_invalid_colors += 1;
                        }
                    }
                }
                if count_invalid_colors > 0 {
                    return Err(anyhow::anyhow!("computed output for test pair must not contain Color::CannotCompute"));
                }
            }
        }

        // Loop over the `train` pairs.
        // Stop if the size of the computed_outputs doesn't match the size of the expected_output.
        // We want to preserve pixel values from the expected output.
        // In order to do so, both the computed_image and the expected_output must have the same size.
        // Cannot copy pixels between images with different sizes.
        // Bail out before starting to mutate things.
        {
            let mut count_train: usize = 0;
            for pair in &self.task.pairs {
                if pair.pair_type != arc_work_model::PairType::Train {
                    continue;
                }
                let index: usize = count_train;
                count_train += 1;
    
                let computed_image: &Image = &computed_images[index];
                let expected_image: Image = pair.output.image.clone();
                if computed_image.size() != expected_image.size() {
                    return Ok(());
                }
            }

        }

        // Loop over the training pairs.
        // Replace the `Color::CannotCompute` with the expected_output
        {
            let mut count_train: usize = 0;
            for pair in &self.task.pairs {
                if pair.pair_type != arc_work_model::PairType::Train {
                    continue;
                }
                let index: usize = count_train;
                count_train += 1;
    
                let computed_image: &mut Image = &mut computed_images[index];
                let size: ImageSize = computed_image.size();
            
                let expected_image: Image = pair.output.image.clone();
                if computed_image.size() != expected_image.size() {
                    return Err(anyhow::anyhow!("size does not match"));
                }
    
                let mut count_replacements: u16 = 0;
                for y in 0..size.height {
                    for x in 0..size.width {
                        let xx = x as i32;
                        let yy = y as i32;
                        let computed_pixel: u8 = computed_image.get(xx, yy).unwrap_or(255);
                        if computed_pixel != Color::CannotCompute as u8 {
                            continue;
                        }
                        let expected_pixel: u8 = expected_image.get(xx, yy).unwrap_or(255);
                        _ = computed_image.set(xx, yy, expected_pixel);
                        count_replacements += 1;
                    }
                }

                // Copying the expected output is dangerous and may yield false positives.
                // It's not a solution just outputting `Color::CannotCompute` on all pixels and the task seems solved.
                // To prevent that scenario, only allow for few pixels being copied.
                let area: u16 = (size.width as u16) * (size.height as u16);
                if count_replacements == area {
                    return Err(anyhow::anyhow!("Replacing everything is illegal."));
                }
                let max_replacements: u16 = area / 5;
                if count_replacements >= max_replacements {
                    return Err(anyhow::anyhow!("Performed too many replacements. The majority of pixels must be computed. count: {} limit: {}", count_replacements, max_replacements));
                }
                // println!("count: {} limit: {}", count_replacements, max_replacements);
            }
        }

        Ok(())
    }

    fn check_all_outputs_use_valid_colors(computed_images: &Vec<Image>) -> anyhow::Result<()> {
        // Loop over the train+test pairs.
        for computed_image in computed_images {
            let size: ImageSize = computed_image.size();
            let mut count_invalid_colors: u16 = 0;
            for y in 0..size.height {
                for x in 0..size.width {
                    let xx = x as i32;
                    let yy = y as i32;
                    let computed_pixel: u8 = computed_image.get(xx, yy).unwrap_or(255);
                    if computed_pixel > 9 {
                        count_invalid_colors += 1;
                    }
                }
            }
            if count_invalid_colors > 0 {
                return Err(anyhow::anyhow!("colors must be in the range [0..9], but encountered an image with {} illegal colors", count_invalid_colors));
            }
        }
        // All the images contain valid colors.
        Ok(())
    }

    fn postprocess_fix_orientation(&self, computed_images: &mut Vec<Image>) -> anyhow::Result<()> {
        // Determine if fixing the orientation makes sense with the training pairs
        {
            let mut count_train: usize = 0;
            for pair in &self.task.pairs {
                if pair.pair_type != arc_work_model::PairType::Train {
                    continue;
                }
    
                let index: usize = count_train;
                count_train += 1;
    
                let computed_size: ImageSize = computed_images[index].size();
                if computed_size.width == computed_size.height {
                    return Err(anyhow::anyhow!("postprocess_fix_orientation requires all images to be rotated. However one or more images square."));
                }
                
                let expected_size: ImageSize = pair.output.image.size();
                let is_rotated: bool = 
                    computed_size.width == expected_size.height && 
                    computed_size.height == expected_size.width;
                
                if !is_rotated {
                    return Err(anyhow::anyhow!("postprocess_fix_orientation requires all images to be rotated. However one or more images is not rotated."));
                }
                
                // fixing the orientation may be possible with this training pair
            }
        }

        // Rotate all the images
        for computed_image in computed_images.iter_mut() {
            let rotated_image: Image = computed_image.rotate_cw()?;
            computed_image.set_image(rotated_image);
        }
        Ok(())
    }

    fn postprocess_recolor(&self, computed_images: &mut Vec<Image>) -> anyhow::Result<()> {
        // Determine if recoloring may be applied to training pairs
        {
            let mut count_train: usize = 0;
            for pair in &self.task.pairs {
                if pair.pair_type != arc_work_model::PairType::Train {
                    continue;
                }
    
                let index: usize = count_train;
                count_train += 1;
    
                let computed_image: &Image = &computed_images[index];
                
                let expected_image: Image = pair.output.image.clone();
                if computed_image.size() != expected_image.size() {
                    return Err(anyhow::anyhow!("recoloring not possible. Can only recolor if all the computed images have the expected size, but one or more images has the wrong size"));
                }
                
                if *computed_image == expected_image {
                    return Err(anyhow::anyhow!("recoloring not possible. Can only recolor if all the computed images have different colors than the expected image, but one or more images has the correct colors"));
                }
    
                // recoloring may be possible with this training pair
            }
        }
        // println!("recoloring may be possible");

        // Obtain all the color mappings from training pairs
        // Detects ambiguous color mappings detected and aborts
        let mut color_mapping: [i16; 256] = [-1; 256];
        {
            let mut count_train: usize = 0;
            for pair in &self.task.pairs {
                if pair.pair_type != arc_work_model::PairType::Train {
                    continue;
                }
                let index: usize = count_train;
                count_train += 1;
    
                let computed_image: &Image = &computed_images[index];
                let size: ImageSize = computed_image.size();
            
                let expected_image: Image = pair.output.image.clone();
                if computed_image.size() != expected_image.size() {
                    return Err(anyhow::anyhow!("recoloring not possible. should not happen. the size should be the same"));
                }
    
                for y in 0..size.height as i32 {
                    for x in 0..size.width as i32 {
                        let computed_pixel: u8 = computed_image.get(x, y).unwrap_or(255);
                        let expected_pixel: u8 = expected_image.get(x, y).unwrap_or(255);
                        let current_color: i16 = color_mapping[computed_pixel as usize];
                        if current_color == -1 {
                            color_mapping[computed_pixel as usize] = expected_pixel as i16;
                            continue;
                        }
                        if current_color != expected_pixel as i16 {
                            // there already exist another color mappings. 
                            // It not possible to recolor.
                            return Err(anyhow::anyhow!("recoloring not possible. there already exist another color mapping for this color"));
                        }
                        // the color mapping already exist for this
                    }
                }
            }
        }

        // Recolor all the computed images
        for computed_image in computed_images.iter_mut() {
            let size: ImageSize = computed_image.size();

            for y in 0..size.height as i32 {
                for x in 0..size.width as i32 {
                    let computed_pixel: u8 = computed_image.get(x, y).unwrap_or(255);
                    let target_color_i16: i16 = color_mapping[computed_pixel as usize];
                    if target_color_i16 < 0 || target_color_i16 > (u8::MAX as i16) {
                        // recoloring of this pixel is not possible. Leave the original pixel as it is.
                        continue;
                    }
                    let target_color: u8 = target_color_i16 as u8;
                    let _ = computed_image.set(x, y, target_color);
                }
            }
        }

        Ok(())
    }

    fn postprocess_flip_and_recolor(&self, computed_images: &mut Vec<Image>) -> anyhow::Result<()> {
        match self.postprocess_recolor(computed_images) {
            Ok(()) => {
                // println!("successfully applied recolor postprocessing for flip mode: normal");
                return Ok(());
            },
            Err(_) => {}
        }

        // Go through the variations: flip x, flip y, flip xy.
        for i in 1..4u8 {
            let mut computed_images_clone: Vec<Image> = computed_images.clone();
            for computed_image in computed_images_clone.iter_mut() {
                let mut image: Image = computed_image.clone();
                if (i & 1) != 0 {
                    image = image.flip_x()?;
                }
                if (i & 2) != 0 {
                    image = image.flip_y()?;
                }
                computed_image.set_image(image);
            }
            match self.postprocess_recolor(&mut computed_images_clone) {
                Ok(()) => {
                    // println!("successfully applied recolor postprocessing for flip mode: {}", i);
                    computed_images.truncate(0);
                    computed_images.extend(computed_images_clone);
                    return Ok(());
                },
                Err(_) => {}
            }
        }

        Err(anyhow::anyhow!("did not find a flip recolor combination that works"))
    }

    /// Solutions that are very close to the expected output.
    /// 
    /// Can such a program be tweaked in minor ways so that it gets even closer to the expected output.
    /// - Is it a rotate that is missing?
    /// - Is it a recoloring that is missing?
    /// - Is it a flip that is missing?
    fn postprocess(&self, computed_images: &mut Vec<Image>) -> anyhow::Result<()> {
        let mut one_or_more_postprocessing_applied = false;
        match self.postprocess_fix_orientation(computed_images) {
            Ok(()) => {
                // println!("successfully applied fix orientation postprocessing");
                one_or_more_postprocessing_applied = true;
            },
            Err(_) => {}
        }
        match self.postprocess_flip_and_recolor(computed_images) {
            Ok(()) => {
                // println!("successfully applied flip and recolor postprocessing");
                one_or_more_postprocessing_applied = true;
            },
            Err(_) => {}
        }
        if !one_or_more_postprocessing_applied {
            return Err(anyhow::anyhow!("Didn't apply any postprocessing"));
        }
        
        // Did apply one or more postprocessing steps.
        Ok(())
    }

    fn process_computed_images(&self, mut computed_images: Vec<Image>) -> anyhow::Result<RunWithProgramResult> {
        let pretty_print = false;

        match self.preserve_output_for_traindata(&mut computed_images) {
            Ok(()) => {
                // println!("did preserve output");
            },
            Err(_error) => {
                // println!("unable to preserve output: {:?}", error);
            }
        }

        match self.postprocess(&mut computed_images) {
            Ok(()) => {
                // println!("successfully applied postprocessing");
                // pretty_print = true;
            },
            Err(_) => {}
        }

        Self::check_all_outputs_use_valid_colors(&computed_images)
            .context("process_computed_images")?;

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
            if computed_image.size() != expected_image.size() {
                count_train_incorrect += 1;
                let s = format!("train. Size of the computed output, doesn't match train[{}].output.size.\nExpected {:?}\nActual {:?}", index, expected_image.size(), computed_image.size());
                message_items.push(s);
                if pretty_print {
                    status_texts.push("Incorrect size");
                }
                continue;
            }

            if *computed_image != expected_image {
                count_train_incorrect += 1;
                let s = format!("train. The computed output, doesn't match train[{}].output.\nExpected {:?}\nActual {:?}", index, expected_image, computed_image);
                message_items.push(s);
                if pretty_print {
                    status_texts.push("Incorrect");
                }
                continue;
            }

            count_train_correct += 1;
            if pretty_print {
                status_texts.push("OK");
            }
        }
        let all_train_pairs_are_correct: bool = (count_train_correct == count_train) && (count_train_incorrect == 0);

        // if count_train_correct >= 2 {
        //     pretty_print = true;
        // }

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

            // Ideas for preventing false positives:
            // Reject if, computed output image is identical to input image.
            // Reject solution, if the Test pairs, has a computed_image.size() different than the predicted size.
            // Reject solution, if the Test pairs, has a computed_image.histogram() different than the predicted palette.
            // Reject solution, if the Test pairs, has a computed_image.only_one_color() and all the training data also has only_one_color.

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
        let all_test_pairs_are_correct: bool = count_test_correct == count_test;

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
            all_train_pairs_are_correct,
            all_test_pairs_are_correct,
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
            let address: u64 = (index as u64) * 100 + 100 + (MemoryLayoutItem::ComputedOutputImage as u64);
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
