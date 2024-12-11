use loda_rust_core::oeis::OeisId;
use bastion::prelude::*;

const UPLOAD_MINER_PROFILE_LODA_RUST: &'static str = "\n; Miner Profile: loda-rust\n";

#[derive(Clone, Debug)]
pub struct UploadWorkerItem {
    pub file_content: String,
    pub oeis_id: OeisId,
}

pub async fn upload_worker(ctx: BastionContext, upload_endpoint: String) -> Result<(), ()> {
    debug!("upload_worker is ready");
    loop {
        let mut upload_worker_item: Option<UploadWorkerItem> = None;
        MessageHandler::new(ctx.recv().await?)
            .on_tell(|item: UploadWorkerItem, _| {
                debug!(
                    "upload_worker {}, received file for upload!:\n{:?}",
                    ctx.current().id(),
                    item.file_content
                );
                upload_worker_item = Some(item.clone());
            })
            .on_fallback(|unknown, _sender_addr| {
                error!(
                    "upload_worker {}, received an unknown message!:\n{:?}",
                    ctx.current().id(),
                    unknown
                );
            });
        if let Some(item) = upload_worker_item {
            let mut upload_content: String = item.file_content.trim_end().to_string();
            upload_content += UPLOAD_MINER_PROFILE_LODA_RUST;
            let client = reqwest::Client::new();
            let upload_result = client.post(&upload_endpoint)
                .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
                .body(upload_content.clone())
                .send()
                .await;
            match upload_result {
                Ok(res) => {
                    let upload_success: bool = res.status() == 200 || res.status() == 201;
                    if !upload_success {
                        error!("upload_worker: uploaded program {}. body: {:?}", item.oeis_id, upload_content);
                        error!("upload_worker: response: {:?} {}, expected status 2xx.", res.version(), res.status());
                        error!("upload_worker: response headers: {:#?}\n", res.headers());
                    }
                },
                Err(error) => {
                    error!("upload_worker: failed program upload of {}, error: {:?}", item.oeis_id, error);
                }
            }
        }
    }
}
