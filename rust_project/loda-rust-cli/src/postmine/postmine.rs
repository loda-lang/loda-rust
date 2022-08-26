use crate::config::Config;
use crate::common::{oeis_ids_from_program_string, OeisIdStringMap};
use crate::common::{find_asm_files_recursively, load_program_ids_csv_file, SimpleLog};
use crate::oeis::{OeisId, OeisIdHashSet, ProcessStrippedFile, StrippedRow};
use crate::lodacpp::{LodaCpp, LodaCppCheck, LodaCppCheckResult, LodaCppCheckStatus, LodaCppEvalTermsExecute, LodaCppEvalTerms, LodaCppMinimize};
use super::{batch_lookup_names, ProgramSerializerContextWithSequenceName, terms_from_program};
use super::{CandidateProgram, CompareTwoPrograms, CompareTwoProgramsResult, find_pending_programs, ParentDirAndChildFile, PostMineError, State, ValidateSingleProgram, ValidateSingleProgramError};
use loda_rust_core::execute::{ProgramId, ProgramRunner, ProgramSerializer};
use loda_rust_core::parser::ParsedProgram;
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use loda_rust_core::util::BigIntVec;
use loda_rust_core::util::BigIntVecToString;
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
use anyhow::Context;

type CandidateProgramItem = Rc<RefCell<CandidateProgram>>;

pub struct PostMine {
    config: Config,
    loda_submitted_by: String,
    lodacpp: LodaCpp,
    path_timestamped_postmine_dir: PathBuf,
    paths_for_processing: Vec<PathBuf>,
    candidate_programs: Vec<CandidateProgramItem>,
    dontmine_hashset: OeisIdHashSet,
    invalid_program_ids_hashset: OeisIdHashSet,
    oeis_id_name_map: OeisIdStringMap,
    oeis_id_terms_map: OeisIdStringMap,
    loda_programs_oeis_dir: PathBuf,
    loda_outlier_programs_repository_oeis_divergent: PathBuf,
    validate_single_program: ValidateSingleProgram,
    iteration: usize,
}

impl PostMine {
    const LIMIT_NUMBER_OF_PROGRAMS_FOR_PROCESSING: usize = 40;
    const MAX_LOOKUP_TERM_COUNT: usize = 100;
    const EVAL_TERM_COUNT: usize = 40;
    const MINIMUM_NUMBER_OF_REQUIRED_TERMS: usize = 10;
    const LODACPP_EVAL_TIME_LIMIT_IN_SECONDS: u64 = 10;
    const LODACPP_MINIMIZE_TIME_LIMIT_IN_SECONDS: u64 = 5;
    const LODACPP_CHECK_TIME_LIMIT_IN_SECONDS: u64 = 120;
    const LODACPP_COMPARE_NUMBER_OF_TERM_COUNT: usize = 60;
    const LODACPP_STEPS_TIME_LIMIT_IN_SECONDS: u64 = 120;

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
        self.obtain_sequence_names()?;
        self.process_candidate_programs()?;
        Ok(())
    }
    
    fn new() -> Result<Self, Box<dyn Error>> {
        let config = Config::load();
        let loda_programs_oeis_dir = config.loda_programs_oeis_dir();
        let validate_single_program = ValidateSingleProgram::new(loda_programs_oeis_dir.clone());

        // Ensure that the `postmine` dir exist
        let postmine_dir_path: PathBuf = config.postmine_dir();
        if !postmine_dir_path.is_dir() {
            fs::create_dir(&postmine_dir_path)?;
        }
        assert!(postmine_dir_path.is_dir());

        // Create dir in which the postmine can store its temp files
        let dirname: String = Self::format_timestamped_postmine_dirname();
        let path_timestamped_postmine_dir: PathBuf = postmine_dir_path.join(dirname);
        fs::create_dir(&path_timestamped_postmine_dir)?;
        assert!(path_timestamped_postmine_dir.is_dir());

        let loda_cpp_executable: PathBuf = config.loda_cpp_executable();
        let lodacpp = LodaCpp::new(loda_cpp_executable);

        let loda_outlier_programs_repository_oeis_divergent: PathBuf = config.loda_outlier_programs_repository_oeis_divergent();
        assert!(loda_outlier_programs_repository_oeis_divergent.is_absolute());
        assert!(loda_outlier_programs_repository_oeis_divergent.is_dir());

        let loda_submitted_by: String = config.loda_submitted_by();

        let instance = Self {
            config: config,
            loda_submitted_by: loda_submitted_by,
            lodacpp: lodacpp,
            path_timestamped_postmine_dir: path_timestamped_postmine_dir,
            paths_for_processing: vec!(),
            candidate_programs: vec!(),
            dontmine_hashset: HashSet::new(),
            invalid_program_ids_hashset: HashSet::new(),
            oeis_id_name_map: OeisIdStringMap::new(),
            oeis_id_terms_map: OeisIdStringMap::new(),
            loda_programs_oeis_dir: loda_programs_oeis_dir,
            loda_outlier_programs_repository_oeis_divergent: loda_outlier_programs_repository_oeis_divergent,
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

    /// Processes all the pending programs inside the `mine-event` dir.
    /// It looks for all the LODA assembly programs there are.
    /// If programs already contain `keep` or `reject` then the files are ignored.
    fn obtain_paths_for_processing(&mut self) -> Result<(), Box<dyn Error>> {
        let mine_event_dir: PathBuf = self.config.mine_event_dir();
        let paths_all: Vec<PathBuf> = find_asm_files_recursively(&mine_event_dir);
        let mut paths_for_processing: Vec<PathBuf> = find_pending_programs(&paths_all, true)?;
        let length0: usize = paths_for_processing.len();
        paths_for_processing.truncate(Self::LIMIT_NUMBER_OF_PROGRAMS_FOR_PROCESSING);
        let length1: usize = paths_for_processing.len();
        if length0 != length1 {
            println!("Found {} pending programs. Truncating to {}.", length0, length1);
        }
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
        let hashset: OeisIdHashSet = HashSet::from_iter(program_ids.iter().cloned());
        println!("loaded dontmine file. number of records: {}", hashset.len());
        self.dontmine_hashset = hashset;
        Ok(())
    }    

    fn obtain_invalid_program_ids(&mut self) -> Result<(), Box<dyn Error>> {
        let path = self.config.analytics_dir_programs_invalid_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: OeisIdHashSet = HashSet::from_iter(program_ids.iter().cloned());
        println!("loaded invalid program_ids file. number of records: {}", hashset.len());
        self.invalid_program_ids_hashset = hashset;
        Ok(())
    }

    fn eval_using_loda_cpp(&mut self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();
        let time_limit = Duration::from_secs(Self::LODACPP_EVAL_TIME_LIMIT_IN_SECONDS);

        let number_of_pending_programs: usize = self.candidate_programs.len();
        let pb = ProgressBar::new(number_of_pending_programs as u64);

        let mut count_success: usize = 0;
        let mut count_failure: usize = 0;
        for candidate_program in self.candidate_programs.iter_mut() {
            let result = self.lodacpp.eval_terms(
                Self::EVAL_TERM_COUNT, 
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

        let mut oeis_id_terms_map = OeisIdStringMap::new();

        let pb = ProgressBar::new(filesize as u64);
        let padding_value_i64: i64 = 0xC0FFEE;
        let padding_value: BigInt = padding_value_i64.to_bigint().unwrap();
        let mut number_of_possible_matches: usize = 0;
        let process_callback = |row: &StrippedRow, count_bytes: usize| {
            pb.set_position(count_bytes as u64);
            let mut stripped_terms: BigIntVec = row.terms().clone();
            stripped_terms.truncate(Self::EVAL_TERM_COUNT);
            let mut is_possible_match = false;
            for candidate_program in self.candidate_programs.iter_mut() {
                let mut candidate_program_mut = candidate_program.borrow_mut();
                let terms: &BigIntVec = candidate_program_mut.lodacpp_terms();
                if terms.starts_with(&stripped_terms) {
                    // let s = format!("program: {} is possible match with A{}  number of identical terms: {}", candidate_program, row.oeis_id, stripped_terms.len());
                    // pb.println(s);
                    candidate_program_mut.possible_id_insert(row.oeis_id());
                    number_of_possible_matches += 1;
                    is_possible_match = true;
                }
            }
            if is_possible_match {
                let terms: String = row.terms().to_compact_comma_string();
                // let message = format!("{}: {}", row.oeis_id().a_number(), terms);
                // pb.println(message);
                oeis_id_terms_map.insert(row.oeis_id(), terms);
            }
        };
        let oeis_ids_to_ignore = HashSet::<OeisId>::new();
        let mut stripped_sequence_processor = ProcessStrippedFile::new();
        stripped_sequence_processor.execute(
            &mut oeis_stripped_file_reader,
            Self::MINIMUM_NUMBER_OF_REQUIRED_TERMS,
            Self::MAX_LOOKUP_TERM_COUNT,
            &oeis_ids_to_ignore,
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
        
        debug!("number of items in oeis_id_terms_map: {}", oeis_id_terms_map.len());
        self.oeis_id_terms_map = oeis_id_terms_map;

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
        let time_limit = Duration::from_secs(Self::LODACPP_MINIMIZE_TIME_LIMIT_IN_SECONDS);
        let result = self.lodacpp.minimize(&candidate_program.borrow().path_original(), time_limit);
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

    fn obtain_sequence_names(&mut self) -> anyhow::Result<()> {
        if self.candidate_programs.is_empty() {
            return Ok(());
        }

        // Extract possible OeisId's
        let mut oeis_ids_possible = OeisIdHashSet::new();
        for candidate_program in &self.candidate_programs {
            oeis_ids_possible.extend(candidate_program.borrow().possible_ids());
        }
        if oeis_ids_possible.is_empty() {
            warn!("None of the {} candidate programs have not been assigned possible_ids.", self.candidate_programs.len());
        }
        debug!("obtain_sequence_names. oeis_ids_possible: {:?}", oeis_ids_possible);
        
        // Extract OeisId's from program source code
        let mut oeis_ids_programs = OeisIdHashSet::new();
        for candidate_program in &self.candidate_programs {
            let program: String = candidate_program.borrow().minimized_program().clone();
            let oeis_ids: OeisIdHashSet = match oeis_ids_from_program_string(&program) {
                Ok(value) => value,
                Err(error) => {
                    return Err(anyhow::anyhow!("Unable to extract all OeisId's from minimized version of this program: {:?} error: {:?}", candidate_program.borrow().path_original(), error));
                }
            };
            oeis_ids_programs.extend(oeis_ids);
        }
        debug!("obtain_sequence_names. oeis_ids_programs: {:?}", oeis_ids_programs);

        // UNION(oeis_ids_possible, oeis_ids_programs)
        let mut oeis_ids = OeisIdHashSet::new();
        oeis_ids.extend(oeis_ids_possible);
        oeis_ids.extend(oeis_ids_programs);
        debug!("obtain_sequence_names. will look up names for {} sequences", oeis_ids.len());

        // Lookup sequence names
        let oeis_names_file: PathBuf = self.config.oeis_names_file();
        let file = File::open(&oeis_names_file)
            .with_context(|| format!("Failed to open OEIS 'names' file: {:?}", oeis_names_file))?;
        let metadata: fs::Metadata = file.metadata()
            .with_context(|| format!("Failed to extract metadata for OEIS 'names' file: {:?}", oeis_names_file))?;
        let filesize: usize = metadata.len() as usize;
        let mut reader = BufReader::new(file);
        let oeis_id_name_map: OeisIdStringMap = batch_lookup_names(
            &mut reader,
            filesize,
            &oeis_ids
        ).map_err(|e| anyhow::anyhow!("Unable to lookup names for OeisId's. error: {:?}", e))?;
        debug!("oeis_id_name_map: {:?}", oeis_id_name_map);
        println!("obtained {} sequence names", oeis_id_name_map.len());
        self.oeis_id_name_map = oeis_id_name_map;
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
    fn path_for_oeis_program(&self, program_id: OeisId) -> ParentDirAndChildFile {
        assert!(self.loda_programs_oeis_dir.is_dir());
        assert!(self.loda_programs_oeis_dir.is_absolute());
        let dir_index: u32 = program_id.raw() / 1000;
        let dir_index_string: String = format!("{:0>3}", dir_index);
        let filename_string: String = format!("{}.asm", program_id.a_number());
        let dir_path: PathBuf = self.loda_programs_oeis_dir.join(dir_index_string);
        let file_path: PathBuf = dir_path.join(filename_string);
        ParentDirAndChildFile::new(dir_path, file_path)
    }

    /// Construct a path, like this: `/absolute/path//041/A041009_30_0.asm`
    fn path_to_mismatch(&self, program_id: OeisId, correct_term_count: usize) -> Result<ParentDirAndChildFile, Box<dyn Error>> {
        assert!(self.loda_outlier_programs_repository_oeis_divergent.is_dir());
        assert!(self.loda_outlier_programs_repository_oeis_divergent.is_absolute());
        let dir_index: u32 = program_id.raw() / 1000;
        let dir_index_string: String = format!("{:0>3}", dir_index);
        let dir_path: PathBuf = self.loda_outlier_programs_repository_oeis_divergent.join(&dir_index_string);
        let name = program_id.a_number();
        for index in 0..1000 {
            let filename = format!("{}_{}_{}.asm", name, correct_term_count, index);
            let file_path: PathBuf = dir_path.join(filename);
            if !file_path.is_file() {
                return Ok(ParentDirAndChildFile::new(dir_path, file_path))
            }
        }
        Err(Box::new(PostMineError::CannotConstructUniqueFilenameForMismatch))
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
    
    fn analyze_candidate(
        &mut self, 
        candidate_program: CandidateProgramItem, 
        possible_id: OeisId, 
        _progressbar: ProgressBar
    ) -> anyhow::Result<()> {
        self.iteration += 1;

        let log_filename = format!("iteration{}_log.txt", self.iteration);
        let log_path: PathBuf = self.path_timestamped_postmine_dir.join(log_filename);
        let simple_log = SimpleLog::new(&log_path)
            .map_err(|e| anyhow::anyhow!("Unable to create log file at path: {:?}. error: {:?}", log_path, e))?;

        let message = format!("Comparing {} with {}", candidate_program.borrow(), possible_id);
        // progressbar.println(message.clone());
        simple_log.println(message);
    
        if self.dontmine_hashset.contains(&possible_id) {
            let message = format!("Maybe keep/reject. The candidate program is contained in the 'dont_mine.csv' file. {}, Analyzing it anyways.", possible_id);
            // progressbar.println(message.clone());
            simple_log.println(message);
        }
    
        if self.invalid_program_ids_hashset.contains(&possible_id) {
            let message = format!("Program {} is listed in the 'programs_invalid.csv'", possible_id);
            // progressbar.println(message.clone());
            simple_log.println(message);
        }

        let oeis_program_path: ParentDirAndChildFile = self.path_for_oeis_program(possible_id);
        let terms_from_oeis_program: Option<String>;
        if oeis_program_path.child_file().is_file() {
            terms_from_oeis_program = terms_from_program(&oeis_program_path.child_file())
                .map_err(|e| anyhow::anyhow!("Unable to extract terms-comment from the existing program. path: {:?} error: {:?}", oeis_program_path, e))?;
        } else {
            terms_from_oeis_program = None;
        }
        self.remove_existing_loda_program_if_its_invalid(possible_id, oeis_program_path.child_file())
            .map_err(|e| anyhow::anyhow!("Unable to remove existing invalid program. path: {:?} error: {:?}", oeis_program_path, e))?;

        let check_program_filename = format!("iteration{}_program.asm", self.iteration);
        let check_program_path: PathBuf = self.path_timestamped_postmine_dir.join(check_program_filename);

        let check_output_filename = format!("iteration{}_loda_check.txt", self.iteration);
        let check_output_path: PathBuf = self.path_timestamped_postmine_dir.join(check_output_filename);
        
        let compare_output_filename = format!("iteration{}_compare.txt", self.iteration);
        let compare_output_path: PathBuf = self.path_timestamped_postmine_dir.join(compare_output_filename);
        

        let program_contents: String = candidate_program.borrow().minimized_program().clone();

        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&program_contents) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Parse program from {:?} error: {:?}", &oeis_program_path, error));
            }
        };
    
        // Don't load dependencies from the file system
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::Virtual,
            PathBuf::from("non-existing-dir"),
        );
        for (oeis_id, _name) in &self.oeis_id_name_map {
            let program_id: u64 = oeis_id.raw() as u64;
            dm.virtual_filesystem_insert_file(program_id, "".to_string());
        }
    
        // Create program from instructions
        let result_parse = dm.parse_stage2(
            ProgramId::ProgramWithoutId, 
            &parsed_program
        );
        let runner: ProgramRunner = match result_parse {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("parse_stage2 with program {:?} error: {:?}", &oeis_program_path, error));
            }
        };

        let mut serializer = ProgramSerializer::new();

        // Pass on the `oeis_id_name_map` all the way to the formatting code
        // of the `seq` instruction, so that the sequence name can be inserted as a comment.
        // Like this: `seq $2,40 ; The prime numbers.`
        let context = ProgramSerializerContextWithSequenceName::new(self.oeis_id_name_map.clone());
        serializer.set_context(Box::new(context));
    
        // Insert the sequence name
        let optional_name: Option<&String> = self.oeis_id_name_map.get(&possible_id);
        let mut resolved_name: String = "Missing sequence name".to_string();
        if let Some(name) = optional_name {
            resolved_name = name.clone();
        }
        serializer.append_comment(format!("{}: {}", possible_id, resolved_name));
    
        // Submitted by Euler
        serializer.append_comment(format!("Submitted by {}", self.loda_submitted_by));
    
        // Prefer using the terms of the original program file, as they are.
        // so that terms don't show up as a git-diff.
        let mut optional_terms: Option<String> = terms_from_oeis_program.clone();
        if optional_terms == None {
            // If it's a newly discovered program without any previous program
            // then there is no term-comment.
            // Of if it's an existing program where comment with terms can be found,
            // then use take the terms from the OEIS 'stripped' file.
            if let Some(terms) = self.oeis_id_terms_map.get(&possible_id) {
                optional_terms = Some(terms.clone());
                simple_log.println("Using terms from the OEIS stripped file");
            }
        }
        if optional_terms == None {
            // Fallback using the terms from the candidate program
            let terms: String = candidate_program.borrow().lodacpp_terms().to_compact_comma_string();
            optional_terms = Some(terms);
            simple_log.println("Fallback using terms from the candidate program");
        }
        let resolved_terms: String;
        if let Some(terms) = optional_terms {
            resolved_terms = terms.clone();
        } else {
            return Err(anyhow::anyhow!("Unable to resolve terms for the program: {:?}", &oeis_program_path));
        }
        serializer.append_comment(resolved_terms);
    
        serializer.append_empty_line();
        runner.serialize(&mut serializer);
        serializer.append_empty_line();
        let file_content: String = serializer.to_string();
        
        // Save the program to disk
        let mut check_program_file = File::create(&check_program_path)?;
        check_program_file.write_all(file_content.as_bytes())?;
        check_program_file.sync_all()?;
        // debug!("Created program file: {:?}", check_program_path);
    
        // Execute `loda-check check <PATH> -b`
        let time_limit = Duration::from_secs(Self::LODACPP_CHECK_TIME_LIMIT_IN_SECONDS);
        let ok_error = self.lodacpp.perform_check_and_save_output(&check_program_path, time_limit, &check_output_path);
        let check_result: LodaCppCheckResult = match ok_error {
            Ok(value) => {
                // debug!("checked program: {:?}", value);
                let message = format!("check success: {:?}", value);
                // progressbar.println(message.clone());
                simple_log.println(message);
                value
            },
            Err(error) => {
                debug!("Unable to check program: {:?} at path: {:?}", error, &check_program_path);
                let message = format!("check error: {:?}", error);
                // progressbar.println(message.clone());
                simple_log.println(message);
                return Ok(());
            }
        };
        match check_result.status {
            LodaCppCheckStatus::PartialMatch => {
                self.process_partial_match(
                    simple_log.clone(),
                    candidate_program,
                    &check_program_path,
                    possible_id,
                    check_result.number_of_correct_terms
                )?;
            },
            LodaCppCheckStatus::FullMatch => {
                self.process_full_match(
                    simple_log.clone(),
                    candidate_program,
                    possible_id,
                    &check_program_path,
                    oeis_program_path,
                    &compare_output_path,
                    check_result.number_of_correct_terms as usize
                )?;
            }
        }

        Ok(())
    }
    
    fn process_partial_match(
        &self, 
        simple_log: SimpleLog, 
        candidate_program: CandidateProgramItem, 
        path_program0: &Path, 
        program_id: OeisId, 
        number_of_correct_terms: u32
    ) -> anyhow::Result<()> {
        let mismatch_path: ParentDirAndChildFile = self.path_to_mismatch(program_id, number_of_correct_terms as usize)
            .map_err(|e| anyhow::anyhow!("Unable to construct mistmatc_path. program_id: {:?} error: {:?}", program_id, e))?;
        mismatch_path.create_parent_dir()
            .map_err(|e| anyhow::anyhow!("Unable to create parent dir for mismatch. program_id: {:?} error: {:?}", program_id, e))?;

        let message = format!("Keeping. This program is a mismatch, it has correct {} terms, followed by mismatch. Saving at: {:?}", number_of_correct_terms, mismatch_path.child_file());
        simple_log.println(message);
        fs::copy(path_program0, mismatch_path.child_file())?;
        candidate_program.borrow_mut().keep_id_insert(program_id);
        Ok(())
    }
    
    fn process_full_match(
        &self, 
        simple_log: SimpleLog, 
        candidate_program: CandidateProgramItem, 
        possible_id: OeisId, 
        path_program0: &Path, 
        path_program1: ParentDirAndChildFile, 
        path_benchmark: &Path,
        number_of_correct_terms: usize
    ) -> anyhow::Result<()> {
        if number_of_correct_terms < Self::MINIMUM_NUMBER_OF_REQUIRED_TERMS {
            let message = format!("process_full_match: Rejecting program with too few terms. Expected {} terms, but got {} terms.", number_of_correct_terms, Self::MINIMUM_NUMBER_OF_REQUIRED_TERMS);
            simple_log.println(message);
            return Ok(());
        }

        // Don't attempt to compute more terms than what the b-file already contains
        // since this can cause huge numbers.
        let term_count: usize = Self::LODACPP_COMPARE_NUMBER_OF_TERM_COUNT.min(number_of_correct_terms);
        let message = format!("process_full_match: will compare steps count for {} terms.", term_count);
        simple_log.println(message);

        let time_limit = Duration::from_secs(Self::LODACPP_STEPS_TIME_LIMIT_IN_SECONDS);
        let instance = CompareTwoPrograms::new();
        let ok_error = instance.compare(
            &self.lodacpp,    
            path_program0, 
            path_program1.child_file(), 
            path_benchmark, 
            time_limit,
            term_count
        );

        let result: CompareTwoProgramsResult = match ok_error {
            Ok(value) => {
                let message = format!("process_full_match: compare result ok: {:?}", value);
                simple_log.println(message);
                value
            },
            Err(error) => {
                let message = format!("process_full_match: compare result error: {:?}", error);
                simple_log.println(message);
                return Ok(());
            }
        };

        // If the new program is faster, then keep it, otherwise reject it.
        match result {
            CompareTwoProgramsResult::Program0 => {
                simple_log.println("Keeping. This new program is an improvement.");
            },
            CompareTwoProgramsResult::Program1 => {
                simple_log.println("Rejecting. This program isn't better than the existing program.");
                return Ok(());
            }
        }
        path_program1.create_parent_dir()
            .map_err(|e| anyhow::anyhow!("Unable to create parent dir for matching program. program_id: {:?} error: {:?}", possible_id, e))?;
        fs::copy(path_program0, path_program1.child_file())?;
        candidate_program.borrow_mut().keep_id_insert(possible_id);
        Ok(())
    }
}
