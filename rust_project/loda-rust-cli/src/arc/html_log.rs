use super::{Image, ImageToHTML};
use serde::{Serialize};
use std::time::Duration;
use std::thread;
use lazy_static::lazy_static;

lazy_static! {
    static ref HTML_LOG_INNER: HtmlLogInner = HtmlLogInner::new();
}

#[derive(Debug, Serialize)]
struct PostMessage {
    message: String,
}

pub struct HtmlLogInner {
    client: reqwest::blocking::Client,
}

impl HtmlLogInner {
    fn new() -> Self {
        let client: reqwest::blocking::Client = reqwest::blocking::Client::new();
        Self {
            client
        }
    }

    fn post_event(&self, html: String) {
        let message = PostMessage {
            message: html
        };

        let result = self.client.post("http://localhost:9000/event")
            .timeout(Duration::from_secs(5))
            .json(&message)
            .send();
        let response = match result {
            Ok(v) => v,
            Err(error) => {
                error!("could not POST request. {:?}", error);
                thread::sleep(Duration::from_millis(300));
                return;
            }
        };
        match response.status() {
            reqwest::StatusCode::OK => {
                // do nothing
            },
            reqwest::StatusCode::PAYLOAD_TOO_LARGE => {
                error!("Request payload is too large! {:#?}", response);
                thread::sleep(Duration::from_millis(300));
            },
            s => {
                error!("Expected status 200, but got something else. status: {:?} {:#?}", s, response);
                thread::sleep(Duration::from_millis(300));
            }
        }
    }
}

pub struct HtmlLog;

impl HtmlLog {
    /// Example: 
    ///
    /// HtmlLog::image(&input);
    pub fn image(image: &Image) {
        let s = image.to_html();
        HTML_LOG_INNER.post_event(s);
    }    

    /// Example: 
    /// 
    /// HtmlLog::compare_images(vec![input.clone(), output.clone()]);
    pub fn compare_images(images: Vec<Image>) {
        let html_vec: Vec<String> = images.iter().map(|image| image.to_html()).collect();
        let compare_item_vec: Vec<String> = html_vec.iter().map(|html| format!("<span class=\"compare-item\">{}</span>", html)).collect();
        let compare_items: String = compare_item_vec.join("");
        let s = format!("<div class=\"compare\">{}</div>", compare_items);
        HTML_LOG_INNER.post_event(s);
    }
}
