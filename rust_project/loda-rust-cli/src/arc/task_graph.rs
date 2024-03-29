//! Graph representation of a single ARC task.
//! 
//! The flow of the solver is as follows:
//! 
//! Compare input images with each other, train+test
//!        
//! Compare output images with each other, train only
//!
//! Establish links between input image and output image, train only
//! - place an agent that walks the graph as if it was a game level.
//! - where does the output gets its size from?
//! - where does the output gets its color from?
//! - do the same object appear in both input and output, but with a different offset?
//! - do the same object appear in both input and output, but with a different color?
//! - do an object only appear across the output images, but not in the input image?
//!
//! Recreate output images for the train pairs
//! - reapply transformations to the input images.
//! - keep the best transformations, reject bad transformations.
//! - make sure that the output image can be recreated with the transformations.
//! - if it cannot be recreated, then establish even more links between input and output, and try again.
//!
//! Create output images for the test pairs
//! - reapply the same transformations to the input images.        
//!
use super::prompt_compact::PromptCompactSerializer;
use super::prompt_position::PromptPositionSerializer;
use super::prompt_run_length_encoding::PromptRLESerializer;
use super::{Image, ImageSize, PixelConnectivity, SingleColorObject, ShapeType, ShapeIdentificationFromSingleColorObject, ColorAndShape, Rectangle, ShapeTransformation};
use super::arc_work_model::{Task, Pair, PairType};
use super::prompt_shape_transform::PromptShapeTransformSerializer;
use super::prompt::{PromptSerialize, PromptType};
use anyhow::Context;
use petgraph::{stable_graph::{NodeIndex, EdgeIndex}, visit::EdgeRef};
use std::collections::{HashSet, HashMap};

/// The number of pair-wise comparisons that are allowed between the same shape type.
/// 
/// This is to prevent the solver from getting stuck in an almost infinite loop.
/// if it's all single pixel shapes that are to be compared.
static SHAPETYPE_COMPARISON_LIMIT: usize = 100;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageType {
    Input,
    Output,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum NodeData {
    Task,
    Id { id: String },
    Pair { pair_index: u8 },
    Image { size: ImageSize, image_type: ImageType },
    Pixel,
    Color { color: u8 },
    Mass { mass: u16 },
    PositionX { x: u8 },
    PositionY { y: u8 },
    PositionReverseX { x: u8 },
    PositionReverseY { y: u8 },
    ObjectsInsideImage { connectivity: PixelConnectivity },
    Object { connectivity: PixelConnectivity },
    ShapeType { shape_type: ShapeType },
    ShapeType45 { shape_type: ShapeType },
    ShapeScale { x: u8, y: u8 },
    ShapeSize { width: u8, height: u8 },
    ShapeTransformations { transformations: Vec<ShapeTransformation> },
    // CenterOfMassUncompressedObject { x, y }
    // Input,
    // Output,
    // PairTrain,
    // PairTest,
    // PairInputImage,
    // PairOutputImage,
    // ImageWall { side: ImageWallSide },
    // ImageCorner { corner: ImageCornerType },
    // PixelColumn,
    // PixelRow,
    // ObjectStaysLocked,
    // ObjectMoves,
    // GridLine,
    // GridCell,
    // GridCorner,
    // TemplateObject and edges to all the places the template occur.
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum PixelNeighborEdgeType {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum EdgeData {
    Link,
    PixelNeighbor { edge_type: PixelNeighborEdgeType },
    Parent,
    Child,
    ObjectTouching { connectivity: PixelConnectivity },

    /// When comparing objects inside a single image, what objects are similar.
    ImageObjectSimilarity { numerator: u8, denominator: u8 },

    /// When comparing input and output, what objects are similar.
    ObjectSimilarity { numerator: u8, denominator: u8 },

    /// When input size is the same the the output size for all pairs.
    /// Has the same color. The mask is the same. The object does not move.
    ObjectIdentical,

    /// When input size is the same the the output size for all pairs.
    /// The mask is the same. The object does not move.
    /// However the colors do change.
    ObjectChangeColor { color_input: u8, color_output: u8 },

    /// When input size is the same the the output size for all pairs.
    /// Has the same color. The mask is the same. 
    /// However the object has moved to another position.
    ObjectChangePosition { relative_x: i8, relative_y: i8 },

    /// When input size is the same the the output size for all pairs.
    /// The mask is the same. 
    /// However the colors have changed.
    /// However the object has moved to another position.
    ObjectChangePositionAndColor { relative_x: i8, relative_y: i8, color_input: u8, color_output: u8 },

    /// When input size is the same the the output size for all pairs.
    /// When a pixel in the input image and a pixel in the output image, both have the same color.
    /// However the pixels belong to different objects.
    ObjectChangeShape { shape_type_input: ShapeType, shape_type_output: ShapeType },

    /// When input size is the same the the output size for all pairs.
    /// the pixel has the same color in both input and output image.
    PixelIdentical,

    /// When input size is the same the the output size for all pairs.
    /// the pixel color differs between input and output.
    PixelChangeColor { color_input: u8, color_output: u8 },

    // PixelNearbyWithSameColor { edge_type: PixelNeighborEdgeType, distance: u8 },
    // PixelNearbyWithDifferentColor { edge_type: PixelNeighborEdgeType, distance: u8 },
    // SymmetricPixel { edge_type: PixelNeighborEdgeType },
    // IsTouchingAnotherObject { edge_type: PixelNeighborEdgeType },
    // InsideHoleOfAnotherObject,
    // InsideBoundingBoxOfAnotherObject,
}

pub type GraphNodeDataEdgeData = petgraph::Graph<NodeData, EdgeData>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TaskGraph {
    graph: GraphNodeDataEdgeData,
    task: Option<Task>,
}

impl TaskGraph {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            graph: petgraph::Graph::new(),
            task: None,
        }
    }

    #[allow(dead_code)]
    pub fn graph(&self) -> &GraphNodeDataEdgeData {
        &self.graph
    }

    #[allow(dead_code)]
    pub fn task(&self) -> &Option<Task> {
        &self.task
    }

    /// Returns the `NodeIndex` of the created image node.
    pub fn add_image(&mut self, image: &Image, image_type: ImageType) -> anyhow::Result<NodeIndex> {
        let node_image = NodeData::Image { size: image.size(), image_type };
        let image_index: NodeIndex = self.graph.add_node(node_image);

        let mut indexes_pixels: Vec<NodeIndex> = Vec::new();
        for y in 0..image.height() {
            for x in 0..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                let pixel_node = NodeData::Pixel;
                let pixel_index: NodeIndex = self.graph.add_node(pixel_node);
                self.graph.add_edge(image_index, pixel_index, EdgeData::Link);
                {
                    let property = NodeData::Color { color };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(pixel_index, index, EdgeData::Link);
                }
                {
                    let property = NodeData::PositionX { x };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(pixel_index, index, EdgeData::Link);
                }
                {
                    let property = NodeData::PositionY { y };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(pixel_index, index, EdgeData::Link);
                }
                {
                    let property = NodeData::PositionReverseX { x: image.width() - x - 1 };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(pixel_index, index, EdgeData::Link);
                }
                {
                    let property = NodeData::PositionReverseY { y: image.height() - y - 1 };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(pixel_index, index, EdgeData::Link);
                }

                indexes_pixels.push(pixel_index);
            }
        }

        // Establish `Left` and `Right` edges between neighbor pixels.
        for y in 0..image.height() {
            for x in 1..image.width() {
                let x0: u8 = x - 1;
                let x1: u8 = x;
                let address0: usize = (y as usize) * (image.width() as usize) + (x0 as usize);
                let address1: usize = (y as usize) * (image.width() as usize) + (x1 as usize);
                let index0: NodeIndex = indexes_pixels[address0];
                let index1: NodeIndex = indexes_pixels[address1];
                self.graph.add_edge(index1, index0, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::Left });
                self.graph.add_edge(index0, index1, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::Right });
            }
        }

        // Establish `Up` and `Down` edges between neighbor pixels.
        for x in 0..image.width() {
            for y in 1..image.height() {
                let y0: u8 = y - 1;
                let y1: u8 = y;
                let address0: usize = (y0 as usize) * (image.width() as usize) + (x as usize);
                let address1: usize = (y1 as usize) * (image.width() as usize) + (x as usize);
                let index0: NodeIndex = indexes_pixels[address0];
                let index1: NodeIndex = indexes_pixels[address1];
                self.graph.add_edge(index1, index0, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::Up });
                self.graph.add_edge(index0, index1, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::Down });
            }
        }

        // Establish `UpLeft` and `DownRight` edges between neighbor pixels.
        for y in 1..image.height() {
            for x in 1..image.width() {
                let x0: u8 = x - 1;
                let x1: u8 = x;
                let y0: u8 = y - 1;
                let y1: u8 = y;
                let address0: usize = (y0 as usize) * (image.width() as usize) + (x0 as usize);
                let address1: usize = (y1 as usize) * (image.width() as usize) + (x1 as usize);
                let index0: NodeIndex = indexes_pixels[address0];
                let index1: NodeIndex = indexes_pixels[address1];
                self.graph.add_edge(index1, index0, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::UpLeft });
                self.graph.add_edge(index0, index1, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::DownRight });
            }
        }

        // Establish `DownLeft` and `UpRight` edges between neighbor pixels.
        for y in 1..image.height() {
            for x in 1..image.width() {
                let x0: u8 = x - 1;
                let x1: u8 = x;
                let y0: u8 = y - 1;
                let y1: u8 = y;
                let address0: usize = (y1 as usize) * (image.width() as usize) + (x0 as usize);
                let address1: usize = (y0 as usize) * (image.width() as usize) + (x1 as usize);
                let index0: NodeIndex = indexes_pixels[address0];
                let index1: NodeIndex = indexes_pixels[address1];
                self.graph.add_edge(index1, index0, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::DownLeft });
                self.graph.add_edge(index0, index1, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::UpRight });
            }
        }

        Ok(image_index)
    }

    /// Generates an image from the graph for the given `NodeIndex`.
    #[allow(dead_code)]
    pub fn to_image(&self, image_index: NodeIndex) -> anyhow::Result<Image> {
        let size: ImageSize = match &self.graph[image_index] {
            NodeData::Image { size , image_type: _ } => *size,
            _ => { 
                return Err(anyhow::anyhow!("Expected NodeData::Image at index {:?}.", image_index)); 
            }
        };
        let mut result_image = Image::color(size.width, size.height, 255);

        for edge_image in self.graph.edges(image_index) {
            let pixel_index: NodeIndex = edge_image.target();
            match &self.graph[pixel_index] {
                NodeData::Pixel => {},
                _ => continue
            }

            let mut found_x: Option<u8> = None;
            let mut found_y: Option<u8> = None;
            let mut found_color: Option<u8> = None;
            for edge_pixel in self.graph.edges(pixel_index) {
                let child_index: NodeIndex = edge_pixel.target();
                match &self.graph[child_index] {
                    NodeData::PositionX { x } => { found_x = Some(*x); },
                    NodeData::PositionY { y } => { found_y = Some(*y); },
                    NodeData::Color { color } => { found_color = Some(*color); },
                    _ => {}
                }
            }
            match (found_x, found_y, found_color) {
                (Some(x), Some(y), Some(color)) => {
                    result_image.set(x as i32, y as i32, color);
                },
                _ => {}
            }
        }

        Ok(result_image)
    }

    /// Describe an object inside an image.
    /// 
    /// Create an object node.
    /// 
    /// Establishes links from pixel nodes to the object node.
    /// 
    /// Returns the `NodeIndex` of the created `Object` node.
    #[allow(dead_code)]
    pub fn add_object(&mut self, image_index: NodeIndex, object_mask: &Image, connectivity: PixelConnectivity) -> anyhow::Result<NodeIndex> {
        let node_object = NodeData::Object { connectivity };
        let object_index: NodeIndex = self.graph.add_node(node_object);

        let size = match &self.graph[image_index] {
            NodeData::Image { size, image_type: _ } => *size,
            _ => { 
                return Err(anyhow::anyhow!("Expected NodeData::Image at index {:?}.", image_index)); 
            }
        };
        if size != object_mask.size() {
            return Err(anyhow::anyhow!("Expected object_mask to have same size as the image its describing."));
        }

        let mut pixel_indexes = Vec::<NodeIndex>::new();
        for edge_image in self.graph.edges(image_index) {
            let pixel_index: NodeIndex = edge_image.target();
            match &self.graph[pixel_index] {
                NodeData::Pixel => {},
                _ => continue
            }

            let mut found_x: Option<u8> = None;
            let mut found_y: Option<u8> = None;
            for edge_pixel in self.graph.edges(pixel_index) {
                let child_index: NodeIndex = edge_pixel.target();
                match &self.graph[child_index] {
                    NodeData::PositionX { x } => { found_x = Some(*x); },
                    NodeData::PositionY { y } => { found_y = Some(*y); },
                    _ => {}
                }
            }
            let (x, y) = match (found_x, found_y) {
                (Some(x), Some(y)) => (x, y),
                _ => {
                    return Err(anyhow::anyhow!("Expected all pixels to have PositionX and PositionY properties."));
                }
            };

            let color: u8 = match object_mask.get(x as i32, y as i32) {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("No pixel for coordinate. Expected object_mask to have same size as the image its describing."));
                }
            };
            if color == 0 {
                continue;
            }
            pixel_indexes.push(pixel_index);
        }

        // println!("pixel to object: adding {} edges", pixel_indexes.len());
        for pixel_index in pixel_indexes {
            let _edge_index: EdgeIndex = self.graph.add_edge(pixel_index, object_index, EdgeData::Parent);
            let _edge_index: EdgeIndex = self.graph.add_edge(object_index, pixel_index, EdgeData::Child);
        }
        Ok(object_index)
    }

    fn process_shapes(&mut self, image_index: NodeIndex, _name: &str, sco: &Option<SingleColorObject>, connectivity: PixelConnectivity) -> anyhow::Result<ProcessShapes> {
        let objectsinsideimage_index: NodeIndex;
        {
            let node = NodeData::ObjectsInsideImage { connectivity };
            let index: NodeIndex = self.graph.add_node(node);
            objectsinsideimage_index = index;
            self.graph.add_edge(image_index, index, EdgeData::Link);
        }

        let sco: &SingleColorObject = match sco {
            Some(value) => value,
            None => {
                // println!("{}: no sco", name);
                let instance = ProcessShapes {
                    objectsinsideimage_index,
                    color_and_shape_vec: vec!(),
                    color_and_shape_to_object_nodeindex: HashMap::new(),
                };
                return Ok(instance);
            }
        };

        let sifsco: ShapeIdentificationFromSingleColorObject = ShapeIdentificationFromSingleColorObject::find_shapes(sco, connectivity)?;
        let mut color_and_shape_to_object_nodeindex = HashMap::<usize, NodeIndex>::new();
        for (color_and_shape_index, color_and_shape) in sifsco.color_and_shape_vec.iter().enumerate() {
            let object_nodeindex: NodeIndex = match self.add_object(image_index, &color_and_shape.shape_identification.mask_uncropped, connectivity) {
                Ok(value) => value,
                Err(error) => {
                    println!("unable to add object to graph. error: {:?}", error);
                    continue;
                }
            };
            // println!("name: {} object_index: {:?}", name, object_index);
            color_and_shape_to_object_nodeindex.insert(color_and_shape_index, object_nodeindex);
            {
                self.graph.add_edge(objectsinsideimage_index, object_nodeindex, EdgeData::Child);
                self.graph.add_edge(object_nodeindex, objectsinsideimage_index, EdgeData::Parent);
            }

            {
                let node = NodeData::ShapeType { shape_type: color_and_shape.shape_identification.shape_type.clone() };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            {
                let node = NodeData::ShapeType45 { shape_type: color_and_shape.shape_identification.shape_type45.clone() };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            {
                let node = NodeData::Color { color: color_and_shape.color };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            if let Some(scale) = &color_and_shape.shape_identification.scale {
                let node = NodeData::ShapeScale { x: scale.x, y: scale.y };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }
            
            {
                let size: ImageSize = color_and_shape.shape_identification.rect.size();
                let node = NodeData::ShapeSize { width: size.width, height: size.height };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            {
                let node = NodeData::PositionX { x: color_and_shape.position_x };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            {
                let node = NodeData::PositionY { y: color_and_shape.position_y };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            if let Some(x) = color_and_shape.position_x_reverse {
                let property = NodeData::PositionReverseX { x };
                let index: NodeIndex = self.graph.add_node(property);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            if let Some(y) = color_and_shape.position_y_reverse {
                let property = NodeData::PositionReverseY { y };
                let index: NodeIndex = self.graph.add_node(property);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            {
                let node = NodeData::Mass { mass: color_and_shape.shape_identification.mass };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }

            {
                let transformations: Vec<ShapeTransformation> = color_and_shape.shape_identification.transformations_sorted_vec();
                let node = NodeData::ShapeTransformations { transformations };
                let index: NodeIndex = self.graph.add_node(node);
                self.graph.add_edge(object_nodeindex, index, EdgeData::Link);
            }
        }

        let instance = ProcessShapes {
            objectsinsideimage_index,
            color_and_shape_vec: sifsco.color_and_shape_vec,
            color_and_shape_to_object_nodeindex,
        };
        Ok(instance)
    }

    fn compare_objects_between_input_and_output(&mut self, task: &Task, pair: &Pair, input_process_shapes: &ProcessShapes, output_process_shapes: &ProcessShapes, connectivity: PixelConnectivity) -> anyhow::Result<()> {
        if pair.pair_type == PairType::Test {
            return Ok(());
        }

        let is_output_size_same_as_input_size: bool = task.is_output_size_same_as_input_size();

        let verbose = false;
        if verbose {
            println!("connectivity: {:?}", connectivity);
        }

        let input_shapetype_set: HashSet<ShapeType> = input_process_shapes.shapetype_set();
        let output_shapetype_set: HashSet<ShapeType> = output_process_shapes.shapetype_set();        
        let intersection_shapetype_set: HashSet<ShapeType> = input_shapetype_set.intersection(&output_shapetype_set).cloned().collect();

        if verbose {
            println!("input_shapetype_set: {:?}", input_shapetype_set);
            println!("output_shapetype_set: {:?}", output_shapetype_set);
            println!("intersection_shapetype_set: {:?}", intersection_shapetype_set);
        }

        // loop over the `intersection_shapetype_set`
        // with the input shapetype, obtain reference to its ShapeIdentification instance
        // with the output shapetype, obtain reference to its ShapeIdentification instance
        // compare the two ShapeIdentification instances
        //
        // Add edge between the input and output objects if they have the same color.
        //
        // Add edge between the input and output objects if they have the same shape.
        //
        // Add edge between the input and output objects if they have the same size of bounding box.
        //
        // Add edge between the input and output objects if they are similar, with the edge data being the similarity score:
        // - identical (same size, no transformation, same color), `HighlySimilarObject { mode: Identical }`
        // - scaled (scaled size, no transformation, same color), `HighlySimilarObject { mode: Scaled }`
        // - transformed (same size, transformed, same color), `HighlySimilarObject { mode: Transformed { transformation } }`
        // - scaled and transformed (scaled size, transformed, same color), `MediumSimilarObject { mode: ScaledAndTransformed { transformation } }`
        // - identical (same size, no transformation, different color), `MediumSimilarObject { mode: Identical }`
        // - scaled (scaled size, no transformation, different color), `MediumSimilarObject { mode: Scaled }`
        // - transformed (same size, transformed, different color), `MediumSimilarObject { mode: Transformed { transformation } }`
        // - scaled and transformed (scaled size, transformed, different color), `MediumSimilarObject { mode: ScaledAndTransformed { transformation } }`
        // - compressed (different size, no transformation, same color), `WeaklySimilarObject { mode: Identical }`
        // - compressed (different size, transformed, same color), `WeaklySimilarObject { mode: Transformed }`
        // - compressed (different size, transformed, different color), `WeaklySimilarObject { mode: TransformedAndDifferentColor }`
        for shapetype in &intersection_shapetype_set {
            self.compare_objects_between_input_and_output_shapetype(
                is_output_size_same_as_input_size, 
                shapetype, 
                input_process_shapes, 
                output_process_shapes
            )?;
        }

        Ok(())
    }

    fn compare_objects_between_input_and_output_shapetype(&mut self, is_output_size_same_as_input_size: bool, shapetype: &ShapeType, input_process_shapes: &ProcessShapes, output_process_shapes: &ProcessShapes) -> anyhow::Result<()> {
        let verbose = false;

        let input_items: Vec<(usize, &ColorAndShape)> = input_process_shapes.obtain_color_and_shape_vec(shapetype);
        let output_items: Vec<(usize, &ColorAndShape)> = output_process_shapes.obtain_color_and_shape_vec(shapetype);

        if verbose {
            println!("shapetype: {:?}  compare {} input shapes with {} output shapes", shapetype, input_items.len(), output_items.len());
        }

        let number_of_comparisons: usize = input_items.len() * output_items.len();
        if number_of_comparisons > SHAPETYPE_COMPARISON_LIMIT {
            if verbose {
                println!("too many comparisons, skipping");
            }
            return Ok(());
        }

        for (input_index, input_color_and_shape) in input_items {
            for (output_index, output_color_and_shape) in &output_items {
                let nodeindex0_option: Option<&NodeIndex> = input_process_shapes.color_and_shape_to_object_nodeindex.get(&input_index);
                let nodeindex1_option: Option<&NodeIndex> = output_process_shapes.color_and_shape_to_object_nodeindex.get(&output_index);
                let (nodeindex0, nodeindex1) = match (nodeindex0_option, nodeindex1_option) {
                    (Some(nodeindex0), Some(nodeindex1)) => (nodeindex0, nodeindex1),
                    _ => continue,
                };

                let same_color: bool = input_color_and_shape.color == output_color_and_shape.color;

                // Determine if the shapes have the same mass or a clean multiple of each other.
                let mass0: u16 = input_color_and_shape.shape_identification.mass;
                let mass1: u16 = output_color_and_shape.shape_identification.mass;
                let mut same_mass: bool = false;
                if mass0 > 0 && mass1 > 0 {
                    let value_max: u16 = mass0.max(mass1);
                    let value_min: u16 = mass0.min(mass1);
                    let remain: u16 = value_max % value_min;
                    if remain == 0 {
                        same_mass = true;
                    }
                }

                let rect0: Rectangle = input_color_and_shape.shape_identification.rect;
                let rect1: Rectangle = output_color_and_shape.shape_identification.rect;
                let same_rect: bool = rect0 == rect1;
                let size0: ImageSize = rect0.size();
                let size1: ImageSize = rect1.size();
                let same_size: bool = size0 == size1;
                let same_width: bool = size0.width == size1.width;
                let same_height: bool = size0.height == size1.height;
                let same_transformations: bool = input_color_and_shape.shape_identification.transformations == output_color_and_shape.shape_identification.transformations;

                // for very similar objects, then check if the mask pixel data is identical.
                let mut same_mask: bool = false;
                if same_size && same_transformations && same_mass {
                    if verbose {
                        println!("same_size: {}  same_transformations: {}", same_size, same_transformations);
                    }
                    let mask0: &Image = &input_color_and_shape.shape_identification.mask_cropped;
                    let mask1: &Image = &output_color_and_shape.shape_identification.mask_cropped;
                    same_mask = mask0 == mask1;
                }

                // When the input and output are the same size, then we can do this strong similarity check.
                if same_rect && same_mask && same_transformations && is_output_size_same_as_input_size {
                    let edge_data: EdgeData;
                    if same_color {
                        edge_data = EdgeData::ObjectIdentical;
                    } else {
                        edge_data = EdgeData::ObjectChangeColor { color_input: input_color_and_shape.color, color_output: output_color_and_shape.color };
                    }
                    self.graph.add_edge(*nodeindex0, *nodeindex1, edge_data);
                    self.graph.add_edge(*nodeindex1, *nodeindex0, edge_data);
                    continue;
                }

                // When the input and output are the same size, then we can do this strong similarity check.
                let relative_x_i32: i32 = (rect1.min_x() - rect0.min_x()) as i32;
                let relative_y_i32: i32 = (rect1.min_y() - rect0.min_y()) as i32;
                let has_moved: bool = relative_x_i32 != 0 || relative_y_i32 != 0;
                if has_moved && same_size && same_mask && same_transformations && is_output_size_same_as_input_size {
                    let relative_x: i8 = relative_x_i32.max(i8::MIN as i32).min(i8::MAX as i32) as i8;
                    let relative_y: i8 = relative_y_i32.max(i8::MIN as i32).min(i8::MAX as i32) as i8;
                    let edge_data: EdgeData;
                    if same_color {
                        edge_data = EdgeData::ObjectChangePosition { relative_x, relative_y };
                    } else {
                        edge_data = EdgeData::ObjectChangePositionAndColor { relative_x, relative_y, color_input: input_color_and_shape.color, color_output: output_color_and_shape.color };
                    }
                    self.graph.add_edge(*nodeindex0, *nodeindex1, edge_data);
                    self.graph.add_edge(*nodeindex1, *nodeindex0, edge_data);
                    continue;
                }

                // Future experiments:
                // for very similar objects, then check if the mask pixel data is the same after transformation.
                // if input.histogram and output.histogram has the same mass for the color and the current shape has that color, then it's likely to be the same object.
                // for same size input/output - does the shape occupy the exact same pixels in both input and output.
                // for same size input/output - does the input shape overlap with the output shape. Then it's likely the shapes are related.
                let same_data = [same_color, same_mass, same_width, same_height, same_transformations, same_mask];
                let same_count: usize = same_data.into_iter().filter(|x| *x).count();
                let numerator_u8: u8 = same_count as u8;
                let denominator_u8: u8 = same_data.len() as u8;
                let similarity_score_f64: f64 = numerator_u8 as f64 / denominator_u8 as f64;
                let similarity_score_percent: usize = (numerator_u8 as usize) * 100 / (denominator_u8 as usize);
                if verbose {
                    println!("  input_index: {}  output_index: {}  same_count: {}  similarity_score: {}", input_index, output_index, same_count, similarity_score_f64);
                }
                if similarity_score_percent < 20 {
                    // Too dissimilar. Don't add an edge.
                    continue;
                }
                if verbose {
                    println!("  adding edge between input and output objects");
                }
                let edge_data = EdgeData::ObjectSimilarity { numerator: numerator_u8, denominator: denominator_u8 };
                self.graph.add_edge(*nodeindex0, *nodeindex1, edge_data);
                self.graph.add_edge(*nodeindex1, *nodeindex0, edge_data);
            }
        }
        Ok(())
    }

    /// Determine what objects are occurring multiple times in an image.
    fn find_similar_objects_inside_image(&mut self, process_shapes: &ProcessShapes) -> anyhow::Result<()> {
        let shapetype_set: HashSet<ShapeType> = process_shapes.shapetype_set();
        for shapetype in &shapetype_set {
            self.find_similar_objects_inside_image_shapetype(
                shapetype, 
                process_shapes, 
            )?;
        }
        Ok(())
    }

    fn find_similar_objects_inside_image_shapetype(&mut self, shapetype: &ShapeType, process_shapes: &ProcessShapes) -> anyhow::Result<()> {
        let verbose = false;

        let items: Vec<(usize, &ColorAndShape)> = process_shapes.obtain_color_and_shape_vec(shapetype);

        if verbose {
            println!("shapetype: {:?}  compare {}x{} shapes", shapetype, items.len(), items.len());
        }

        let number_of_comparisons: usize = items.len() * items.len();
        if number_of_comparisons > SHAPETYPE_COMPARISON_LIMIT {
            if verbose {
                println!("too many comparisons, skipping");
            }
            return Ok(());
        }

        for (index0, color_and_shape0) in &items {
            for (index1, color_and_shape1) in &items {
                if index0 >= index1 {
                    // ignore the diagonal by using == because we don't want to compare the object with itself.
                    // ignore half of the triangle, by using `>` so that we don't compare the same pair of objects twice.
                    continue;
                }

                let nodeindex0_option: Option<&NodeIndex> = process_shapes.color_and_shape_to_object_nodeindex.get(&index0);
                let nodeindex1_option: Option<&NodeIndex> = process_shapes.color_and_shape_to_object_nodeindex.get(&index1);
                let (nodeindex0, nodeindex1) = match (nodeindex0_option, nodeindex1_option) {
                    (Some(nodeindex0), Some(nodeindex1)) => (nodeindex0, nodeindex1),
                    _ => continue,
                };

                let same_color: bool = color_and_shape0.color == color_and_shape1.color;

                // Determine if the shapes have the same mass or a clean multiple of each other.
                let mass0: u16 = color_and_shape0.shape_identification.mass;
                let mass1: u16 = color_and_shape1.shape_identification.mass;
                let mut same_mass: bool = false;
                if mass0 > 0 && mass1 > 0 {
                    let value_max: u16 = mass0.max(mass1);
                    let value_min: u16 = mass0.min(mass1);
                    let remain: u16 = value_max % value_min;
                    if remain == 0 {
                        same_mass = true;
                    }
                }

                let rect0: Rectangle = color_and_shape0.shape_identification.rect;
                let rect1: Rectangle = color_and_shape1.shape_identification.rect;
                let size0: ImageSize = rect0.size();
                let size1: ImageSize = rect1.size();
                let same_size: bool = size0 == size1;
                let same_width: bool = size0.width == size1.width;
                let same_height: bool = size0.height == size1.height;
                let same_transformations: bool = color_and_shape0.shape_identification.transformations == color_and_shape1.shape_identification.transformations;

                // for very similar objects, then check if the mask pixel data is identical.
                let mut same_mask: bool = false;
                if same_size && same_transformations && same_mass {
                    if verbose {
                        println!("same_size: {}  same_transformations: {}", same_size, same_transformations);
                    }
                    let mask0: &Image = &color_and_shape0.shape_identification.mask_cropped;
                    let mask1: &Image = &color_and_shape1.shape_identification.mask_cropped;
                    same_mask = mask0 == mask1;
                }

                // Future experiments:
                // for very similar objects, then check if the mask pixel data is the same after transformation.
                // if input.histogram and output.histogram has the same mass for the color and the current shape has that color, then it's likely to be the same object.
                let same_data = [same_color, same_mass, same_width, same_height, same_transformations, same_mask];
                let same_count: usize = same_data.into_iter().filter(|x| *x).count();
                let numerator_u8: u8 = same_count as u8;
                let denominator_u8: u8 = same_data.len() as u8;
                let similarity_score_f64: f64 = numerator_u8 as f64 / denominator_u8 as f64;
                let similarity_score_percent: usize = (numerator_u8 as usize) * 100 / (denominator_u8 as usize);
                if verbose {
                    println!("  input_index: {}  output_index: {}  same_count: {}  similarity_score: {}", index0, index1, same_count, similarity_score_f64);
                }
                if similarity_score_percent < 20 {
                    // Too dissimilar. Don't add an edge.
                    continue;
                }
                if verbose {
                    println!("  adding edge between 2 similar objects");
                }
                let edge_data = EdgeData::ImageObjectSimilarity { numerator: numerator_u8, denominator: denominator_u8 };
                self.graph.add_edge(*nodeindex0, *nodeindex1, edge_data);
                self.graph.add_edge(*nodeindex1, *nodeindex0, edge_data);
            }
        }
        Ok(())
    }

    fn pixel_nodeindexes_from_image(&self, image_nodeindex: NodeIndex) -> anyhow::Result<HashMap<(u8, u8), NodeIndex>> {
        match &self.graph[image_nodeindex] {
            NodeData::Image { size: _, image_type: _ } => {},
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Image"));
            }
        }

        let mut position_to_pixelnodeindex: HashMap<(u8, u8), NodeIndex> = HashMap::new();

        for edge_image in self.graph.edges(image_nodeindex) {
            let pixel_index: NodeIndex = edge_image.target();
            match &self.graph[pixel_index] {
                NodeData::Pixel => {},
                _ => continue
            }

            // Obtain coordinates (x, y) for this pixel.
            let mut found_x: Option<u8> = None;
            let mut found_y: Option<u8> = None;
            for edge_pixel in self.graph.edges(pixel_index) {
                let node_index: NodeIndex = edge_pixel.target();
                match &self.graph[node_index] {
                    NodeData::PositionX { x } => { found_x = Some(*x); },
                    NodeData::PositionY { y } => { found_y = Some(*y); },
                    _ => continue
                }
            }

            let (x, y) = match (found_x, found_y) {
                (Some(x), Some(y)) => (x, y),
                _ => continue
            };
            position_to_pixelnodeindex.insert((x, y), pixel_index);
        }

        Ok(position_to_pixelnodeindex)
    }

    /// Get the node index of the pixel at the given coordinate.
    pub fn get_pixel_nodeindex_at_xy_coordinate(&self, image_node_index: NodeIndex, find_x: u8, find_y: u8) -> anyhow::Result<NodeIndex> {
        let mut found_pixel_node_index: Option<NodeIndex> = None;
        for edge_image in self.graph.edges(image_node_index) {
            let node_index: NodeIndex = edge_image.target();
            match &self.graph[node_index] {
                NodeData::Pixel => {},
                _ => continue
            }
            let pixel_index: NodeIndex = node_index;

            let mut found_x: Option<u8> = None;
            let mut found_y: Option<u8> = None;
            for edge_pixel in self.graph.edges(pixel_index) {
                let child_index: NodeIndex = edge_pixel.target();
                let child_node: &NodeData = &self.graph[child_index];
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
            if pixel_x != find_x || pixel_y != find_y {
                continue;
            }
            if found_pixel_node_index.is_some() {
                return Err(anyhow::anyhow!("multiple candidates found. x: {} y: {}", find_x, find_y));
            }
            found_pixel_node_index = Some(node_index);
        }
        match found_pixel_node_index {
            Some(pixel_node_index) => Ok(pixel_node_index),
            None => Err(anyhow::anyhow!("Cannot find the pixel in the graph"))
        }
    }

    fn get_color_of_pixel(&self, pixel_nodeindex: NodeIndex) -> anyhow::Result<u8> {
        match &self.graph[pixel_nodeindex] {
            NodeData::Pixel => {},
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Pixel"));
            }
        }

        let mut found_color: Option<u8> = None;
        for edge_pixel in self.graph.edges(pixel_nodeindex) {
            let node_index: NodeIndex = edge_pixel.target();
            match &self.graph[node_index] {
                NodeData::Color { color } => { found_color = Some(*color); },
                _ => continue
            }
        }

        let color: u8 = match found_color {
            Some(color) => color,
            None => {
                return Err(anyhow::anyhow!("expected NodeData::Color"));
            }
        };

        Ok(color)
    }

    fn get_object_from_pixel(&self, pixel_nodeindex: NodeIndex, connectivity: PixelConnectivity) -> anyhow::Result<NodeIndex> {
        match &self.graph[pixel_nodeindex] {
            NodeData::Pixel => {},
            _ => { 
                return Err(anyhow::anyhow!("get_object_from_pixel. Expected NodeData::Pixel"));
            }
        }

        let mut found_object_nodeindex: Option<NodeIndex> = None;
        let mut ambiguous_count: usize = 0;
        for edge_pixel in self.graph.edges(pixel_nodeindex) {
            let node_index: NodeIndex = edge_pixel.target();
            match &self.graph[node_index] {
                NodeData::Object { connectivity: the_connectivity } => { 
                    if *the_connectivity != connectivity {
                        continue;
                    }
                    found_object_nodeindex = Some(node_index);
                    ambiguous_count += 1;
                },
                _ => continue
            }
        }
        if ambiguous_count > 1 {
            return Err(anyhow::anyhow!("get_object_from_pixel. Pixel is linked with multiple objects. Is supposed to be linked with only one object."));
        }
        if let Some(nodeindex) = found_object_nodeindex {
            return Ok(nodeindex);
        }
        Err(anyhow::anyhow!("get_object_from_pixel. Pixel is not linked with an object. Is supposed to be linked with only one object."))
    }

    /// Find the `Pair` node with the given `pair_index`.
    fn get_pair(&self, pair_index: u8) -> anyhow::Result<NodeIndex> {
        let mut found_nodeindex: Option<NodeIndex> = None;
        let mut ambiguous_count: usize = 0;
        for node_index in self.graph.node_indices() {
            match &self.graph[node_index] {
                NodeData::Pair { pair_index: the_pair_index } => {
                    if *the_pair_index != pair_index {
                        continue;
                    }
                    found_nodeindex = Some(node_index);
                    ambiguous_count += 1;
                },
                _ => {}
            }
        }
        if ambiguous_count > 1 {
            return Err(anyhow::anyhow!("Multiple pairs with the same index. There is supposed to be only one pair with the same index."));
        }
        let pair_nodeindex: NodeIndex = match found_nodeindex {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("No pair with index {}", pair_index));
            }
        };
        Ok(pair_nodeindex)
    }

    /// Get the object `NodeIndex` for pixel.
    fn get_object_for_input_pixel(&self, pair_index: u8, x: u8, y: u8, connectivity: PixelConnectivity) -> anyhow::Result<NodeIndex> {
        let pair_node: NodeIndex = self.get_pair(pair_index).context("get_object_for_input_pixel pair_node")?;
        let image_node: NodeIndex = self.get_image_for_pair(pair_node, ImageType::Input).context("get_object_for_input_pixel image_node")?;
        let pixel_node: NodeIndex = self.get_pixel_nodeindex_at_xy_coordinate(image_node, x, y).context("get_object_for_input_pixel pixel_node")?;
        let object_node: NodeIndex = self.get_object_from_pixel(pixel_node, connectivity).context("get_object_for_input_pixel object_node")?;
        Ok(object_node)
    }

    /// Get the `ShapeType` for pixel.
    pub fn get_shapetype_for_input_pixel(&self, pair_index: u8, x: u8, y: u8, connectivity: PixelConnectivity) -> anyhow::Result<ShapeType> {
        let object_node: NodeIndex = self.get_object_for_input_pixel(pair_index, x, y, connectivity).context("get_shapetype_for_input_pixel object_node")?;
        let shape_type: ShapeType = self.get_shapetype_from_object(object_node).context("get_shapetype_for_input_pixel shape_type")?;
        Ok(shape_type)
    }

    /// Get the `ShapeType45` for pixel.
    pub fn get_shapetype45_for_input_pixel(&self, pair_index: u8, x: u8, y: u8, connectivity: PixelConnectivity) -> anyhow::Result<ShapeType> {
        let object_node: NodeIndex = self.get_object_for_input_pixel(pair_index, x, y, connectivity).context("get_shapetype45_for_input_pixel object_node")?;
        let shape_type: ShapeType = self.get_shapetype45_from_object(object_node).context("get_shapetype45_for_input_pixel shape_type")?;
        Ok(shape_type)
    }

    /// Get the `ShapeTransformation` vector for pixel.
    #[allow(dead_code)]
    pub fn get_shapetransformations_for_input_pixel(&self, pair_index: u8, x: u8, y: u8, connectivity: PixelConnectivity) -> anyhow::Result<Vec<ShapeTransformation>> {
        let object_node: NodeIndex = self.get_object_for_input_pixel(pair_index, x, y, connectivity).context("get_shapetransformations_for_input_pixel object_node")?;
        let transformations: Vec<ShapeTransformation> = self.get_shapetransformations_from_object(object_node).context("get_shapetransformations_for_input_pixel transformations")?;
        Ok(transformations)
    }

    /// Get the object ID for pixel.
    #[allow(dead_code)]
    pub fn get_objectid_for_input_pixel(&self, pair_index: u8, x: u8, y: u8, connectivity: PixelConnectivity) -> anyhow::Result<usize> {
        let object_node: NodeIndex = self.get_object_for_input_pixel(pair_index, x, y, connectivity).context("get_objectid_for_input_pixel object_node")?;
        let index: usize = object_node.index();
        Ok(index)
    }

    /// Get the width and height of the shape for pixel.
    pub fn get_shapesize_for_input_pixel(&self, pair_index: u8, x: u8, y: u8, connectivity: PixelConnectivity) -> anyhow::Result<ImageSize> {
        let object_node: NodeIndex = self.get_object_for_input_pixel(pair_index, x, y, connectivity).context("get_shapesize_for_input_pixel object_node")?;
        let shape_size: ImageSize = self.get_shapesize_from_object(object_node).context("get_shapesize_for_input_pixel shape_size")?;
        Ok(shape_size)
    }

    /// Get the position (x, y) of the shape for pixel.
    pub fn get_objectposition_for_input_pixel(&self, pair_index: u8, x: u8, y: u8, connectivity: PixelConnectivity) -> anyhow::Result<(u8, u8)> {
        let object_node: NodeIndex = self.get_object_for_input_pixel(pair_index, x, y, connectivity).context("get_objectposition_for_input_pixel object_node")?;
        let xy_tupple: (u8, u8) = self.get_position_from_object(object_node).context("get_objectposition_for_input_pixel xy_tupple")?;
        Ok(xy_tupple)
    }

    /// Find the `Image` node via the `Pair` node.
    fn get_image_for_pair(&self, pair_nodeindex: NodeIndex, image_type: ImageType) -> anyhow::Result<NodeIndex> {
        let mut found_nodeindex: Option<NodeIndex> = None;
        let mut ambiguous_count: usize = 0;
        for edge in self.graph.edges(pair_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::Image { size: _, image_type: the_image_type } => {
                    if *the_image_type != image_type {
                        continue;
                    }
                    found_nodeindex = Some(node_index);
                    ambiguous_count += 1;
                },
                _ => {}
            }
        }
        if ambiguous_count > 1 {
            return Err(anyhow::anyhow!("Multiple images with the same ImageType. A pair node is supposed to have have images with unique ImageTypes"));
        }
        let image_nodeindex: NodeIndex = match found_nodeindex {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("No image for pair"));
            }
        };
        Ok(image_nodeindex)
    }

    /// Find the `ObjectsInsideImage` node via the `Image` node.
    fn get_objectsinsideimage_for_image(&self, image_nodeindex: NodeIndex, connectivity: PixelConnectivity) -> anyhow::Result<NodeIndex> {
        let mut found_nodeindex: Option<NodeIndex> = None;
        let mut ambiguous_count: usize = 0;
        for edge in self.graph.edges(image_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::ObjectsInsideImage { connectivity: the_connectivity } => {
                    if *the_connectivity != connectivity {
                        continue;
                    }
                    found_nodeindex = Some(node_index);
                    ambiguous_count += 1;
                },
                _ => {}
            }
        }
        if ambiguous_count > 1 {
            return Err(anyhow::anyhow!("Multiple ObjectsInsideImage nodes with the same connectivity. Connectivity is supposed to be unique."));
        }
        let nodeindex: NodeIndex = match found_nodeindex {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Cannot find ObjectsInsideImage"));
            }
        };
        Ok(nodeindex)
    }

    fn get_objectsinsideimage_for_pair(&self, pair_index: u8, image_type: ImageType, connectivity: PixelConnectivity) -> anyhow::Result<NodeIndex> {
        // Find the pair node
        let pair_nodeindex: NodeIndex = self.get_pair(pair_index)?;

        // Find the image node
        let image_nodeindex: NodeIndex = self.get_image_for_pair(pair_nodeindex, image_type)?;

        // Find the ObjectsInsideImage node
        let objectsinsideimage_nodeindex: NodeIndex = self.get_objectsinsideimage_for_image(image_nodeindex, connectivity)?;
        Ok(objectsinsideimage_nodeindex)
    }

    fn get_shapetype_from_object(&self, object_nodeindex: NodeIndex) -> anyhow::Result<ShapeType> {
        match &self.graph[object_nodeindex] {
            NodeData::Object { connectivity: _ } => {},
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Object"));
            }
        }

        let mut found_shapetype: Option<ShapeType> = None;
        let mut ambiguous_count: usize = 0;
        for edge in self.graph.edges(object_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::ShapeType { shape_type } => { 
                    found_shapetype = Some(*shape_type);
                    ambiguous_count += 1;
                },
                _ => continue
            }
        }
        if ambiguous_count > 1 {
            return Err(anyhow::anyhow!("Object is linked with multiple ShapeType's. Is supposed to be linked with only one ShapeType."));
        }
        if let Some(shapetype) = found_shapetype {
            return Ok(shapetype);
        }
        Err(anyhow::anyhow!("Object is not linked with a ShapeType. Is supposed to be linked with only one ShapeType."))
    }

    fn get_shapetype45_from_object(&self, object_nodeindex: NodeIndex) -> anyhow::Result<ShapeType> {
        match &self.graph[object_nodeindex] {
            NodeData::Object { connectivity: _ } => {},
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Object"));
            }
        }

        let mut found_shapetype: Option<ShapeType> = None;
        let mut ambiguous_count: usize = 0;
        for edge in self.graph.edges(object_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::ShapeType45 { shape_type } => { 
                    found_shapetype = Some(*shape_type);
                    ambiguous_count += 1;
                },
                _ => continue
            }
        }
        if ambiguous_count > 1 {
            return Err(anyhow::anyhow!("Object is linked with multiple ShapeType45's. Is supposed to be linked with only one ShapeType45."));
        }
        if let Some(shapetype) = found_shapetype {
            return Ok(shapetype);
        }
        Err(anyhow::anyhow!("Object is not linked with a ShapeType45. Is supposed to be linked with only one ShapeType45."))
    }

    fn get_shapesize_from_object(&self, object_nodeindex: NodeIndex) -> anyhow::Result<ImageSize> {
        match &self.graph[object_nodeindex] {
            NodeData::Object { connectivity: _ } => {},
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Object"));
            }
        }

        let mut found_shapesize: Option<ImageSize> = None;
        let mut ambiguous_count: usize = 0;
        for edge in self.graph.edges(object_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::ShapeSize { width, height } => { 
                    found_shapesize = Some(ImageSize::new(*width, *height));
                    ambiguous_count += 1;
                },
                _ => continue
            }
        }
        if ambiguous_count > 1 {
            return Err(anyhow::anyhow!("Object is linked with multiple ShapeSize's. Is supposed to be linked with only one ShapeSize."));
        }
        if let Some(shapesize) = found_shapesize {
            return Ok(shapesize);
        }
        Err(anyhow::anyhow!("Object is not linked with a ShapeSize. Is supposed to be linked with only one ShapeSize."))
    }

    fn get_position_from_object(&self, object_nodeindex: NodeIndex) -> anyhow::Result<(u8, u8)> {
        match &self.graph[object_nodeindex] {
            NodeData::Object { connectivity: _ } => {},
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Object"));
            }
        }

        let mut found_x: Option<u8> = None;
        let mut found_y: Option<u8> = None;
        let mut ambiguous_x_count: usize = 0;
        let mut ambiguous_y_count: usize = 0;
        for edge in self.graph.edges(object_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::PositionX { x } => { 
                    found_x = Some(*x);
                    ambiguous_x_count += 1;
                },
                NodeData::PositionY { y } => { 
                    found_y = Some(*y);
                    ambiguous_y_count += 1;
                },
                _ => continue
            }
        }
        if ambiguous_x_count > 1 || ambiguous_y_count > 1 {
            return Err(anyhow::anyhow!("Object is linked with multiple PositionX's or multiple PositionY's. Is supposed to be linked with only one."));
        }
        match (found_x, found_y) {
            (Some(x), Some(y)) => {
                return Ok((x, y));
            },
            _ => {
                return Err(anyhow::anyhow!("Object is not linked with both a PositionX and a PositionY. Is supposed to be linked with only one."));
            }
        }
    }

    fn get_shapetransformations_from_object(&self, object_nodeindex: NodeIndex) -> anyhow::Result<Vec<ShapeTransformation>> {
        match &self.graph[object_nodeindex] {
            NodeData::Object { connectivity: _ } => {},
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Object"));
            }
        }

        let mut found_shapetransformations: Option<Vec<ShapeTransformation>> = None;
        let mut ambiguous_count: usize = 0;
        for edge in self.graph.edges(object_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::ShapeTransformations { transformations } => { 
                    found_shapetransformations = Some(transformations.clone());
                    ambiguous_count += 1;
                },
                _ => continue
            }
        }
        if ambiguous_count > 1 {
            return Err(anyhow::anyhow!("Object is linked with multiple ShapeTransformations. Is supposed to be linked with only one ShapeTransformations."));
        }
        if let Some(transformations) = found_shapetransformations {
            return Ok(transformations);
        }
        Err(anyhow::anyhow!("Object is not linked with a ShapeTransformations. Is supposed to be linked with only one ShapeTransformations."))
    }

    fn find_shapetype_changes_between_input_and_output(&mut self, task: &Task, pair: &Pair, input_image_nodeindex: NodeIndex, output_image_nodeindex: NodeIndex, connectivity: PixelConnectivity) -> anyhow::Result<()> {
        if pair.pair_type == PairType::Test {
            return Ok(());
        }

        if !task.is_output_size_same_as_input_size() {
            return Ok(());
        }

        let input_pixel_nodeindex_hashmap: HashMap<(u8, u8), NodeIndex> = self.pixel_nodeindexes_from_image(input_image_nodeindex)?;
        let output_pixel_nodeindex_hashmap: HashMap<(u8, u8), NodeIndex> = self.pixel_nodeindexes_from_image(output_image_nodeindex)?;
        
        let size: ImageSize = pair.input.image.size();

        let mut same_color_object_pairs: HashSet<(usize, usize)> = HashSet::new();

        // loop over the pixels in both input and output image.
        // if the pixels are different, then continue.
        // we now have 2 pixels wit the same color value.
        // obtain the object id for the pixels.
        // obtain the shape id for the objects.
        // insert an edge (ObjectChangeShape) between the 2 object nodes.
        for y in 0..size.height {
            for x in 0..size.width {
                let pixel_nodeindex0: NodeIndex = match input_pixel_nodeindex_hashmap.get(&(x, y)) {
                    Some(nodeindex) => *nodeindex,
                    None => continue
                };
                let pixel_nodeindex1: NodeIndex = match output_pixel_nodeindex_hashmap.get(&(x, y)) {
                    Some(nodeindex) => *nodeindex,
                    None => continue
                };

                let color0: u8 = self.get_color_of_pixel(pixel_nodeindex0)?;
                let color1: u8 = self.get_color_of_pixel(pixel_nodeindex1)?;
                let same_color: bool = color0 == color1;

                if !same_color {
                    continue;
                }

                let object_nodeindex0: NodeIndex = self.get_object_from_pixel(pixel_nodeindex0, connectivity)
                    .context("find_shapetype_changes_between_input_and_output object_nodeindex0")?;

                let object_nodeindex1: NodeIndex = self.get_object_from_pixel(pixel_nodeindex1, connectivity)
                    .context("find_shapetype_changes_between_input_and_output object_nodeindex1")?;
        
                let value0: usize = object_nodeindex0.index();
                let value1: usize = object_nodeindex1.index();
                let value_min: usize = value0.min(value1);
                let value_max: usize = value0.max(value1);
                same_color_object_pairs.insert((value_min, value_max));
            }
        }
        // println!("connectivity: {:?} object_pairs: {:?}", connectivity, same_color_object_pairs.len());

        for (value_min, value_max) in same_color_object_pairs {
            let nodeindex0: NodeIndex = NodeIndex::new(value_min);
            let nodeindex1: NodeIndex = NodeIndex::new(value_max);

            let shapetype0: ShapeType = self.get_shapetype_from_object(nodeindex0)?;
            let shapetype1: ShapeType = self.get_shapetype_from_object(nodeindex1)?;
            if shapetype0 == shapetype1 {
                continue;
            }
            let edge = EdgeData::ObjectChangeShape { shape_type_input: shapetype0, shape_type_output: shapetype1 };
            self.graph.add_edge(nodeindex0, nodeindex1, edge);
        }

        Ok(())
    }

    /// Establish edges between the input pixels and the output pixels.
    /// 
    /// This is only relevant when the task uses same size for input and output.
    fn find_pixel_color_changes_between_input_and_output(&mut self, task: &Task, pair: &Pair, input_image_nodeindex: NodeIndex, output_image_nodeindex: NodeIndex) -> anyhow::Result<()> {
        if pair.pair_type == PairType::Test {
            return Ok(());
        }

        if !task.is_output_size_same_as_input_size() {
            return Ok(());
        }
        
        let input_pixel_nodeindex_hashmap: HashMap<(u8, u8), NodeIndex> = self.pixel_nodeindexes_from_image(input_image_nodeindex)?;
        let output_pixel_nodeindex_hashmap: HashMap<(u8, u8), NodeIndex> = self.pixel_nodeindexes_from_image(output_image_nodeindex)?;
        
        let size: ImageSize = pair.input.image.size();

        // loop over the pixels in both input and output image.
        // if the pixels are the same then insert: `EdgeData::PixelIdentical`
        // if the pixels are different, then insert: `EdgeData::PixelColorChange`
        for y in 0..size.height {
            for x in 0..size.width {
                let pixel_nodeindex0: NodeIndex = match input_pixel_nodeindex_hashmap.get(&(x, y)) {
                    Some(nodeindex) => *nodeindex,
                    None => continue
                };
                let pixel_nodeindex1: NodeIndex = match output_pixel_nodeindex_hashmap.get(&(x, y)) {
                    Some(nodeindex) => *nodeindex,
                    None => continue
                };

                let color0: u8 = self.get_color_of_pixel(pixel_nodeindex0)?;
                let color1: u8 = self.get_color_of_pixel(pixel_nodeindex1)?;
                let same_color: bool = color0 == color1;

                if same_color {
                    let edge_data = EdgeData::PixelIdentical;
                    self.graph.add_edge(pixel_nodeindex0, pixel_nodeindex1, edge_data);
                    self.graph.add_edge(pixel_nodeindex1, pixel_nodeindex0, edge_data);
                } else{
                    let edge_data = EdgeData::PixelChangeColor { color_input: color0, color_output: color1 };
                    self.graph.add_edge(pixel_nodeindex0, pixel_nodeindex1, edge_data);
                    self.graph.add_edge(pixel_nodeindex1, pixel_nodeindex0, edge_data);
                }
            }
        }

        Ok(())
    }

    /// Determine which objects are touching each other.
    /// 
    /// Establishes `ObjectTouching` edges between these objects.
    fn determine_what_objects_are_touching(&mut self, image_nodeindex: NodeIndex, connectivity: PixelConnectivity) -> anyhow::Result<()> {
        // Future experiments:
        // Parameter for treating a particular color as transparent, so these pixels does not count as touching.
        // Parameter for considering diagonal pixels as touching.

        // println!("determine_what_objects_are_touching image_nodeindex: {:?}", image_nodeindex);

        let image_size: ImageSize;
        match &self.graph[image_nodeindex] {
            NodeData::Image { size, image_type: _ } => { image_size = *size; },
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Image"));
            }
        }

        let mut position_to_object: HashMap<(u8, u8), NodeIndex> = HashMap::new();

        for edge_image in self.graph.edges(image_nodeindex) {
            let pixel_index: NodeIndex = edge_image.target();
            match &self.graph[pixel_index] {
                NodeData::Pixel => {},
                _ => continue
            }

            // Obtain coordinates (x, y) and object index for this pixel.
            let mut found_object: Option<NodeIndex> = None;
            let mut found_x: Option<u8> = None;
            let mut found_y: Option<u8> = None;
            for edge_pixel in self.graph.edges(pixel_index) {
                let node_index: NodeIndex = edge_pixel.target();
                match &self.graph[node_index] {
                    NodeData::Object { connectivity: node_connectivity } => {
                        if *node_connectivity != connectivity {
                            continue;
                        }
                        found_object = Some(node_index);
                    },
                    NodeData::PositionX { x } => { found_x = Some(*x); },
                    NodeData::PositionY { y } => { found_y = Some(*y); },
                    _ => continue
                }
            }

            let (x, y, object_index) = match (found_x, found_y, found_object) {
                (Some(x), Some(y), Some(object_index)) => (x, y, object_index),
                _ => continue
            };
            position_to_object.insert((x, y), object_index);
        }
        // println!("position_to_object: {:?}", position_to_object);

        let mut object_touching_object: HashSet<(NodeIndex, NodeIndex)> = HashSet::new();

        // Determine if two objects are connected, going from left to right.
        for y in 0..image_size.height {
            for x in 0..image_size.width {
                let object_index0: NodeIndex = match position_to_object.get(&(x, y)) {
                    Some(object_index) => *object_index,
                    None => continue
                };
                let object_index1: NodeIndex = match position_to_object.get(&((x + 1), y)) {
                    Some(object_index) => *object_index,
                    None => continue
                };
                let min_index: NodeIndex = object_index0.min(object_index1);
                let max_index: NodeIndex = object_index0.max(object_index1);
                if min_index == max_index {
                    continue;
                }
                object_touching_object.insert((min_index, max_index));
            }
        }

        // Determine if two objects are connected, going from top to bottom.
        for y in 0..image_size.height {
            for x in 0..image_size.width {
                let object_index0: NodeIndex = match position_to_object.get(&(x, y)) {
                    Some(object_index) => *object_index,
                    None => continue
                };
                let object_index1: NodeIndex = match position_to_object.get(&(x, (y + 1))) {
                    Some(object_index) => *object_index,
                    None => continue
                };
                let min_index: NodeIndex = object_index0.min(object_index1);
                let max_index: NodeIndex = object_index0.max(object_index1);
                if min_index == max_index {
                    continue;
                }
                object_touching_object.insert((min_index, max_index));
            }
        }

        // println!("object_touching_object: {:?}", object_touching_object.len());

        for (object_index0, object_index1) in object_touching_object {
            let edge_data = EdgeData::ObjectTouching { connectivity };
            self.graph.add_edge(object_index0, object_index1, edge_data);
            self.graph.add_edge(object_index1, object_index0, edge_data);
        }
        Ok(())
    }

    fn populate_with_pair(&mut self, task: &Task, pair: &Pair, task_node_index: NodeIndex) -> anyhow::Result<()> {
        let pair_node_index: NodeIndex;
        {
            let node = NodeData::Pair { pair_index: pair.pair_index };
            pair_node_index = self.graph.add_node(node);
            self.graph.add_edge(task_node_index, pair_node_index, EdgeData::Child);
            self.graph.add_edge(pair_node_index, task_node_index, EdgeData::Parent);
        }

        let image_input_node_index: NodeIndex = match self.add_image(&pair.input.image, ImageType::Input) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("pair[{}].input.image cannot add image. {:?}", pair.pair_index, error));
            }
        };
        // println!("image_input_node_index: {:?}", image_input_node_index);
        self.graph.add_edge(pair_node_index, image_input_node_index, EdgeData::Child);
        self.graph.add_edge(image_input_node_index, pair_node_index, EdgeData::Parent);

        let image_output_node_index: NodeIndex = match self.add_image(&pair.output.image, ImageType::Output) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("pair[{}].output.image cannot add image. {:?}", pair.pair_index, error));
            }
        };
        // println!("image_output_node_index: {:?}", image_output_node_index);
        self.graph.add_edge(pair_node_index, image_output_node_index, EdgeData::Child);
        self.graph.add_edge(image_output_node_index, pair_node_index, EdgeData::Parent);

        {
            let id: String = pair.id_input_image();
            let node = NodeData::Id { id };
            let id_node_index: NodeIndex = self.graph.add_node(node);
            self.graph.add_edge(id_node_index, image_input_node_index, EdgeData::Link);
        }

        {
            let id: String = pair.id_output_image();
            let node = NodeData::Id { id };
            let id_node_index: NodeIndex = self.graph.add_node(node);
            self.graph.add_edge(id_node_index, image_output_node_index, EdgeData::Link);
        }

        self.find_pixel_color_changes_between_input_and_output(
            task,
            pair,
            image_input_node_index, 
            image_output_node_index, 
        )?;

        let connectivity_vec = vec![PixelConnectivity::Connectivity4, PixelConnectivity::Connectivity8];
        for connectivity in connectivity_vec {
            let input_process_shapes: ProcessShapes = self.process_shapes(
                image_input_node_index, 
                "input", 
                &pair.input.image_meta.single_color_object, 
                connectivity
            )?;
            let output_process_shapes: ProcessShapes = self.process_shapes(
                image_output_node_index, 
                "output", 
                &pair.output.image_meta.single_color_object, 
                connectivity
            )?;

            // Future experiments:
            // Determine if an object is contained inside another object.
            // Example ARC task 776ffc46, where the plus sign is inside a grey box.
            // Example ARC task 6b9890af, where the output image is box with an object scaled up to fill the box.

            // Determine if an object is touching another object.
            // Example ARC task 48d8fb45, where the object that is to be extracted is touching a grey pixel.
            self.determine_what_objects_are_touching(image_input_node_index, connectivity)?;
            self.determine_what_objects_are_touching(image_output_node_index, connectivity)?;

            // Determine what objects are occurring multiple times.
            self.find_similar_objects_inside_image(&input_process_shapes)?;
            if pair.pair_type == PairType::Train {
                self.find_similar_objects_inside_image(&output_process_shapes)?;
            }

            self.compare_objects_between_input_and_output(
                task,
                pair,
                &input_process_shapes, 
                &output_process_shapes, 
                connectivity
            )?;

            self.find_shapetype_changes_between_input_and_output(
                task,
                pair,
                image_input_node_index, 
                image_output_node_index, 
                connectivity
            )?;

            // Future experiment:
            // When input size == output size.
            // Detect if there is an another shape type that is occupying the same area both in the input image and output image.
            // if so, then add an edge between the two shapes, since it may be a transformation of the input shape.
            // such as change from a solid rectangle to a hollow box, as in ARC task 4347f46a
        }

        Ok(())
    }

    pub fn populate_with_task(&mut self, task: &Task) -> anyhow::Result<()> {
        let task_node_index: NodeIndex;
        {
            let node = NodeData::Task;
            task_node_index = self.graph.add_node(node);
        }

        for pair in &task.pairs {
            self.populate_with_pair(task, pair, task_node_index)?;
        }

        self.task = Some(task.clone());

        // Future experiments:
        // input images: Compare all the shapes with each other, are there any shapes that are similar?
        // output images: Compare all the shapes with each other, are there any shapes that are similar?
        // Determine what objects gets passed on from the input to the output.
        // Determine if an object gets inserted just once or multiple times into the output.
        // Determine if all the pairs use the same objects, or if each pair use its own objects.
        // Determine if a new object gets inserted into the output, that is not present in the input.
        // Determine if an object gets removed from the input, that is not present in the output.
        // Determine if one/multiple object gets moved around and all the other objects stay in the same place.
        // Do they move in the same direction? Do they move the same distance? Do they align with each other?
        Ok(())
    }

    fn prompt_serializer(prompt_type: &PromptType) -> Box<dyn PromptSerialize> {
        match prompt_type {
            PromptType::Compact => Box::new(PromptCompactSerializer),
            PromptType::Position => Box::new(PromptPositionSerializer),
            PromptType::RunLengthEncoding => Box::new(PromptRLESerializer),
            PromptType::ShapeAndTransformConnectivity4 => Box::new(PromptShapeTransformSerializer::new_connectivity4()),
            PromptType::ShapeAndTransformConnectivity8 => Box::new(PromptShapeTransformSerializer::new_connectivity8()),
        }
    }

    pub fn to_prompt(&self, prompt_type: &PromptType) -> anyhow::Result<String> {
        let t: Box<dyn PromptSerialize> = Self::prompt_serializer(prompt_type);
        t.to_prompt(&self)
    }

    /// When `Connectivity4` is specified, then it's only shapes that are connected via the 4 pixels above, below, left and right.
    /// 
    /// When `Connectivity8` is specified, then it's shapes that are connected via all the surrounding 8 pixels.
    pub fn get_object_nodeindex_vec(&self, pair_index: u8, image_type: ImageType, connectivity: PixelConnectivity) -> anyhow::Result<Vec<NodeIndex>> {
        // Find the ObjectsInsideImage for the current pair's input image.
        let objectsinsideimage_nodeindex: NodeIndex = self.get_objectsinsideimage_for_pair(pair_index, image_type, connectivity)?;
        let mut object_nodeindex_vec = Vec::<NodeIndex>::new();
        for edge in self.graph.edges(objectsinsideimage_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::Object { connectivity: _ } => {
                    object_nodeindex_vec.push(node_index);
                },
                _ => {}
            }
        }

        // Ignore objects where the background color is black.
        let mut object_nodeindex_vec2 = Vec::<NodeIndex>::new();
        for object_nodeindex in &object_nodeindex_vec {
            for edge in self.graph.edges(*object_nodeindex) {
                let node_index: NodeIndex = edge.target();
                match &self.graph[node_index] {
                    NodeData::Color { color } => {
                        if *color == 0 {
                            continue;
                        }
                        object_nodeindex_vec2.push(*object_nodeindex);
                    },
                    _ => {}
                }
            }
        }

        Ok(object_nodeindex_vec2)
    }
}

struct ProcessShapes {
    #[allow(dead_code)]
    objectsinsideimage_index: NodeIndex,

    color_and_shape_vec: Vec<ColorAndShape>,
    color_and_shape_to_object_nodeindex: HashMap<usize, NodeIndex>,
}

impl ProcessShapes {
    fn shapetype_set(&self) -> HashSet<ShapeType> {
        let mut shapetype_set = HashSet::<ShapeType>::new();
        for color_and_shape in &self.color_and_shape_vec {
            shapetype_set.insert(color_and_shape.shape_identification.shape_type.clone());
        }
        shapetype_set
    }

    fn obtain_color_and_shape_vec(&self, filter_shapetype: &ShapeType) -> Vec<(usize, &ColorAndShape)> {
        let mut items = Vec::<(usize, &ColorAndShape)>::new();
        for (index, color_and_shape) in self.color_and_shape_vec.iter().enumerate() {
            if color_and_shape.shape_identification.shape_type == *filter_shapetype {
                items.push((index, color_and_shape));
            }
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_add_image_and_to_image() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 5, 5,
            0, 1, 3, 3,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let mut instance = TaskGraph::new();

        // Act
        let image_index: NodeIndex = instance.add_image(&input, ImageType::Input).expect("NodeIndex");
        let actual: Image = instance.to_image(image_index).expect("Image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 5, 5,
            0, 1, 3, 3,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_compare_input_output() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 0, 1, 0,
            1, 1, 0, 1,
            1, 0, 1, 1,
            1, 0, 0, 0,
        ];
        let image0: Image = Image::try_create(4, 4, pixels0).expect("image");
        let pixels1: Vec<u8> = vec![
            1, 0, 0, 0,
            1, 0, 0, 0,
            1, 0, 1, 1,
            1, 1, 1, 1,
        ];
        let image1: Image = Image::try_create(4, 4, pixels1).expect("image");
        let mut instance = TaskGraph::new();

        // Act
        let image_index0: NodeIndex = instance.add_image(&image0, ImageType::Input).expect("NodeIndex");
        let _image_index1: NodeIndex = instance.add_image(&image1, ImageType::Output).expect("NodeIndex");

        // Future experiment
        // compare both images, update metadata

        let actual: Image = instance.to_image(image_index0).expect("Image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 1, 0,
            1, 1, 0, 1,
            1, 0, 1, 1,
            1, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
