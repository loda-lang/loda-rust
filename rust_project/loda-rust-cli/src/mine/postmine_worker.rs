use super::{MineEventDirectoryState, SharedWorkerState, UploadWorkerItem};
use crate::postmine::PostMine;
use loda_rust_core::oeis::OeisId;
use bastion::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PostmineWorkerMessage {
    StartPostmineJob,
}

pub async fn postmine_worker(
    ctx: BastionContext,
    mine_event_dir_state: Arc<Mutex<MineEventDirectoryState>>,
    shared_worker_state: Arc<Mutex<SharedWorkerState>>,
) -> Result<(), ()> {
    loop {
        MessageHandler::new(ctx.recv().await?)
            .on_tell(|message: PostmineWorkerMessage, _| {
                println!(
                    "postmine_worker: child {}, received broadcast message: {:?}",
                    ctx.current().id(),
                    message
                );
                match message {
                    PostmineWorkerMessage::StartPostmineJob => {
                        println!("BEFORE PostMine::run()");
                        let mut postmine: PostMine = match PostMine::new() {
                            Ok(value) => value,
                            Err(error) => {
                                error!("Could not create PostMine instance. error: {:?}", error);
                                return;
                            }
                        };
                        let callback = move |file_content: String, oeis_id: OeisId| {
                            let distributor = Distributor::named("upload_worker");
                            let upload_worker_item = UploadWorkerItem { 
                                file_content: file_content,
                                oeis_id: oeis_id,
                            };
                            let tell_result = distributor
                                .tell_everyone(upload_worker_item);
                            if let Err(error) = tell_result {
                                error!("postmine_worker: Unable to send UploadWorkerItem. oeis_id: {} error: {:?}", oeis_id, error);
                            }
                        }; 
                        postmine.set_found_program_callback(callback);
                        let result = postmine.run_inner();
                        println!("AFTER PostMine::run()");
                        match result {
                            Ok(()) => {
                                println!("postmine Ok");
                            },
                            Err(error) => {
                                error!("postmine error: {:?}", error);
                            }
                        }

                        // update/reset mine_event_dir_state, so counters are 0.
                        match mine_event_dir_state.lock() {
                            Ok(mut state) => {
                                state.reset();
                            },
                            Err(error) => {
                                error!("postmine_worker: mine_event_dir_state.lock() failed. {:?}", error);
                            }
                        }

                        // Resume the miner_workers
                        println!("trigger resume mining");
                        thread::sleep(Duration::from_millis(1000));
                        match shared_worker_state.lock() {
                            Ok(mut state) => {
                                *state = SharedWorkerState::Mining;
                            },
                            Err(error) => {
                                error!("postmine_worker: Unable to change state=Mining. error: {:?}", error);
                            }
                        }
                    }
                }
            });
    }
}
