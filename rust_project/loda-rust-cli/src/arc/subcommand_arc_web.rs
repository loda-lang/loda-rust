use crate::common::find_json_files_recursively;
use crate::config::Config;
use super::arc_work_model::{PairType, Task};
use super::{Image, ImageSize};
use tera::Tera;
use tide::http::mime;
use tide::{Request, Response};
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "petgraph")]
use super::{ExperimentWithPetgraph, NodeData, EdgeData, PixelNeighborEdgeType};

#[cfg(feature = "petgraph")]
use petgraph::{Graph, stable_graph::NodeIndex, visit::EdgeRef};

#[derive(Clone)]
struct State {
    config: Config,
    tera: Arc<Tera>,
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

    /// The `arc-web` subcommand when invoked from the command line.
    /// 
    /// This starts a web server, where a human can explore the ARC data.
    pub async fn run_web_server() -> anyhow::Result<()> {
        let instance = Self::new()?;
        instance.run_web_server_inner().await?;
        Ok(())
    }

    async fn run_web_server_inner(&self) -> anyhow::Result<()> {
        println!("Starting the web server...");
        let e = env!("CARGO_MANIFEST_DIR");
        let dir_static: String = format!("{}/web/static/", e);

        let tera_arc: Arc<Tera>;
        {
            let dir: String = format!("{}/web/templates/arc/**/*.html", e);
            let mut tera = match Tera::new(&dir) {
                Ok(t) => t,
                Err(e) => {
                    println!("Parsing error(s): {}", e);
                    ::std::process::exit(1);
                }
            };
            tera.autoescape_on(vec![".html"]);
            tera_arc = Arc::new(tera);
        }

        let mut app = tide::with_state(State {
            config: self.config.clone(),
            tera: tera_arc,
        });
        app.at("/").get(demo1);
        app.at("/task").get(Self::get_task_list);
        app.at("/task/:task_id").get(Self::get_task_with_id);

        #[cfg(feature = "petgraph")]
        app.at("/task/:task_id/graph/:node_id").get(Self::get_node);

        app.at("/static").serve_dir(&dir_static)?;
        app.listen("127.0.0.1:8090").await?;

        Ok(())
    }

    async fn get_task_list(req: Request<State>) -> tide::Result {
        let config: &Config = &req.state().config;
        let tera: &Tera = &req.state().tera;
    
        let repo_path: PathBuf = config.arc_repository_data();
        let all_json_paths: Vec<PathBuf> = find_json_files_recursively(&repo_path);
        debug!("all_json_paths: {:?}", all_json_paths.len());

        let mut task_list = String::new();
        for path in &all_json_paths {
            let task_name: String = match path.file_stem() {
                Some(value) => String::from(value.to_string_lossy()),
                None => continue,
            };
            task_list.push_str(&format!("<li><a href=\"/task/{}\">{}</a></li>\n", task_name, task_name));
        }

        let mut context = tera::Context::new();
        context.insert("task_list", &task_list);
        let html: String = tera.render("page_task_list.html", &context).unwrap();
    
        let response = Response::builder(200)
            .body(html)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

    async fn get_task_with_id(req: Request<State>) -> tide::Result {
        let config: &Config = &req.state().config;
        let tera: &Tera = &req.state().tera;
        let task_id: &str = req.param("task_id").unwrap_or("world");
        let find_filename: String = format!("{}.json", task_id);
    
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

        let inspect_html: String = match task.inspect_to_html() {
            Ok(value) => value,
            Err(_error) => {
                let response = tide::Response::builder(500)
                    .body("unable to inspect the task.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        let mut context = tera::Context::new();
        context.insert("inspect_html", &inspect_html);
        context.insert("task_id", task_id);
        let html: String = tera.render("page_inspect_task.html", &context).unwrap();
    
        let response = Response::builder(200)
            .body(html)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

    #[cfg(feature = "petgraph")]
    async fn get_node(req: Request<State>) -> tide::Result {
        let config: &Config = &req.state().config;
        let tera: &Tera = &req.state().tera;
        let task_id: &str = req.param("task_id").unwrap_or("world");
        let node_id: &str = req.param("node_id").unwrap_or("world");

        let node_id_usize: usize = match node_id.parse::<usize>() {
            Ok(value) => value,
            Err(_error) => {
                let response = tide::Response::builder(400)
                    .body("invalid node_id.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        let find_filename: String = format!("{}.json", task_id);
    
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

        let mut image = Image::empty();
        for pair in &task.pairs {
            image = pair.input.image.clone();
            break;
        }
        let mut instance = ExperimentWithPetgraph::new();

        let image_node_index = match instance.add_image(&image) {
            Ok(value) => value,
            Err(error) => {
                debug!("error: {:?}", error);
                let response = tide::Response::builder(500)
                    .body("unable to populate graph.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };
        println!("image_node_index: {:?}", image_node_index);

        let graph: &Graph<NodeData, EdgeData> = instance.graph();

        let node_index = NodeIndex::new(node_id_usize);

        let pixel_node: &NodeData = &graph[node_index];
        println!("node: {:?}", pixel_node);

        match pixel_node {
            NodeData::Pixel => {},
            _ => {
                let response = tide::Response::builder(500)
                    .body("The node is not a pixel")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        }

        let mut node_index_up: Option<NodeIndex> = None;
        let mut node_index_down: Option<NodeIndex> = None;
        let mut node_index_left: Option<NodeIndex> = None;
        let mut node_index_right: Option<NodeIndex> = None;
        let mut node_index_upleft: Option<NodeIndex> = None;
        let mut node_index_upright: Option<NodeIndex> = None;
        let mut node_index_downleft: Option<NodeIndex> = None;
        let mut node_index_downright: Option<NodeIndex> = None;
        for edge_pixel in graph.edges(node_index) {
            let child_index: NodeIndex = edge_pixel.target();

            match edge_pixel.weight() {
                EdgeData::PixelNeighbor { edge_type } => {
                    match edge_type {
                        PixelNeighborEdgeType::Up => { 
                            node_index_up = Some(child_index); 
                        },
                        PixelNeighborEdgeType::Down => { 
                            node_index_down = Some(child_index); 
                        },
                        PixelNeighborEdgeType::Left => { 
                            node_index_left = Some(child_index); 
                        },
                        PixelNeighborEdgeType::Right => { 
                            node_index_right = Some(child_index); 
                        },
                        PixelNeighborEdgeType::UpLeft => { 
                            node_index_upleft = Some(child_index); 
                        },
                        PixelNeighborEdgeType::UpRight => { 
                            node_index_upright = Some(child_index);
                        },
                        PixelNeighborEdgeType::DownLeft => { 
                            node_index_downleft = Some(child_index); 
                        },
                        PixelNeighborEdgeType::DownRight => { 
                            node_index_downright = Some(child_index); 
                        },
                        _ => {}
                    }
                },
                _ => {},
            }
        }

        let mut center_wrap_pixel = WrapPixel::default();
        center_wrap_pixel.task_id = Some(task_id.to_string());
        center_wrap_pixel.node_index = Some(node_index);
        center_wrap_pixel.load(&graph);
        let pixel_center: String = tera.render("wrap_pixel.html", &center_wrap_pixel.to_context()).unwrap();
    
        let mut up_wrap_pixel = WrapPixel::default();
        up_wrap_pixel.task_id = Some(task_id.to_string());
        up_wrap_pixel.node_index = node_index_up;
        up_wrap_pixel.load(&graph);
        let pixel_up: String = tera.render("wrap_pixel.html", &up_wrap_pixel.to_context()).unwrap();

        let mut down_wrap_pixel = WrapPixel::default();
        down_wrap_pixel.task_id = Some(task_id.to_string());
        down_wrap_pixel.node_index = node_index_down;
        down_wrap_pixel.load(&graph);
        let pixel_down: String = tera.render("wrap_pixel.html", &down_wrap_pixel.to_context()).unwrap();

        let mut left_wrap_pixel = WrapPixel::default();
        left_wrap_pixel.task_id = Some(task_id.to_string());
        left_wrap_pixel.node_index = node_index_left;
        left_wrap_pixel.load(&graph);
        let pixel_left: String = tera.render("wrap_pixel.html", &left_wrap_pixel.to_context()).unwrap();

        let mut right_wrap_pixel = WrapPixel::default();
        right_wrap_pixel.task_id = Some(task_id.to_string());
        right_wrap_pixel.node_index = node_index_right;
        right_wrap_pixel.load(&graph);
        let pixel_right: String = tera.render("wrap_pixel.html", &right_wrap_pixel.to_context()).unwrap();

        let mut upleft_wrap_pixel = WrapPixel::default();
        upleft_wrap_pixel.task_id = Some(task_id.to_string());
        upleft_wrap_pixel.node_index = node_index_upleft;
        upleft_wrap_pixel.load(&graph);
        let pixel_upleft: String = tera.render("wrap_pixel.html", &upleft_wrap_pixel.to_context()).unwrap();

        let mut upright_wrap_pixel = WrapPixel::default();
        upright_wrap_pixel.task_id = Some(task_id.to_string());
        upright_wrap_pixel.node_index = node_index_upright;
        upright_wrap_pixel.load(&graph);
        let pixel_upright: String = tera.render("wrap_pixel.html", &upright_wrap_pixel.to_context()).unwrap();

        let mut downleft_wrap_pixel = WrapPixel::default();
        downleft_wrap_pixel.task_id = Some(task_id.to_string());
        downleft_wrap_pixel.node_index = node_index_downleft;
        downleft_wrap_pixel.load(&graph);
        let pixel_downleft: String = tera.render("wrap_pixel.html", &downleft_wrap_pixel.to_context()).unwrap();

        let mut downright_wrap_pixel = WrapPixel::default();
        downright_wrap_pixel.task_id = Some(task_id.to_string());
        downright_wrap_pixel.node_index = node_index_downright;
        downright_wrap_pixel.load(&graph);
        let pixel_downright: String = tera.render("wrap_pixel.html", &downright_wrap_pixel.to_context()).unwrap();

        let mut context_edge_horizontal = tera::Context::new();
        context_edge_horizontal.insert("key", "value");
        let edge_horizontal: String = tera.render("edge_horizontal.html", &context_edge_horizontal).unwrap();
    
        let mut context_edge_vertical = tera::Context::new();
        context_edge_vertical.insert("key", "value");
        let edge_vertical: String = tera.render("edge_vertical.html", &context_edge_vertical).unwrap();
    
        let mut context_edge_diagonal_a = tera::Context::new();
        context_edge_diagonal_a.insert("key", "value");
        let edge_diagonal_a: String = tera.render("edge_diagonal_a.html", &context_edge_diagonal_a).unwrap();
    
        let mut context_edge_diagonal_b = tera::Context::new();
        context_edge_diagonal_b.insert("key", "value");
        let edge_diagonal_b: String = tera.render("edge_diagonal_b.html", &context_edge_diagonal_b).unwrap();
    
        let mut context = tera::Context::new();
        context.insert("pixel_center", &pixel_center);
        context.insert("pixel_up", &pixel_up);
        context.insert("pixel_down", &pixel_down);
        context.insert("pixel_left", &pixel_left);
        context.insert("pixel_right", &pixel_right);
        context.insert("pixel_upleft", &pixel_upleft);
        context.insert("pixel_upright", &pixel_upright);
        context.insert("pixel_downleft", &pixel_downleft);
        context.insert("pixel_downright", &pixel_downright);
        context.insert("edge_left_center", &edge_horizontal);
        context.insert("edge_center_right", &edge_horizontal);
        context.insert("edge_center_up", &edge_vertical);
        context.insert("edge_center_down", &edge_vertical);
        context.insert("edge_center_upleft", &edge_diagonal_a);
        context.insert("edge_center_upright", &edge_diagonal_b);
        context.insert("edge_center_downleft", &edge_diagonal_b);
        context.insert("edge_center_downright", &edge_diagonal_a);
    
        let pretty_pixel: String = tera.render("inspect_pixel.html", &context).unwrap();
    
        let mut context2 = tera::Context::new();
        context2.insert("left_side", &pretty_pixel);
        context2.insert("right_side", "hi");
    
        let body: String = tera.render("side_by_side.html", &context2).unwrap();
        
        let response = Response::builder(200)
            .body(body)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

}

#[cfg(feature = "petgraph")]
#[derive(Clone, Debug, Default)]
struct WrapPixel {
    color: Option<u8>,
    x: Option<u8>,
    y: Option<u8>,
    task_id: Option<String>,
    node_index: Option<NodeIndex>,
}

impl WrapPixel {
    fn load(&mut self, graph: &Graph<NodeData, EdgeData>) {
        let node_index: NodeIndex = match self.node_index {
            Some(node_index) => node_index.clone(),
            None => return,
        };
        for edge_pixel in graph.edges(node_index) {
            let child_index: NodeIndex = edge_pixel.target();
            let child_node: NodeData = graph[child_index];
            match child_node {
                NodeData::PositionX { x } => { self.x = Some(x); },
                NodeData::PositionY { y } => { self.y = Some(y); },
                NodeData::Color { color } => { self.color = Some(color); },
                _ => {}
            }
        }
    }

    fn to_context(&self) -> tera::Context {
        let color: String;
        if let Some(value) = self.color {
            color = format!("{}", value);
        } else {
            color = "missing".to_string();
        }

        let href: String;
        match (&self.task_id, &self.node_index) {
            (Some(task_id), Some(node_index)) => {
                href = format!("/task/{}/graph/{}", task_id, node_index.index());
            },
            _ => {
                href = "#".to_string();
            }
        }

        let mut context = tera::Context::new();
        context.insert("color", &color);
        context.insert("href", &href);
        context.insert("infoid", "pixel1");
        context
    }
}

async fn demo1(req: Request<State>) -> tide::Result {
    println!("demo1");
    let tera: &Tera = &req.state().tera;

    let mut context_pixel_center = tera::Context::new();
    context_pixel_center.insert("color", "2");
    context_pixel_center.insert("href", "#");
    let pixel_center: String = tera.render("wrap_pixel.html", &context_pixel_center).unwrap();

    let mut context_pixel_mock1 = tera::Context::new();
    context_pixel_mock1.insert("color", "3");
    context_pixel_mock1.insert("href", "/task/662c240a/graph/5");
    let pixel_mock1: String = tera.render("wrap_pixel.html", &context_pixel_mock1).unwrap();

    let mut context_pixel_mock2 = tera::Context::new();
    context_pixel_mock2.insert("color", "4");
    context_pixel_mock2.insert("href", "/task/662c240a/graph/5");
    let pixel_mock2: String = tera.render("wrap_pixel.html", &context_pixel_mock2).unwrap();

    let mut context_edge_horizontal = tera::Context::new();
    context_edge_horizontal.insert("key", "value");
    let edge_horizontal: String = tera.render("edge_horizontal.html", &context_edge_horizontal).unwrap();

    let mut context_edge_vertical = tera::Context::new();
    context_edge_vertical.insert("key", "value");
    let edge_vertical: String = tera.render("edge_vertical.html", &context_edge_vertical).unwrap();

    let mut context_edge_diagonal_a = tera::Context::new();
    context_edge_diagonal_a.insert("key", "value");
    let edge_diagonal_a: String = tera.render("edge_diagonal_a.html", &context_edge_diagonal_a).unwrap();

    let mut context_edge_diagonal_b = tera::Context::new();
    context_edge_diagonal_b.insert("key", "value");
    let edge_diagonal_b: String = tera.render("edge_diagonal_b.html", &context_edge_diagonal_b).unwrap();

    let mut context = tera::Context::new();
    context.insert("pixel_center", &pixel_center);
    context.insert("pixel_up", &pixel_mock1);
    context.insert("pixel_down", &pixel_mock1);
    context.insert("pixel_left", &pixel_mock1);
    context.insert("pixel_right", &pixel_mock1);
    context.insert("pixel_upleft", &pixel_mock2);
    context.insert("pixel_upright", &pixel_mock2);
    context.insert("pixel_downleft", &pixel_mock2);
    context.insert("pixel_downright", &pixel_mock2);
    context.insert("edge_left_center", &edge_horizontal);
    context.insert("edge_center_right", &edge_horizontal);
    context.insert("edge_center_up", &edge_vertical);
    context.insert("edge_center_down", &edge_vertical);
    context.insert("edge_center_upleft", &edge_diagonal_a);
    context.insert("edge_center_upright", &edge_diagonal_b);
    context.insert("edge_center_downleft", &edge_diagonal_b);
    context.insert("edge_center_downright", &edge_diagonal_a);

    let pretty_pixel: String = tera.render("inspect_pixel.html", &context).unwrap();

    let mut context2 = tera::Context::new();
    context2.insert("left_side", &pretty_pixel);
    context2.insert("right_side", "hi");

    let body: String = tera.render("side_by_side.html", &context2).unwrap();

    let response = Response::builder(200)
        .body(body)
        .content_type(mime::HTML)
        .build();

    Ok(response)
}
