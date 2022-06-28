use std::io;
use std::io::BufRead;
use std::collections::{HashMap, HashSet};
use crate::common::SimpleLog;
use super::{parse_stripped_sequence_line, StrippedSequence};

pub struct ProcessStrippedSequenceFile {
    count_bytes: usize,
    count_lines: usize,
    count_junk: usize,
    count_callback: usize,
    count_tooshort: usize,
    count_ignored_program_id: usize,
    count_grow_to_term_count: usize,
    histogram_length: HashMap<usize,u32>,
}

impl ProcessStrippedSequenceFile {
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
        simple_log.println(format!("Number of lines in oeis 'stripped' file: {}", self.count_lines));
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

    pub fn execute<F>(
        &mut self,
        reader: &mut dyn io::BufRead,
        minimum_number_of_required_terms: usize,
        term_count: usize, 
        program_ids_to_ignore: &HashSet<u32>, 
        mut callback: F
    )
        where F: FnMut(&StrippedSequence, usize)
    {
        assert!(term_count >= 1);
        assert!(term_count <= 100);
        for line in reader.lines() {
            let line: String = line.unwrap();
            self.count_bytes += line.len();
            self.count_lines += 1;
            let mut stripped_sequence: StrippedSequence = match parse_stripped_sequence_line(&line, Some(term_count)) {
                Some(value) => value,
                None => {
                    self.count_junk += 1;
                    continue;
                }
            };
            let number_of_terms: usize = stripped_sequence.len();
            let counter = self.histogram_length.entry(number_of_terms).or_insert(0);
            *counter += 1;
            if number_of_terms < minimum_number_of_required_terms {
                self.count_tooshort += 1;
                continue;
            }
            if program_ids_to_ignore.contains(&stripped_sequence.sequence_number) {
                self.count_ignored_program_id += 1;
                continue;
            }
            if number_of_terms < term_count {
                self.count_grow_to_term_count += 1;
                stripped_sequence.grow_to_length(term_count);
            }
            assert!(stripped_sequence.len() == term_count);
            callback(&stripped_sequence, self.count_bytes);
            self.count_callback += 1;
        }
    }
}
