use super::{Image, ImageCompare, ImagePadding, ImageSize, ImageMaskCount};
use petgraph::stable_graph::NodeIndex;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum NodeData {
    Image,
    Pixel,
    Color { color: u8 },
    PositionX { x: u8 },
    PositionY { y: u8 },
    // PositionReverseX { x: u8 },
    // PositionReverseY { y: u8 },
    // PixelColumn,
    // PixelRow,
    // Object,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum PixelNeighborEdgeType {
    UpDown,
    LeftRight,
    // UpLeftDownRight,
    // UpRightDownLeft,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum EdgeData {
    Link,
    PixelNeighbor { edge_type: PixelNeighborEdgeType },
}

#[allow(dead_code)]
struct ExperimentWithPetgraph {
    graph: petgraph::Graph<NodeData, EdgeData, petgraph::Undirected>,
}

impl ExperimentWithPetgraph {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            graph: petgraph::Graph::new_undirected(),
        }
    }

    #[allow(dead_code)]
    fn add_image(&mut self, image: &Image) -> anyhow::Result<()> {
        let node_image = NodeData::Image;
        let index_image: NodeIndex = self.graph.add_node(node_image);

        let mut indexes_pixels: Vec<NodeIndex> = Vec::new();
        for y in 0..image.height() {
            for x in 0..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                let node_pixel = NodeData::Pixel;
                let index_pixel: NodeIndex = self.graph.add_node(node_pixel);
                self.graph.add_edge(index_image, index_pixel, EdgeData::Link);
                {
                    let property = NodeData::Color { color };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(index_pixel, index, EdgeData::Link);
                }
                {
                    let property = NodeData::PositionX { x };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(index_pixel, index, EdgeData::Link);
                }
                {
                    let property = NodeData::PositionY { y };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(index_pixel, index, EdgeData::Link);
                }

                indexes_pixels.push(index_pixel);
            }
        }

        // Establish horizontal edges between neighbor pixels.
        for y in 0..image.height() {
            for x in 1..image.width() {
                let x0: u8 = x - 1;
                let x1: u8 = x;
                let address0: usize = (y as usize) * (image.width() as usize) + (x0 as usize);
                let address1: usize = (y as usize) * (image.width() as usize) + (x1 as usize);
                let index0: NodeIndex = indexes_pixels[address0];
                let index1: NodeIndex = indexes_pixels[address1];
                self.graph.add_edge(index0, index1, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::LeftRight });
            }
        }

        // Establish vertical edges between neighbor pixels.
        for x in 0..image.width() {
            for y in 1..image.height() {
                let y0: u8 = y - 1;
                let y1: u8 = y;
                let address0: usize = (y0 as usize) * (image.width() as usize) + (x as usize);
                let address1: usize = (y1 as usize) * (image.width() as usize) + (x as usize);
                let index0: NodeIndex = indexes_pixels[address0];
                let index1: NodeIndex = indexes_pixels[address1];
                self.graph.add_edge(index0, index1, EdgeData::PixelNeighbor { edge_type: PixelNeighborEdgeType::UpDown });
            }
        }

        println!("graph: {:?}", self.graph);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_analyze() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let mut instance = ExperimentWithPetgraph::new();

        // Act
        instance.add_image(&input).expect("ok");
    }
}
