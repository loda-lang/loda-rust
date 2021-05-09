use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use serde::Deserialize;
use rand::Rng;
use rand::seq::SliceRandom;

// Divide up the programs into equal sized clusters with similar age.
const NUMBER_OF_CLUSTERS: u8 = 10;


// Some programs are newer than older programs.
//
// This is a data structure for picking a recently added program.
//
// Without this data structure, it would be terrible time consuming
// making a weighted choice among the programs.
//
// Inside the file `program_creation_dates.csv`:
// The column `creation date` contain a date with the format yyyymmdd.
// Example: "19841230" is a program created on: 1984 dec 30.
// This may be old or new, depending on what other dates are present in this csv file.
//
// A new program that have just been added to the repository,
// may be useful or not useful. The goal with this `RecentProgramContainer`
// is to exercise the most recent programs.
// The recent programs are assigned `cluster_id` 9.
//
// The oldest program that have been in the repository for years,
// these programs are assigned `cluster_id` 0.
//
// Programs with an age in between the oldest and the newest,
// These programs are assigned `cluster_id` 1..8.
//
// On initialization the `program_creation_dates.csv` is loaded.
// This CSV file have been generated by using `git log` for 
// extracting the creation date.
pub struct RecentProgramContainer {
    cluster_program_ids: Vec<Vec<u32>>,
}

impl RecentProgramContainer {
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        process_csv_into_clusters(&mut reader)
    }

    #[allow(dead_code)]
    fn cluster_program_ids(&self) -> &Vec<Vec<u32>> {
        &self.cluster_program_ids
    }

    #[allow(dead_code)]
    pub fn choose<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        let cluster_weight_vec: Vec<(usize,usize)> = vec![
            (0, 1), // Low probability for choosing an old program.
            (1, 2),
            (2, 4),
            (3, 8),
            (4, 16),
            (5, 32),
            (6, 64),
            (7, 128),
            (8, 256),
            (9, 512), // High probablility for choosing a recent program.
        ];
        assert!(cluster_weight_vec.len() == (NUMBER_OF_CLUSTERS as usize));
        let cluster_id: &usize = &cluster_weight_vec.choose_weighted(rng, |item| item.1).unwrap().0;
        let program_ids: &Vec<u32> = &self.cluster_program_ids[*cluster_id];
        if program_ids.is_empty() {
            // The CSV file is supposed to have hundreds of rows, so there
            // should be several program_ids for every cluster_id.
            // No matter what cluster_id is picked, there should be at least 1 program.
            // Return None, in the unfortunate case there isn't any program_ids for the picked cluser_id.
            return None;
        }
        let program_id: u32 = match program_ids.choose(rng) {
            Some(program_id) => *program_id,
            None => {
                // For a non-empty vector, this shouldn't happen.
                return None;
            }
        };
        Some(program_id)
    }

    #[cfg(test)]
    fn cluster_lengths(&self) -> String {
        let strings: Vec<String> = self.cluster_program_ids.iter().map(|program_ids| {
            format!("{}", program_ids.len())
        }).collect();
        strings.join(",")
    }
}

fn process_csv_into_clusters(reader: &mut dyn BufRead) -> Result<RecentProgramContainer, Box<dyn Error>> {
    let records: Vec<Record> = process_csv_data(reader)?;
    convert_records_to_clusters(records)
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "program id")]
    program_id: u32,

    #[serde(rename = "creation date")]
    creation_date: u32, // format like this: yyyymmdd, eg. 19841230
}

impl Record {
    #[cfg(test)]
    fn new(program_id: u32, creation_date: u32) -> Self {
        Self {
            program_id: program_id,
            creation_date: creation_date,
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

fn convert_records_to_clusters(mut records: Vec<Record>) -> Result<RecentProgramContainer, Box<dyn Error>> {
    // Order program_ids by their creation date
    records.sort_by(|a,b| a.creation_date.cmp(&b.creation_date));

    // Create clusters with empty vectors
    let mut clusters: Vec<Vec<u32>> = vec!();
    for _ in 0..NUMBER_OF_CLUSTERS {
        let program_ids: Vec<u32> = vec!();
        clusters.push(program_ids);
    }
    
    // Place program_ids evenly in each cluster
    let count: usize = records.len();
    for (index, record) in records.iter().enumerate() {
        let cluster_id = (index * ((NUMBER_OF_CLUSTERS) as usize)) / count;
        if cluster_id >= clusters.len() {
            panic!("cluster_id is out of bounds");
        }
        clusters[cluster_id].push(record.program_id);
    }

    let container = RecentProgramContainer {
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
program id;creation date
4;20190115
5;20190119

6;20210316
7;20181012
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<Record> = process_csv_data(&mut input).unwrap();
        let strings: Vec<String> = records.iter().map(|record| {
            format!("{} {}", record.program_id, record.creation_date)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "4 20190115,5 20190119,6 20210316,7 20181012");
    }

    #[test]
    fn test_10010_convert_records_to_clusters4() {
        let records: Vec<Record> = vec![
            Record::new(101, 19840101),
            Record::new(102, 19840102),
            Record::new(103, 19840103),
            Record::new(104, 19840104),
        ];
        let container: RecentProgramContainer = convert_records_to_clusters(records).unwrap();
        assert_eq!(container.cluster_lengths(), "1,0,1,0,0,1,0,1,0,0");
    }

    #[test]
    fn test_10011_convert_records_to_clusters10() {
        let records: Vec<Record> = vec![
            Record::new(101, 19840101),
            Record::new(102, 19840102),
            Record::new(103, 19840103),
            Record::new(104, 19840104),
            Record::new(105, 19840105),
            Record::new(106, 19840106),
            Record::new(107, 19840107),
            Record::new(108, 19840108),
            Record::new(109, 19840109),
            Record::new(110, 19840110),
        ];
        let container: RecentProgramContainer = convert_records_to_clusters(records).unwrap();
        assert_eq!(container.cluster_lengths(), "1,1,1,1,1,1,1,1,1,1");
        let cluster_program_ids = container.cluster_program_ids();
        {
            // cluster 9 contains the newest program_ids
            let newest_program_id: Option<&u32> = (cluster_program_ids[9]).first();
            assert_eq!(*newest_program_id.unwrap(), 110);
        }
        {
            // cluster 0 contains the oldest program_ids
            let newest_program_id: Option<&u32> = (cluster_program_ids[0]).first();
            assert_eq!(*newest_program_id.unwrap(), 101);
        }
    }

    #[test]
    fn test_10012_convert_records_to_clusters11() {
        let records: Vec<Record> = vec![
            Record::new(101, 19840101),
            Record::new(102, 19840102),
            Record::new(103, 19840103),
            Record::new(104, 19840104),
            Record::new(105, 19840105),
            Record::new(106, 19840106),
            Record::new(107, 19840107),
            Record::new(108, 19840108),
            Record::new(109, 19840109),
            Record::new(110, 19840110),
            Record::new(111, 19840111),
        ];
        let container: RecentProgramContainer = convert_records_to_clusters(records).unwrap();
        assert_eq!(container.cluster_lengths(), "2,1,1,1,1,1,1,1,1,1");
    }

    #[test]
    fn test_10013_convert_records_to_clusters19() {
        let records: Vec<Record> = vec![
            Record::new(101, 19840101),
            Record::new(102, 19840102),
            Record::new(103, 19840103),
            Record::new(104, 19840104),
            Record::new(105, 19840105),
            Record::new(106, 19840106),
            Record::new(107, 19840107),
            Record::new(108, 19840108),
            Record::new(109, 19840109),
            Record::new(110, 19840110),
            Record::new(111, 19840111),
            Record::new(112, 19840112),
            Record::new(113, 19840113),
            Record::new(114, 19840114),
            Record::new(115, 19840115),
            Record::new(116, 19840116),
            Record::new(117, 19840117),
            Record::new(118, 19840118),
            Record::new(119, 19840119),
        ];
        let container: RecentProgramContainer = convert_records_to_clusters(records).unwrap();
        assert_eq!(container.cluster_lengths(), "2,2,2,2,2,2,2,2,2,1");
    }

    #[test]
    fn test_10014_convert_records_to_clusters21() {
        let records: Vec<Record> = vec![
            Record::new(101, 19840101),
            Record::new(102, 19840102),
            Record::new(103, 19840103),
            Record::new(104, 19840104),
            Record::new(105, 19840105),
            Record::new(106, 19840106),
            Record::new(107, 19840107),
            Record::new(108, 19840108),
            Record::new(109, 19840109),
            Record::new(110, 19840110),
            Record::new(111, 19840111),
            Record::new(112, 19840112),
            Record::new(113, 19840113),
            Record::new(114, 19840114),
            Record::new(115, 19840115),
            Record::new(116, 19840116),
            Record::new(117, 19840117),
            Record::new(118, 19840118),
            Record::new(119, 19840119),
            Record::new(120, 19840120),
            Record::new(121, 19840121),
        ];
        let container: RecentProgramContainer = convert_records_to_clusters(records).unwrap();
        assert_eq!(container.cluster_lengths(), "3,2,2,2,2,2,2,2,2,2");
    }
}