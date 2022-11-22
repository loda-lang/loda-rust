use super::{CoordinatorWorkerMessage, UploadWorkerItem};
use crate::postmine::PostMine;
use loda_rust_core::oeis::OeisId;
use bastion::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PostmineWorkerMessage {
    StartPostmineJob,
}

pub async fn postmine_worker(
    ctx: BastionContext,
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

                        // Tell the coordinator that postmine has completed
                        let distributor = Distributor::named("coordinator_worker");
                        let tell_result = distributor.tell_everyone(CoordinatorWorkerMessage::PostmineJobComplete);
                        if let Err(error) = tell_result {
                            error!("postmine_worker: Unable to send PostmineJobComplete to coordinator_worker_distributor. error: {:?}", error);
                        }
                    }
                }
            });
    }
}
