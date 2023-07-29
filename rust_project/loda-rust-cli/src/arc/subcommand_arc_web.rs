use crate::common::find_json_files_recursively;
use crate::config::Config;
use super::image_line_spans::PromptRLEDeserializer;
use super::{Image, ImageToHTML};
use super::arc_work_model::{Task, PairType};
use super::{TaskGraph, NodeData, EdgeData, PixelNeighborEdgeType, natural_language::NaturalLanguage};
use super::prompt::PromptType;
use http_types::Url;
use serde::{Deserialize, Serialize};
use tera::Tera;
use tide::http::mime;
use tide::{Request, Response};
use tokio::sync::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use std::num::NonZeroUsize;
use cached::{SizedCache, Cached};
use petgraph::{Graph, stable_graph::NodeIndex, visit::EdgeRef};

const DEFAULT_CACHE_CAPACITY: usize = 10;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct CacheKey {
    task_name: String,
}

#[derive(Clone, Debug, Default)]
pub struct CacheValue {
    task: Option<Task>,
    task_graph: Option<TaskGraph>,
}

type Cache = SizedCache<CacheKey, CacheValue>;

#[derive(Clone)]
struct State {
    #[allow(dead_code)]
    config: Config,

    tera: Arc<Tera>,

    all_json_paths: Vec<PathBuf>,

    cache: Arc<RwLock<Cache>>,
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

        let repo_path: PathBuf = self.config.arc_repository_data();
        let all_json_paths: Vec<PathBuf> = find_json_files_recursively(&repo_path);
        debug!("all_json_paths: {:?}", all_json_paths.len());

        let cache_arc: Arc<RwLock<Cache>>;
        {
            let capacity = NonZeroUsize::new(DEFAULT_CACHE_CAPACITY).unwrap();
            let cache: Cache = SizedCache::with_size(capacity.get());
            cache_arc = Arc::new(RwLock::new(cache));
        }

        let mut app = tide::with_state(State {
            config: self.config.clone(),
            tera: tera_arc,
            all_json_paths,
            cache: cache_arc,
        });
        app.at("/task").get(Self::get_task_list);
        app.at("/task/:task_id").get(Self::get_task_with_id);
        app.at("/task/:task_id/prompt").get(Self::get_prompt);
        app.at("/task/:task_id/reply").get(Self::get_reply).post(Self::post_reply);
        app.at("/task/:task_id/find-node-pixel").get(Self::find_node_pixel);
        app.at("/task/:task_id/node/:node_id").get(Self::get_node);
        app.at("/static").serve_dir(&dir_static)?;
        app.listen("127.0.0.1:8090").await?;

        Ok(())
    }

    async fn get_task_list(req: Request<State>) -> tide::Result {
        let tera: &Tera = &req.state().tera;
        let mut all_json_paths: Vec<PathBuf> = req.state().all_json_paths.clone();    

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

    async fn load_task(req: &Request<State>, task_id: &str) -> anyhow::Result<Task> {
        let key = CacheKey {
            task_name: task_id.to_string(),
        };
        let mut write_guard = req.state().cache.write().await;
        let cache_value: &mut CacheValue = write_guard.cache_get_or_set_with(key, || CacheValue::default());

        if let Some(task) = &cache_value.task {
            debug!("cache hit. task: {:?}", task_id);
            return Ok(task.clone());
        }

        debug!("cache miss. task: {:?}", task_id);
        let find_filename: String = format!("{}.json", task_id);
        let all_json_paths: Vec<PathBuf> = req.state().all_json_paths.clone();    
        let found_path: Option<PathBuf> = all_json_paths
            .into_iter()
            .find(|path| {
                if let Some(filename) = path.file_name() {
                    if filename.to_string_lossy() == find_filename {
                        // debug!("found the task. path: {:?}", path);
                        return true;
                    }
                }
                false
            });

        let task_json_file: PathBuf = match found_path {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("cannot find the task."));
            }
        };
        // debug!("task_json_file: {:?}", task_json_file);

        let task: Task = Task::load_with_json_file(&task_json_file)?;
        cache_value.task = Some(task.clone());
        Ok(task)
    }

    async fn load_task_graph(req: &Request<State>, task_id: &str) -> anyhow::Result<TaskGraph> {
        let task: Task = Self::load_task(req, task_id).await?;

        let key = CacheKey {
            task_name: task_id.to_string(),
        };
        let mut write_guard = req.state().cache.write().await;
        let cache_value: &mut CacheValue = write_guard.cache_get_or_set_with(key, || CacheValue::default());

        if let Some(task_graph) = &cache_value.task_graph {
            debug!("cache hit. task: {:?}", task_id);
            return Ok(task_graph.clone());
        }

        debug!("cache miss. task: {:?}", task_id);

        let mut task_graph = TaskGraph::new();
        match task_graph.populate_with_task(&task) {
            Ok(()) => {},
            Err(error) => {
                return Err(anyhow::anyhow!("unable to populate graph. {:?}", error));
            }
        }

        cache_value.task_graph = Some(task_graph.clone());
        Ok(task_graph)
    }

    async fn get_task_with_id(req: Request<State>) -> tide::Result {
        let tera: &Tera = &req.state().tera;
        let task_id: &str = req.param("task_id").unwrap_or("world");
    
        let task: Task = match Self::load_task(&req, task_id).await {
            Ok(value) => value,
            Err(error) => {
                error!("cannot load the task. error: {:?}", error);
                let response = tide::Response::builder(404)
                    .body("cannot load the task.")
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
        context.insert("node_href", &format!("/task/{}/node/0", task_id));
        context.insert("prompt_href", &format!("/task/{}/prompt", task_id));
        let html: String = tera.render("page_inspect_task.html", &context).unwrap();
    
        let response = Response::builder(200)
            .body(html)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

    async fn find_node_pixel(req: Request<State>) -> tide::Result {
        let task_id: &str = req.param("task_id").unwrap_or("world");
        let query: FindNodePixel = req.query()?;
        // println!("find_node_pixel x: {}, y: {} id: {}", query.x, query.y, query.id);

        let task_graph: TaskGraph = match Self::load_task_graph(&req, task_id).await {
            Ok(value) => value,
            Err(error) => {
                error!("cannot load the task_graph. error: {:?}", error);
                let response = tide::Response::builder(404)
                    .body("cannot load the task_graph.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        // find the pixel in the graph and get its corresponding node id.
        let graph: &Graph<NodeData, EdgeData> = task_graph.graph();

        // Find the `Id` node with the `query.id`.
        let mut found_id_node_index: Option<NodeIndex> = None;
        for node_index in graph.node_indices() {
            match &graph[node_index] {
                NodeData::Id { id } => {
                    if *id != query.id {
                        continue;
                    }
                    if found_id_node_index.is_some() {
                        println!("found multiple nodes with the same id: {}", id);
                    }
                    found_id_node_index = Some(node_index);
                },
                _ => continue
            }
        }
        let id_node_index: NodeIndex = match found_id_node_index {
            Some(value) => value,
            None => {
                let response = tide::Response::builder(404)
                    .body("cannot find the node with the given id.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };
        // println!("id_node_index: {:?}", id_node_index);

        // Find the `Image` node that is a child of the `Id` node.
        let mut found_image_node_index: Option<NodeIndex> = None;
        for edge in graph.edges(id_node_index) {
            let child_index: NodeIndex = edge.target();
            if found_image_node_index.is_some() {
                println!("found multiple image nodes for the given id.");
            }
            found_image_node_index = Some(child_index);
        }
        let image_node_index: NodeIndex = match found_image_node_index {
            Some(value) => value,
            None => {
                let response = tide::Response::builder(404)
                    .body("did find the Id node, but cannot find the Image node.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };
        // println!("image_node_index: {:?}", image_node_index);

        // Find the `Pixel` node that is a child of the `Image` node.
        let mut found_pixel_node_index: Option<NodeIndex> = None;
        for edge_image in graph.edges(image_node_index) {
            let node_index: NodeIndex = edge_image.target();
            match &graph[node_index] {
                NodeData::Pixel => {},
                _ => continue
            }
            let pixel_index: NodeIndex = node_index;

            let mut found_x: Option<u8> = None;
            let mut found_y: Option<u8> = None;
            for edge_pixel in graph.edges(pixel_index) {
                let child_index: NodeIndex = edge_pixel.target();
                let child_node: &NodeData = &graph[child_index];
                match child_node {
                    NodeData::PositionX { x } => { found_x = Some(*x); },
                    NodeData::PositionY { y } => { found_y = Some(*y); },
                    _ => {}
                }
            }
            let (pixel_x, pixel_y) = match (found_x, found_y) {
                (Some(x), Some(y)) => (x, y),
                _ => continue
            };
            if pixel_x != query.x || pixel_y != query.y {
                continue;
            }
            if found_pixel_node_index.is_some() {
                println!("multiple candidates found. x: {} y: {}", query.x, query.y);
                continue;
            }
            found_pixel_node_index = Some(node_index);
        }
        let pixel_node_index: usize = match found_pixel_node_index {
            Some(value) => value.index(),
            None => {
                return Err(tide::Error::from_str(500, "Cannot find the pixel in the graph"));
            }
        };
        // println!("pixel_node_index: {:?}", pixel_node_index);

        let current_url: &Url = req.url();
        let redirect_url: Url;
        if let Some(domain) = current_url.domain() {
            let base_url = match current_url.port() {
                Some(port) => format!("{}://{}:{}", current_url.scheme(), domain, port),
                None => format!("{}://{}", current_url.scheme(), domain),
            };
            redirect_url = Url::parse(&format!("{}/task/{}/node/{}", base_url, task_id, pixel_node_index))?;
        } else {
            return Err(tide::Error::from_str(500, "URL does not have a base URL"));
        }

        let response = Response::builder(303)
            .header("Location", redirect_url.as_str())
            .build();
        Ok(response)
    }

    async fn get_node(req: Request<State>) -> tide::Result {
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

        let task_graph: TaskGraph = match Self::load_task_graph(&req, task_id).await {
            Ok(value) => value,
            Err(error) => {
                error!("cannot load the task_graph. error: {:?}", error);
                let response = tide::Response::builder(404)
                    .body("cannot load the task_graph.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        let graph: &Graph<NodeData, EdgeData> = task_graph.graph();

        let node_index = NodeIndex::new(node_id_usize);
        if node_id_usize >= graph.node_count() {
            let response = tide::Response::builder(500)
                .body("node_index out of bounds. The graph doesn't have that many nodes.")
                .content_type("text/plain; charset=utf-8")
                .build();
            return Ok(response);
        }

        match &graph[node_index] {
            NodeData::Pixel => {
                return Self::page_node_pixel(graph, node_index, task_id, tera);
            },
            _ => {
                return Self::page_node_nonpixel(graph, node_index, task_id, tera);
            }
        }
    }

    fn page_node_nonpixel(graph: &Graph<NodeData, EdgeData>, node_index: NodeIndex, task_id: &str, tera: &Tera) -> tide::Result {
        let node_href_prefix: String = format!("/task/{}/node", task_id);

        let mut wrap = WrapNonPixelNode::default();
        wrap.task_id = Some(task_id.to_string());
        wrap.node_index = node_index;
        wrap.load(&graph, &node_href_prefix);
    
        let node: &NodeData = &graph[node_index];

        let mut s = String::new();
        s += &format!("{:?}", node);
        s += "\n";
        s += "\n";

        let mut context2 = tera::Context::new();
        context2.insert("inspect_data", &s);
        context2.insert("task_id", &task_id);
        context2.insert("task_href", &format!("/task/{}", task_id));
        context2.insert("node_id", &node_index.index().to_string());
        context2.insert("outgoing_edges", &wrap.outgoing_edges);
        context2.insert("incoming_edges", &wrap.incoming_edges);
        let body: String = tera.render("page_node_nonpixel.html", &context2).unwrap();
        
        let response = Response::builder(200)
            .body(body)
            .content_type(mime::HTML)
            .build();
        Ok(response)
    }

    fn page_node_pixel(graph: &Graph<NodeData, EdgeData>, node_index: NodeIndex, task_id: &str, tera: &Tera) -> tide::Result {
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
                    }
                },
                _ => {},
            }
        }

        let node_href_prefix: String = format!("/task/{}/node", task_id);

        let mut center_wrap_pixel = WrapPixel::default();
        center_wrap_pixel.task_id = Some(task_id.to_string());
        center_wrap_pixel.node_index = Some(node_index);
        center_wrap_pixel.is_center_pixel = true;
        center_wrap_pixel.load(&graph, &node_href_prefix);
        let pixel_center: String = tera.render("wrap_pixel.html", &center_wrap_pixel.to_context()).unwrap();
    
        let mut up_wrap_pixel = WrapPixel::default();
        up_wrap_pixel.task_id = Some(task_id.to_string());
        up_wrap_pixel.node_index = node_index_up;
        up_wrap_pixel.load(&graph, &node_href_prefix);
        let pixel_up: String = tera.render("wrap_pixel.html", &up_wrap_pixel.to_context()).unwrap();

        let mut down_wrap_pixel = WrapPixel::default();
        down_wrap_pixel.task_id = Some(task_id.to_string());
        down_wrap_pixel.node_index = node_index_down;
        down_wrap_pixel.load(&graph, &node_href_prefix);
        let pixel_down: String = tera.render("wrap_pixel.html", &down_wrap_pixel.to_context()).unwrap();

        let mut left_wrap_pixel = WrapPixel::default();
        left_wrap_pixel.task_id = Some(task_id.to_string());
        left_wrap_pixel.node_index = node_index_left;
        left_wrap_pixel.load(&graph, &node_href_prefix);
        let pixel_left: String = tera.render("wrap_pixel.html", &left_wrap_pixel.to_context()).unwrap();

        let mut right_wrap_pixel = WrapPixel::default();
        right_wrap_pixel.task_id = Some(task_id.to_string());
        right_wrap_pixel.node_index = node_index_right;
        right_wrap_pixel.load(&graph, &node_href_prefix);
        let pixel_right: String = tera.render("wrap_pixel.html", &right_wrap_pixel.to_context()).unwrap();

        let mut upleft_wrap_pixel = WrapPixel::default();
        upleft_wrap_pixel.task_id = Some(task_id.to_string());
        upleft_wrap_pixel.node_index = node_index_upleft;
        upleft_wrap_pixel.load(&graph, &node_href_prefix);
        let pixel_upleft: String = tera.render("wrap_pixel.html", &upleft_wrap_pixel.to_context()).unwrap();

        let mut upright_wrap_pixel = WrapPixel::default();
        upright_wrap_pixel.task_id = Some(task_id.to_string());
        upright_wrap_pixel.node_index = node_index_upright;
        upright_wrap_pixel.load(&graph, &node_href_prefix);
        let pixel_upright: String = tera.render("wrap_pixel.html", &upright_wrap_pixel.to_context()).unwrap();

        let mut downleft_wrap_pixel = WrapPixel::default();
        downleft_wrap_pixel.task_id = Some(task_id.to_string());
        downleft_wrap_pixel.node_index = node_index_downleft;
        downleft_wrap_pixel.load(&graph, &node_href_prefix);
        let pixel_downleft: String = tera.render("wrap_pixel.html", &downleft_wrap_pixel.to_context()).unwrap();

        let mut downright_wrap_pixel = WrapPixel::default();
        downright_wrap_pixel.task_id = Some(task_id.to_string());
        downright_wrap_pixel.node_index = node_index_downright;
        downright_wrap_pixel.load(&graph, &node_href_prefix);
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
        context2.insert("node_id", &node_index.index().to_string());
        let body: String = tera.render("page_node_pixel.html", &context2).unwrap();
        
        let response = Response::builder(200)
            .body(body)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

    async fn get_prompt(req: Request<State>) -> tide::Result {
        let tera: &Tera = &req.state().tera;
        let task_id: &str = req.param("task_id").unwrap_or("world");

        let task_graph: TaskGraph = match Self::load_task_graph(&req, task_id).await {
            Ok(value) => value,
            Err(error) => {
                error!("cannot load the task_graph. error: {:?}", error);
                let response = tide::Response::builder(404)
                    .body("cannot load the task_graph.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        let prompt_type: PromptType = PromptType::RunLengthEncoding;
        // let prompt_type: PromptType = PromptType::ShapeAndTransform;
        let prompt: String = task_graph.to_prompt(prompt_type)?;

        let mut context2 = tera::Context::new();
        context2.insert("prompt_text", &prompt);
        context2.insert("task_id", &task_id);
        context2.insert("task_href", &format!("/task/{}", task_id));
        context2.insert("reply_href", &format!("/task/{}/reply", task_id));
        let body: String = tera.render("page_prompt.html", &context2).unwrap();
        
        let response = Response::builder(200)
            .body(body)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

    async fn get_reply(req: Request<State>) -> tide::Result {
        let tera: &Tera = &req.state().tera;
        let task_id: &str = req.param("task_id").unwrap_or("world");

        let _task_graph: TaskGraph = match Self::load_task_graph(&req, task_id).await {
            Ok(value) => value,
            Err(error) => {
                error!("cannot load the task_graph. error: {:?}", error);
                let response = tide::Response::builder(404)
                    .body("cannot load the task_graph.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        let task: Task = Self::load_task(&req, task_id).await?;
        let mut expected_image = Image::empty();
        for pair in &task.pairs {
            if pair.pair_type != PairType::Test {
                continue;
            }
            // Extract the first test image.
            expected_image = pair.output.test_image.clone();
            break;
        }
        let expected_image_html: String = expected_image.to_html();

        // let initial_reply_text: String = NaturalLanguage::reply_example1();
        let initial_reply_text = String::new();

        let mut context2 = tera::Context::new();
        context2.insert("task_id", &task_id);
        context2.insert("task_href", &format!("/task/{}", task_id));
        context2.insert("prompt_href", &format!("/task/{}/prompt", task_id));
        context2.insert("reply_text", &initial_reply_text);
        context2.insert("expected_image_html", &expected_image_html);
        context2.insert("predicted_image_html", "Nothing submitted yet");
        let body: String = tera.render("page_reply.html", &context2).unwrap();
        
        let response = Response::builder(200)
            .body(body)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }

    async fn post_reply(mut req: Request<State>) -> tide::Result {
        let reply_data: PostReplyData = req.body_form().await?;

        let task_id: &str = req.param("task_id").unwrap_or("world");
        
        let tera: &Tera = &req.state().tera;

        let _task_graph: TaskGraph = match Self::load_task_graph(&req, task_id).await {
            Ok(value) => value,
            Err(error) => {
                error!("cannot load the task_graph. error: {:?}", error);
                let response = tide::Response::builder(404)
                    .body("cannot load the task_graph.")
                    .content_type("text/plain; charset=utf-8")
                    .build();
                return Ok(response);
            }
        };

        let task: Task = Self::load_task(&req, task_id).await?;
        let mut expected_image = Image::empty();
        for pair in &task.pairs {
            if pair.pair_type != PairType::Test {
                continue;
            }
            // Extract the first test image.
            expected_image = pair.output.test_image.clone();
            break;
        }
        let expected_image_html: String = expected_image.to_html();
        
        let multiline_text: &str = &reply_data.replyText;
        let status_text: String;
        let predicted_image_html: String;
        // match NaturalLanguage::try_from(multiline_text) {
        //     Ok(natural_language) => {
        //         status_text = format!("parsed the reply text. natural_language: {:?}", natural_language);
        //         predicted_image_html = natural_language.to_html();
        //     },
        //     Err(error) => {
        //         status_text = format!("cannot parse the reply text. error: {:?}", error);
        //         predicted_image_html = "Problem in reply text. No image generated.".to_string();
        //     }
        // }
        match PromptRLEDeserializer::try_from(multiline_text) {
            Ok(natural_language) => {
                status_text = format!("parsed the reply text. natural_language: {:?}", natural_language);
                predicted_image_html = natural_language.to_html();
            },
            Err(error) => {
                status_text = format!("cannot parse the reply text. error: {:?}", error);
                predicted_image_html = "Problem in reply text. No image generated.".to_string();
            }
        }

        let mut context2 = tera::Context::new();
        context2.insert("task_id", &task_id);
        context2.insert("task_href", &format!("/task/{}", task_id));
        context2.insert("prompt_href", &format!("/task/{}/prompt", task_id));
        context2.insert("reply_text", &reply_data.replyText);
        context2.insert("post_reply_result", &status_text);
        context2.insert("expected_image_html", &expected_image_html);
        context2.insert("predicted_image_html", &predicted_image_html);
        let body: String = tera.render("page_reply.html", &context2).unwrap();
        
        let response = Response::builder(200)
            .body(body)
            .content_type(mime::HTML)
            .build();
    
        Ok(response)
    }
}

#[derive(Deserialize)]
#[serde(default)]
struct PostReplyData {
    replyText: String,
}

impl Default for PostReplyData {
    fn default() -> Self {
        Self {
            replyText: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct WrapNonPixelNode {
    task_id: Option<String>,
    node_index: NodeIndex,
    outgoing_edges: Vec<TemplateItemEdge>,
    incoming_edges: Vec<TemplateItemEdge>,
}

impl WrapNonPixelNode {
    fn load(&mut self, graph: &Graph<NodeData, EdgeData>, node_href_prefix: &str) {
        self.outgoing_edges = TemplateItemEdge::outgoing(graph, node_href_prefix, self.node_index, false);
        self.incoming_edges = TemplateItemEdge::incoming(graph, node_href_prefix, self.node_index, false);
    }
}

#[derive(Clone, Debug, Default)]
struct WrapPixel {
    is_center_pixel: bool,
    color: Option<u8>,
    x: Option<u8>,
    y: Option<u8>,
    task_id: Option<String>,
    node_index: Option<NodeIndex>,
    outgoing_edges: Vec<TemplateItemEdge>,
    incoming_edges: Vec<TemplateItemEdge>,
}

impl WrapPixel {
    fn load(&mut self, graph: &Graph<NodeData, EdgeData>, node_href_prefix: &str) {
        let node_index: NodeIndex = match self.node_index {
            Some(node_index) => node_index.clone(),
            None => return,
        };
        for edge_pixel in graph.edges(node_index) {
            let child_index: NodeIndex = edge_pixel.target();
            match &graph[child_index] {
                NodeData::PositionX { x } => { self.x = Some(*x); },
                NodeData::PositionY { y } => { self.y = Some(*y); },
                NodeData::Color { color } => { self.color = Some(*color); },
                _ => {}
            }
        }

        self.outgoing_edges = TemplateItemEdge::outgoing(graph, node_href_prefix, node_index, true);
        self.incoming_edges = TemplateItemEdge::incoming(graph, node_href_prefix, node_index, true);
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
                href = format!("/task/{}/node/{}", task_id, node_index.index());
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
        context.insert("is_center_pixel", &self.is_center_pixel);
        context.insert("outgoing_edges", &self.outgoing_edges);
        context.insert("incoming_edges", &self.incoming_edges);
        context
    }
}

#[derive(Clone, Debug, Serialize)]
struct TemplateItemEdge {
    edge_index: usize,
    edge_name: String,
    node_href: String,
    node_index: usize,
    node_name: String,
}

impl TemplateItemEdge {
    fn outgoing(graph: &Graph<NodeData, EdgeData>, node_href_prefix: &str, node_index: NodeIndex, ignore_pixel_neighbor: bool) -> Vec<Self> {
        let edges = graph.edges_directed(node_index, petgraph::Outgoing);
        let mut items = Vec::<TemplateItemEdge>::new();
        for edge in edges {
            match edge.weight() {
                EdgeData::PixelNeighbor { edge_type: _ } => {
                    if ignore_pixel_neighbor {
                        continue;
                    }
                },
                _ => {}
            };
            let child_index: NodeIndex = edge.target();
            let node_name: String = format!("{:?}", graph[child_index]);
            let item = Self {
                edge_index: edge.id().index(),
                edge_name: format!("{:?}", edge.weight()),
                node_href: format!("{}/{}", node_href_prefix, child_index.index()),
                node_index: child_index.index(),
                node_name,
            };
            items.push(item);
        }
        items
    }

    fn incoming(graph: &Graph<NodeData, EdgeData>, node_href_prefix: &str, node_index: NodeIndex, ignore_pixel_neighbor: bool) -> Vec<Self> {
        let edges = graph.edges_directed(node_index, petgraph::Incoming);
        let mut items = Vec::<TemplateItemEdge>::new();
        for edge in edges {
            match edge.weight() {
                EdgeData::PixelNeighbor { edge_type: _ } => {
                    if ignore_pixel_neighbor {
                        continue;
                    }
                },
                _ => {}
            };
            let child_index: NodeIndex = edge.source();
            let node_name: String = format!("{:?}", graph[child_index]);
            let item = Self {
                edge_index: edge.id().index(),
                edge_name: format!("{:?}", edge.weight()),
                node_href: format!("{}/{}", node_href_prefix, child_index.index()),
                node_index: child_index.index(),
                node_name,
            };
            items.push(item);
        }
        items
    }
}

#[derive(Deserialize)]
struct FindNodePixel {
    x: u8,
    y: u8,
    id: String,
}
