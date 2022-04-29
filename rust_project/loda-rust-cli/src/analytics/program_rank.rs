use loda_rust_core::config::Config;
use simple_pagerank::Pagerank;
use crate::common::parse_csv_file;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use csv::WriterBuilder;

const HIGHEST_POPULARITY_VALUE: usize = 9;

#[derive(Debug, Deserialize)]
struct RecordDependency {
    #[serde(rename = "caller program id")]
    source: u32,
    #[serde(rename = "callee program id")]
    target: u32,
}

impl RecordDependency {
    pub fn parse_csv(path: &Path) -> Result<Vec<RecordDependency>, Box<dyn Error>> {
        parse_csv_file(path)
    }
}

fn create_csv_file<S: Serialize>(records: &Vec<S>, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_path(output_path)?;
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}


pub fn compute_program_rank() {
    let config = Config::load();
    let input_path: PathBuf = config.analytics_dir_dependencies_file();
    let output_rank_path: PathBuf = config.analytics_dir_program_rank_file();
    let output_popularity_path: PathBuf = config.analytics_dir_program_popularity_file();
    let dependency_vec: Vec<RecordDependency> = RecordDependency::parse_csv(&input_path).expect("Unable to load input file");

    let mut pr = Pagerank::<u32>::new();
    for dependency in &dependency_vec {
        pr.add_edge(dependency.source, dependency.target);
    }
    pr.calculate();

    create_dependencies_csv_file(&pr, &output_rank_path);
    create_popularity_csv_file(&pr, &output_popularity_path);
}

#[derive(Serialize)]
struct ProgramRankResult {
    #[serde(rename = "program id")]
    program_id: u32,
    score: String,
}

fn create_dependencies_csv_file(pagerank: &Pagerank::<u32>, output_path: &Path) {
    let mut rows = Vec::<ProgramRankResult>::new();
    for node in pagerank.nodes() {
        let row = ProgramRankResult {
            program_id: *node.0,
            score: format!("{:.4}", node.1),
        };
        rows.push(row);
    }

    match create_csv_file(&rows, &output_path) {
        Ok(()) => {},
        Err(error) => {
            error!("Unable to save csv file: {:?}", error);
        }
    }
}

#[derive(Clone, Copy)]
struct ProgramScoreItem {
    program_id: u32,
    score: i64,
}

#[derive(Clone, Copy, Serialize)]
struct ProgramPopularityItem {
    #[serde(rename = "program id")]
    program_id: u32,
    popularity: usize,
}

fn create_popularity_csv_file(pagerank: &Pagerank::<u32>, output_path: &Path) {
    let mut program_score_items = Vec::<ProgramScoreItem>::new();
    for node in pagerank.nodes() {
        let score_float: f64 = node.1;
        let score_int = (score_float * 1000000.0).floor() as i64; 
        let item = ProgramScoreItem {
            program_id: *node.0,
            score: score_int,
        };
        program_score_items.push(item);
    }

    // Find lowest score
    let mut histogram = HashMap::<i64,u32>::new();
    for item in &program_score_items {
        let counter = histogram.entry(item.score).or_insert(0);
        *counter += 1;
    }
    let mut found_key: i64 = -666;
    let mut found_value: u32 = 0;
    for (key, value) in histogram.iter() {
        if *value > found_value {
            found_value = *value;
            found_key = *key;
        }
    }
    let lowest_score: i64 = found_key;

    // Separate the unused programs, from the used programs
    let (items_with_lowest_score, items_remaining): (Vec<ProgramScoreItem>, Vec<ProgramScoreItem>) = 
        program_score_items.iter()
        .partition(|&row| row.score == lowest_score);

    let mut result = Vec::<ProgramPopularityItem>::new();

    // bin=0: There a lot of unused programs, so they get their own bin
    for item in items_with_lowest_score {
        let result_item = ProgramPopularityItem { program_id: item.program_id, popularity: 0 };
        result.push(result_item);
    }

    // bin=1..9: Split into k approx evenly sized bins
    // bin 9 is for the highly popular and influential programs
    // bin 5 is for the medium popular programs
    // bin 1 is for the rarely used programs
    // bin 0 is for the unused and unpopular programs. This cluster is huge.
    let count: usize = items_remaining.len();
    for (index, item) in items_remaining.iter().enumerate() {
        let popularity = HIGHEST_POPULARITY_VALUE - (index * HIGHEST_POPULARITY_VALUE / count);
        let result_item = ProgramPopularityItem { program_id: item.program_id, popularity: popularity };
        result.push(result_item);
    }

    // sort by program id
    result.sort_unstable_by_key(|item| (item.program_id));

    match create_csv_file(&result, &output_path) {
        Ok(()) => {},
        Err(error) => {
            error!("Unable to save csv file: {:?}", error);
        }
    }
}
