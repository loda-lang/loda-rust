use super::{Image, ImageToHTML};
use serde::Serialize;
use std::time::Duration;
use std::thread;
use lazy_static::lazy_static;
use html_escape;

lazy_static! {
    static ref HTML_LOG_INNER: HtmlLogInner = HtmlLogInner::new();
}

#[derive(Debug, Serialize)]
struct PostMessage {
    message: String,
}

struct HtmlLogInner {
    client: reqwest::blocking::Client,
}

impl HtmlLogInner {
    fn new() -> Self {
        // let optional_proxy_url: Option<String> = Some("http://localhost:8888".to_string());
        let optional_proxy_url: Option<String> = None;
        let mut builder = reqwest::blocking::ClientBuilder::new();
        if let Some(proxy_url) = optional_proxy_url {
            let proxy: reqwest::Proxy = reqwest::Proxy::http(proxy_url).expect("proxy");
            builder = builder.proxy(proxy);
        }
        let client: reqwest::blocking::Client = builder.build().expect("client");
        
        Self {
            client
        }
    }

    #[allow(dead_code)]
    fn post_event(&self, html: String) {
        let message = PostMessage {
            message: html
        };

        let url = "http://localhost:9000/event";

        let result = self.client.post(url)
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

#[allow(dead_code)]
pub struct HtmlLog;

impl HtmlLog {
    /// Example: 
    ///
    /// HtmlLog::text("hello world");
    #[allow(dead_code)]
    pub fn text<S: AsRef<str>>(text: S) {
        let text_str: &str = text.as_ref();
        let s = html_escape::encode_text(text_str);
        HTML_LOG_INNER.post_event(s.to_string());
    }

    /// Example: 
    ///
    /// HtmlLog::html("hello <b>world</b>");
    #[allow(dead_code)]
    pub fn html<S: AsRef<str>>(html: S) {
        let html_str: &str = html.as_ref();
        HTML_LOG_INNER.post_event(html_str.to_string());
    }

    /// Example: 
    ///
    /// HtmlLog::image(&input);
    #[allow(dead_code)]
    pub fn image(image: &Image) {
        let s = image.to_html();
        HTML_LOG_INNER.post_event(s);
    }    

    /// Example: 
    /// 
    /// HtmlLog::compare_images(vec![input.clone(), output.clone()]);
    #[allow(dead_code)]
    pub fn compare_images(images: Vec<Image>) {
        let html_vec: Vec<String> = images.iter().map(|image| image.to_html()).collect();
        let compare_item_vec: Vec<String> = html_vec.iter().map(|html| format!("<span class=\"themearc compare-item\">{}</span>", html)).collect();
        let compare_items: String = compare_item_vec.join("");
        let s = format!("<div class=\"themearc compare\">{}</div>", compare_items);
        HTML_LOG_INNER.post_event(s);
    }
}
