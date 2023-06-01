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
}

#[allow(dead_code)]
struct ExperimentWithPetgraph {
    graph: petgraph::Graph<NodeData, f32, petgraph::Undirected>,
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

        for y in 0..image.height() {
            for x in 0..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                let node_pixel = NodeData::Pixel;
                let index_pixel: NodeIndex = self.graph.add_node(node_pixel);
                self.graph.add_edge(index_image, index_pixel, 1.0);
                {
                    let property = NodeData::Color { color };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(index_pixel, index, 1.0);
                }
                {
                    let property = NodeData::PositionX { x };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(index_pixel, index, 1.0);
                }
                {
                    let property = NodeData::PositionY { y };
                    let index: NodeIndex = self.graph.add_node(property);
                    self.graph.add_edge(index_pixel, index, 1.0);
                }

                // edge to neighbor pixels.
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
