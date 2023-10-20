use super::{Image, convolution2x2, ImageMaskCount, Rectangle, ImageSymmetry};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct LargestInteriorRectangle {
    pub rectangles: HashSet<Rectangle>,
    pub mass: u16,
}

impl LargestInteriorRectangle {
    pub fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut candidates = HashSet::<Rectangle>::new();
        let mut biggest_area: u16 = 0;

        // Find the longest horizontal slices
        {
            let slices: LongestHorizontalSlices = LongestHorizontalSlices::analyze(&image)?;
            let mass: u16 = slices.mass as u16;
            if mass > biggest_area {
                biggest_area = slices.mass as u16;
                candidates.clear();
            }
            if mass == biggest_area {
                for (x, y) in &slices.positions {
                    let rect: Rectangle = Rectangle::new(*x, *y, slices.mass, 1);
                    candidates.insert(rect);
                }
            }
        }

        // Find the longest vertical slices
        {
            let layer2: Image = image.flip_diagonal_a()?;
            let slices: LongestHorizontalSlices = LongestHorizontalSlices::analyze(&layer2)?;
            let mass: u16 = slices.mass as u16;
            if mass > biggest_area {
                biggest_area = slices.mass as u16;
                candidates.clear();
            }
            if mass == biggest_area {
                for (x, y) in &slices.positions {
                    let rect: Rectangle = Rectangle::new(*y, *x, 1, slices.mass);
                    candidates.insert(rect);
                }
            }
        }
        let mut current_layer: Image = image.clone();
        let mut scale: u8 = 2;
        loop {
            if current_layer.width() < 2 || current_layer.height() < 2 {
                // The image is too small, so no more convolution 2x2 operations can be applied.
                break;
            }
            let layer: Image = convolution2x2(&current_layer, conv2x2_is_full)?;
            let count: u16 = layer.mask_count_nonzero();
            if count == 0 {
                // The image is all zeros, so it makes no sense to continue with the convolution 2x2 operations.
                break;
            }

            // Find the longest horizontal slices
            {
                let slices: LongestHorizontalSlices = LongestHorizontalSlices::analyze(&layer)?;
                let mass: u16 = ((slices.mass as u16) + ((scale - 1) as u16)) * (scale as u16);
                if mass > biggest_area {
                    biggest_area = mass;
                    candidates.clear();
                }
                if mass == biggest_area {
                    for (x, y) in &slices.positions {
                        let rect: Rectangle = Rectangle::new(
                            *x, 
                            *y, 
                            slices.mass + scale - 1, 
                            scale,
                        );
                        candidates.insert(rect);
                    }
                }
            }

            // Find the longest vertical slices
            {
                let layer2: Image = layer.flip_diagonal_a()?;
                let slices: LongestHorizontalSlices = LongestHorizontalSlices::analyze(&layer2)?;
                let mass: u16 = ((slices.mass as u16) + ((scale - 1) as u16)) * (scale as u16);
                if mass > biggest_area {
                    biggest_area = mass;
                    candidates.clear();
                }
                if mass == biggest_area {
                    for (x, y) in &slices.positions {
                        let rect: Rectangle = Rectangle::new(
                            *y, 
                            *x, 
                            scale,
                            slices.mass + scale - 1, 
                        );
                        candidates.insert(rect);
                    }
                }
            }

            current_layer = layer;
            scale += 1;
        }
        let instance = Self {
            rectangles: candidates,
            mass: biggest_area,
        };
        Ok(instance)
    }
}

#[derive(Clone, Debug)]
struct LongestHorizontalSlices {
    positions: Vec<(u8, u8)>,
    mass: u8,
}

impl LongestHorizontalSlices {
    fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut candidates: Vec<(u8, u8)> = vec!();
        let mut biggest_mass: u8 = 0;
        for y in 0..image.height() {
            let mut mass: u8 = 0;
            let mut start_x: u8 = 0;
            for x in 0..image.width() {
                let pixel: u8 = image.get(x as i32, y as i32).unwrap_or(0);
                if pixel > 0 {
                    if mass == 0 {
                        start_x = x;
                    }
                    mass += 1;
                } else {
                    if mass > 0 {
                        if mass > biggest_mass {
                            biggest_mass = mass;
                            candidates.clear();
                        }
                        if mass == biggest_mass {
                            candidates.push((start_x, y));
                        }
                    }
                    mass = 0;
                }
            }
            if mass > 0 {
                if mass > biggest_mass {
                    biggest_mass = mass;
                    candidates.clear();
                }
                if mass == biggest_mass {
                    candidates.push((start_x, y));
                }
            }
        }
        let instance = Self {
            positions: candidates,
            mass: biggest_mass,
        };
        Ok(instance)
    }
}

fn conv2x2_is_full(image: &Image) -> anyhow::Result<u8> {
    for pixel in image.pixels() {
        if *pixel == 0 {
            return Ok(0);
        }
    }
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_longest_horizontal_slices() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 0, 1,
            1, 1, 0, 1, 1, 0,
            1, 1, 1, 1, 1, 1,
            0, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 0, 
            1, 1, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 6, input_pixels).expect("image");

        // Act
        let actual: LongestHorizontalSlices = LongestHorizontalSlices::analyze(&input).expect("ok");

        // Assert
        assert_eq!(actual.mass, 6);
        assert_eq!(actual.positions, vec![(0, 2)]);
    }

    #[test]
    fn test_20000_largest_interior_rectangle() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 0, 1,
            1, 1, 0, 1, 1, 0,
            1, 1, 1, 1, 1, 1,
            0, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 0, 
            1, 1, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 6, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(1, 2, 4, 3));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 12,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_largest_interior_rectangle() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 1, 1, 0, 1,
            1, 1, 1, 0, 1, 1,
            0, 1, 0, 1, 1, 1,
            1, 0, 1, 1, 1, 0,
            1, 1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(6, 5, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(4, 1, 1, 4));
        rectangles.insert(Rectangle::new(4, 1, 2, 2));
        rectangles.insert(Rectangle::new(3, 2, 2, 2));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 4,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_largest_interior_rectangle() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 1, 1, 0, 1,
            1, 1, 1, 0, 1, 1,
            0, 1, 0, 1, 0, 1,
            1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 5, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(0, 3, 6, 1));
        rectangles.insert(Rectangle::new(3, 3, 3, 2));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 6,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_largest_interior_rectangle() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 1, 0, 1, 0, 1,
            0, 1, 0, 0, 0, 1, 0,
            1, 0, 1, 0, 1, 0, 1,
            1, 0, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(7, 4, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(2, 3, 5, 1));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 5,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20004_largest_interior_rectangle() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 1, 1, 1, 1, 1,
            0, 1, 1, 1, 1, 1, 0,
            1, 0, 1, 1, 1, 1, 0,
            1, 0, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(7, 4, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(2, 0, 4, 4));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 16,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20005_largest_interior_rectangle() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1,
            1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1,
            1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1,
            1, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(11, 5, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(8, 0, 3, 5));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 15,
        };
        assert_eq!(actual, expected);
    }
}
