use csv::WriterBuilder;
use serde::Serialize;
use std::error::Error;
use std::path::Path;

pub fn create_csv_file<S: Serialize>(records: &Vec<S>, output_path: &Path) -> Result<(), Box<dyn Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;

    #[derive(Serialize)]
    struct Record {
        count: u32,
        instruction: String,
        constant: i32,
    }

    fn create_record(count: u32, instruction: &str, constant: i32) -> Record {
        Record {
            count: count,
            instruction: instruction.to_string(),
            constant: constant
        }
    }

    #[test]
    fn test_10000_create_csv_file() -> Result<(), Box<dyn Error>> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_create_csv_file");
        fs::create_dir(&basedir)?;
        let csv_path: PathBuf = basedir.join("data.csv");

        let records: Vec<Record> = vec![
            create_record(5, "mov", -3),
            create_record(17, "add", 8),
            create_record(99, "gcd", 42),
        ];

        // Act
        create_csv_file(&records, &csv_path)?;

        // Assert
        let expected_file_content = 
r#"count;instruction;constant
5;mov;-3
17;add;8
99;gcd;42
"#;
        let actual_file_content: String = fs::read_to_string(&csv_path)?;

        assert_eq!(actual_file_content, expected_file_content);
        Ok(())
    }
}
