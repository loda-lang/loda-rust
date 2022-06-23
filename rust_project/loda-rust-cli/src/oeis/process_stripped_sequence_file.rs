use std::io;
use std::io::BufRead;
use std::collections::HashSet;
use crate::common::SimpleLog;
use super::{parse_stripped_sequence_line, StrippedSequence};

pub fn process_stripped_sequence_file<F>(
    simple_log: SimpleLog,
    reader: &mut dyn io::BufRead, 
    term_count: usize, 
    program_ids_to_ignore: &HashSet<u32>, 
    mut callback: F
)
    where F: FnMut(&StrippedSequence, usize)
{
    assert!(term_count >= 1);
    assert!(term_count <= 100);
    
    let mut count_callback: usize = 0;
    let mut count_junk: usize = 0;
    let mut count_tooshort: usize = 0;
    let mut count_ignore: usize = 0;
    let mut count_wildcard: usize = 0;
    let mut count_bytes: usize = 0;
    for line in reader.lines() {
        let line: String = line.unwrap();
        count_bytes += line.len();
        let mut stripped_sequence: StrippedSequence = match parse_stripped_sequence_line(&line, Some(term_count)) {
            Some(value) => value,
            None => {
                count_junk += 1;
                continue;
            }
        };
        if program_ids_to_ignore.contains(&stripped_sequence.sequence_number) {
            count_ignore += 1;
            continue;
        }
        if term_count == 40 && stripped_sequence.len() >= 30 && stripped_sequence.len() < 40 {
            count_wildcard += 1;
            stripped_sequence.grow_to_length(40);
        }
        if stripped_sequence.len() != term_count {
            count_tooshort += 1;
            continue;
        }
        callback(&stripped_sequence, count_bytes);
        count_callback += 1;
    }
    simple_log.println(format!("count_sequences: {}", count_callback));
    simple_log.println(format!("count_ignore: {}", count_ignore));
    simple_log.println(format!("count_tooshort: {}", count_tooshort));
    simple_log.println(format!("count_wildcard: {}", count_wildcard));
    simple_log.println(format!("count_junk: {}", count_junk));
}
