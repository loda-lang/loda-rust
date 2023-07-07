use crate::common::find_json_files_recursively;
use crate::config::Config;
use super::arc_work_model::{PairType, Task};
use super::{Image, ImageSize, Histogram};
use super::{ShapeIdentification, ShapeTransformation, ShapeType};
use super::{SingleColorObject, PixelConnectivity, ImageHistogram, ImageMask};
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
        let mut all_json_paths: Vec<PathBuf> = find_json_files_recursively(&repo_path);
        debug!("all_json_paths: {:?}", all_json_paths.len());

        alphanumeric_sort::sort_path_slice(&mut all_json_paths);

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
        context.insert("tasklist_href", "/task");
        context.insert("graph_href", &format!("/task/{}/graph/1", task_id));
        let html: String = tera.render("page_inspect_task.html", &context).unwrap();
    
        let response = Response::builder(200)
            .body(html)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

    fn inspect_shapes(name: &str, sco: &Option<SingleColorObject>) {
        let sco: &SingleColorObject = match sco {
            Some(value) => value,
            None => {
                println!("{}: no sco", name);
                return;
            }
        };
        for color in 0..=9 {
            let enumerated_objects: Image = match sco.enumerate_clusters(color, PixelConnectivity::Connectivity4) {
                Ok(value) => value,
                Err(_error) => {
                    // println!("error: {:?}", error);
                    continue;
                }
            };
            let histogram: Histogram = enumerated_objects.histogram_all();
            for (count, object_id) in histogram.pairs_ordered_by_color() {
                if count == 0 || object_id == 0 {
                    continue;
                }
                let mask: Image = enumerated_objects.to_mask_where_color_is(object_id);
                let shape_id: ShapeIdentification = match ShapeIdentification::compute(&mask) {
                    Ok(value) => value,
                    Err(error) => {
                        println!("unable to find shape. error: {:?}", error);
                        continue;
                    }
                };
                println!("{} {}, {}, {}  shape: {}  rect: {:?}", name, count, color, object_id, shape_id, shape_id.rect);
            }
        }
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

        let mut image_input = Image::empty();
        let mut image_output = Image::empty();
        let mut sco_input: Option<SingleColorObject> = None;
        let mut sco_output: Option<SingleColorObject> = None;
        for pair in &task.pairs {
            image_input = pair.input.image.clone();
            image_output = pair.output.image.clone();
            sco_input = pair.input.image_meta.single_color_object.clone();
            sco_output = pair.output.image_meta.single_color_object.clone();
            break;
        }

        Self::inspect_shapes("input", &sco_input);
        Self::inspect_shapes("output", &sco_output);

        let mut instance = ExperimentWithPetgraph::new();

        let image_input_node_index = match instance.add_image(&image_input) {
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
        println!("image_input_node_index: {:?}", image_input_node_index);

        let image_output_node_index = match instance.add_image(&image_output) {
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
        println!("image_output_node_index: {:?}", image_output_node_index);

        let graph: &Graph<NodeData, EdgeData> = instance.graph();

        let node_index = NodeIndex::new(node_id_usize);
        if node_id_usize >= graph.node_count() {
            let response = tide::Response::builder(500)
                .body("node_index out of bounds. The graph doesn't have that many nodes.")
                .content_type("text/plain; charset=utf-8")
                .build();
            return Ok(response);
        }

        let node: &NodeData = &graph[node_index];
        println!("node: {:?}", node);

        match node {
            NodeData::Pixel => {
                return Self::page_graph_pixel(graph, node_index, task_id, tera);
            },
            _ => {
                let mut s = String::new();
                s += &format!("{:?}", node);
                s += "\n";
                s += "\n";

                let count_outgoing: usize = graph.edges_directed(node_index, petgraph::Outgoing).count();
                s += &format!("count_outgoing: {:?}", count_outgoing);
                s += "\n";
    
                let count_incoming: usize = graph.edges_directed(node_index, petgraph::Incoming).count();
                s += &format!("count_incoming: {:?}", count_incoming);
                s += "\n";
    
                let mut context2 = tera::Context::new();
                context2.insert("inspect_data", &s);
                context2.insert("task_id", &task_id);
                context2.insert("task_href", &format!("/task/{}", task_id));
                let body: String = tera.render("page_graph_node.html", &context2).unwrap();
                
                let response = Response::builder(200)
                    .body(body)
                    .content_type(mime::HTML)
                    .build();
                return Ok(response);
            }
        }
    }

    fn page_graph_pixel(graph: &Graph<NodeData, EdgeData>, node_index: NodeIndex, task_id: &str, tera: &Tera) -> tide::Result {
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
        context_edge_horizontal.insert("htmlcharacter", "&#x22EF;");
        context_edge_horizontal.insert("infoid", "edgexyz");
        let edge_horizontal: String = tera.render("wrap_edge.html", &context_edge_horizontal).unwrap();
    
        let mut context_edge_vertical = tera::Context::new();
        context_edge_vertical.insert("htmlcharacter", "&#x22EE;");
        context_edge_vertical.insert("infoid", "edgexyz");
        let edge_vertical: String = tera.render("wrap_edge.html", &context_edge_vertical).unwrap();
    
        let mut context_edge_diagonal_a = tera::Context::new();
        context_edge_diagonal_a.insert("htmlcharacter", "&#x22F1;");
        context_edge_diagonal_a.insert("infoid", "edgexyz");
        let edge_diagonal_a: String = tera.render("wrap_edge.html", &context_edge_diagonal_a).unwrap();
    
        let mut context_edge_diagonal_b = tera::Context::new();
        context_edge_diagonal_b.insert("htmlcharacter", "&#x22F0;");
        context_edge_diagonal_b.insert("infoid", "edgexyz");
        let edge_diagonal_b: String = tera.render("wrap_edge.html", &context_edge_diagonal_b).unwrap();
    
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

        let info_pixel_center: String = tera.render("info_pixel.html", &center_wrap_pixel.to_info_context()).unwrap();
        let info_pixel_up: String = tera.render("info_pixel.html", &up_wrap_pixel.to_info_context()).unwrap();
        let info_pixel_down: String = tera.render("info_pixel.html", &down_wrap_pixel.to_info_context()).unwrap();
        let info_pixel_left: String = tera.render("info_pixel.html", &left_wrap_pixel.to_info_context()).unwrap();
        let info_pixel_right: String = tera.render("info_pixel.html", &right_wrap_pixel.to_info_context()).unwrap();
        let info_pixel_upleft: String = tera.render("info_pixel.html", &upleft_wrap_pixel.to_info_context()).unwrap();
        let info_pixel_upright: String = tera.render("info_pixel.html", &upright_wrap_pixel.to_info_context()).unwrap();
        let info_pixel_downleft: String = tera.render("info_pixel.html", &downleft_wrap_pixel.to_info_context()).unwrap();
        let info_pixel_downright: String = tera.render("info_pixel.html", &downright_wrap_pixel.to_info_context()).unwrap();

        let mut context_edge1 = tera::Context::new();
        context_edge1.insert("infoid", "edgexyz");
        let info_edge1: String = tera.render("info_edge.html", &context_edge1).unwrap();

        let mut info_divs = String::new();
        info_divs += &info_pixel_center;
        info_divs += &info_pixel_up;
        info_divs += &info_pixel_down;
        info_divs += &info_pixel_left;
        info_divs += &info_pixel_right;
        info_divs += &info_pixel_upleft;
        info_divs += &info_pixel_upright;
        info_divs += &info_pixel_downleft;
        info_divs += &info_pixel_downright;
        info_divs += &info_edge1;

        let mut context2 = tera::Context::new();
        context2.insert("left_side", &pretty_pixel);
        context2.insert("right_side", &info_divs);
        context2.insert("task_id", &task_id);
        context2.insert("task_href", &format!("/task/{}", task_id));
        let body: String = tera.render("page_graph_pixel.html", &context2).unwrap();
        
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

    fn infoid(&self) -> Option<String> {
        let node_index: &NodeIndex = match &self.node_index {
            Some(value) => value,
            None => return None,
        };
        let s = format!("pixel{}", node_index.index());
        Some(s)
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
        if let Some(infoid) = self.infoid() {
            context.insert("infoid", &infoid);
        }
        context
    }

    fn to_info_context(&self) -> tera::Context {
        let color: String;
        if let Some(value) = self.color {
            color = format!("{}", value);
        } else {
            color = "missing".to_string();
        }

        let x: String;
        if let Some(value) = self.x {
            x = format!("{}", value);
        } else {
            x = "missing".to_string();
        }

        let y: String;
        if let Some(value) = self.y {
            y = format!("{}", value);
        } else {
            y = "missing".to_string();
        }

        let mut context = tera::Context::new();
        context.insert("color", &color);
        context.insert("x", &x);
        context.insert("y", &y);
        if let Some(infoid) = self.infoid() {
            context.insert("infoid", &infoid);
        }
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
    context_edge_horizontal.insert("htmlcharacter", "&#x22EF;");
    context_edge_horizontal.insert("infoid", "edgexyz");
    let edge_horizontal: String = tera.render("wrap_edge.html", &context_edge_horizontal).unwrap();

    let mut context_edge_vertical = tera::Context::new();
    context_edge_vertical.insert("htmlcharacter", "&#x22EE;");
    context_edge_vertical.insert("infoid", "edgexyz");
    let edge_vertical: String = tera.render("wrap_edge.html", &context_edge_vertical).unwrap();

    let mut context_edge_diagonal_a = tera::Context::new();
    context_edge_diagonal_a.insert("htmlcharacter", "&#x22F1;");
    context_edge_diagonal_a.insert("infoid", "edgexyz");
    let edge_diagonal_a: String = tera.render("wrap_edge.html", &context_edge_diagonal_a).unwrap();

    let mut context_edge_diagonal_b = tera::Context::new();
    context_edge_diagonal_b.insert("htmlcharacter", "&#x22F0;");
    context_edge_diagonal_b.insert("infoid", "edgexyz");
    let edge_diagonal_b: String = tera.render("wrap_edge.html", &context_edge_diagonal_b).unwrap();

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
    context2.insert("task_id", "demo1");
    context2.insert("task_href", "#");
    let body: String = tera.render("page_graph_pixel.html", &context2).unwrap();

    let response = Response::builder(200)
        .body(body)
        .content_type(mime::HTML)
        .build();

    Ok(response)
}
