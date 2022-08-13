use std::io;
use std::io::BufRead;
use std::collections::{HashMap, HashSet};
use num_bigint::BigInt;
use crate::common::SimpleLog;
use super::StrippedRow;

pub struct ProcessStrippedFile {
    count_bytes: usize,
    count_lines: usize,
    count_junk: usize,
    count_callback: usize,
    count_tooshort: usize,
    count_ignored_program_id: usize,
    count_grow_to_term_count: usize,
    histogram_length: HashMap<usize,u32>,
}

impl ProcessStrippedFile {
    pub fn new() -> Self {
        Self {
            count_bytes: 0,
            count_lines: 0,
            count_junk: 0,
            count_callback: 0,
            count_tooshort: 0,
            count_ignored_program_id: 0,
            count_grow_to_term_count: 0,
            histogram_length: HashMap::new(),
        }        
    }

    pub fn print_summary(&self, simple_log: SimpleLog) {
        simple_log.println(format!("Number of rows in oeis 'stripped' file: {}", self.count_lines));
        simple_log.println(format!("count_callback: {}", self.count_callback));
        simple_log.println(format!("count_tooshort: {}", self.count_tooshort));
        simple_log.println(format!("count_ignored_program_id: {}", self.count_ignored_program_id));
        simple_log.println(format!("count_grow_to_term_count: {}", self.count_grow_to_term_count));
        simple_log.println(format!("count_junk: {}", self.count_junk));

        let mut histogram_length_vec = Vec::<u32>::new();
        for i in 1..41 {
            match self.histogram_length.get(&i) {
                Some(length) => { histogram_length_vec.push(*length); },
                None => { histogram_length_vec.push(0); }
            }
        }
        simple_log.println(format!("sequence_lengths: {:?}", histogram_length_vec));
    }

    /// Traverse all the rows of the OEIS `stripped` file.
    pub fn execute<F>(
        &mut self,
        reader: &mut dyn io::BufRead,
        minimum_number_of_required_terms: usize,
        term_count: usize, 
        program_ids_to_ignore: &HashSet<u32>,
        padding_value: &BigInt,
        should_grow_to_length: bool, 
        mut callback: F
    )
        where F: FnMut(&StrippedRow, usize)
    {
        assert!(term_count >= 1);
        assert!(term_count <= 100);
        for line in reader.lines() {
            let line: String = line.unwrap();
            self.count_bytes += line.len();
            self.count_lines += 1;
            let mut row: StrippedRow = match StrippedRow::parse(&line, Some(term_count)) {
                Some(value) => value,
                None => {
                    self.count_junk += 1;
                    continue;
                }
            };
            let number_of_terms: usize = row.len();
            let counter = self.histogram_length.entry(number_of_terms).or_insert(0);
            *counter += 1;
            if number_of_terms < minimum_number_of_required_terms {
                self.count_tooshort += 1;
                continue;
            }
            if program_ids_to_ignore.contains(&row.sequence_number) {
                self.count_ignored_program_id += 1;
                continue;
            }
            if should_grow_to_length {
                if number_of_terms < term_count {
                    self.count_grow_to_term_count += 1;
                    row.grow_to_length(term_count, padding_value);
                }
                assert!(row.len() == term_count);
            }
            callback(&row, self.count_bytes);
            self.count_callback += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;
    
    const INPUT_MOCKDATA: &str = r#"
# OEIS Sequence Data (http://oeis.org/stripped.gz)
# Last Modified: January 32 01:01 UTC 1984
# Use of this content is governed by the
# OEIS End-User License: http://oeis.org/LICENSE
A000040 ,2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,101,103,107,109,113,127,131,137,139,149,151,157,163,167,173,179,181,191,193,197,199,211,223,227,229,233,239,241,251,257,263,269,271,
A000045 ,0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,1597,2584,4181,6765,10946,17711,28657,46368,75025,121393,196418,317811,514229,832040,1346269,2178309,3524578,5702887,9227465,14930352,24157817,39088169,63245986,102334155,
A000231 ,3,7,46,4336,134281216,288230380379570176,2658455991569831764110243006194384896,452312848583266388373324160190187140390789016525312000869601987902398529536,
A000241 ,0,0,0,0,0,1,3,9,18,36,60,100,150,
A000615 ,2,2,8,72,1536,86080,14487040,8274797440,17494930604032,
A000633 ,1,1,3,7,18,42,109,
A112088 ,2,3,5,7,11,16,24,36,54,81,122,183,274,411,617,925,1388,2082,3123,4684,7026,10539,15809,23713,35570,53355,80032,120048,180072,270108,405162,607743,911615,1367422,2051133,3076700,4615050,6922575,10383862,
A117093 ,2,3,5,7,11,13,16,17,18,19,23,28,29,30,31,37,38,39,40,41,43,47,53,58,59,61,67,71,72,73,78,79,81,82,83,88,89,95,96,97,98,99,100,
"#;

    #[test]
    fn test_10000_execute() {
        // Arrange
        let mut input: &[u8] = INPUT_MOCKDATA.as_bytes();

        let mut callback_items = Vec::<String>::new();
        let callback = |row: &StrippedRow, _| {
            callback_items.push(format!("{}", row.sequence_number));
        };
        let minimum_number_of_required_terms = 8;
        let term_count = 20;
        let mut program_ids_to_ignore = HashSet::<u32>::new();
        program_ids_to_ignore.insert(45);
        program_ids_to_ignore.insert(112088);

        let padding_value = BigInt::zero();
        let mut processor = ProcessStrippedFile::new();

        // Act
        processor.execute(
            &mut input,
            minimum_number_of_required_terms,
            term_count,
            &program_ids_to_ignore,
            &padding_value,
            true, 
            callback
        );

        // Assert
        let callback_dump = callback_items.join(",");
        assert_eq!(callback_dump, "40,231,241,615,117093");
        assert_eq!(processor.count_ignored_program_id, 2);
        assert_eq!(processor.count_junk, 5);
        assert_eq!(processor.count_grow_to_term_count, 3);
        assert_eq!(processor.count_tooshort, 1);
    }
}
