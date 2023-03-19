use super::arc_json_model;
use super::arc_json_model::GridToImage;
use super::ImageToHTML;

pub trait ModelToHTML {
    fn to_html(&self) -> String;
}

impl ModelToHTML for arc_json_model::Task {
    fn to_html(&self) -> String {
        fn format_grid(grid: &arc_json_model::Grid) -> String {
            match grid.to_image() {
                Ok(image) => image.to_html(),
                Err(_error) => "to_image error".to_string()
            }
        }

        let model_id: String = self.id().identifier();
        let mut s: String = format!("<div>Model: {}</div>", model_id);
        s += "<h3>Train</h3>";
        s += "<div class=\"themearc model rows\">";
        for pair in self.train() {
            let html0: String = format_grid(pair.input());
            let html1: String = format_grid(pair.output());
            let column0: String = format!("<div class=\"themearc model train input\">{}</div>", html0);
            let column1: String = format!("<div class=\"themearc model train output\">{}</div>", html1);
            let row: String = format!("<div class=\"themearc model row\">{}{}</div>", column0, column1);
            s += &row;
        }
        s += "</div>";
        s += "<h3>Test</h3>";
        s += "<div class=\"themearc model rows\">";
        for pair in self.test() {
            let html0: String = format_grid(pair.input());
            let html1: String = format_grid(pair.output());
            let column0: String = format!("<div class=\"themearc model test input\">{}</div>", html0);
            let column1: String = format!("<div class=\"themearc model test output\">{}</div>", html1);
            let row: String = format!("<div class=\"themearc model row\">{}{}</div>", column0, column1);
            s += &row;
        }
        s += "</div>";
        s
    }
}
