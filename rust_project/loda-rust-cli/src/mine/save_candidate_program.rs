use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use chrono::{DateTime, Utc};

pub fn save_candidate_program(
    mine_event_dir: &Path,
    iteration: usize,
    content: &String,
) -> std::io::Result<()> 
{
    // Format filename as "19841231-235959-1234.asm"
    let now: DateTime<Utc> = Utc::now();
    let filename: String = format!("{}-{}.asm", now.format("%Y%m%d-%H%M%S"), iteration);

    // Write the file to the output dir
    let path = mine_event_dir.join(Path::new(&filename));
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;

    println!("candidate: {:?}", filename);
    Ok(())
}
