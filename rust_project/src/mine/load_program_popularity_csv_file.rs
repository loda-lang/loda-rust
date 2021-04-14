use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use serde::Deserialize;

// Inside the file `program_popularity.csv`:
// The column `popularity` contain values in the range 0 to 9.
// where 0 is the unpopular program_ids.
// where 5 is the medium used program_ids.
// where 9 is the most used program_ids.
const NUMBER_OF_CLUSTERS: u8 = 10;

pub struct PopularProgramContainer {
    cluster_program_ids: Vec<Vec<u32>>,
}

impl PopularProgramContainer {
    fn cluster_program_ids(&self) -> &Vec<Vec<u32>> {
        &self.cluster_program_ids
    }
}

#[derive(Debug)]
pub enum ProgramPopularityError {
    PopularityClusterIdOutOfBounds,
}

impl fmt::Display for ProgramPopularityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PopularityClusterIdOutOfBounds => 
                write!(f, "Cluster id is out of bounds")
        }
    }
}

impl Error for ProgramPopularityError {}

pub fn load_program_popularity_csv_file(path: &Path) -> Result<PopularProgramContainer, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    process_csv_into_clusters(&mut reader)
}

fn process_csv_into_clusters(reader: &mut dyn BufRead) -> Result<PopularProgramContainer, Box<dyn Error>> {
    let records: Vec<Record> = process_csv_data(reader)?;
    convert_records_to_clusters(records)
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "program id")]
    program_id: u32,

    #[serde(rename = "popularity")]
    popularity_cluster_id: u8,
}

impl Record {
    fn new(program_id: u32, popularity_cluster_id: u8) -> Self {
        Self {
            program_id: program_id,
            popularity_cluster_id: popularity_cluster_id,
        }
    }
}

fn process_csv_data(reader: &mut dyn BufRead) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut records = Vec::<Record>::new();
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(reader);
    for result in csv_reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    Ok(records)
}

fn convert_records_to_clusters(records: Vec<Record>) -> Result<PopularProgramContainer, Box<dyn Error>> {
    // Ensure there isn't too many clusters
    let mut max_cluster_id: u8 = 0;
    for record in &records {
        if max_cluster_id < record.popularity_cluster_id {
            max_cluster_id = record.popularity_cluster_id;
        }
    }
    if max_cluster_id >= NUMBER_OF_CLUSTERS {
        return Err(Box::new(ProgramPopularityError::PopularityClusterIdOutOfBounds));
    }

    // Identify program_ids for each cluster
    let mut clusters: Vec<Vec<u32>> = vec!();
    for cluster_id in 0..NUMBER_OF_CLUSTERS {
        let mut program_ids: Vec<u32> = vec!();
        for record in &records {
            if record.popularity_cluster_id == cluster_id {
                program_ids.push(record.program_id);
            }
        }
        clusters.push(program_ids);
    }

    let container = PopularProgramContainer {
        cluster_program_ids: clusters
    };
    Ok(container)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_10000_process_csv_data() {
        let data = "\
program id;popularity
4;1

5;9
6;8
7;3
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = process_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {}", record.program_id, record.popularity_cluster_id)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "4 1,5 9,6 8,7 3");
    }

    #[test]
    fn test_10001_convert_records_to_clusters_error_too_many_clusters() {
        let records: Vec<Record> = vec![
            // Cluster 9 is the highest allowed cluster.
            // Here using cluster 10 is beyond the max cluster and should trigger an error.
            Record::new(666, 10),
        ];
        let result = convert_records_to_clusters(records);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_10002_convert_records_to_clusters_success() {
        let records: Vec<Record> = vec![
            // 3 items in cluster 0
            Record::new(101, 0),
            Record::new(102, 0),
            Record::new(103, 0),

            // 1 item in cluster 4
            Record::new(301, 4),

            // 2 items in cluster 9
            Record::new(901, 9),
            Record::new(902, 9),
        ];
        let container: PopularProgramContainer = convert_records_to_clusters(records).unwrap();
        let cluster_program_ids: &Vec<Vec<u32>> = container.cluster_program_ids();
        assert_eq!(cluster_program_ids.len(), 10);
        assert_eq!(cluster_program_ids[0].len(), 3);
        assert_eq!(cluster_program_ids[4].len(), 1);
        assert_eq!(cluster_program_ids[9].len(), 2);
    }
}
