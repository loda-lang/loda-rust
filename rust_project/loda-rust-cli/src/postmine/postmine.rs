use crate::config::{Config, MinerFilterMode};
use crate::common::{oeis_ids_from_program_string, OeisIdStringMap};
use crate::common::{load_program_ids_csv_file, PendingProgramsWithPriority, SimpleLog};
use crate::oeis::{ProcessStrippedFile, StrippedRow};
use crate::lodacpp::{LodaCpp, LodaCppCheck, LodaCppCheckResult, LodaCppCheckStatus, LodaCppEvalTermsExecute, LodaCppEvalTerms, LodaCppMinimize};
use super::{batch_lookup_names, terms_from_program, FormatProgram, path_for_oeis_program};
use super::{CandidateProgram, CompareTwoPrograms, CompareTwoProgramsResult, ParentDirAndChildFile, State, StatusOfExistingProgram, ValidateSingleProgram};
use loda_rust_core::oeis::{OeisId, OeisIdHashSet};
use loda_rust_core::util::BigIntVec;
use loda_rust_core::util::BigIntVecToString;
use num_bigint::{BigInt, ToBigInt};
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::fs;
use std::fs::{File, Metadata};
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

/// Process the pending programs inside the `mine-event` dir.
/// 
/// Ignores programs that have already been processed.
/// 
/// If the mined program is faster than what's already 
/// inside `loda-programs` repo. Then it's a keeper and gets added to the repo.
/// the input file gets renamed to `20220826-210221-120462594.keep.asm`
/// 
/// If the mined program is sharing lots of terms with OEIS sequences,
/// but isn't quite a full match, then it gets added to the `loda-outlier-programs/oeis_divergent` repo. 
/// This is also a keeper.
/// the input file gets renamed to `20220826-210221-120462594.keep.asm`
/// 
/// Rejection, there are lots of ways the mined program is not a keeper.
/// then the input file gets renamed to `20220826-210851-140750305.reject.asm`
pub struct PostMine {
    config: Config,
    loda_submitted_by: String,
    lodacpp: LodaCpp,
    path_timestamped_postmine_dir: PathBuf,
    paths_for_processing: Vec<PathBuf>,
    candidate_programs: Vec<CandidateProgramItem>,
    dontmine_hashset: OeisIdHashSet,
    invalid_program_ids_hashset: OeisIdHashSet,
    valid_program_ids_hashset: OeisIdHashSet,
    oeis_id_name_map: OeisIdStringMap,
    oeis_id_terms_map: OeisIdStringMap,
    loda_programs_oeis_dir: PathBuf,
    loda_outlier_programs_repository_oeis_divergent: PathBuf,
    validate_single_program: ValidateSingleProgram,
    iteration: usize,
    focus_only_on_new_programs: bool,
    found_program_callback: Option<Box<dyn Fn(String, OeisId)>>,
}

impl PostMine {
    const LIMIT_NUMBER_OF_PROGRAMS_FOR_PROCESSING: usize = 100;
    const MAX_LOOKUP_TERM_COUNT: usize = 100;
    const EVAL_TERM_COUNT: usize = 40;
    const MINIMUM_NUMBER_OF_REQUIRED_TERMS: usize = 10;
    const LODACPP_EVAL_TIME_LIMIT_IN_SECONDS: u64 = 10;
    const LODACPP_MINIMIZE_TIME_LIMIT_IN_SECONDS: u64 = 5;
    const LODACPP_CHECK_TIME_LIMIT_IN_SECONDS: u64 = 240;
    const LODACPP_COMPARE_NUMBER_OF_TERM_COUNT: usize = 60;
    const LODACPP_STEPS_TIME_LIMIT_IN_SECONDS: u64 = 120;

    /// The dir "~/.loda-rust/mine-event" holds candidate programs, that have completed the mining funnel.
    /// When running "postmine" each candidate program is checked with the b-file.
    /// 
    /// If it matches with the b-file, there is a chance it's new program that has been discovered, 
    /// or it's an improvement to an existing program.
    /// 
    /// If it's doesn't match with the b-file then it gets added to the "loda-outlier-programs".
    /// Originally the number of variants was 1000, but this caused the repo to quickly grow to 300k files.
    /// Now the limit is smaller and fewer files to deal with.
    /// 
    /// Examples of filenames with a `variant index`:
    /// ```
    /// OEIS ID _ NUMBER OF CORRECT TERMS _ VARIANT INDEX . asm
    /// A144414_32_1.asm
    /// A132337_63_12.asm
    /// A168741_18_303.asm
    /// ```
    const MAX_NUMBER_OF_OUTLIER_VARIANTS: usize = 10;

    pub fn run() -> anyhow::Result<()> {
        let mut instance = Self::new()?;
        instance.run_inner()?;
        Ok(())
    }

    pub fn run_inner(&mut self) -> anyhow::Result<()> {
        self.obtain_paths_for_processing()?;    
        self.populate_candidate_programs()?;
        self.obtain_dontmine_program_ids()?;
        self.obtain_invalid_program_ids()?;
        self.obtain_valid_program_ids()?;
        self.eval_using_loda_cpp()?;
        self.lookup_in_oeis_stripped_file()?;
        self.minimize_candidate_programs()?;
        self.obtain_sequence_names()?;
        self.process_candidate_programs()?;
        Ok(())
    }
    
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let loda_programs_oeis_dir = config.loda_programs_oeis_dir();
        let validate_single_program = ValidateSingleProgram::new(loda_programs_oeis_dir.clone());

        let focus_only_on_new_programs: bool = match config.miner_filter_mode() {
            MinerFilterMode::All => false,
            MinerFilterMode::New => true
        };

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
            valid_program_ids_hashset: HashSet::new(),
            oeis_id_name_map: OeisIdStringMap::new(),
            oeis_id_terms_map: OeisIdStringMap::new(),
            loda_programs_oeis_dir: loda_programs_oeis_dir,
            loda_outlier_programs_repository_oeis_divergent: loda_outlier_programs_repository_oeis_divergent,
            validate_single_program: validate_single_program,
            iteration: 0,
            focus_only_on_new_programs: focus_only_on_new_programs,
            found_program_callback: None,
        };
        Ok(instance)
    }

    pub fn set_found_program_callback(&mut self, c: impl Fn(String, OeisId) + 'static) {
        self.found_program_callback = Some(Box::new(c));
    }

    /// Format dirname ala `19841231-235959-postmine`
    fn format_timestamped_postmine_dirname() -> String {
        let now: DateTime<Utc> = Utc::now();
        format!("{}-postmine", now.format("%Y%m%d-%H%M%S"))
    }

    /// Processes all the pending programs inside the `mine-event` dir.
    /// It looks for all the LODA assembly programs there are.
    /// If programs already contain `keep` or `reject` then the files are ignored.
    fn obtain_paths_for_processing(&mut self) -> anyhow::Result<()> {
        let pending = PendingProgramsWithPriority::create(&self.config)?;
        println!("Arrange programs by priority. high prio: {}, low prio: {}", pending.paths_high_prio().len(), pending.paths_low_prio().len());

        // Get references to the Path which is fixed length. PathBuf is variable length.
        let paths_high_prio: Vec<&Path> = pending.paths_high_prio().iter().map(|path|path.as_path()).collect();
        let paths_low_prio: Vec<&Path> = pending.paths_low_prio().iter().map(|path|path.as_path()).collect();

        // High priority items at the front of the queue, so they get processed first.
        // Low priority items at the end of the queue.
        let mut paths_queue = Vec::<&Path>::new();
        paths_queue.extend(paths_high_prio);
        paths_queue.extend(paths_low_prio);

        // Take only a few items from the start of the queue.
        let length0: usize = paths_queue.len();
        paths_queue.truncate(Self::LIMIT_NUMBER_OF_PROGRAMS_FOR_PROCESSING);
        let length1: usize = paths_queue.len();
        if length0 != length1 {
            println!("Number of programs in queue {}. Truncating to {}.", length0, length1);
        }

        let paths_for_processing: Vec<PathBuf> = paths_queue.iter().map(|&path|PathBuf::from(path)).collect();
        if paths_for_processing.is_empty() {
            return Err(anyhow::anyhow!("No pending programs in the 'mine-event' dir."));
        }
        self.paths_for_processing = paths_for_processing;
        Ok(())
    }

    fn populate_candidate_programs(&mut self) -> anyhow::Result<()> {
        let mut candidate_programs = Vec::<CandidateProgramItem>::with_capacity(self.paths_for_processing.len());
        for path in &self.paths_for_processing {
            let candidate_program = CandidateProgram::new(path)
                .map_err(|e| anyhow::anyhow!("Unable to create CandidateProgram. error: {:?}", e))?;

            let candidate_program_item = Rc::new(RefCell::new(candidate_program));
            candidate_programs.push(candidate_program_item);
        }
        self.candidate_programs = candidate_programs;
        Ok(())
    }

    /// OeisId's that are to be ignored.
    /// 
    /// OEIS contains duplicates, sequences that have later been withdrawn, legacy sequences.
    /// It makes no sense wasting resources on those sequences.
    /// 
    /// The list loaded from `~/.loda-rust/analytics/dont_mine.csv`
    /// which is populated with the content of `loda-program/oeis/deny.txt`.
    fn obtain_dontmine_program_ids(&mut self) -> anyhow::Result<()> {
        let path = self.config.analytics_dir_dont_mine_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)
            .map_err(|e| anyhow::anyhow!("obtain_dontmine_program_ids - unable to load program_ids. error: {:?}", e))?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: OeisIdHashSet = HashSet::from_iter(program_ids.iter().cloned());
        debug!("loaded dontmine file. number of records: {}", hashset.len());
        self.dontmine_hashset = hashset;
        Ok(())
    }    

    fn obtain_invalid_program_ids(&mut self) -> anyhow::Result<()> {
        let path = self.config.analytics_dir_programs_invalid_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)
            .map_err(|e| anyhow::anyhow!("obtain_invalid_program_ids - unable to load program_ids. error: {:?}", e))?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: OeisIdHashSet = HashSet::from_iter(program_ids.iter().cloned());
        debug!("loaded invalid program_ids file. number of records: {}", hashset.len());
        self.invalid_program_ids_hashset = hashset;
        Ok(())
    }

    fn obtain_valid_program_ids(&mut self) -> anyhow::Result<()> {
        let path = self.config.analytics_dir_programs_valid_file();
        let program_ids_raw: Vec<u32> = load_program_ids_csv_file(&path)
            .map_err(|e| anyhow::anyhow!("obtain_valid_program_ids - unable to load program_ids. error: {:?}", e))?;
        let program_ids: Vec<OeisId> = program_ids_raw.iter().map(|x| OeisId::from(*x)).collect();
        let hashset: OeisIdHashSet = HashSet::from_iter(program_ids.iter().cloned());
        debug!("loaded valid program_ids file. number of records: {}", hashset.len());
        self.valid_program_ids_hashset = hashset;
        Ok(())
    }

    fn eval_using_loda_cpp(&mut self) -> anyhow::Result<()> {
        let start = Instant::now();
        let time_limit = Duration::from_secs(Self::LODACPP_EVAL_TIME_LIMIT_IN_SECONDS);

        let number_of_pending_programs: usize = self.candidate_programs.len();
        let pb = ProgressBar::new(number_of_pending_programs as u64);

        let mut count_success: usize = 0;
        let mut count_failure: usize = 0;
        for candidate_program in self.candidate_programs.iter_mut() {
            let path_original = PathBuf::from(candidate_program.borrow().path_original());
            let result = self.lodacpp.eval_terms(
                Self::EVAL_TERM_COUNT, 
                &path_original,
                time_limit
            );
            let evalterms: LodaCppEvalTerms = match result {
                Ok(value) => value,
                Err(error) => {
                    let reason = format!("Couldn't eval program with loda-cpp, {:?}", error);
                    let msg = format!("Rejecting {}, {}", candidate_program.borrow(), reason);
                    pb.println(msg);
                    candidate_program.borrow_mut().perform_reject(reason)
                        .map_err(|e| anyhow::anyhow!("eval_using_loda_cpp -> perform_reject. path_original: {:?} error: {:?}", path_original, e))?;
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

        debug!("evaluate: count_success: {}", count_success);
        if count_failure > 0 {
            error!("evaluate: count_failure: {}", count_failure);
        }
        Ok(())
    }

    /// Look up the initial terms in the OEIS `stripped` file and gather the corresponding program ids.
    fn lookup_in_oeis_stripped_file(&mut self) -> anyhow::Result<()> {
        let start = Instant::now();
        println!("Looking up in the OEIS 'stripped' file");

        let mut oeis_ids_to_ignore: OeisIdHashSet = self.dontmine_hashset.clone();
        if self.focus_only_on_new_programs {
            oeis_ids_to_ignore.extend(&self.valid_program_ids_hashset);
        }

        let oeis_stripped_file: PathBuf = self.config.oeis_stripped_file();
        assert!(oeis_stripped_file.is_absolute());
        assert!(oeis_stripped_file.is_file());
        let file: File = File::open(&oeis_stripped_file)
            .with_context(|| format!("lookup_in_oeis_stripped_file - Failed to open OEIS 'stripped' file: {:?}", oeis_stripped_file))?;
        let filemetadata: Metadata = file.metadata()
            .with_context(|| format!("lookup_in_oeis_stripped_file - Failed to obtain metadata about the OEIS 'stripped' file: {:?}", oeis_stripped_file))?;
        let filesize: usize = filemetadata.len() as usize;
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

        for candidate_program in programs_without_possible_ids {
            if candidate_program.borrow().state() != State::PendingProcessing {
                continue;
            }
            debug!("Rejected {}, where terms cannot be found in OEIS 'stripped' file", candidate_program.borrow());
            candidate_program.borrow_mut().perform_reject("lookup_in_oeis_stripped_file, Terms cannot be found in OEIS 'stripped' file")
                .map_err(|e| anyhow::anyhow!("lookup_in_oeis_stripped_file -> perform_reject. error: {:?}", e))?;
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

    fn minimize_candidate_programs(&mut self) -> anyhow::Result<()> {
        let start = Instant::now();

        let candidate_programs: Vec<CandidateProgramItem> = self.pending_candidate_programs();
        if candidate_programs.is_empty() {
            println!("There are no pending candidate programs in the 'mine-event' dir. Stopping.");
            return Ok(());
        }

        let number_of_candidate_programs: usize = candidate_programs.len();
        println!("Minimizing programs");
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

    fn minimize_candidate_program(&mut self, candidate_program: CandidateProgramItem) -> anyhow::Result<()> {
        let time_limit = Duration::from_secs(Self::LODACPP_MINIMIZE_TIME_LIMIT_IN_SECONDS);
        let path_original = PathBuf::from(candidate_program.borrow().path_original());
        let result = self.lodacpp.minimize(&path_original, time_limit);
        match result {
            Ok(value) => {
                // debug!("minimized program successfully:\n{}", value);
                candidate_program.borrow_mut().assign_minimized_program(value);
            },
            Err(error) => {
                let reason = format!("Unable to minimize program: {:?}", error);
                // debug!("program: {:?}, rejection reason {}", candidate_program.borrow().path_original(), reason);
                candidate_program.borrow_mut().perform_reject(&reason)
                    .map_err(|e| anyhow::anyhow!("minimize_candidate_program -> perform_reject. path_original: {:?} reason: {:?} error: {:?}", path_original, reason, e))?;
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
        debug!("obtained {} sequence names", oeis_id_name_map.len());
        self.oeis_id_name_map = oeis_id_name_map;
        Ok(())
    }

    fn process_candidate_programs(&mut self) -> anyhow::Result<()> {
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

            candidate_program.borrow_mut().perform_keep_or_reject_based_result()
                .map_err(|e| anyhow::anyhow!("process_candidate_programs -> perform_keep_or_reject_based_result. error: {:?}", e))?;
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
        path_for_oeis_program(&self.loda_programs_oeis_dir, program_id)
    }

    /// Construct a path, like this: `/absolute/path//041/A041009_30_5.asm`
    fn path_to_mismatch(&self, oeis_id: OeisId, correct_term_count: usize) -> anyhow::Result<ParentDirAndChildFile> {
        self.unique_path_for_saving_into_loda_outlier_programs(oeis_id, correct_term_count, "")
    }

    /// Construct a path, like this: `/absolute/path//041/A041009_timeout_33333_5.asm`
    /// 
    /// The `A041009` is the OeisId.
    /// 
    /// The `timeout` indicates that `loda check` exceeded the time limit, and thus it's
    /// undecided if this is a full match or a partial match.
    /// 
    /// The `33333` is the number of terms that are correct.
    /// 
    /// The `5` is the index that prevents the name from clashing with similar names.
    fn path_to_timeout(&self, oeis_id: OeisId, correct_term_count: usize) -> anyhow::Result<ParentDirAndChildFile> {
        self.unique_path_for_saving_into_loda_outlier_programs(oeis_id, correct_term_count, "_timeout")
    }

    /// The destination path, where to mined program is saved.
    /// There are already lots of programs with similar names.
    /// In order to ensure the name is unique, an index gets incremented until a name becomes available.
    fn unique_path_for_saving_into_loda_outlier_programs(&self, oeis_id: OeisId, correct_term_count: usize, name_suffix: &str) -> anyhow::Result<ParentDirAndChildFile> {
        assert!(self.loda_outlier_programs_repository_oeis_divergent.is_dir());
        assert!(self.loda_outlier_programs_repository_oeis_divergent.is_absolute());
        let dir_index: u32 = oeis_id.raw() / 1000;
        let dir_index_string: String = format!("{:0>3}", dir_index);
        let dir_path: PathBuf = self.loda_outlier_programs_repository_oeis_divergent.join(&dir_index_string);
        let name = oeis_id.a_number();
        for variant_index in 0..Self::MAX_NUMBER_OF_OUTLIER_VARIANTS {
            let filename = format!("{}{}_{}_{}.asm", name, name_suffix, correct_term_count, variant_index);
            let file_path: PathBuf = dir_path.join(filename);
            if !file_path.is_file() {
                return Ok(ParentDirAndChildFile::new(dir_path, file_path))
            }
        }
        Err(anyhow::anyhow!("loda_outlier_programs repo: Cannot construct unique filename for {:?} inside dir: {:?}", oeis_id.a_number(), dir_path))
    }

    fn determine_status_of_existing_program(&self, program_id: OeisId, path: &Path) -> StatusOfExistingProgram {
        if !path.is_file() {
            debug!("There is no existing file in loda-programs repo for: {}", program_id);
            return StatusOfExistingProgram::NoExistingProgram;
        }
        match self.validate_single_program.run(path) {
            Ok(_) => {
                debug!("The existing file in loda-programs repo {} seems ok", program_id);
                return StatusOfExistingProgram::CompareNewWithExisting;
            },
            Err(error) => {
                debug!("The existing program {} in loda-programs repo. error: {:?}", program_id, error);
                let reason = format!("There is a problem with the existing program {} in loda-programs repo. error: {:?}", program_id.a_number(), error);
                return StatusOfExistingProgram::IgnoreExistingProgram { ignore_reason: reason };
            }
        };
    }
    
    /// Decide wether to keep or reject a candidate program.
    /// 
    /// Determines if the candidate program is correct.
    /// 
    /// Determines if the candidate program is an improvement over an eventual existing program.
    fn analyze_candidate(
        &mut self, 
        candidate_program: CandidateProgramItem, 
        possible_id: OeisId, 
        progressbar: ProgressBar
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

        let status_of_existing_program: StatusOfExistingProgram = self.determine_status_of_existing_program(possible_id, oeis_program_path.child_file());

        let check_program_filename = format!("iteration{}_program.asm", self.iteration);
        let check_program_path: PathBuf = self.path_timestamped_postmine_dir.join(check_program_filename);

        let check_output_filename = format!("iteration{}_loda_check.txt", self.iteration);
        let check_output_path: PathBuf = self.path_timestamped_postmine_dir.join(check_output_filename);
        
        let compare_output_filename = format!("iteration{}_compare.txt", self.iteration);
        let compare_output_path: PathBuf = self.path_timestamped_postmine_dir.join(compare_output_filename);
        
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

        let program_contents: String = candidate_program.borrow().minimized_program().clone();

        // Format the program
        let mut fp = FormatProgram::new(program_contents.clone());
        fp.program_oeis_id(possible_id);
        fp.loda_submitted_by(self.loda_submitted_by.clone());
        fp.oeis_id_name_map(self.oeis_id_name_map.clone());
        fp.program_path(oeis_program_path.child_file());
        fp.terms(resolved_terms.clone());
        let file_content: String = fp.build()?;
        
        // Save the program to disk
        let mut check_program_file = File::create(&check_program_path)?;
        check_program_file.write_all(file_content.as_bytes())?;
        check_program_file.sync_all()?;
        // debug!("Created program file: {:?}", check_program_path);
    
        // Execute `loda-check check <PATH> -b`
        let time_limit = Duration::from_secs(Self::LODACPP_CHECK_TIME_LIMIT_IN_SECONDS);
        let check_start = Instant::now();
        let ok_error = self.lodacpp.perform_check_and_save_output(&check_program_path, time_limit, &check_output_path);
        simple_log.println(format!("check - elapsed {}", HumanDuration(check_start.elapsed())));
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
            LodaCppCheckStatus::FullMatch => {
                self.process_full_match(
                    simple_log.clone(),
                    candidate_program,
                    &file_content,
                    possible_id,
                    &check_program_path,
                    &oeis_program_path,
                    status_of_existing_program,
                    &compare_output_path,
                    check_result.number_of_correct_terms as usize,
                    progressbar.clone(),
                )?;
            },
            LodaCppCheckStatus::PartialMatch => {
                self.process_partial_match(
                    simple_log.clone(),
                    candidate_program,
                    &check_program_path,
                    possible_id,
                    check_result.number_of_correct_terms
                )?;
            },
            LodaCppCheckStatus::Timeout => {
                self.process_timeout(
                    simple_log.clone(),
                    candidate_program,
                    &check_program_path,
                    possible_id,
                    check_result.number_of_correct_terms
                )?;
            },
        }

        Ok(())
    }

    fn process_full_match(
        &self, 
        simple_log: SimpleLog, 
        candidate_program: CandidateProgramItem, 
        file_content: &String,
        oeis_id: OeisId, 
        path_program0: &Path, 
        path_program1: &ParentDirAndChildFile,
        status_of_existing_program: StatusOfExistingProgram,
        path_comparison: &Path,
        number_of_correct_terms: usize,
        progressbar: ProgressBar
    ) -> anyhow::Result<()> {
        if number_of_correct_terms < Self::MINIMUM_NUMBER_OF_REQUIRED_TERMS {
            let message = format!("process_full_match: Rejecting program with too few terms. Expected {} or more terms, but got {} terms.", Self::MINIMUM_NUMBER_OF_REQUIRED_TERMS, number_of_correct_terms);
            simple_log.println(message);
            return Ok(());
        }

        // Don't attempt to compute more terms than what the b-file already contains
        // since this can cause huge numbers.
        let term_count: usize = Self::LODACPP_COMPARE_NUMBER_OF_TERM_COUNT.min(number_of_correct_terms);
        let message = format!("process_full_match: will compare steps count for {} terms.", term_count);
        simple_log.println(message);

        let time_limit = Duration::from_secs(Self::LODACPP_STEPS_TIME_LIMIT_IN_SECONDS);
        let ok_error = CompareTwoPrograms::compare(
            simple_log.clone(),
            &self.lodacpp,    
            path_program0, 
            path_program1.child_file(),
            &status_of_existing_program,
            path_comparison, 
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
                simple_log.println("Keeping. The new program is an improvement.");
            },
            CompareTwoProgramsResult::Program1 => {
                simple_log.println("Rejecting. The new program isn't better than the existing program.");
                return Ok(());
            }
        }

        // Program0 (The candidate program) is an improvement.

        // Human readable message above the progressbar,
        // that explains why Program0 is an improvement.
        match status_of_existing_program {
            StatusOfExistingProgram::NoExistingProgram => {
                progressbar.println(format!("miner discovered a \"new\" program. {}", oeis_id));
            },
            StatusOfExistingProgram::IgnoreExistingProgram { ignore_reason: _ } => {
                progressbar.println(format!("miner discovered an \"improved\" program. {}", oeis_id));
            },
            StatusOfExistingProgram::CompareNewWithExisting => {
                progressbar.println(format!("miner discovered a \"faster\" program. {}", oeis_id));
            }
        }

        // Save program to disk
        path_program1.create_parent_dir()
            .map_err(|e| anyhow::anyhow!("Unable to create parent dir for matching program. program_id: {:?} error: {:?}", oeis_id, e))?;
        fs::copy(path_program0, path_program1.child_file())?;
        candidate_program.borrow_mut().keep_id_insert(oeis_id);

        // Invoke callback with the discovered program
        if let Some(ref callback) = self.found_program_callback {
            callback(file_content.clone(), oeis_id);
        }
        Ok(())
    }

    fn process_partial_match(
        &self, 
        simple_log: SimpleLog, 
        candidate_program: CandidateProgramItem, 
        path_program0: &Path, 
        oeis_id: OeisId, 
        number_of_correct_terms: u32
    ) -> anyhow::Result<()> {
        let destination_path: ParentDirAndChildFile = match self.path_to_mismatch(oeis_id, number_of_correct_terms as usize) {
            Ok(value) => value,
            Err(error) => {
                let message = format!("process_partial_match: discarding program. path_to_mismatch. oeis_id: {} error: {:?}", oeis_id, error);
                simple_log.println(message);
                return Ok(());
            }
        };
        destination_path.create_parent_dir()
            .map_err(|e| anyhow::anyhow!("process_partial_match: Unable to create parent dir. oeis_id: {:?} error: {:?}", oeis_id, e))?;

        let message = format!("Keeping. This program is a mismatch, it has correct {} terms, followed by mismatch. Saving at: {:?}", number_of_correct_terms, destination_path.child_file());
        simple_log.println(message);
        fs::copy(path_program0, destination_path.child_file())?;
        candidate_program.borrow_mut().keep_id_insert(oeis_id);
        Ok(())
    }

    fn process_timeout(
        &self, 
        simple_log: SimpleLog, 
        candidate_program: CandidateProgramItem, 
        path_program0: &Path, 
        oeis_id: OeisId, 
        number_of_correct_terms: u32
    ) -> anyhow::Result<()> {
        let destination_path: ParentDirAndChildFile = match self.path_to_timeout(oeis_id, number_of_correct_terms as usize) {
            Ok(value) => value,
            Err(error) => {
                let message = format!("process_timeout: discarding program. path_to_timeout. error: {:?}", error);
                simple_log.println(message);
                return Ok(());
            }
        };
        destination_path.create_parent_dir()
            .map_err(|e| anyhow::anyhow!("process_timeout: Unable to create parent dir. oeis_id: {:?} error: {:?}", oeis_id, e))?;

        let message = format!("Keeping. Timeout while checking this program. Undecided if this is a full match or a partial match. It has correct {} terms. Saving at: {:?}", number_of_correct_terms, destination_path.child_file());
        simple_log.println(message);
        fs::copy(path_program0, destination_path.child_file())?;
        candidate_program.borrow_mut().keep_id_insert(oeis_id);
        Ok(())
    }
}
