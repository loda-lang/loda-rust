use super::{AnalyticsDirectory, BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};
use crate::common::create_csv_file;
use chrono::{DateTime, Utc, Datelike};
use std::path::PathBuf;
use std::error::Error;
use std::time::SystemTime;
use serde::Serialize;
use std::fs::{self, Metadata};

/// This analyzer determines when was the program files last modified and 
/// generates a `program_modified.csv` with this format:
/// 
/// ```
/// program id;modified
/// 4;20190115
/// 5;20190119
/// 6;20210316
/// 7;20181012
/// 8;20210118
/// 10;20210225
/// 12;20190115
/// ```
pub struct AnalyzeProgramModified {
    analytics_directory: AnalyticsDirectory,
    record_vec: Vec<Record>,
    error_count_metadata_without_modified: usize,
}

impl AnalyzeProgramModified {
    pub fn new(analytics_directory: AnalyticsDirectory) -> Self {
        Self {
            analytics_directory,
            record_vec: vec!(),
            error_count_metadata_without_modified: 0,
        }
    }

    fn append_record(&mut self, program_id: u32, modified: String) {
        let record = Record {
            program_id,
            modified,
        };
        self.record_vec.push(record);
    }

    /// Format timestamp as "20001231" or "19840101"
    fn format_timestamp(datetime: DateTime<Utc>) -> String {
        let year: u32 = i32::max(datetime.year(), 0) as u32;
        let month: u32 = datetime.month();
        let day: u32 = datetime.day();
        format!("{year}{month:02}{day:02}")
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeProgramModified {
    fn plugin_name(&self) -> &'static str {
        "AnalyzeProgramModified"
    }

    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> Result<(), Box<dyn Error>> {
        let metadata: Metadata = fs::metadata(&context.program_path)?;
        let time: SystemTime = match metadata.modified() {
            Ok(value) => value,
            Err(error) => {
                if self.error_count_metadata_without_modified < 5 {
                    error!("AnalyzeProgramModified: Not supported on this platform. {:?}", error);
                }
                self.error_count_metadata_without_modified += 1;
                return Ok(());
            }
        };
        let datetime: DateTime<Utc> = DateTime::<Utc>::from(time);
        let s = Self::format_timestamp(datetime);
        self.append_record(context.program_id, s);
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut records: Vec<Record> = self.record_vec.clone();
        records.sort_unstable_by_key(|item| (item.program_id));

        // Save as a CSV file
        let output_path: PathBuf = self.analytics_directory.program_modified_file();
        create_csv_file(&records, &output_path)?;
        Ok(())
    }

    fn human_readable_summary(&self) -> String {
        let mut s = format!("timestamps: {}", self.record_vec.len());
        if self.error_count_metadata_without_modified > 0 {
            s += &format!("\nNumber of times modified() could not be obtained: {}", self.error_count_metadata_without_modified);
        }
        s
    }
}


#[derive(Clone, Serialize)]
struct Record {
    #[serde(rename = "program id")]
    program_id: u32,
    modified: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_10000_obtain_timestamps() {
        let dt: DateTime<Utc> = Utc.ymd(1999, 3, 9).and_hms_micro(21, 59, 33, 453_829);
        let s = AnalyzeProgramModified::format_timestamp(dt);
        assert_eq!(s, "19990309");
    }
}
