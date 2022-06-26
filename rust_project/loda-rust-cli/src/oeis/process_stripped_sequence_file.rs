use std::io;
use std::io::BufRead;
use std::collections::HashSet;
use crate::common::SimpleLog;
use super::{parse_stripped_sequence_line, StrippedSequence};

pub fn process_stripped_sequence_file<F>(
    simple_log: SimpleLog,
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
    
    let mut count_bytes: usize = 0;
    let mut count_lines: usize = 0;
    let mut count_junk: usize = 0;
    let mut count_callback: usize = 0;
    let mut count_tooshort: usize = 0;
    let mut count_ignored_program_id: usize = 0;
    let mut count_grow_to_term_count: usize = 0;
    for line in reader.lines() {
        let line: String = line.unwrap();
        count_bytes += line.len();
        count_lines += 1;
        let mut stripped_sequence: StrippedSequence = match parse_stripped_sequence_line(&line, Some(term_count)) {
            Some(value) => value,
            None => {
                count_junk += 1;
                continue;
            }
        };
        if stripped_sequence.len() < minimum_number_of_required_terms {
            count_tooshort += 1;
            continue;
        }
        if program_ids_to_ignore.contains(&stripped_sequence.sequence_number) {
            count_ignored_program_id += 1;
            continue;
        }
        if stripped_sequence.len() < term_count {
            count_grow_to_term_count += 1;
            stripped_sequence.grow_to_length(term_count);
        }
        assert!(stripped_sequence.len() == term_count);
        callback(&stripped_sequence, count_bytes);
        count_callback += 1;
    }
    simple_log.println(format!("Number of lines in oeis 'stripped' file: {}", count_lines));
    simple_log.println(format!("count_callback: {}", count_callback));
    simple_log.println(format!("count_tooshort: {}", count_tooshort));
    simple_log.println(format!("count_ignored_program_id: {}", count_ignored_program_id));
    simple_log.println(format!("count_grow_to_term_count: {}", count_grow_to_term_count));
    simple_log.println(format!("count_junk: {}", count_junk));
}
