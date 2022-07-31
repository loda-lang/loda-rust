use crate::config::Config;
use crate::common::{find_asm_files_recursively, load_program_ids_csv_file};
use crate::postmine::{CandidateProgram, find_pending_programs, State, ValidateSingleProgram, ValidateSingleProgramError};
use crate::oeis::{OeisId, ProcessStrippedSequenceFile, StrippedSequence};
use crate::lodacpp::{LodaCpp, LodaCppCheck, LodaCppEvalTermsExecute, LodaCppEvalTerms, LodaCppMinimize};
use loda_rust_core::util::BigIntVec;
use num_bigint::{BigInt, ToBigInt};
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::rc::Rc;
use core::cell::RefCell;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

type CandidateProgramItem = Rc<RefCell<CandidateProgram>>;

pub struct PostMine {
    config: Config,
    path_timestamped_postmine_dir: PathBuf,
    paths_for_processing: Vec<PathBuf>,
    candidate_programs: Vec<CandidateProgramItem>,
    dontmine_hashset: HashSet<OeisId>,
    invalid_program_ids_hashset: HashSet<OeisId>,
    loda_programs_oeis_dir: PathBuf,
    validate_single_program: ValidateSingleProgram,
    iteration: usize,
}

impl PostMine {
    const LOOKUP_TERM_COUNT: usize = 40;
    const MINIMUM_NUMBER_OF_REQUIRED_TERMS: usize = 10;
    const LODACPP_EVAL_TIME_LIMIT_IN_SECONDS: u64 = 10;
    const LODACPP_MINIMIZE_TIME_LIMIT_IN_SECONDS: u64 = 5;
    const LODACPP_CHECK_TIME_LIMIT_IN_SECONDS: u64 = 120;

    pub fn run() -> Result<(), Box<dyn Error>> {
        let mut instance = Self::new()?;
        instance.run_inner()?;
        Ok(())
    }

    fn run_inner(&mut self) -> Result<(), Box<dyn Error>> {
        self.obtain_paths_for_processing()?;    
        self.populate_candidate_programs()?;
        self.obtain_dontmine_program_ids()?;
        self.obtain_invalid_program_ids()?;
        self.eval_using_loda_cpp()?;
        self.lookup_in_oeis_stripped_file()?;
        self.minimize_candidate_programs()?;
        self.process_candidate_programs()?;
        Ok(())
    }
    
    fn new() -> Result<Self, Box<dyn Error>> {
        let config = Config::load();
        let loda_programs_oeis_dir = config.loda_programs_oeis_dir();
        let validate_single_program = ValidateSingleProgram::new(loda_programs_oeis_dir.clone());

        // Create dir in which the postmine can store its temp files
        let dirname: String = Self::format_timestamped_postmine_dirname();
        let path_timestamped_postmine_dir: PathBuf = config.postmine_dir().join(dirname);
        fs::create_dir(&path_timestamped_postmine_dir)?;

        let instance = Self {
            config: config,
            path_timestamped_postmine_dir: path_timestamped_postmine_dir,
            paths_for_processing: vec!(),
            candidate_programs: vec!(),
            dontmine_hashset: HashSet::new(),
            invalid_program_ids_hashset: HashSet::new(),
            loda_programs_oeis_dir: loda_programs_oeis_dir,
            validate_single_program: validate_single_program,
            iteration: 0,
        };
        Ok(instance)
    }

    /// Format dirname ala `19841231-235959-postmine`
    fn format_timestamped_postmine_dirname() -> String {
        let now: DateTime<Utc> = Utc::now();
        format!("{}-postmine", now.format("%Y%m%d-%H%M%S"))
    }

    fn obtain_paths_for_processing(&mut self) -> Result<(), Box<dyn Error>> {
        let postmine_dir: PathBuf = self.config.postmine_dir();
        let paths_all: Vec<PathBuf> = find_asm_files_recursively(&postmine_dir);
        let paths_for_processing: Vec<PathBuf> = find_pending_programs(&paths_all, true)?;
        self.paths_for_processing = paths_for_processing;
        Ok(())
    }

    fn populate_candidate_programs(&mut self) -> Result<(), Box<dyn Error>> {
        let mut candidate_programs = Vec::<CandidateProgramItem>::with_capacity(self.paths_for_processing.len());
        for path in &self.paths_for_processing {
            let candidate_program = CandidateProgram::new(path)?;
            let candidate_program_item = Rc::new(RefCell::new(candidate_program));
            candidate_programs.push(candidate_program_item);
        }
        self.candidate_programs = candidate_programs;
        Ok(())
    }

    fn obtain_dontmine_program_ids(&mut self) -> Result<(), Box<dyn Error>> {
        let path = self.config.analytics_dir_dont_mine_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: HashSet<OeisId> = HashSet::from_iter(program_ids.iter().cloned());
        println!("loaded dontmine file. number of records: {}", hashset.len());
        self.dontmine_hashset = hashset;
        Ok(())
    }    

    fn obtain_invalid_program_ids(&mut self) -> Result<(), Box<dyn Error>> {
        let path = self.config.analytics_dir_programs_invalid_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: HashSet<OeisId> = HashSet::from_iter(program_ids.iter().cloned());
        println!("loaded invalid program_ids file. number of records: {}", hashset.len());
        self.invalid_program_ids_hashset = hashset;
        Ok(())
    }

    fn eval_using_loda_cpp(&mut self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();
        let time_limit = Duration::from_secs(Self::LODACPP_EVAL_TIME_LIMIT_IN_SECONDS);

        let loda_cpp_executable: PathBuf = self.config.loda_cpp_executable();
        let lodacpp = LodaCpp::new(loda_cpp_executable);

        let number_of_pending_programs: usize = self.candidate_programs.len();
        let pb = ProgressBar::new(number_of_pending_programs as u64);

        let mut count_success: usize = 0;
        let mut count_failure: usize = 0;
        for candidate_program in self.candidate_programs.iter_mut() {
            let result = lodacpp.eval_terms(
                Self::LOOKUP_TERM_COUNT, 
                candidate_program.borrow().path_original(),
                time_limit
            );
            let evalterms: LodaCppEvalTerms = match result {
                Ok(value) => value,
                Err(error) => {
                    let reason = format!("Couldn't eval program with loda-cpp, {:?}", error);
                    let msg = format!("Rejecting {}, {}", candidate_program.borrow(), reason);
                    candidate_program.borrow_mut().perform_reject(reason)?;
                    pb.println(msg);
                    count_failure += 1;
                    pb.inc(1);
                    continue;
                }
            };

            count_success += 1;
            candidate_program.borrow_mut().update_lodacpp_terms(evalterms.terms().clone());
            pb.inc(1);
        }
        pb.finish_and_clear();
    
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} Ran loda-cpp with pending programs, in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        println!("evaluate: count_success: {} count_failure: {}", count_success, count_failure);
        Ok(())
    }

    /// Look up the initial terms in the OEIS `stripped` file and gather the corresponding program ids.
    fn lookup_in_oeis_stripped_file(&mut self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();
        println!("Looking up in the OEIS 'stripped' file");

        let oeis_stripped_file: PathBuf = self.config.oeis_stripped_file();
        assert!(oeis_stripped_file.is_absolute());
        assert!(oeis_stripped_file.is_file());
        let file = File::open(oeis_stripped_file)?;
        let filesize: usize = file.metadata()?.len() as usize;
        let mut oeis_stripped_file_reader = BufReader::new(file);

        let pb = ProgressBar::new(filesize as u64);
        let padding_value_i64: i64 = 0xC0FFEE;
        let padding_value: BigInt = padding_value_i64.to_bigint().unwrap();
        let mut number_of_possible_matches: usize = 0;
        let process_callback = |stripped_sequence: &StrippedSequence, count_bytes: usize| {
            pb.set_position(count_bytes as u64);
            let all_vec: &BigIntVec = stripped_sequence.bigint_vec_ref();
            for candidate_program in self.candidate_programs.iter_mut() {
                let mut candidate_program_mut = candidate_program.borrow_mut();
                let terms: &BigIntVec = candidate_program_mut.lodacpp_terms();
                if terms.starts_with(all_vec) {
                    // let s = format!("program: {} is possible match with A{}  number of identical terms: {}", candidate_program, stripped_sequence.sequence_number, all_vec.len());
                    // pb.println(s);
                    let oeis_id = OeisId::from(stripped_sequence.sequence_number);
                    candidate_program_mut.possible_id_insert(oeis_id);
                    number_of_possible_matches += 1;
                }
            }
        };
        let program_ids_to_ignore = HashSet::<u32>::new();
        let mut stripped_sequence_processor = ProcessStrippedSequenceFile::new();
        stripped_sequence_processor.execute(
            &mut oeis_stripped_file_reader,
            Self::MINIMUM_NUMBER_OF_REQUIRED_TERMS,
            Self::LOOKUP_TERM_COUNT,
            &program_ids_to_ignore,
            &padding_value,
            false,
            process_callback
        );
        pb.finish_and_clear();
    
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} Lookups in the OEIS 'stripped' file, in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );

        debug!("found number of possible matches: {}", number_of_possible_matches);

        // Reject programs that has not been assigned any OEIS ids
        let programs_without_possible_ids: Vec<CandidateProgramItem> = self.candidate_programs
            .iter()
            .filter(|candidate_program| candidate_program.borrow().is_possible_ids_empty())
            .map(|x| x.clone())
            .collect();

        if !programs_without_possible_ids.is_empty() {
            println!("number of programs without possible ids: {}", programs_without_possible_ids.len());
        }
        for candidate_program in programs_without_possible_ids {
            debug!("Rejected {}, where terms cannot be found in OEIS 'stripped' file", candidate_program.borrow());
            candidate_program.borrow_mut().perform_reject("lookup_in_oeis_stripped_file, Terms cannot be found in OEIS 'stripped' file")?;
        }

        Ok(())
    }

    fn pending_candidate_programs(&self) -> Vec<CandidateProgramItem> {
        let pending_programs: Vec<CandidateProgramItem> = self.candidate_programs
            .iter()
            .filter(|candidate_program| candidate_program.borrow().state() == State::PendingProcessing)
            .map(|x| x.clone())
            .collect();
        pending_programs
    }

    fn minimize_candidate_programs(&mut self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();

        let candidate_programs: Vec<CandidateProgramItem> = self.pending_candidate_programs();
        if candidate_programs.is_empty() {
            println!("There are no pending candidate programs in the 'mine-event' dir. Stopping.");
            return Ok(());
        }

        let number_of_candidate_programs: usize = candidate_programs.len();
        println!("Minimizing {} programs", number_of_candidate_programs);
        let pb = ProgressBar::new(number_of_candidate_programs as u64);
        for candidate_program in candidate_programs {
            self.minimize_candidate_program(candidate_program.clone())?;
            pb.inc(1);
        }
        pb.finish_and_clear();
    
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} Minimized programs, in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );
        Ok(())
    }

    fn minimize_candidate_program(&mut self, candidate_program: CandidateProgramItem) -> Result<(), Box<dyn Error>> {
        let loda_cpp_executable: PathBuf = self.config.loda_cpp_executable();
        let lodacpp = LodaCpp::new(loda_cpp_executable);
        let time_limit = Duration::from_secs(Self::LODACPP_MINIMIZE_TIME_LIMIT_IN_SECONDS);
        let result = lodacpp.minimize(&candidate_program.borrow().path_original(), time_limit);
        match result {
            Ok(value) => {
                // debug!("minimized program successfully:\n{}", value);
                candidate_program.borrow_mut().assign_minimized_program(value);
            },
            Err(error) => {
                let reason = format!("Unable to minimize program: {:?}", error);
                // debug!("program: {:?}, rejection reason {}", candidate_program.borrow().path_original(), reason);
                candidate_program.borrow_mut().perform_reject(reason)?;
            }
        }
        Ok(())
    }

    fn process_candidate_programs(&mut self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();

        let candidate_programs: Vec<CandidateProgramItem> = self.pending_candidate_programs();
        if candidate_programs.is_empty() {
            println!("There are no pending candidate programs in the 'mine-event' dir. Stopping.");
            return Ok(());
        }

        let mut number_of_program_ids_to_be_analyzed: usize = 0;
        for program in &candidate_programs {
            number_of_program_ids_to_be_analyzed += program.borrow().possible_ids().len();
        }
        if number_of_program_ids_to_be_analyzed == 0 {
            println!("There are no program ids to be analyzed. Stopping.");
            return Ok(());
        }

        println!("Analyzing {} program ids", number_of_program_ids_to_be_analyzed);
        let pb = ProgressBar::new(number_of_program_ids_to_be_analyzed as u64);
        for candidate_program in candidate_programs {
            let possible_ids: Vec<OeisId> = candidate_program.borrow().possible_id_vec();
            for possible_id in possible_ids {
                self.analyze_candidate(candidate_program.clone(), possible_id, pb.clone())?;
                pb.inc(1);
            }

            candidate_program.borrow_mut().perform_keep_or_reject_based_result()?;
        }
        pb.finish_and_clear();
    
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} Analyzed pending programs, in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );
        Ok(())
    }

    /// Construct a path, like this: `/absolute/path/123/A123456.asm`
    fn path_for_oeis_program(&self, program_id: OeisId) -> PathBuf {
        let dir_index: u32 = program_id.raw() / 1000;
        let dir_index_string: String = format!("{:0>3}", dir_index);
        let filename_string: String = format!("{}.asm", program_id.a_number());
        let dirname = Path::new(&dir_index_string);
        let filename = Path::new(&filename_string);
        let pathbuf: PathBuf = self.loda_programs_oeis_dir.join(dirname).join(filename);
        pathbuf
    }

    fn remove_existing_loda_program(&mut self, program_id: OeisId, source_path: &Path, remove_reason: String) -> Result<(), Box<dyn Error>> {
        info!("removing existing loda program: {} reason: {}", program_id, remove_reason);
        let destination_name = format!("iteration{}_remove_existing_{}.asm", self.iteration, program_id);
        let destination_path: PathBuf = self.path_timestamped_postmine_dir.join(destination_name);
        fs::rename(source_path, &destination_path)?;
        Ok(())
    }

    fn remove_existing_loda_program_if_its_invalid(&mut self, program_id: OeisId, path: &Path) -> Result<(), Box<dyn Error>> {
        let error = match self.validate_single_program.run(path) {
            Ok(_) => {
                debug!("The existing file in loda-programs repo {} seems ok", program_id);
                return Ok(());
            },
            Err(error) => error
        };
        if let Some(vsp_error) = error.downcast_ref::<ValidateSingleProgramError>() {
            match vsp_error {
                ValidateSingleProgramError::MissingFile => {
                    debug!("There is no existing file in loda-programs repo for: {}", program_id);
                    return Ok(());
                },
                ValidateSingleProgramError::IndirectMemoryAccess => {
                    let reason = format!("The existing program {} in loda-programs repo uses indirect memory access, which LODA-RUST doesn't yet support.", program_id);
                    self.remove_existing_loda_program(program_id, path, reason)?;
                    return Ok(());
                },
                ValidateSingleProgramError::CyclicDependency => {
                    let reason = format!("The existing program {} in loda-programs repo has a cyclic dependency and cannot be loaded.", program_id);
                    self.remove_existing_loda_program(program_id, path, reason)?;
                    return Ok(());
                },
                ValidateSingleProgramError::Load => {
                    let reason = format!("The existing program {} in loda-programs repo cannot be loaded for other reasons.", program_id);
                    self.remove_existing_loda_program(program_id, path, reason)?;
                    return Ok(());
                },
                ValidateSingleProgramError::Run => {
                    let reason = format!("The existing program {} in loda-programs repo cannot run.", program_id);
                    self.remove_existing_loda_program(program_id, path, reason)?;
                    return Ok(());
                }
            }
        }
        error!("The file in loda-programs repo {} has problems: {}", program_id, error);
        Err(error)
    }
    
    fn analyze_candidate(&mut self, candidate_program: CandidateProgramItem, possible_id: OeisId, progressbar: ProgressBar) -> Result<(), Box<dyn Error>> {
        self.iteration += 1;

        let message = format!("Comparing {} with {}", candidate_program.borrow(), possible_id);
        progressbar.println(message);

        if self.dontmine_hashset.contains(&possible_id) {
            let message = format!("Maybe keep/reject. The candidate program is contained in the 'dont_mine.csv' file. {}, Analyzing it anyways.", possible_id);
            progressbar.println(message);
        }
    
        if self.invalid_program_ids_hashset.contains(&possible_id) {
            let message = format!("Program {} is listed in the 'programs_invalid.csv'", possible_id);
            progressbar.println(message);
        }

        let path: PathBuf = self.path_for_oeis_program(possible_id);

        self.remove_existing_loda_program_if_its_invalid(possible_id, &path)?;

        let has_original_file: bool = path.is_file();
        if has_original_file {
            // debug!("There already exist program: {}, Renaming from: {} to: {}", possible_id, path, path_original);
        }

        // Save the minimized program to disk
        let check_filename = format!("iteration{}_{}", self.iteration, candidate_program.borrow().filename_original());
        let check_path: PathBuf = self.path_timestamped_postmine_dir.join(check_filename);

        // Prefix with a-number
        let file_content: String = format!(
            "; {}:\n\n{}\n", 
            possible_id.a_number(), 
            candidate_program.borrow().minimized_program()
        );

        let mut check_file = File::create(&check_path)?;
        check_file.write_all(file_content.as_bytes())?;
        check_file.sync_all()?;
        debug!("Created file: {:?}", check_path);
    
        // Execute `loda-check check <PATH> -b 0`
        let time_limit = Duration::from_secs(Self::LODACPP_CHECK_TIME_LIMIT_IN_SECONDS);
        let loda_cpp_executable: PathBuf = self.config.loda_cpp_executable();
        let lodacpp = LodaCpp::new(loda_cpp_executable);
        let result = lodacpp.check(&check_path, time_limit);
        match result {
            Ok(value) => {
                debug!("checked program successfully:\n{}", value);
            },
            Err(error) => {
                debug!("Unable to check program: {:?}", error);
            }
        }

        // candidate_program.borrow_mut().keep_program_ids_insert(program_id);
        Ok(())
    }
}
