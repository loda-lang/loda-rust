use std::time::Instant;

pub fn subcommand_pattern() {
    let start_time = Instant::now();

    println!("pattern, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
