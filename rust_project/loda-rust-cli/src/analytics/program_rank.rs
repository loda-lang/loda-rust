use super::AnalyticsDirectory;
use crate::common::{create_csv_file, parse_csv_file};
use simple_pagerank::Pagerank;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

const HIGHEST_POPULARITY_VALUE: usize = 9;

#[derive(Debug, Deserialize)]
struct RecordDependency {
    #[serde(rename = "caller program id")]
    source: u32,
    #[serde(rename = "callee program id")]
    target: u32,
}

/// Determine the popularity group which each program belong to.
/// 
/// The miner is more likely to call a popular program, eg. the Fibonacci program, or the sqrt2 function.
/// 
/// The miner is less likely to call an unpopular program, eg. an unreferenced program, or few insignificant references. 
/// 
/// This saves a `program_popularity.csv` file, with this format:
/// 
/// ```csv
/// program id;popularity
/// 5;9
/// 6;9
/// 8;8
/// 10;9
/// 15;9
/// 30;4
/// 31;8
/// 32;9
/// 33;6
/// ```
pub fn compute_program_rank(analytics_directory: AnalyticsDirectory) {
    let input_path: PathBuf = analytics_directory.dependencies_file();
    let output_rank_path: PathBuf = analytics_directory.program_rank_file();
    let output_popularity_path: PathBuf = analytics_directory.program_popularity_file();
    let pr: Pagerank<u32> = calculate_pagerank(&input_path);
    create_dependencies_csv_file(&pr, &output_rank_path);
    create_popularity_csv_file(&pr, &output_popularity_path, HIGHEST_POPULARITY_VALUE);
}

fn calculate_pagerank(input_path: &Path) -> Pagerank::<u32> {
    let dependency_vec: Vec<RecordDependency> = parse_csv_file(input_path).expect("Unable to load input file");
    let mut pr = Pagerank::<u32>::new();
    for dependency in &dependency_vec {
        pr.add_edge(dependency.source, dependency.target);
    }
    pr.calculate();
    pr
}

#[derive(Debug, Deserialize, Serialize)]
struct ProgramRankItem {
    #[serde(rename = "program id")]
    program_id: u32,
    score: String,
}

fn create_dependencies_csv_file(pagerank: &Pagerank::<u32>, output_path: &Path) {
    let mut items = Vec::<ProgramRankItem>::new();
    for node in pagerank.nodes() {
        let item = ProgramRankItem {
            program_id: *node.0,
            score: format!("{:.4}", node.1),
        };
        items.push(item);
    }

    match create_csv_file(&items, &output_path) {
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

#[derive(Clone, Copy, Deserialize, Serialize)]
struct ProgramPopularityItem {
    #[serde(rename = "program id")]
    program_id: u32,
    popularity: usize,
}

fn create_popularity_csv_file(pagerank: &Pagerank::<u32>, output_path: &Path, highest_popularity_value: usize) {
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
    let mut lowest_score: i64 = 0;
    if let Some(item0) = program_score_items.first() {
        lowest_score = item0.score;
    }
    for item in &program_score_items {
        if lowest_score > item.score {
            lowest_score = item.score;
        }
    }

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
        let popularity = highest_popularity_value - (index * highest_popularity_value / count);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_10000_create_dependencies_csv_file() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_create_dependencies_csv_file");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("input.csv");
        let output_path: PathBuf = basedir.join("output.csv");

        let input_content = 
r#"caller program id;callee program id
6;4
4;5
3;6
2;5
1;2
1;5
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;

        // Act
        let pr = calculate_pagerank(&input_path);
        create_dependencies_csv_file(&pr, &output_path);

        // Assert
        let result_records: Vec<ProgramRankItem> = parse_csv_file(&output_path)?;
        let mut result_items = Vec::<String>::new();
        for record in result_records {
            result_items.push(format!("{}", record.program_id));
        }
        let result = result_items.join(",");
        assert_eq!(result, "5,4,6,2,3,1");
        Ok(())
    }

    #[test]
    fn test_20000_create_popularity_csv_file() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_20000_create_popularity_csv_file");
        fs::create_dir(&basedir)?;
        let input_path: PathBuf = basedir.join("input.csv");
        let output_path: PathBuf = basedir.join("output.csv");

        let input_content = 
r#"caller program id;callee program id
8;9
7;8
6;7
5;6
4;5
3;4
2;3
1;2
104;1
100;1
103;1
101;1
102;1
"#;
        let mut input_file = File::create(&input_path)?;
        input_file.write_all(input_content.as_bytes())?;

        // Act
        let pr = calculate_pagerank(&input_path);
        create_popularity_csv_file(&pr, &output_path, 3);

        // Assert
        let result_records: Vec<ProgramPopularityItem> = parse_csv_file(&output_path)?;
        let mut result_items = Vec::<String>::new();
        for record in result_records {
            result_items.push(format!("{} {}", record.program_id, record.popularity));
        }
        let result = result_items.join(",");
        assert_eq!(result, "1 1,2 1,3 1,4 2,5 2,6 2,7 3,8 3,9 3,100 0,101 0,102 0,103 0,104 0");
        Ok(())
    }
}
