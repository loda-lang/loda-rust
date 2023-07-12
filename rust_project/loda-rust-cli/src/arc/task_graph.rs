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
use super::{Image, ImageSize, Histogram, PixelConnectivity, SingleColorObject, ImageHistogram, ShapeType};
use super::{ImageMask, ShapeIdentification};
use super::arc_work_model::{Task, Pair};
use petgraph::{stable_graph::{NodeIndex, EdgeIndex}, visit::EdgeRef};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum NodeData {
    Task,
    Id { id: String },
    Pair { pair_index: u8 },
    Image { size: ImageSize },
    Pixel,
    Color { color: u8 },
    PositionX { x: u8 },
    PositionY { y: u8 },
    Object { connectivity: PixelConnectivity },
    ShapeType { shape_type: ShapeType },
    // Input,
    // Output,
    // PairTrain,
    // PairTest,
    // PairInputImage,
    // PairOutputImage,
    // PositionReverseX { x: u8 },
    // PositionReverseY { y: u8 },
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

    fn process_shapes_inner(&mut self, image_index: NodeIndex, name: &str, sco: &SingleColorObject, connectivity: PixelConnectivity) -> anyhow::Result<()> {
        for color in 0..=9 {
            let enumerated_objects: Image = match sco.enumerate_clusters(color, connectivity) {
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
                let object_index: NodeIndex = match self.add_object(image_index, &shape_id.mask_uncropped, connectivity) {
                    Ok(value) => value,
                    Err(error) => {
                        println!("unable to add object to graph. error: {:?}", error);
                        continue;
                    }
                };
                println!("name: {} object_index: {:?}", name, object_index);

                {
                    let property = NodeData::ShapeType { shape_type: shape_id.shape_type.clone() };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(object_index, index, EdgeData::Link);
                }

                {
                    let property = NodeData::Color { color };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(object_index, index, EdgeData::Link);
                }
            }
        }
        Ok(())
    }

    fn process_shapes(&mut self, image_index: NodeIndex, name: &str, sco: &Option<SingleColorObject>) {
        let connectivity: PixelConnectivity = PixelConnectivity::Connectivity8;
        let sco: &SingleColorObject = match sco {
            Some(value) => value,
            None => {
                println!("{}: no sco", name);
                return;
            }
        };
        match self.process_shapes_inner(image_index, name, sco, connectivity) {
            Ok(()) => {},
            Err(error) => {
                println!("{}: unable to process shapes error: {:?}", name, error);
                return;
            }
        }
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
        println!("image_input_node_index: {:?}", image_input_node_index);
        self.graph.add_edge(pair_node_index, image_input_node_index, EdgeData::Child);
        self.graph.add_edge(image_input_node_index, pair_node_index, EdgeData::Parent);

        let image_output_node_index: NodeIndex = match self.add_image(&pair.output.image) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("pair[{}].output.image cannot add image. {:?}", pair.pair_index, error));
            }
        };
        println!("image_output_node_index: {:?}", image_output_node_index);
        self.graph.add_edge(pair_node_index, image_output_node_index, EdgeData::Child);
        self.graph.add_edge(image_output_node_index, pair_node_index, EdgeData::Parent);

        self.process_shapes(image_input_node_index, "input", &pair.input.image_meta.single_color_object);
        self.process_shapes(image_output_node_index, "output", &pair.output.image_meta.single_color_object);

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
