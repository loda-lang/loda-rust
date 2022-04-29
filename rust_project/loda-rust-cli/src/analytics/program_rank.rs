use loda_rust_core::config::Config;
use simple_pagerank::Pagerank;
use crate::common::parse_csv_file;
use std::path::{Path, PathBuf};
use std::error::Error;
use serde::{Serialize, Deserialize};
use csv::WriterBuilder;

#[derive(Debug, Deserialize)]
struct RecordDependency {
    #[serde(rename = "caller program id")]
    source: String,
    #[serde(rename = "callee program id")]
    target: String,
}

impl RecordDependency {
    pub fn parse_csv(path: &Path) -> Result<Vec<RecordDependency>, Box<dyn Error>> {
        parse_csv_file(path)
    }
}

#[derive(Serialize)]
struct ProgramRankResult {
    #[serde(rename = "program id")]
    program_id: String,
    score: String,
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
    let output_path: PathBuf = config.analytics_dir_program_rank_file();
    let dependency_vec: Vec<RecordDependency> = RecordDependency::parse_csv(&input_path).expect("Unable to load input file");

    let mut pr = Pagerank::<&str>::new();
    for dependency in &dependency_vec {
        pr.add_edge(&dependency.source, &dependency.target);
    }
    pr.calculate();

    create_dependencies_csv_file(&pr, &output_path);
}

fn create_dependencies_csv_file(pagerank: &Pagerank::<&str>, output_path: &Path) {
    let mut rows = Vec::<ProgramRankResult>::new();
    for node in pagerank.nodes() {
        let row = ProgramRankResult {
            program_id: node.0.to_string(),
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
