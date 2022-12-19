use super::{Image, ImageToHTML};
use serde::{Serialize};

#[derive(Debug, Serialize)]
struct PostMessage {
    message: String,
}

pub struct HtmlLog;

impl HtmlLog {
    pub fn post_event(html: String) {
        let message = PostMessage {
            message: html
        };

        let client = reqwest::blocking::Client::new();
        let res = client.post("http://localhost:9000/event")
            .json(&message)
            .send().expect("could not POST event");
        if res.status() != 200 {
            error!("{:#?}", res);
        }
    }

    /// Example: 
    ///
    /// HtmlLog::image(&input);
    pub fn image(image: &Image) {
        let s = image.to_html();
        Self::post_event(s);
    }    

    /// Example: 
    /// 
    /// HtmlLog::compare_images(vec![input.clone(), output.clone()]);
    pub fn compare_images(images: Vec<Image>) {
        let html_vec: Vec<String> = images.iter().map(|image| image.to_html()).collect();
        let compare_item_vec: Vec<String> = html_vec.iter().map(|html| format!("<span class=\"compare-item\">{}</span>", html)).collect();
        let compare_items: String = compare_item_vec.join("");
        let s = format!("<div class=\"compare\">{}</div>", compare_items);
        Self::post_event(s);
    }
}
