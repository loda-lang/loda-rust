use loda_rust_core::util::BigIntVec;
use crate::mine::FunnelConfig;
use crate::config::Config;
use crate::common::{create_csv_file, SimpleLog};
use crate::oeis::{ProcessStrippedSequenceFile, StrippedSequence};
use num_bigint::{BigInt, ToBigInt};
use std::convert::TryFrom;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use serde::Serialize;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

static DISCARD_EXTREME_VALUES_BEYOND_THIS_LIMIT: i64 = 400;

/// This code identifies a good magic value for the bloomfilter.
/// It should not be a value that is used a lot, so any value from 
/// the top 100 most used terms will be a terrible choice.
/// I made the mistake of choosing zero as magic value, causing
/// +400.000 files to be generated in less than 20 minutes!
///
/// It should be a value that is rarely used, so that there are as
/// few false-positives as possible.
/// At the same time it shouldn't be a huge value, like `0xCAFEBABE`.
/// BigInt/String manipulation is expensive, and there is a lot of it,
/// thus the wildcard value should be as few bytes as possible, 
/// so there are fewer bytes to be allocated/compared.
///
/// The most frequent occuring terms in the OEIS 'stripped' file are:
///
/// ```csv
/// count;value
/// 3277144;0
/// 791230;1
/// 402661;2
/// 295319;3
/// 251879;4
/// 207336;5
/// 187158;6
/// 161854;7
/// 155826;8
/// 135863;9
/// 78968;10
/// snip
/// 39094;-1
/// snip
/// 13576;-2
/// snip
/// 8044;-3
/// snip
/// 85;-67
/// snip
/// 61;-86
/// ```
///
/// The value `-67` only occurs 85 times, and `-86` occurs 61 times,
/// so these may be good choices for use as a magic value.
///
/// Number of extreme values: 4084887,
/// that are outside the range -400 .. +400.
pub struct HistogramStrippedFile {
    config: Config,
    simple_log: SimpleLog,
    histogram: HashMap<i64,u32>,
}

impl HistogramStrippedFile {
    pub fn run(simple_log: SimpleLog) -> Result<(), Box<dyn Error>> {
        let config = Config::load();
        let mut instance = Self {
            config: config,
            simple_log: simple_log,
            histogram: HashMap::new(),
        };
        instance.run_inner()?;
        Ok(())
    }

    fn run_inner(&mut self) -> Result<(), Box<dyn Error>> {
        self.simple_log.println("\nHistogram of OEIS 'stripped' file");
        println!("Histogram of OEIS 'stripped' file");

        let oeis_stripped_file: PathBuf = self.config.oeis_stripped_file();
        assert!(oeis_stripped_file.is_absolute());
        assert!(oeis_stripped_file.is_file());

        let file = File::open(oeis_stripped_file).unwrap();
        let filesize: usize = file.metadata().unwrap().len() as usize;
        let mut reader = BufReader::new(file);
        Self::histogram_of_terms_in_oeis_stripped_file(
            self.simple_log.clone(),
            &mut reader, 
            filesize,
            &mut self.histogram,
        )?;
        self.save()?;
        Ok(())
    }

    fn histogram_of_terms_in_oeis_stripped_file(
        simple_log: SimpleLog,
        oeis_stripped_file_reader: &mut dyn io::BufRead, 
        filesize: usize,
        histogram: &mut HashMap::<i64,u32>,
    ) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();
        let mut count_big: u32 = 0;
        let mut count_small: u32 = 0;
        let mut count_wildcard: u32 = 0;
        let pb = ProgressBar::new(filesize as u64);
        let padding_value_i64: i64 = 0xC0FFEE;
        let padding_value: BigInt = padding_value_i64.to_bigint().unwrap();
        let process_callback = |stripped_sequence: &StrippedSequence, count_bytes: usize| {
            pb.set_position(count_bytes as u64);
            let all_vec: &BigIntVec = stripped_sequence.bigint_vec_ref();
            for value in all_vec {
                let key: i64 = match i64::try_from(value).ok() {
                    Some(value) => value,
                    None => {
                        count_big += 1;
                        continue;
                    }
                };
                if key == padding_value_i64 {
                    count_wildcard += 1;
                    continue;
                }
                if key.abs() > DISCARD_EXTREME_VALUES_BEYOND_THIS_LIMIT {
                    count_big += 1;
                    continue;
                }
                let counter = histogram.entry(key).or_insert(0);
                *counter += 1;
                count_small += 1;
            }
        };
        let program_ids_to_ignore = HashSet::<u32>::new();
        let mut stripped_sequence_processor = ProcessStrippedSequenceFile::new();
        stripped_sequence_processor.execute(
            oeis_stripped_file_reader,
            FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS,
            FunnelConfig::TERM_COUNT,
            &program_ids_to_ignore,
            &padding_value,
            true,
            process_callback
        );
        pb.finish_and_clear();
    
        let green_bold = Style::new().green().bold();        
        println!(
            "{:>12} Histogram of OEIS 'stripped' file, in {}",
            green_bold.apply_to("Finished"),
            HumanDuration(start.elapsed())
        );
    
        simple_log.println(format!("Number of small values: {}", count_small));
        simple_log.println(format!("Number of big values: {}", count_big));
        simple_log.println(format!("Number of wildcard values: {}", count_wildcard));
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut records = Vec::<Record>::new();
        for (histogram_key, histogram_count) in &self.histogram {
            let record = Record {
                count: *histogram_count,
                value: *histogram_key,
            };
            records.push(record);
        }
        for i in -DISCARD_EXTREME_VALUES_BEYOND_THIS_LIMIT..(DISCARD_EXTREME_VALUES_BEYOND_THIS_LIMIT+1) {
            if !self.histogram.contains_key(&i) {
                let record = Record {
                    count: 0,
                    value: i,
                };
                records.push(record);
            }
        }
    
        // Move the most frequently occuring items to the top
        // Move the lesser used items to the bottom
        records.sort_unstable_by_key(|item| (item.count, item.value.clone()));
        records.reverse();
    
        // Save as a CSV file
        let output_path: PathBuf = self.config.analytics_dir_histogram_oeis_stripped_file();
        create_csv_file(&records, &output_path)
    }
}

#[derive(Debug, Serialize)]
struct Record {
    count: u32,
    value: i64,
}
