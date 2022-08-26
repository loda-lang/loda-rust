use crate::common::OeisIdStringMap;
use crate::oeis::{OeisIdHashSet, ProcessStrippedFile, StrippedRow};
use loda_rust_core::util::BigIntVecToString;
use std::error::Error;
use std::io;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};
use num_bigint::BigInt;
use num_traits::Zero;

#[allow(dead_code)]
pub fn batch_lookup_terms(
    reader: &mut dyn io::BufRead,
    filesize: usize,
    oeis_ids: &OeisIdHashSet
) -> Result<OeisIdStringMap, Box<dyn Error>> {
    let start = Instant::now();
    println!("Looking up in the OEIS 'stripped' file");

    let mut oeis_id_terms_map = OeisIdStringMap::new();
    let pb = ProgressBar::new(filesize as u64);
    let callback = |row: &StrippedRow, count_bytes: usize| {
        pb.set_position(count_bytes as u64);
        if oeis_ids.contains(&row.oeis_id()) {
            let terms: String = row.terms().to_compact_comma_string();
            // let message = format!("{}: {}", row.oeis_id().a_number(), terms);
            // pb.println(message);
            oeis_id_terms_map.insert(row.oeis_id(), terms);
        }
    };
    
    let minimum_number_of_required_terms: usize = 1;
    let term_count: usize = 100;

    let oeis_ids_to_ignore = OeisIdHashSet::new();
    let mut processor = ProcessStrippedFile::new();
    let padding_value: BigInt = BigInt::zero();
    processor.execute(
        reader, 
        minimum_number_of_required_terms,
        term_count, 
        &oeis_ids_to_ignore,
        &padding_value,
        false,
        callback
    );
    pb.finish_and_clear();

    let green_bold = Style::new().green().bold();        
    println!(
        "{:>12} Lookups in the OEIS 'stripped' file, in {}",
        green_bold.apply_to("Finished"),
        HumanDuration(start.elapsed())
    );

    Ok(oeis_id_terms_map)
}
