use loda_rust_core::oeis::OeisIdHashSet;
use crate::common::OeisIdStringMap;
use crate::oeis::{NameRow, ProcessNamesFile};
use std::error::Error;
use std::io;
use std::time::Instant;
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

pub fn batch_lookup_names(
    reader: &mut dyn io::BufRead,
    filesize: usize,
    oeis_ids: &OeisIdHashSet
) -> Result<OeisIdStringMap, Box<dyn Error>> {
    let start = Instant::now();
    println!("Looking up in the OEIS 'names' file");

    let mut oeis_id_name_map = OeisIdStringMap::new();
    let pb = ProgressBar::new(filesize as u64);
    let callback = |row: &NameRow, count_bytes: usize| {
        pb.set_position(count_bytes as u64);
        if oeis_ids.contains(&row.oeis_id()) {
            // let message = format!("{}: {}", row.oeis_id().a_number(), row.name());
            // pb.println(message);
            oeis_id_name_map.insert(row.oeis_id(), row.name().to_string());
        }
    };
    
    let oeis_ids_to_ignore = OeisIdHashSet::new();
    let mut processor = ProcessNamesFile::new();
    processor.execute(
        reader, 
        &oeis_ids_to_ignore,
        callback
    );
    pb.finish_and_clear();

    let green_bold = Style::new().green().bold();        
    println!(
        "{:>12} Lookups in the OEIS 'names' file, in {}",
        green_bold.apply_to("Finished"),
        HumanDuration(start.elapsed())
    );

    Ok(oeis_id_name_map)
}
