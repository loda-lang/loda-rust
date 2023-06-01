use super::{Image, ImageCompare, ImagePadding, ImageSize, ImageMaskCount};
use petgraph::stable_graph::NodeIndex;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum NodeData {
    Pixel,
    Color { color: u8 },
    PositionX { x: u8 },
    PositionY { y: u8 },
    // PositionReverseX { x: u8 },
    // PositionReverseY { y: u8 },
}

// #[derive(Clone, Debug)]
// enum EdgeData {
//     Link,
//     Property,
// }

#[allow(dead_code)]
struct ExperimentWithPetgraph;

impl ExperimentWithPetgraph {
    #[allow(dead_code)]
    fn analyze(image: &Image) -> anyhow::Result<()> {
        let mut graph: petgraph::Graph<NodeData, f32, petgraph::Undirected> = petgraph::Graph::new_undirected();
        for y in 0..image.height() {
            for x in 0..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                let node_pixel = NodeData::Pixel;
                let index_pixel: NodeIndex = graph.add_node(node_pixel);
                {
                    let property = NodeData::Color { color };
                    let index: NodeIndex = graph.add_node(property);
                    graph.add_edge(index_pixel, index, 1.0);
                }
                {
                    let property = NodeData::PositionX { x };
                    let index: NodeIndex = graph.add_node(property);
                    graph.add_edge(index_pixel, index, 1.0);
                }
                {
                    let property = NodeData::PositionY { y };
                    let index: NodeIndex = graph.add_node(property);
                    graph.add_edge(index_pixel, index, 1.0);
                }

                // edge to neighbor pixels.
            }
        }
        println!("graph: {:?}", graph);
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

        // Act
        ExperimentWithPetgraph::analyze(&input).expect("ok");
    }
}
