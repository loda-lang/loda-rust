use std::io;
use std::io::BufRead;
use std::collections::HashSet;
use std::time::Instant;
use crate::common::SimpleLog;
use super::{parse_stripped_sequence_line, StrippedSequence};

pub fn process_stripped_sequence_file<F>(
    simple_log: SimpleLog,
    reader: &mut dyn io::BufRead, 
    filesize: usize,
    term_count: usize, 
    program_ids_to_ignore: &HashSet<u32>, 
    print_progress: bool, 
    mut callback: F
)
    where F: FnMut(&StrippedSequence)
{
    assert!(filesize >= 1);
    assert!(term_count >= 1);
    assert!(term_count <= 100);
    if print_progress {
        simple_log.println(format!("number of bytes to be processed: {}", filesize));
    }
    let mut count_callback: usize = 0;
    let mut count_junk: usize = 0;
    let mut count_tooshort: usize = 0;
    let mut count_ignore: usize = 0;
    let mut count_bytes: usize = 0;
    let mut progress_time = Instant::now();
    for line in reader.lines() {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if print_progress && elapsed >= 1000 {
            let percent: usize = (count_bytes * 100) / filesize;
            println!("progress: {}%  {} of {}", percent, count_bytes, filesize);
            progress_time = Instant::now();
        }
        let line: String = line.unwrap();
        count_bytes += line.len();
        let stripped_sequence: StrippedSequence = match parse_stripped_sequence_line(&line, Some(term_count)) {
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
        if stripped_sequence.len() != term_count {
            count_tooshort += 1;
            continue;
        }
        callback(&stripped_sequence);
        count_callback += 1;
    }
    simple_log.println(format!("count_sequences: {}", count_callback));
    simple_log.println(format!("count_ignore: {}", count_ignore));
    simple_log.println(format!("count_tooshort: {}", count_tooshort));
    simple_log.println(format!("count_junk: {}", count_junk));
}
