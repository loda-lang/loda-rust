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
use super::{Image, ImageSize, PixelConnectivity, SingleColorObject, ShapeType, ShapeIdentificationFromSingleColorObject, ColorAndShape, Rectangle, ShapeTransformation};
use super::arc_work_model::{Task, Pair, PairType};
use super::natural_language::FieldId;
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
    ShapeScale { x: u8, y: u8 },
    ShapeSize { width: u8, height: u8 },
    ShapeTransformations { transformations: Vec<ShapeTransformation> },
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

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TaskGraph {
    graph: petgraph::Graph<NodeData, EdgeData>,
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
    pub fn graph(&self) -> &petgraph::Graph<NodeData, EdgeData> {
        &self.graph
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
                return Err(anyhow::anyhow!("expected NodeData::Pixel"));
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
            return Err(anyhow::anyhow!("Pixel is linked with multiple objects. Is supposed to be linked with only one object."));
        }
        if let Some(nodeindex) = found_object_nodeindex {
            return Ok(nodeindex);
        }
        Err(anyhow::anyhow!("Pixel is not linked with an object. Is supposed to be linked with only one object."))
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

    fn get_shapetype_from_object(&self, object_nodeindex: NodeIndex, connectivity: PixelConnectivity) -> anyhow::Result<ShapeType> {
        match &self.graph[object_nodeindex] {
            NodeData::Object { connectivity: the_connectivity } => { 
                if *the_connectivity != connectivity {
                    return Err(anyhow::anyhow!("connectivity mismatch"));
                }
            },
            _ => { 
                return Err(anyhow::anyhow!("expected NodeData::Pixel"));
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
            return Err(anyhow::anyhow!("Object is linked with multiple shapetypes. Is supposed to be linked with only one shapetype."));
        }
        if let Some(shapetype) = found_shapetype {
            return Ok(shapetype);
        }
        Err(anyhow::anyhow!("Object is not linked with a shapetype. Is supposed to be linked with only one shapetype."))
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

                let object_nodeindex0: NodeIndex = self.get_object_from_pixel(pixel_nodeindex0, connectivity)?;
                let object_nodeindex1: NodeIndex = self.get_object_from_pixel(pixel_nodeindex1, connectivity)?;
        
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

            let shapetype0: ShapeType = self.get_shapetype_from_object(nodeindex0, connectivity)?;
            let shapetype1: ShapeType = self.get_shapetype_from_object(nodeindex1, connectivity)?;
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

    pub fn to_prompt(&self) -> anyhow::Result<String> {
        let task: &Task = match &self.task {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("graph is not initialized with a task"));
            }
        };

        let mut rows = Vec::<String>::new();

        rows.push("I'm doing Prolog experiments.\n\n".to_string());
        // rows.push("I'm doing Prolog experiments with grids.\n\n".to_string());
        // rows.push("I'm doing Prolog experiments and you are going to emit a prolog program with the predicted output objects.\n\n".to_string());
        // rows.push("You are a Prolog expert developer.\n\n".to_string());
        
        rows.push("The grid is 1-indexed.\n\n".to_string());
        // rows.push("The grid is 1-indexed and does allow negative indices and coordinates outside the grid size. The top left corner has coordinate x=1, y=1.\n\n".to_string());

        // rows.push("The coordinates are topleftx_toplefty_bottomrightx_bottomrighty.\n\n".to_string());
        // rows.push("The coordinates are provided as tlX_Y_brX_Y where tl is topleft and br is bottomright.\n\n".to_string());
        // rows.push("The coordinates are TLBR formatted, tY_lX_bY_rX where t=top l=left b=bottom r=right.\n\n".to_string());
        rows.push("Top-Left Bottom-Right (TLBR) is used for object bounding boxes, like tY_lX_bY_rX where t=top l=left b=bottom r=right.\n\n".to_string());
        // rows.push("Top-Left Bottom-Right (TLBR) is used for object rectangles, like tY_lX_bY_rX where t=top l=left b=bottom r=right.\n\n".to_string());

        rows.push("The width of the object bounding box has a 'w' prefix, like 'w5' is width=5.".to_string());
        rows.push("The height of the object bounding box has a 'h' prefix, like 'h3' is height=3.\n".to_string());
        // rows.push("The x coordinates has this relationship: left + width - 1 = right.".to_string());
        // rows.push("The y coordinates has this relationship: top + height - 1 = bottom.\n\n".to_string());

        rows.push("The number of solid pixels in the object has a 'm' prefix, like 'm12' is mass=12.\n".to_string());

        // rows.push("The `id` prefixed text has no integer value and should not be considered. The number of unique IDs may be relevant. The mass of each ID is sometimes preserved in the output.".to_string());
        rows.push("The `id` prefixed text has no integer value and should not be considered for sorting. The number of unique IDs may be relevant. The total mass of certain IDs is sometimes preserved in the output.".to_string());

        // rows.push("No rectangle overlap with other rectangles.\n\n".to_string());

        // rows.push("There number of output objects may be different than the input objects.".to_string());
        // rows.push("The output objects may have to be sorted by coordinates or mass or some other property.".to_string());
        // rows.push("The output objects have experience gravity towards other objects. An object with a specific `id` may attract other objects.".to_string());

        rows.push("```prolog".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let pair_index_u8: u8 = pair_index.min(255) as u8;
            let natural_language_pair_index: String = format!("{}", pair_index + 1);
            if pair.pair_type == PairType::Test {
                continue;
            }
            if pair_index > 0 {
                rows.push("".to_string());
            }

            {
                let size: ImageSize = pair.input.image.size();
                let s = format!("% Example {} input grid_width{}_height{}", natural_language_pair_index, size.width, size.height);
                rows.push(s);
            }

            {
                let object_nodeindex_vec: Vec::<NodeIndex> = self.get_object_nodeindex_vec(pair_index_u8, ImageType::Input)?;
                for object_nodeindex in &object_nodeindex_vec {
                    let s0: String = self.natural_language_of_object(*object_nodeindex)?;
                    let s1: String = format!("object(input{}_{}).", natural_language_pair_index, s0);
                    rows.push(s1);
                }        
            }

            rows.push("".to_string());
            {
                let size: ImageSize = pair.output.image.size();
                let s = format!("% Example {} output grid_width{}_height{}", natural_language_pair_index, size.width, size.height);
                rows.push(s);
            }

            {
                let object_nodeindex_vec: Vec::<NodeIndex> = self.get_object_nodeindex_vec(pair_index_u8, ImageType::Output)?;
                for object_nodeindex in &object_nodeindex_vec {
                    let s0: String = self.natural_language_of_object(*object_nodeindex)?;
                    let s1: String = format!("object(output{}_{}).", natural_language_pair_index, s0);
                    rows.push(s1);
                }        
            }
        }
        rows.push("```".to_string());
        rows.push("".to_string());
        rows.push("".to_string());
        // rows.push("\n\nWhat example has the biggest number of columns?".to_string());
        // rows.push("\n\nWhat are the transformations across all the examples, that goes from the input to the output?".to_string());
        // rows.push("The shapeRectangle is solid and cannot overlap with other objects. Create more shapeRectangle objects in order to ensure no overlap.".to_string());
        // rows.push("There number of output objects may be different than the input objects.".to_string());

        // rows.push("Assumptions:".to_string());
        // rows.push("- Assume that shapeRectangle is solid and have no holes.".to_string());
        // rows.push("- Assume that shapeBox has 1 rectangular hole.".to_string());
        // rows.push("- Assume that shapeBoxWithTwoHoles has a middle separator and 2 rectangular holes.".to_string());
        // rows.push("".to_string());

        rows.push("There number of output objects can be different than the input objects. Also consider the rules with clockwise rotation.".to_string());
        rows.push("Check if a condition is satisfied only for objects with a certain shape.".to_string());

        rows.push("A shape can occlude another shape, so shapeL may appear as shapeRectangle. Sometimes it's the occluded object that gets transformed.".to_string());
        rows.push("Consider both euclidian distance and manhatten distance between objects.".to_string());
        rows.push("Check how much an object moves relative x, y.".to_string());
        rows.push("Check how IDs gets swapped. Don't invent new IDs.".to_string());
        // rows.push("The output objects may have to be sorted by coordinates or mass or some other property.".to_string());
        rows.push("Objects that stay stationary may be a useful landmark. A landmark may be a starting point for inserting a new object next to. Check that the examples have landmarks.\n\n".to_string());
        rows.push("Check if objects are aligned to a certain edge. Check if the mass is preserved. Count the number of object groups.".to_string());
        rows.push("Check if all the output objects agree on the same shape.".to_string());
        rows.push("Check if the shape may be a shortest path drawn between two landmarks and navigates around obstacle objects.".to_string());
        rows.push("For the objects that make it to the output, check if their shape is preserved.".to_string());
        rows.push("A line that goes from edge to edge, some condition may be satisfied above the line, but not below it.".to_string());
        // rows.push("Consider what side an object is touching another object, maybe in the output the touch points are different.".to_string());
        rows.push("Consider where two objects are touching, maybe the touch points are different in the output, such as moving from left side to the opposite side.".to_string());
        rows.push("Consider the touch point may be at the center of another object, so objects can be moved to the center point.".to_string());
        // rows.push("Transformations: sort, gravity towards, rotate, flipx, flipy, move, merge objects, split objects and so on.".to_string());
        // rows.push("Transformations: sort, gravity towards, rotate, flipx, flipy, move, merge objects, split objects, extract object, fit object inside another object, and so on.".to_string());
        rows.push("Transformations: sort, gravity towards, rotate, flipx, flipy, move, merge objects, split objects, extract object, fit object inside another object, layout 2..5 objects in a direction while keeping the objects aligned and evenly spaced (remember to update tlbr coordinates). The transformation can be anything, it just have to be simple.".to_string());
        // rows.push("Check for splitview, sometimes the examples have a line spanning from edge to edge, dividing the grid into two halfs. Compare properties between the halfs and determine what properties are relevant.\n\n".to_string());
        // rows.push("\n\nThink step by step, what are the transformations across all the examples, that goes from the input to the output. Explain your reasoning for inserting new objects.".to_string());

        rows.push("\n\n# Task A".to_string());
        rows.push("Think step by step, what does the examples output objects have in common. Check if they are all horizontal lines. Are they sorted in a particular way.".to_string());

        rows.push("\n\n# Task B".to_string());
        rows.push("Think step by step, what are the transformations across all the examples, that goes from the input to the output. Write down your observations.".to_string());
        rows.push("Explain your reasoning for inserting new objects.".to_string());
        rows.push("Explain your reasoning for deleting existing objects.".to_string());
        // rows.push("or the predicted output the object ordering dictates the drawing order, the first output objects gets drawn first.".to_string());
        
        rows.push("\n\n# Task C".to_string());
        rows.push("Think step by step about the orientation of the output objects. Does it make sense to layout the objects in another direction. Update the object coordinates accordingly.".to_string());

        rows.push("\n\n# Task D".to_string());
        rows.push("With the following example, I want you to predict what the output should be. Print your reasoning before the prolog code.\n\n".to_string());
        rows.push("```prolog".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let pair_index_u8: u8 = pair_index.min(255) as u8;
            let natural_language_pair_index: String = format!("{}", pair_index + 1);
            if pair.pair_type == PairType::Train {
                continue;
            }

            // Future experiment:
            // How do I loop over all the pairs. Can I do it with a single prompt, that contains all the pairs?
            // Or do I have to do prompting for each pair individually?
            // if pair_index == 4 {
            //     continue;
            // }
            // if pair_index == 5 {
            //     continue;
            // }
            
            {
                let size: ImageSize = pair.input.image.size();
                let s = format!("% Example {} input grid_width{}_height{}", natural_language_pair_index, size.width, size.height);
                rows.push(s);
            }

            {
                let object_nodeindex_vec: Vec::<NodeIndex> = self.get_object_nodeindex_vec(pair_index_u8, ImageType::Input)?;
                for object_nodeindex in &object_nodeindex_vec {
                    let s0: String = self.natural_language_of_object(*object_nodeindex)?;
                    let s1: String = format!("object(input{}_{}).", natural_language_pair_index, s0);
                    rows.push(s1);
                }        
            }

            rows.push("".to_string());
            {
                let grid_size: String = match task.predict_output_size_for_pair(pair) {
                    Ok(size) => {
                        format!("width{}_height{}", size.width, size.height)
                    },
                    Err(_) => {
                        format!("widthPREDICT_heightPREDICT")
                    }
                };
                let s = format!("% Example {} output grid_{}", natural_language_pair_index, grid_size);
                rows.push(s);
            }

            {
                rows.push(format!("PREDICT the text that starts with object(output{}_", natural_language_pair_index));
            }

            // Future experiment:
            // process all the test pairs. Currently it's only 1 test pair.
            break;
        }
        rows.push("```".to_string());
        // rows.push("Repeat the previous example prolog code, with PREDICT replaced with your predictions. Print reasoning first followed by the prolog code block".to_string());
        // rows.push("Repeat the previous example prolog code, with PREDICT replaced with your predictions. Leave out the input from the prolog code.".to_string());
        // rows.push("The task for you: Repeat the previous example prolog code, with PREDICT replaced with your predictions. Leave out the input from the prolog code.".to_string());
        rows.push("Repeat the previous example prolog code, with PREDICT replaced with your predictions.".to_string());

        Ok(rows.join("\n"))
    }

    fn get_object_nodeindex_vec(&self, pair_index: u8, image_type: ImageType) -> anyhow::Result<Vec<NodeIndex>> {
        // Future experiment
        // Do prompting for connectivity4 and connectivity8, so there are 2 different prompts.

        // Find the ObjectsInsideImage { connectivity: Connectivity8 } for the current pair's input image.
        let objectsinsideimage_nodeindex: NodeIndex = self.get_objectsinsideimage_for_pair(pair_index, image_type, PixelConnectivity::Connectivity4)?;
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

    fn natural_language_of_object(&self, object_nodeindex: NodeIndex) -> anyhow::Result<String> {
        let mut found_position_x: Option<u8> = None;
        let mut found_position_y: Option<u8> = None;
        let mut found_shapesize: Option<ImageSize> = None;
        let mut found_mass: Option<u16> = None;
        let mut found_color: Option<u8> = None;
        let mut found_shapetype: Option<ShapeType> = None;
        let mut found_shapetransformations: Option<String> = None;
        let mut found_shapescale: Option<String> = None;
        for edge in self.graph.edges(object_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &self.graph[node_index] {
                NodeData::PositionX { x } => {
                    found_position_x = Some(*x);
                },
                NodeData::PositionY { y } => {
                    found_position_y = Some(*y);
                },
                NodeData::ShapeSize { width, height } => {
                    found_shapesize = Some(ImageSize::new(*width, *height));
                },
                NodeData::Mass { mass } => {
                    found_mass = Some(*mass);
                },
                NodeData::Color { color } => {
                    found_color = Some(*color);
                },
                NodeData::ShapeType { shape_type } => {
                    found_shapetype = Some(*shape_type);
                },
                NodeData::ShapeTransformations { transformations } => {
                    if transformations.len() == 8 {
                        found_shapetransformations = Some("all".to_string());
                        continue;
                    }
                    let items: Vec<String> = transformations.iter().map(|t| t.natural_language_name().to_string()).collect::<Vec<String>>();
                    let s = format!("{}", items.join("_"));
                    found_shapetransformations = Some(s);
                },
                NodeData::ShapeScale { x, y } => {
                    let s = format!("scalex{}_scaley{}", x, y);
                    found_shapescale = Some(s);
            },
                _ => {}
            }
        }
        let mut items = Vec::<String>::new();
        if let Some(color) = found_color {
            let obfuscated_color: String = FieldId::id_from_value(color);
            items.push(obfuscated_color);
        }
        if let Some(position_x) = found_position_x {
            // items.push(format!("x{}", position_x));
            // items.push(format!("{}", position_x + 1));
        }
        if let Some(position_y) = found_position_y {
            // items.push(format!("y{}", position_y));
            // items.push(format!("{}", position_y + 1));
        }
        if let Some(size) = found_shapesize {
            // items.push(format!("width{}_height{}", size.width, size.height));
            // let x: i32 = size.width as i32 - 1;
            // let y: i32 = size.height as i32 - 1;
            // items.push(format!("{}_{}", x, y));
            // items.push(format!("{}_{}", size.width, size.height));
        }
        match (found_position_x, found_position_y, found_shapesize) {
            (Some(x), Some(y), Some(size)) => {
                // items.push(format!("coord{}_{}_{}_{}", x + 1, y + 1, x + size.width, y + size.height));
                // items.push(format!("tl{}_{}_br{}_{}", x + 1, y + 1, x + size.width, y + size.height));
                items.push(format!("t{}_l{}_b{}_r{}", y + 1, x + 1, y + size.height, x + size.width));
                items.push(format!("w{}_h{}", size.width, size.height));
            },
            _ => {}
        }
        if let Some(mass) = found_mass {
            items.push(format!("m{}", mass));
        }
        if let Some(shapetype) = found_shapetype {
            items.push(format!("shape{:?}", shapetype));
        }
        if let Some(shapescale) = found_shapescale {
            items.push(shapescale);
        } else {
            items.push(format!("scaleUnknown"));
        }
        let mut s = String::new();
        s += &items.join("_");
        if let Some(shapetransformations) = found_shapetransformations {
            s += &format!(", transform({})", shapetransformations);
        }
        Ok(s)
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
