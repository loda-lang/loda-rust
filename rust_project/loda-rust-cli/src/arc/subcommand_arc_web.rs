use std::path::PathBuf;

use crate::common::find_json_files_recursively;
use crate::config::Config;
use super::arc_work_model::{PairType, Task};
use super::ImageSize;
use lazy_static::lazy_static;
use tera::Tera;
use tide::http::mime;
use tide::{Request, Response};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let e = env!("CARGO_MANIFEST_DIR");
        let dir: String = format!("{}/web/templates/arc/**/*.html", e);
        let mut tera = match Tera::new(&dir) {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        tera
    };
}

#[derive(Clone)]
struct State {
    config: Config,
}

pub struct SubcommandARCWeb {
    config: Config,
}

impl SubcommandARCWeb {
    fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let instance = Self {
            config,
        };
        Ok(instance)
    }

    pub async fn run_web_server() -> anyhow::Result<()> {
        let instance = Self::new()?;
        instance.run_web_server_inner().await?;
        Ok(())
    }

    /// The `arc-web` subcommand when invoked from the command line.
    /// 
    /// This starts a web server, where a human can explore the ARC data.
    async fn run_web_server_inner(&self) -> anyhow::Result<()> {
        println!("Starting the web server...");
        let e = env!("CARGO_MANIFEST_DIR");
        let dir_static: String = format!("{}/web/static/", e);

        let mut app = tide::with_state(State {
            config: self.config.clone(),
        });
        app.at("/").get(demo1);
        app.at("/task/:taskname").get(Self::get_task);
        app.at("/static").serve_dir(&dir_static)?;
        app.listen("127.0.0.1:8090").await?;

        Ok(())
    }

    async fn get_task(req: Request<State>) -> tide::Result {
        let config: &Config = &req.state().config;
        let taskname: &str = req.param("taskname").unwrap_or("world");
        let find_filename: String = format!("{}.json", taskname);
    
        let repo_path: PathBuf = config.arc_repository_data();
        let all_json_paths: Vec<PathBuf> = find_json_files_recursively(&repo_path);
        debug!("all_json_paths: {:?}", all_json_paths.len());

        let found_path: Option<PathBuf> = all_json_paths
            .into_iter()
            .find(|path| {
                if let Some(filename) = path.file_name() {
                    if filename.to_string_lossy() == find_filename {
                        debug!("found the task. path: {:?}", path);
                        return true;
                    }
                }
                false
            });

        let task_json_file: PathBuf = match found_path {
            Some(value) => value,
            None => {
                let response = tide::Response::builder(404)
                    .body("cannot find the task.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };
        debug!("task_json_file: {:?}", task_json_file);

        let task: Task = match Task::load_with_json_file(&task_json_file) {
            Ok(value) => value,
            Err(_error) => {
                let response = tide::Response::builder(500)
                    .body("unable to load the task.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        let html: String = match task.inspect_to_html() {
            Ok(value) => value,
            Err(_error) => {
                let response = tide::Response::builder(500)
                    .body("unable to inspect the task.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        let response = Response::builder(200)
            .body(html)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

}

async fn demo1(mut _req: Request<State>) -> tide::Result {
    println!("demo1");

    let mut context_pixel_center = tera::Context::new();
    context_pixel_center.insert("color", "2");
    let pixel_center: String = TEMPLATES.render("wrap_pixel.html", &context_pixel_center).unwrap();

    let mut context_pixel_mock1 = tera::Context::new();
    context_pixel_mock1.insert("color", "3");
    let pixel_mock1: String = TEMPLATES.render("wrap_pixel.html", &context_pixel_mock1).unwrap();

    let mut context_pixel_mock2 = tera::Context::new();
    context_pixel_mock2.insert("color", "4");
    let pixel_mock2: String = TEMPLATES.render("wrap_pixel.html", &context_pixel_mock2).unwrap();

    let mut context_edge_horizontal = tera::Context::new();
    context_edge_horizontal.insert("key", "value");
    let edge_horizontal: String = TEMPLATES.render("edge_horizontal.html", &context_edge_horizontal).unwrap();

    let mut context_edge_vertical = tera::Context::new();
    context_edge_vertical.insert("key", "value");
    let edge_vertical: String = TEMPLATES.render("edge_vertical.html", &context_edge_vertical).unwrap();

    let mut context_edge_diagonal_a = tera::Context::new();
    context_edge_diagonal_a.insert("key", "value");
    let edge_diagonal_a: String = TEMPLATES.render("edge_diagonal_a.html", &context_edge_diagonal_a).unwrap();

    let mut context_edge_diagonal_b = tera::Context::new();
    context_edge_diagonal_b.insert("key", "value");
    let edge_diagonal_b: String = TEMPLATES.render("edge_diagonal_b.html", &context_edge_diagonal_b).unwrap();

    let mut context = tera::Context::new();
    context.insert("pixel_center", &pixel_center);
    context.insert("pixel_top", &pixel_mock1);
    context.insert("pixel_bottom", &pixel_mock1);
    context.insert("pixel_left", &pixel_mock1);
    context.insert("pixel_right", &pixel_mock1);
    context.insert("pixel_topleft", &pixel_mock2);
    context.insert("pixel_topright", &pixel_mock2);
    context.insert("pixel_bottomleft", &pixel_mock2);
    context.insert("pixel_bottomright", &pixel_mock2);
    context.insert("edge_left_center", &edge_horizontal);
    context.insert("edge_center_right", &edge_horizontal);
    context.insert("edge_center_top", &edge_vertical);
    context.insert("edge_center_bottom", &edge_vertical);
    context.insert("edge_center_topleft", &edge_diagonal_a);
    context.insert("edge_center_topright", &edge_diagonal_b);
    context.insert("edge_center_bottomleft", &edge_diagonal_b);
    context.insert("edge_center_bottomright", &edge_diagonal_a);

    let pretty_pixel: String = TEMPLATES.render("inspect_pixel.html", &context).unwrap();

    let mut context2 = tera::Context::new();
    context2.insert("left_side", &pretty_pixel);
    context2.insert("right_side", "hi");

    let body: String = TEMPLATES.render("side_by_side.html", &context2).unwrap();

    let response = Response::builder(200)
        .body(body)
        .content_type(mime::HTML)
        .build();

    Ok(response)
}
