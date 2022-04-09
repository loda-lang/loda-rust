use crate::common::{find_asm_files_recursively, find_csv_files_recursively, program_id_from_path, parse_csv_file};
use crate::pattern::RecordSimilar;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParameterType, ParsedProgram};
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::iter::FromIterator;

const PROGRAM_LENGTH_MINIMUM: usize = 1;
const PROGRAM_LENGTH_MAXIMUM: usize = 80;
const MINIMUM_NUMBER_OF_SIMILAR_PROGRAMS_BEFORE_ITS_A_PATTERN: usize = 15;

pub fn subcommand_pattern() {
    let start_time = Instant::now();

    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let loda_identify_similar_programs_repository_oeis_dir: PathBuf = config.loda_identify_similar_programs_repository_oeis();
    let output_dir: PathBuf = config.loda_patterns_repository_simple_constant();

    // Find all similarity CSV files.
    let mut similarity_csv_paths: Vec<PathBuf> = find_csv_files_recursively(&loda_identify_similar_programs_repository_oeis_dir);
    similarity_csv_paths.sort();
    let number_of_similarity_csv_paths = similarity_csv_paths.len();
    if number_of_similarity_csv_paths <= 0 {
        error!("Expected 1 or more similarity csv files, but there are none to analyze");
        return;
    }
    debug!("number of similarity csv files: {}", number_of_similarity_csv_paths);
    let mut csv_vec = Vec::<Rc<SimilarityCSVFile>>::new();
    for path in similarity_csv_paths {
        let program_id: u32 = match program_id_from_path(&path) {
            Some(value) => value,
            None => { continue; }
        };
        let instance = SimilarityCSVFile::new(program_id, path);
        csv_vec.push(Rc::new(instance));
    }
    let mut program_id_to_csv_hashmap = ProgramIdToSimilarityCSVFile::new();
    for csv_item in csv_vec {
        program_id_to_csv_hashmap.insert(csv_item.program_id, Rc::clone(&csv_item));
    }
    let number_of_items_in_csv_hashmap = program_id_to_csv_hashmap.len();
    if number_of_items_in_csv_hashmap <= 0 {
        error!("Expected 1 or more similarity csv files, but there are none to analyze");
        return;
    }
    debug!("number of unique program_ids in csv hashmap: {:?}", number_of_items_in_csv_hashmap);

    // Find all programs.
    let mut program_asm_paths: Vec<PathBuf> = find_asm_files_recursively(&loda_programs_oeis_dir);
    program_asm_paths.sort();
    let number_of_program_asm_paths = program_asm_paths.len();
    if number_of_program_asm_paths <= 0 {
        error!("Expected 1 or more program asm files, but there are none to analyze");
        return;
    }
    debug!("number of program asm files: {}", number_of_program_asm_paths);

    // Parse all programs.
    // Ignoring too short/long programs.
    let mut program_meta_vec = Vec::<Rc<ProgramMeta>>::new();
    for path in program_asm_paths {
        let program_meta = match analyze_program(&path) {
            Some(value) => value,
            None => {
                continue;
            }
        };
        program_meta_vec.push(Rc::new(program_meta));
    }
    debug!("number of program_meta items: {}", program_meta_vec.len());

    // Obtain the number of lines of all programs.
    let mut line_count_set = HashSet::<u16>::new();
    for program_meta in &program_meta_vec {
        line_count_set.insert(program_meta.line_count);
    }
    let mut line_count_vec: Vec<u16> = line_count_set.into_iter().collect();
    line_count_vec.sort();
    debug!("line_count's: {:?}", line_count_vec);

    traverse_by_program_length(
        &line_count_vec, 
        &program_meta_vec, 
        &program_id_to_csv_hashmap,
        &output_dir,
    );

    println!("elapsed: {:?} ms", start_time.elapsed().as_millis());
}

fn traverse_by_program_length(
    line_count_vec: &Vec<u16>, 
    program_meta_vec: &Vec<Rc<ProgramMeta>>, 
    program_id_to_similarity_csv_file: &ProgramIdToSimilarityCSVFile,
    output_dir: &Path,
) {
    for line_count in line_count_vec {
        let mut programs_with_same_length = Vec::<Rc<ProgramMeta>>::new();
        for program_meta in program_meta_vec {
            if program_meta.line_count != *line_count {
                continue;
            }
            programs_with_same_length.push(Rc::clone(program_meta));
        }
        process_programs_with_same_length(
            *line_count, 
            &programs_with_same_length, 
            program_id_to_similarity_csv_file,
            output_dir,
        );
    }
}

type ProgramIdToProgramIdSet = HashMap<u32, HashSet<u32>>;

fn process_programs_with_same_length(
    line_count: u16, 
    program_meta_vec: &Vec<Rc<ProgramMeta>>, 
    program_id_to_similarity_csv_file: &ProgramIdToSimilarityCSVFile,
    output_dir: &Path,
) {
    println!("line count: {:?}  number of programs: {:?}", line_count, program_meta_vec.len());

    // Build a hashmap of programs with the same number of lines
    let mut program_id_to_program_meta_hashmap = ProgramIdToProgramMeta::new();
    for program_meta_item in program_meta_vec {
        program_id_to_program_meta_hashmap.insert(program_meta_item.program_id, Rc::clone(&program_meta_item));
    }

    let mut number_of_similarity_records: usize = 0;

    // The key is the lowest program_id in the pattern
    // The value is a hashset with the similar program_ids.
    let mut accumulated = ProgramIdToProgramIdSet::new();

    for program_meta in program_meta_vec {
        let program_id: u32 = program_meta.program_id;

        // Find corresponding similarity csv file
        let csv_file: Rc<SimilarityCSVFile> = match program_id_to_similarity_csv_file.get(&program_id) {
            Some(value) => Rc::clone(value),
            None => {
                debug!("ignoring program_id: {}, because it's missing a similarity csv file", program_id);
                continue;
            }
        };

        // Parse the similarity csv file
        let similarity_records: Vec<RecordSimilar> = match parse_csv_file(&csv_file.path) {
            Ok(value) => value,
            Err(error) => {
                debug!("ignoring program_id: {}. cannot load csv file {:?}", program_id, error);
                continue;
            }
        };
        number_of_similarity_records += similarity_records.len();

        // Compare this program with each rows in the csv file
        find_patterns(
            program_id, 
            &similarity_records, 
            &program_id_to_program_meta_hashmap,
            &mut accumulated
        );
    }
    debug!("number of records: {}", number_of_similarity_records);
    println!("number of patterns: {}", accumulated.len());

    for (lowest_program_id, program_id_set) in &accumulated {
        let save_result = save_pattern(line_count, *lowest_program_id, &program_id_set, &program_id_to_program_meta_hashmap, output_dir);
        match save_result {
            Ok(_) => {},
            Err(error) => {
                error!("Unable to save result. {:?}", error);
            }
        }
    }
}

fn save_pattern(
    line_count: u16, 
    lowest_program_id: u32, 
    program_id_set: &HashSet<u32>, 
    program_id_to_program_meta_hashmap: &ProgramIdToProgramMeta,
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let original_program_meta: Rc<ProgramMeta> = match program_id_to_program_meta_hashmap.get(&lowest_program_id) {
        Some(value) => Rc::clone(value),
        None => {
            debug!("ignoring program: {}. there is no asm file.", lowest_program_id);
            return Ok(());
        }
    };

    let mut line_number_to_value_set = HashMap::<usize, HashSet<i64>>::new();

    for program_id_item in program_id_set {
        let similar_program_meta: Rc<ProgramMeta> = match program_id_to_program_meta_hashmap.get(&program_id_item) {
            Some(value) => Rc::clone(value),
            None => {
                continue;
            }
        };

        let instruction_vec0 = &original_program_meta.parsed_program.instruction_vec;
        let instruction_vec1 = &similar_program_meta.parsed_program.instruction_vec;

        // Reject if the number of instructions differs
        if instruction_vec0.len() != instruction_vec1.len() {
            continue;
        }

        // Reject if the instructions differs
        for index in 0..instruction_vec0.len() {
            if instruction_vec0[index].instruction_id != instruction_vec1[index].instruction_id {
                continue;
            }
        }

        for index in 0..instruction_vec0.len() {
            let instruction0: &Instruction = &instruction_vec0[index];
            let instruction1: &Instruction = &instruction_vec1[index];

            // If the instructions have different constants
            // then remember the line number and the constants.
            let diff = ProgramMeta::instruction_diff_between_constants(instruction0, instruction1);
            if let Some((constant0, constant1)) = diff {
                let entry = line_number_to_value_set.entry(index).or_insert_with(|| HashSet::new());
                entry.insert(constant0);
                entry.insert(constant1);
            }
        }
    }

    let mut annotated_program = String::with_capacity(4000);
    let instruction_vec = &original_program_meta.parsed_program.instruction_vec;
    let mut indentation: usize = 0;
    let mut pretty_parameters = Vec::<String>::new();
    for index in 0..instruction_vec.len() {
        let instruction: &Instruction = &instruction_vec[index];

        if index > 0 {
            annotated_program.push_str("\n");
        }

        // Indent nested loops
        if instruction.instruction_id == InstructionId::LoopEnd {
            if indentation > 0 {
                indentation -= 1;
            }
        }
        for _ in 0..indentation {
            annotated_program.push_str("  ");
        }
        if instruction.instruction_id == InstructionId::LoopBegin {
            indentation += 1;
        }

        // The instruction
        annotated_program.push_str(&format!("{}", instruction));

        let value_set: &HashSet<i64> = match line_number_to_value_set.get(&index) {
            Some(value) => value,
            None => {
                continue;
            }
        };

        // The parameter index
        let parameter_index = pretty_parameters.len();
        annotated_program.push_str(" ; ");
        annotated_program.push_str(&format!("source=parameter {}", parameter_index));

        // Format the parameter values
        let mut value_vec: Vec<&i64> = Vec::from_iter(value_set);
        value_vec.sort();
        let value_strings: Vec<String> = value_vec.iter().map(|value| format!("{}", value) ).collect();
        let mut formatted_parameter = String::with_capacity(1000);
        formatted_parameter.push_str(&format!("; parameter {}\n", parameter_index));
        formatted_parameter.push_str(&format!("; number of unique values: {}\n", value_set.len()));
        formatted_parameter.push_str("; value: ");
        formatted_parameter.push_str(&value_strings.join(","));
        pretty_parameters.push(formatted_parameter);
    }

    // Convert program_ids to a formatted string
    let mut program_ids: Vec<u32> = program_id_set.iter().map(|program_id| *program_id).collect();
    program_ids.sort();
    let program_id_strings: Vec<String> = program_ids.iter().map(|program_id| format!("{}", program_id)).collect();
    let formatted_program_ids: String = program_id_strings.join(",");
    
    // File content
    let mut content = String::with_capacity(4000);
    content += &annotated_program;
    content += "\n\n";
    if !pretty_parameters.is_empty() {
        content += &pretty_parameters.join("\n\n");
        content += "\n\n";
    }
    content += "; programs with this pattern\n";
    content += &format!("; number of programs: {:?}\n", program_id_set.len());
    content += "; program id: ";
    content += &formatted_program_ids;
    content += "\n";

    // Version control of the found patterns.
    // Ideally pick a filename that stays the same, no matter how many programs follow the same pattern.
    // The number of lines in the patterns doesn't change.
    // The number of parameters changes, if new programs starts making creative parameter changes.
    // The OEIS sequence id of the lowest program. This changes if it has started using another pattern.
    let filename = format!("lines{}_parameters{}_A{}.asm", line_count, line_number_to_value_set.len(), lowest_program_id);
    let path: PathBuf = output_dir.join(Path::new(&filename));

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

fn find_patterns(
    program_id: u32, 
    similarity_records: &Vec<RecordSimilar>, 
    program_id_to_program_meta_hashmap: &ProgramIdToProgramMeta,
    accumulated: &mut ProgramIdToProgramIdSet,
) {
    let original_program_meta: Rc<ProgramMeta> = match program_id_to_program_meta_hashmap.get(&program_id) {
        Some(value) => Rc::clone(value),
        None => {
            debug!("ignoring program: {}. there is no asm file.", program_id);
            return;
        }
    };

    let mut highly_similar_programs = Vec::<Rc<ProgramMeta>>::with_capacity(26);
    for record in similarity_records {
        let similar_program_meta: Rc<ProgramMeta> = match program_id_to_program_meta_hashmap.get(&record.program_id) {
            Some(value) => Rc::clone(value),
            None => {
                continue;
            }
        };
    
        let similarity = ProgramMeta::measure_similarity(&original_program_meta, &similar_program_meta);
        match similarity {
            ProgramMetaSimilarity::NotSimilar => {
                continue;
            },
            ProgramMetaSimilarity::SimilarWithDifferentConstants(_) => {
                highly_similar_programs.push(similar_program_meta);
            }
        }
    }

    if highly_similar_programs.len() < MINIMUM_NUMBER_OF_SIMILAR_PROGRAMS_BEFORE_ITS_A_PATTERN {
        debug!("ignoring program: {}. there are too few similar programs.", program_id);
        return;
    }

    highly_similar_programs.push(original_program_meta);

    let mut highly_similar_program_ids: Vec<u32> = highly_similar_programs.iter().map(|pm|pm.program_id).collect();
    highly_similar_program_ids.sort();
    // println!("program id: {} has many similar with minor diffs to constants: {:?}", program_id, highly_similar_program_ids);

    // Extend an existing pattern.
    // If no there is no existing pattern, then create a new pattern.
    // Use the lowest program_id of this pattern as the key, so should other 
    // patterns use the same program_id as their key, then the same pattern will be extended.
    let lowest_program_id: u32 = *highly_similar_program_ids.first().unwrap();
    let entry = accumulated.entry(lowest_program_id).or_insert_with(|| HashSet::new());
    for highly_similar_program_id in highly_similar_program_ids {
        entry.insert(highly_similar_program_id);
    }
}

fn analyze_program(
    path: &Path, 
) -> Option<ProgramMeta> {
    let program_id: u32 = match program_id_from_path(path) {
        Some(value) => value,
        None => {
            return None;
        }
    };
    let parsed_program: ParsedProgram = match load_program(path) {
        Some(value) => value,
        None => {
            return None;
        }
    };
    let line_count_raw: usize = parsed_program.instruction_vec.len();
    if line_count_raw < PROGRAM_LENGTH_MINIMUM {
        return None;
    }
    if line_count_raw > PROGRAM_LENGTH_MAXIMUM {
        error!("analyze_program. Skipping a program that is too long. path: {:?}", path);
        return None;
    }
    Some(ProgramMeta::new(program_id, line_count_raw as u16,  parsed_program))
}

fn load_program(path: &Path) -> Option<ParsedProgram> {
    let contents: String = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(error) => {
            error!("load program, error: {:?} path: {:?}", error, path);
            return None;
        }
    };
    let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
        Ok(value) => value,
        Err(error) => {
            error!("load program, error: {:?} path: {:?}", error, path);
            return None;
        }
    };
    Some(parsed_program)
}

enum ProgramMetaSimilarity {
    NotSimilar,
    SimilarWithDifferentConstants(usize),
}


struct ProgramMeta {
    program_id: u32,
    line_count: u16,
    parsed_program: ParsedProgram,
}

impl ProgramMeta {
    fn new(program_id: u32, line_count: u16, parsed_program: ParsedProgram) -> Self {
        Self {
            program_id: program_id,
            line_count: line_count,
            parsed_program: parsed_program,
        }
    }

    fn measure_similarity(pm0: &ProgramMeta, pm1: &ProgramMeta) -> ProgramMetaSimilarity {
        let instruction_vec0 = &pm0.parsed_program.instruction_vec;
        let instruction_vec1 = &pm1.parsed_program.instruction_vec;

        // Reject if the number of instructions differs
        if instruction_vec0.len() != instruction_vec1.len() {
            return ProgramMetaSimilarity::NotSimilar;
        }

        // Reject if the instructions differs
        for index in 0..instruction_vec0.len() {
            if instruction_vec0[index].instruction_id != instruction_vec1[index].instruction_id {
                return ProgramMetaSimilarity::NotSimilar;
            }
        }

        let mut number_of_differencies: usize = 0;
        for index in 0..instruction_vec0.len() {
            let instruction0: &Instruction = &instruction_vec0[index];
            let instruction1: &Instruction = &instruction_vec1[index];
            let parameters0: &Vec<InstructionParameter> = &instruction0.parameter_vec;
            let parameters1: &Vec<InstructionParameter> = &instruction1.parameter_vec;

            // Reject if the number of parameters differs
            if parameters0.len() != parameters1.len() {
                return ProgramMetaSimilarity::NotSimilar;
            }

            for parameter_index in 0..parameters0.len() {
                let parameter0: &InstructionParameter = &parameters0[parameter_index];
                let parameter1: &InstructionParameter = &parameters1[parameter_index];

                // Reject if the parameter type differs
                if parameter0.parameter_type != parameter1.parameter_type {
                    return ProgramMetaSimilarity::NotSimilar;
                }

                let is_same_value = parameter0.parameter_value == parameter1.parameter_value;

                match parameter0.parameter_type {
                    ParameterType::Constant => {
                        if !is_same_value {
                            number_of_differencies += 1;
                        }
                    },
                    ParameterType::Register => {
                        if !is_same_value {
                            return ProgramMetaSimilarity::NotSimilar;
                        }
                    },
                }
            }
        }
        ProgramMetaSimilarity::SimilarWithDifferentConstants(number_of_differencies)
    }

    fn instruction_diff_between_constants(instruction0: &Instruction, instruction1: &Instruction) -> Option<(i64, i64)> {
        let parameters0: &Vec<InstructionParameter> = &instruction0.parameter_vec;
        let parameters1: &Vec<InstructionParameter> = &instruction1.parameter_vec;

        // Reject if the number of parameters differs
        if parameters0.len() != parameters1.len() {
            return None;
        }

        for parameter_index in 0..parameters0.len() {
            let parameter0: &InstructionParameter = &parameters0[parameter_index];
            let parameter1: &InstructionParameter = &parameters1[parameter_index];

            // Reject if the parameter type differs
            if parameter0.parameter_type != parameter1.parameter_type {
                return None;
            }
            let is_same_value = parameter0.parameter_value == parameter1.parameter_value;
            match parameter0.parameter_type {
                ParameterType::Constant => {
                    if !is_same_value {
                        return Some((parameter0.parameter_value, parameter1.parameter_value));
                    }
                },
                ParameterType::Register => {
                    if !is_same_value {
                        return None;
                    }
                },
            }
        }
        None
    }
}

type ProgramIdToProgramMeta = HashMap::<u32, Rc::<ProgramMeta>>;

struct SimilarityCSVFile {
    program_id: u32,
    path: PathBuf,
}

impl SimilarityCSVFile {
    fn new(program_id: u32, path: PathBuf) -> Self {
        Self {
            program_id: program_id,
            path: path
        }
    }
}

type ProgramIdToSimilarityCSVFile = HashMap::<u32, Rc::<SimilarityCSVFile>>;
