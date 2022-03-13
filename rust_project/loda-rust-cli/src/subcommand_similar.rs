use std::time::Instant;

pub fn subcommand_similar() {
    let start_time = Instant::now();
    println!("similar begin");

    println!("similar end, elapsed: {:?} ms", start_time.elapsed().as_millis());
}
