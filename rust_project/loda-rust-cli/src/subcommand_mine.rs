use crate::mine::{MinerThreadMessageToCoordinator, start_miner_loop};
use std::thread;
use std::mem;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver};

extern crate num_cpus;

pub fn subcommand_mine() {
    print_info_about_start_conditions();

    let mut number_of_threads: usize = 1;

    number_of_threads = num_cpus::get();
    assert!(number_of_threads >= 1_usize);
    assert!(number_of_threads < 1000_usize);

    number_of_threads = number_of_threads / 2;
    number_of_threads = number_of_threads.max(1);

    let (sender, receiver) = channel::<MinerThreadMessageToCoordinator>();

    let builder = thread::Builder::new().name("minercoordinator".to_string());
    let join_handle: thread::JoinHandle<_> = builder.spawn(move || {
        miner_coordinator_inner(receiver);
    }).unwrap();

    for j in 0..number_of_threads {
        println!("start thread {} of {}", j, number_of_threads);
        let name = format!("miner{}", j);
        let sender_clone = sender.clone();
        let _ = thread::Builder::new().name(name).spawn(move || {
            start_miner_loop(sender_clone);
        });
        thread::sleep(Duration::from_millis(2000));
    }

    // Drop the original sender that is not being used
    mem::drop(sender);

    join_handle.join().expect("The minercoordinator thread being joined has panicked");
}

fn miner_coordinator_inner(rx: Receiver<MinerThreadMessageToCoordinator>) {
    loop {
        println!("coordinator iteration");
        loop {
            match rx.try_recv() {
                Ok(message) => {
                    println!("received message: {:?}", message);
                    continue;
                },
                Err(_) => {
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(1000));
    }
}

fn print_info_about_start_conditions() {
    let build_mode: &str;
    if cfg!(debug_assertions) {
        error!("Debugging enabled. Wasting cpu cycles. Not good for mining!");
        build_mode = "'DEBUG'  # Terrible inefficient for mining!";
    } else {
        build_mode = "'RELEASE'  # Good";
    }
    println!("[mining info]");
    println!("build_mode = {}", build_mode);
}
