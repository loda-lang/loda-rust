use super::{NameRow, OeisId, OeisIdHashSet};
use crate::common::SimpleLog;
use std::io;
use std::io::BufRead;
use std::collections::HashSet;

pub struct ProcessNamesFile {
    count_bytes: usize,
    count_lines: usize,
    count_junk: usize,
    count_callback: usize,
    count_ignored_program_id: usize,
}

impl ProcessNamesFile {
    pub fn new() -> Self {
        Self {
            count_bytes: 0,
            count_lines: 0,
            count_junk: 0,
            count_callback: 0,
            count_ignored_program_id: 0,
        }        
    }

    pub fn print_summary(&self, simple_log: SimpleLog) {
        simple_log.println(format!("Number of rows in oeis 'names' file: {}", self.count_lines));
        simple_log.println(format!("count_callback: {}", self.count_callback));
        simple_log.println(format!("count_ignored_program_id: {}", self.count_ignored_program_id));
        simple_log.println(format!("count_junk: {}", self.count_junk));
    }

    /// Traverse all the rows of the OEIS `names` file.
    pub fn execute<F>(
        &mut self,
        reader: &mut dyn io::BufRead,
        oeis_ids_to_ignore: &OeisIdHashSet,
        mut callback: F
    )
        where F: FnMut(&NameRow, usize)
    {
        for line in reader.lines() {
            let line: String = line.unwrap();
            self.count_bytes += line.len();
            self.count_lines += 1;
            let row: NameRow = match NameRow::parse(&line) {
                Some(value) => value,
                None => {
                    self.count_junk += 1;
                    continue;
                }
            };
            if oeis_ids_to_ignore.contains(&row.oeis_id()) {
                self.count_ignored_program_id += 1;
                continue;
            }
            callback(&row, self.count_bytes);
            self.count_callback += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const INPUT_MOCKDATA: &str = r#"
# OEIS Sequence Names (http://oeis.org/names.gz)
# Last Modified: Sol 15 00:02 IFC 1984
# Use of this content is governed by the
# OEIS End-User License: http://oeis.org/LICENSE
A000001 Number of groups of order n.
A000002 Kolakoski sequence: a(n) is length of n-th run; a(1) = 1; sequence consists just of 1's and 2's.
A000003 Number of classes of primitive positive definite binary quadratic forms of discriminant D = -4n; or equivalently the class number of the quadratic order of discriminant D = -4n.
A000004 The zero sequence.
A000005 d(n) (also called tau(n) or sigma_0(n)), the number of divisors of n.
A000006 Integer part of square root of n-th prime.
A000007 The characteristic function of {0}: a(n) = 0^n.
A000008 Number of ways of making change for n cents using coins of 1, 2, 5, 10 cents.
A000040 The prime numbers.
A000045 Fibonacci numbers: F(n) = F(n-1) + F(n-2) with F(0) = 0 and F(1) = 1.
"#;

    #[test]
    fn test_10000_execute() {
        // Arrange
        let mut input: &[u8] = INPUT_MOCKDATA.as_bytes();

        let mut callback_items = Vec::<String>::new();
        let callback = |row: &NameRow, _| {
            callback_items.push(format!("{}", row.oeis_id().raw()));
        };
        let mut oeis_ids_to_ignore = HashSet::<OeisId>::new();
        oeis_ids_to_ignore.insert(OeisId::from(5));
        oeis_ids_to_ignore.insert(OeisId::from(40));

        let mut processor = ProcessNamesFile::new();

        // Act
        processor.execute(
            &mut input,
            &oeis_ids_to_ignore,
            callback
        );

        // Assert
        let callback_dump = callback_items.join(",");
        assert_eq!(callback_dump, "1,2,3,4,6,7,8,45");
        assert_eq!(processor.count_ignored_program_id, 2);
        assert_eq!(processor.count_junk, 5);
    }
}
