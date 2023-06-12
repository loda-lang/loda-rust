use super::Image;

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum ImageNeighbourDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl ImageNeighbourDirection {
    fn traversal_positions(&self, width: u8, height: u8) -> Vec<Vec<(i32,i32)>> {
        let width: i32 = width as i32;
        let height: i32 = height as i32;

        let mut outer_positions = Vec::<Vec::<(i32,i32)>>::new();
        match self {
            ImageNeighbourDirection::Left => {
                for y in 0..height {
                    let mut row = Vec::<(i32,i32)>::new();
                    for x in 0..width {
                        row.push((x, y));
                    }
                    outer_positions.push(row);
                }
            },
            ImageNeighbourDirection::Right => {
                let x1: i32 = width - 1;
                for y in 0..height {
                    let mut row = Vec::<(i32,i32)>::new();
                    for x in 0..width {
                        row.push((x1 - x, y));
                    }
                    outer_positions.push(row);
                }
            },
            ImageNeighbourDirection::Up => {
                for x in 0..width {
                    let mut column = Vec::<(i32,i32)>::new();
                    for y in 0..height {
                        column.push((x, y));
                    }
                    outer_positions.push(column);
                }
            },
            ImageNeighbourDirection::Down => {
                let y1: i32 = height - 1;
                for x in 0..width {
                    let mut column = Vec::<(i32,i32)>::new();
                    for y in 0..height {
                        column.push((x, y1 - y));
                    }
                    outer_positions.push(column);
                }
            },
            ImageNeighbourDirection::UpLeft => {
                let xrange: i32 = i32::max(width, height);
                for x in -xrange..xrange {
                    let mut diagonal_positions = Vec::<(i32,i32)>::new();
                    for y in 0..height {
                        let set_x: i32 = x + y;
                        if set_x < 0 || set_x >= width {
                            continue;
                        }
                        diagonal_positions.push((set_x, y));
                    }
                    if !diagonal_positions.is_empty() {
                        outer_positions.push(diagonal_positions);
                    }
                }
            },
            ImageNeighbourDirection::UpRight => {
                let xrange: i32 = i32::max(width, height);
                for x in -xrange..xrange {
                    let mut diagonal_positions = Vec::<(i32,i32)>::new();
                    for y in 0..height {
                        let set_x: i32 = width - 1 - x - y;
                        if set_x < 0 || set_x >= width {
                            continue;
                        }
                        diagonal_positions.push((set_x, y));
                    }
                    if !diagonal_positions.is_empty() {
                        outer_positions.push(diagonal_positions);
                    }
                }
            },
            ImageNeighbourDirection::DownLeft => {
                let xrange: i32 = i32::max(width, height);
                for x in -xrange..xrange {
                    let mut diagonal_positions = Vec::<(i32,i32)>::new();
                    for y in 0..height {
                        let set_x: i32 = x + y;
                        if set_x < 0 || set_x >= width {
                            continue;
                        }
                        diagonal_positions.push((set_x, height - 1 - y));
                    }
                    if !diagonal_positions.is_empty() {
                        outer_positions.push(diagonal_positions);
                    }
                }
            },
            ImageNeighbourDirection::DownRight => {
                let xrange: i32 = i32::max(width, height);
                for x in -xrange..xrange {
                    let mut diagonal_positions = Vec::<(i32,i32)>::new();
                    for y in 0..height {
                        let set_x: i32 = height - 1 - x - y;
                        if set_x < 0 || set_x >= width {
                            continue;
                        }
                        diagonal_positions.push((set_x, height - 1 - y));
                    }
                    if !diagonal_positions.is_empty() {
                        outer_positions.push(diagonal_positions);
                    }
                }
            }
        }
        outer_positions
    }
}

pub trait ImageNeighbour {
    /// Shoot out a ray in `direction` and determine what is the color of nearest visible object.
    /// 
    /// The `ignore_mask` can be used for suppressing pixels that should not be considered visible.
    fn neighbour_color(&self, ignore_mask: &Image, direction: ImageNeighbourDirection, color_when_there_is_no_neighbour: u8) -> anyhow::Result<Image>;

    /// Shoot out a ray in `direction` and determine the distance to the nearest visible object.
    /// 
    /// The `ignore_mask` can be used for suppressing pixels that should not be considered visible.
    /// 
    /// The magic value `255` is used when there is no object visible. Thus the max distance is only `254`.
    fn neighbour_distance(&self, ignore_mask: &Image, direction: ImageNeighbourDirection) -> anyhow::Result<Image>;
}

impl ImageNeighbour for Image {
    fn neighbour_color(&self, ignore_mask: &Image, direction: ImageNeighbourDirection, color_when_there_is_no_neighbour: u8) -> anyhow::Result<Image> {
        if ignore_mask.width() != self.width() || ignore_mask.height() != self.height() {
            return Err(anyhow::anyhow!("The size of the ignore_mask must be the same, but is different"));
        }
        if self.is_empty() {
            return Ok(self.clone());
        }

        // Plan the traversal of pixels
        let outer_positions: Vec<Vec<(i32,i32)>> = direction.traversal_positions(self.width(), self.height());
    
        let mut result_image = Image::color(self.width(), self.height(), color_when_there_is_no_neighbour);
        // Perform traversal of the pixels
        for inner_positions in outer_positions {
            let mut current_color: Option<u8> = None;

            for (x, y) in inner_positions {
                if let Some(color_value) = current_color {
                    let _ = result_image.set(x, y, color_value);
                }

                let mask_value: u8 = ignore_mask.get(x, y).unwrap_or(255);
                if mask_value > 0 {
                    continue;
                }

                let color_value: u8 = self.get(x, y).unwrap_or(255);
                current_color = Some(color_value);
            }
        }
        Ok(result_image)
    }

    fn neighbour_distance(&self, ignore_mask: &Image, direction: ImageNeighbourDirection) -> anyhow::Result<Image> {
        if ignore_mask.width() != self.width() || ignore_mask.height() != self.height() {
            return Err(anyhow::anyhow!("The size of the ignore_mask must be the same, but is different"));
        }
        if self.is_empty() {
            return Ok(self.clone());
        }

        // Plan the traversal of pixels
        let outer_positions: Vec<Vec<(i32,i32)>> = direction.traversal_positions(self.width(), self.height());
    
        let mut result_image = Image::zero(self.width(), self.height());
        // Perform traversal of the pixels
        for inner_positions in outer_positions {
            let mut current_color: Option<u8> = None;
            let mut current_distance: u8 = 255;

            for (x, y) in inner_positions {
                let _ = result_image.set(x, y, current_distance);
                if current_distance < 254 {
                    current_distance += 1;
                }
                let mask_value: u8 = ignore_mask.get(x, y).unwrap_or(255);
                if mask_value > 0 {
                    continue;
                }
                let color_value: u8 = self.get(x, y).unwrap_or(255);
                if current_color != Some(color_value) {
                    current_distance = 0;
                }
                current_color = Some(color_value);
            }
        }
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use crate::arc::ImageMask;

    #[test]
    fn test_10000_traversal_positions_left() {
        // Arrange
        let direction = ImageNeighbourDirection::Left;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(3, 2);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 0), (1, 0), (2, 0)],
            vec![(0, 1), (1, 1), (2, 1)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10001_traversal_positions_right() {
        // Arrange
        let direction = ImageNeighbourDirection::Right;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(3, 2);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(2, 0), (1, 0), (0, 0)],
            vec![(2, 1), (1, 1), (0, 1)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10002_traversal_positions_up() {
        // Arrange
        let direction = ImageNeighbourDirection::Up;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(3, 2);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 0), (0, 1)],
            vec![(1, 0), (1, 1)],
            vec![(2, 0), (2, 1)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10003_traversal_positions_down() {
        // Arrange
        let direction = ImageNeighbourDirection::Down;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(3, 2);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 1), (0, 0)],
            vec![(1, 1), (1, 0)],
            vec![(2, 1), (2, 0)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10004_traversal_positions_upleft_4x3() {
        // Arrange
        let direction = ImageNeighbourDirection::UpLeft;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(4, 3);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 2)],
            vec![(0, 1), (1, 2)],
            vec![(0, 0), (1, 1), (2, 2)],
            vec![(1, 0), (2, 1), (3, 2)],
            vec![(2, 0), (3, 1)],
            vec![(3, 0)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10004_traversal_positions_upleft_4x2() {
        // Arrange
        let direction = ImageNeighbourDirection::UpLeft;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(4, 2);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 1)],
            vec![(0, 0), (1, 1)],
            vec![(1, 0), (2, 1)],
            vec![(2, 0), (3, 1)],
            vec![(3, 0)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10004_traversal_positions_upleft_2x4() {
        // Arrange
        let direction = ImageNeighbourDirection::UpLeft;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(2, 4);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 3)],
            vec![(0, 2), (1, 3)],
            vec![(0, 1), (1, 2)],
            vec![(0, 0), (1, 1)],
            vec![(1, 0)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10005_traversal_positions_upright_4x3() {
        // Arrange
        let direction = ImageNeighbourDirection::UpRight;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(4, 3);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(3, 2)],
            vec![(3, 1), (2, 2)],
            vec![(3, 0), (2, 1), (1, 2)],
            vec![(2, 0), (1, 1), (0, 2)],
            vec![(1, 0), (0, 1)],
            vec![(0, 0)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10005_traversal_positions_upright_4x2() {
        // Arrange
        let direction = ImageNeighbourDirection::UpRight;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(4, 2);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(3, 1)],
            vec![(3, 0), (2, 1)],
            vec![(2, 0), (1, 1)],
            vec![(1, 0), (0, 1)],
            vec![(0, 0)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10005_traversal_positions_upright_2x4() {
        // Arrange
        let direction = ImageNeighbourDirection::UpRight;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(2, 4);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(1, 3)],
            vec![(1, 2), (0, 3)],
            vec![(1, 1), (0, 2)],
            vec![(1, 0), (0, 1)],
            vec![(0, 0)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10006_traversal_positions_downleft_4x3() {
        // Arrange
        let direction = ImageNeighbourDirection::DownLeft;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(4, 3);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 0)],
            vec![(0, 1), (1, 0)],
            vec![(0, 2), (1, 1), (2, 0)],
            vec![(1, 2), (2, 1), (3, 0)],
            vec![(2, 2), (3, 1)],
            vec![(3, 2)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10006_traversal_positions_downleft_4x2() {
        // Arrange
        let direction = ImageNeighbourDirection::DownLeft;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(4, 2);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 0)],
            vec![(0, 1), (1, 0)],
            vec![(1, 1), (2, 0)],
            vec![(2, 1), (3, 0)],
            vec![(3, 1)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10006_traversal_positions_downleft_2x4() {
        // Arrange
        let direction = ImageNeighbourDirection::DownLeft;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(2, 4);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(0, 0)],
            vec![(0, 1), (1, 0)],
            vec![(0, 2), (1, 1)],
            vec![(0, 3), (1, 2)],
            vec![(1, 3)],
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10007_traversal_positions_downright_4x3() {
        // Arrange
        let direction = ImageNeighbourDirection::DownRight;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(4, 3);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(3, 0)],
            vec![(3, 1), (2, 0)], 
            vec![(3, 2), (2, 1), (1, 0)], 
            vec![(2, 2), (1, 1), (0, 0)], 
            vec![(1, 2), (0, 1)], 
            vec![(0, 2)]
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10007_traversal_positions_downright_4x2() {
        // Arrange
        let direction = ImageNeighbourDirection::DownRight;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(4, 2);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(3, 0)],
            vec![(3, 1), (2, 0)], 
            vec![(2, 1), (1, 0)], 
            vec![(1, 1), (0, 0)], 
            vec![(0, 1)]
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_10007_traversal_positions_downright_2x4() {
        // Arrange
        let direction = ImageNeighbourDirection::DownRight;

        // Act
        let positions: Vec<Vec<(i32, i32)>> = direction.traversal_positions(2, 4);

        // Assert
        let expected: Vec<Vec<(i32, i32)>> = vec![
            vec![(1, 0)],
            vec![(1, 1), (0, 0)], 
            vec![(1, 2), (0, 1)], 
            vec![(1, 3), (0, 2)], 
            vec![(0, 3)]
        ];
        assert_eq!(positions, expected);
    }

    #[test]
    fn test_20000_neighbour_color_left_3x4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9,
            0, 9, 0,
            9, 0, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Left, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3,
            3, 3, 9,
            3, 9, 9,
            3, 3, 3,
        ];
        let expected = Image::create_raw(3, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20000_neighbour_color_left_6x3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 5, 0, 0,
            0, 9, 0, 0, 5, 0,
            9, 0, 0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Left, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 9, 5, 5,
            3, 3, 9, 9, 9, 5,
            3, 9, 9, 9, 9, 9,
        ];
        let expected = Image::create_raw(6, 3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20001_neighbour_color_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9,
            0, 9, 0,
            9, 0, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 4, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Right, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 3,
            9, 3, 3,
            3, 3, 3,
            3, 3, 3,
        ];
        let expected = Image::create_raw(3, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20002_neighbour_color_up() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 0,
            0, 9, 0, 0,
            9, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Up, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3,
            3, 3, 9, 3,
            3, 9, 9, 3,
        ];
        let expected = Image::create_raw(4, 3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20003_neighbour_color_down() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 0,
            0, 9, 0, 0,
            9, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Down, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 3, 3,
            9, 3, 3, 3,
            3, 3, 3, 3,
        ];
        let expected = Image::create_raw(4, 3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20004_neighbour_color_upleft() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 0,
            0, 9, 0, 0,
            9, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpLeft, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3,
            3, 3, 3, 9,
            3, 3, 9, 3,
            3, 9, 3, 9,
        ];
        let expected = Image::create_raw(4, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20005_neighbour_color_upright() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 9, 0, 0,
            0, 0, 9, 0,
            0, 0, 0, 9,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpRight, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3,
            9, 3, 3, 3,
            3, 9, 3, 3,
            9, 3, 9, 3,
        ];
        let expected = Image::create_raw(4, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20006_neighbour_color_downleft() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            9, 0, 0, 0,
            0, 9, 0, 0,
            0, 0, 9, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownLeft, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 9, 3, 9,
            3, 3, 9, 3,
            3, 3, 3, 9,
            3, 3, 3, 3,
        ];
        let expected = Image::create_raw(4, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20007_neighbour_color_downright() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 9,
            0, 0, 9, 0,
            0, 9, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownRight, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 3, 9, 3,
            3, 9, 3, 3,
            9, 3, 3, 3,
            3, 3, 3, 3,
        ];
        let expected = Image::create_raw(4, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_30000_neighbour_distance_left_5x7() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 9,
            0, 0, 0, 9, 0,
            0, 0, 9, 0, 9,
            0, 9, 0, 0, 9, 
            9, 0, 9, 0, 9,
            9, 9, 9, 9, 9,
        ];
        let input: Image = Image::try_create(5, 7, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_distance(&ignore_mask, ImageNeighbourDirection::Left).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            255, 255, 255, 255, 255,
            255, 255, 255, 255, 255,
            255, 255, 255, 255,   0,
            255, 255, 255,   0,   1,
            255, 255,   0,   1,   2,
            255,   0,   1,   2,   3,
            255,   0,   1,   2,   3,
        ];
        let expected = Image::create_raw(5, 7, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_30001_neighbour_distance_left_5x7() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 9,
            0, 0, 0, 9, 8,
            0, 0, 9, 0, 8,
            0, 9, 0, 0, 8, 
            9, 0, 8, 0, 9,
            8, 9, 8, 9, 8,
        ];
        let input: Image = Image::try_create(5, 7, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.neighbour_distance(&ignore_mask, ImageNeighbourDirection::Left).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            255, 255, 255, 255, 255,
            255, 255, 255, 255, 255,
            255, 255, 255, 255,   0,
            255, 255, 255,   0,   1,
            255, 255,   0,   1,   2,
            255,   0,   1,   0,   1,
            255,   0,   0,   0,   0,
        ];
        let expected = Image::create_raw(5, 7, expected_pixels);
        assert_eq!(output, expected);
    }
}
