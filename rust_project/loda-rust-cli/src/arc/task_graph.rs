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
use super::{Image, ImageSize, PixelConnectivity, SingleColorObject, ShapeType, ShapeIdentificationFromSingleColorObject, ColorAndShape};
use super::arc_work_model::{Task, Pair, PairType};
use petgraph::{stable_graph::{NodeIndex, EdgeIndex}, visit::EdgeRef};
use std::collections::{HashSet, HashMap};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum NodeData {
    Task,
    Id { id: String },
    Pair { pair_index: u8 },
    Image { size: ImageSize },
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
    ObjectSimilarity { numerator: u8, denominator: u8 },
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
}

impl TaskGraph {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            graph: petgraph::Graph::new(),
        }
    }

    #[allow(dead_code)]
    pub fn graph(&self) -> &petgraph::Graph<NodeData, EdgeData> {
        &self.graph
    }

    /// Returns the `NodeIndex` of the created image node.
    #[allow(dead_code)]
    pub fn add_image(&mut self, image: &Image) -> anyhow::Result<NodeIndex> {
        let node_image = NodeData::Image { size: image.size() };
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
            NodeData::Image { size } => *size,
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
            NodeData::Image { size } => *size,
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
        }

        let instance = ProcessShapes {
            objectsinsideimage_index,
            color_and_shape_vec: sifsco.color_and_shape_vec,
            color_and_shape_to_object_nodeindex,
        };
        Ok(instance)
    }

    fn compare_objects_between_input_and_output(&mut self, input_process_shapes: &ProcessShapes, output_process_shapes: &ProcessShapes, connectivity: PixelConnectivity) -> anyhow::Result<()> {
        let verbose = false;
        if verbose {
            println!("connectivity: {:?}", connectivity);
        }

        let _input_objectsinsideimage_index: NodeIndex = input_process_shapes.objectsinsideimage_index;
        let _output_objectsinsideimage_index: NodeIndex = output_process_shapes.objectsinsideimage_index;

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

            let input_items: Vec<(usize, &ColorAndShape)> = input_process_shapes.obtain_color_and_shape_vec(shapetype);
            let output_items: Vec<(usize, &ColorAndShape)> = output_process_shapes.obtain_color_and_shape_vec(shapetype);

            if verbose {
                println!("shapetype: {:?}  compare {} input shapes with {} output shapes", shapetype, input_items.len(), output_items.len());
            }

            let number_of_comparisons: usize = input_items.len() * output_items.len();
            if number_of_comparisons > 30 {
                if verbose {
                    println!("too many comparisons, skipping");
                }
                continue;
            }

            for (input_index, input_color_and_shape) in input_items {
                for (output_index, output_color_and_shape) in &output_items {
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

                    let size0: ImageSize = input_color_and_shape.shape_identification.rect.size();
                    let size1: ImageSize = output_color_and_shape.shape_identification.rect.size();
                    let same_width: bool = size0.width == size1.width;
                    let same_height: bool = size0.height == size1.height;
                    let same_transformations: bool = input_color_and_shape.shape_identification.transformations == output_color_and_shape.shape_identification.transformations;

                    // for very similar objects, then check if the mask pixel data is identical.
                    let mut same_mask: bool = false;
                    if same_width && same_height && same_transformations && same_mass {
                        if verbose {
                            println!("same_width: {}  same_height: {}  same_transformations: {}", same_width, same_height, same_transformations);
                        }
                        let mask0: &Image = &input_color_and_shape.shape_identification.mask_cropped;
                        let mask1: &Image = &output_color_and_shape.shape_identification.mask_cropped;
                        same_mask = mask0 == mask1;
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
                        continue;
                    }
                    let nodeindex0: Option<&NodeIndex> = input_process_shapes.color_and_shape_to_object_nodeindex.get(&input_index);
                    let nodeindex1: Option<&NodeIndex> = output_process_shapes.color_and_shape_to_object_nodeindex.get(&output_index);
                    if let (Some(nodeindex0), Some(nodeindex1)) = (nodeindex0, nodeindex1) {
                        if verbose {
                            println!("  adding edge between input and output objects");
                        }
                        let edge_data = EdgeData::ObjectSimilarity { numerator: numerator_u8, denominator: denominator_u8 };
                        self.graph.add_edge(*nodeindex0, *nodeindex1, edge_data);
                        self.graph.add_edge(*nodeindex1, *nodeindex0, edge_data);
                    }
                }
            }
        }

        Ok(())
    }

    fn populate_with_pair(&mut self, pair: &Pair, task_node_index: NodeIndex) -> anyhow::Result<()> {
        let pair_node_index: NodeIndex;
        {
            let node = NodeData::Pair { pair_index: pair.pair_index };
            pair_node_index = self.graph.add_node(node);
            self.graph.add_edge(task_node_index, pair_node_index, EdgeData::Child);
            self.graph.add_edge(pair_node_index, task_node_index, EdgeData::Parent);
        }

        let image_input_node_index: NodeIndex = match self.add_image(&pair.input.image) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("pair[{}].input.image cannot add image. {:?}", pair.pair_index, error));
            }
        };
        // println!("image_input_node_index: {:?}", image_input_node_index);
        self.graph.add_edge(pair_node_index, image_input_node_index, EdgeData::Child);
        self.graph.add_edge(image_input_node_index, pair_node_index, EdgeData::Parent);

        let image_output_node_index: NodeIndex = match self.add_image(&pair.output.image) {
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

            if pair.pair_type == PairType::Train {
                self.compare_objects_between_input_and_output(
                    &input_process_shapes, 
                    &output_process_shapes, 
                    connectivity
                )?;
            }
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
            self.populate_with_pair(pair, task_node_index)?;
        }
        Ok(())
    }
}

struct ProcessShapes {
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
        let image_index: NodeIndex = instance.add_image(&input).expect("NodeIndex");
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
        let image_index0: NodeIndex = instance.add_image(&image0).expect("NodeIndex");
        let _image_index1: NodeIndex = instance.add_image(&image1).expect("NodeIndex");

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
