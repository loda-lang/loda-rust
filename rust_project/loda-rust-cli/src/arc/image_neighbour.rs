use super::Image;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ImageNeighbourDirection {
    Up,
    Down,
    Left,
    Right,
}

pub trait ImageNeighbour {
    fn color_of_neighbour(&self, ignore_mask: &Image, direction: ImageNeighbourDirection, color_when_there_is_no_neighbour: u8) -> anyhow::Result<Image>;
}

impl ImageNeighbour for Image {
    fn color_of_neighbour(&self, ignore_mask: &Image, direction: ImageNeighbourDirection, color_when_there_is_no_neighbour: u8) -> anyhow::Result<Image> {
        if ignore_mask.width() != self.width() || ignore_mask.height() != self.height() {
            return Err(anyhow::anyhow!("The size of the ignore_mask must be the same, but is different"));
        }
        if self.is_empty() {
            return Ok(self.clone());
        }

        // Plan the traversal of pixels
        let mut outer_positions = Vec::<Vec::<(i32,i32)>>::new();
        match direction {
            ImageNeighbourDirection::Left => {
                for y in 0..(self.height() as i32) {
                    let mut row = Vec::<(i32,i32)>::new();
                    for x in 0..(self.width() as i32) {
                        row.push((x, y));
                    }
                    outer_positions.push(row);
                }
            },
            ImageNeighbourDirection::Right => {
                let x1: i32 = (self.width() as i32) - 1;
                for y in 0..(self.height() as i32) {
                    let mut row = Vec::<(i32,i32)>::new();
                    for x in 0..(self.width() as i32) {
                        row.push((x1 - x, y));
                    }
                    outer_positions.push(row);
                }
            },
            ImageNeighbourDirection::Up => {
                for x in 0..(self.width() as i32) {
                    let mut column = Vec::<(i32,i32)>::new();
                    for y in 0..(self.height() as i32) {
                        column.push((x, y));
                    }
                    outer_positions.push(column);
                }
            },
            ImageNeighbourDirection::Down => {
                let y1: i32 = (self.height() as i32) - 1;
                for x in 0..(self.width() as i32) {
                    let mut column = Vec::<(i32,i32)>::new();
                    for y in 0..(self.height() as i32) {
                        column.push((x, y1 - y));
                    }
                    outer_positions.push(column);
                }
            }
        }
    
        let mut result_image = Image::color(self.width(), self.height(), color_when_there_is_no_neighbour);
        // Perform traversal of the pixels
        for inner_positions in outer_positions {
            let mut current_color: Option<u8> = None;

            for (x, y) in inner_positions {
                if let Some(color_value) = current_color {
                    let _ = result_image.set(x, y, color_value);
                    continue;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use crate::arc::ImageMask;

    #[test]
    fn test_10000_color_of_neighbour_left() {
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
        let output: Image = input.color_of_neighbour(&ignore_mask, ImageNeighbourDirection::Left, 3).expect("image");

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
    fn test_10001_color_of_neighbour_right() {
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
        let output: Image = input.color_of_neighbour(&ignore_mask, ImageNeighbourDirection::Right, 3).expect("image");

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
    fn test_10002_color_of_neighbour_up() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 0,
            0, 9, 0, 0,
            9, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.color_of_neighbour(&ignore_mask, ImageNeighbourDirection::Up, 3).expect("image");

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
    fn test_10003_color_of_neighbour_down() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 0,
            0, 9, 0, 0,
            9, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        let ignore_mask = input.to_mask_where_color_is(0);

        // Act
        let output: Image = input.color_of_neighbour(&ignore_mask, ImageNeighbourDirection::Down, 3).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            9, 9, 3, 3,
            9, 3, 3, 3,
            3, 3, 3, 3,
        ];
        let expected = Image::create_raw(4, 3, expected_pixels);
        assert_eq!(output, expected);
    }
}
