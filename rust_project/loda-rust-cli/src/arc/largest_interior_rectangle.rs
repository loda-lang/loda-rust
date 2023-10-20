//! Largest interior rectangle.
//! 
//! Only considering the pixels that have the value `1`.
//! This algorithm determines the biggest rectangles that can be drawn inside `1` value area.
use super::{Image, convolution2x2, ImageMaskCount, Rectangle, ImageSymmetry};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct LargestInteriorRectangle {
    pub rectangles: HashSet<Rectangle>,
    pub mass: u16,
}

impl LargestInteriorRectangle {
    #[allow(dead_code)]
    pub fn analyze(image: &Image) -> anyhow::Result<Self> {
        Self::analyze_stack_algorithm(image)
        // Self::analyze_my_own_adhoc_algorithm(image)
    }

    #[allow(dead_code)]
    fn analyze_stack_algorithm(image: &Image) -> anyhow::Result<Self> {
        if image.is_empty() {
            return Ok(Self {
                rectangles: HashSet::new(),
                mass: 0,
            });
        }
    
        let mut max_area: u16 = 0;
        let mut largest_rects = HashSet::<Rectangle>::new();
        let mut height_vec: Vec<u8> = vec![0; image.width() as usize];
        let mut start_y_vec: Vec<u8> = vec![0; image.width() as usize];
    
        for y in 0..image.height() {
            for x in 0..(image.width() as usize) {
                if image.get(x as i32, y as i32) == Some(1) {
                    height_vec[x] += 1;
                    if height_vec[x] == 1 {
                        start_y_vec[x] = y;
                    }
                } else {
                    height_vec[x] = 0;
                    start_y_vec[x] = 0;
                }
            }
            let (current_rects, area) = Self::largest_rectangles_under_histogram(&height_vec, &start_y_vec);
            if area > max_area {
                max_area = area;
                largest_rects = current_rects;
            } else if area == max_area {
                largest_rects.extend(current_rects);
            }
        }

        if max_area == 0 {
            largest_rects.clear();
        }
    
        Ok(Self {
            rectangles: largest_rects,
            mass: max_area,
        })
    }

    /// https://stackoverflow.com/questions/4311694/maximize-the-rectangular-area-under-histogram
    fn largest_rectangles_under_histogram(height_vec: &Vec<u8>, start_y_vec: &Vec<u8>) -> (HashSet<Rectangle>, u16) {
        let mut stack = Vec::<usize>::new();
        let mut max_area: usize = 0;
        let mut largest_rects = HashSet::<Rectangle>::new();
        let mut index: usize = 0;
    
        while index < height_vec.len() {
            if stack.is_empty() || height_vec[index] >= height_vec[*stack.last().unwrap()] {
                stack.push(index);
                index += 1;
            } else {
                let top = stack.pop().unwrap();
                let width = if stack.is_empty() { index } else { index - stack.last().unwrap() - 1 };
                let area = (height_vec[top] as usize) * width;
                if area > max_area {
                    max_area = area;
                    largest_rects.clear();
                }
                if area == max_area {
                    let start_x = if stack.is_empty() { 0 } else { *stack.last().unwrap() + 1 };
                    largest_rects.insert(Rectangle::new(
                        start_x as u8,
                        start_y_vec[top],
                        width as u8,
                        height_vec[top],
                    ));
                }
            }
        }
    
        while !stack.is_empty() {
            let top = stack.pop().unwrap();
            let width = if stack.is_empty() { index } else { index - stack.last().unwrap() - 1 };
            let area = (height_vec[top] as usize) * width;
            if area > max_area {
                max_area = area;
                largest_rects.clear();
            }
            if area == max_area {
                let start_x = if stack.is_empty() { 0 } else { *stack.last().unwrap() + 1 };
                largest_rects.insert(Rectangle::new(
                    start_x as u8,
                    start_y_vec[top],
                    width as u8,
                    height_vec[top],
                ));
            }
        }
    
        (largest_rects, max_area.min(u16::MAX as usize) as u16)
    }

    #[allow(dead_code)]
    fn analyze_my_own_adhoc_algorithm(image: &Image) -> anyhow::Result<Self> {
        let mut candidates = HashSet::<Rectangle>::new();
        let mut biggest_area: u16 = 0;

        // Analyze initial layer
        Self::analyze_slices(&image, &mut biggest_area, &mut candidates, 1, true)?;
        let flipped_image = image.flip_diagonal_a()?;
        Self::analyze_slices(&flipped_image, &mut biggest_area, &mut candidates, 1, false)?;

        let mut current_layer: Image = image.clone();
        let mut scale: u8 = 2;
        loop {
            if current_layer.width() < 2 || current_layer.height() < 2 {
                // The image is too small, so no more convolution 2x2 operations can be applied.
                break;
            }
            let next_layer = convolution2x2(&current_layer, conv2x2_is_full)?;
            if next_layer.mask_count_nonzero() == 0 {
                // The image is all zeros, so it makes no sense to continue with the convolution 2x2 operations.
                break;
            }

            Self::analyze_slices(&next_layer, &mut biggest_area, &mut candidates, scale, true)?;
            let flipped_layer = next_layer.flip_diagonal_a()?;
            Self::analyze_slices(&flipped_layer, &mut biggest_area, &mut candidates, scale, false)?;

            current_layer = next_layer;
            scale += 1;
        }
        let instance = Self {
            rectangles: candidates,
            mass: biggest_area,
        };
        Ok(instance)
    }

    fn analyze_slices(
        image: &Image, 
        biggest_area: &mut u16, 
        candidates: &mut HashSet<Rectangle>, 
        scale: u8,
        horizontal: bool
    ) -> anyhow::Result<()> {
        let slices = LongestHorizontalSlices::analyze(&image)?;
        let mass = ((slices.mass as u16) + ((scale - 1) as u16)) * (scale as u16);

        if mass > *biggest_area {
            *biggest_area = mass;
            candidates.clear();
        }
        
        if mass == *biggest_area {
            for (x, y) in &slices.positions {
                let rect = if horizontal {
                    Rectangle::new(*x, *y, slices.mass + scale - 1, scale)
                } else {
                    Rectangle::new(*y, *x, scale, slices.mass + scale - 1)
                };
                candidates.insert(rect);
            }
        }
        Ok(())
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

/// Determines if the 2x2 convolution window is filled with pixel data.
/// 
/// Returns `1` when all pixels are non-zero.
/// 
/// Returns `0` when one or more pixels are zero.
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

    #[test]
    fn test_20006_largest_interior_rectangle() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(11, 1, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(2, 0, 3, 1));
        rectangles.insert(Rectangle::new(8, 0, 3, 1));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20007_largest_interior_rectangle() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1,
            0,
            1,
            1,
            1,
            0,
            1,
            0,
            1,
            1,
            1,
        ];
        let input: Image = Image::try_create(1, 11, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(0, 2, 1, 3));
        rectangles.insert(Rectangle::new(0, 8, 1, 3));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20008_largest_interior_rectangle() {
        // Arrange
        let input: Image = Image::color(1, 1, 1);

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let mut rectangles = HashSet::<Rectangle>::new();
        rectangles.insert(Rectangle::new(0, 0, 1, 1));
        let expected = LargestInteriorRectangle {
            rectangles,
            mass: 1,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20009_largest_interior_rectangle() {
        // Arrange
        let input: Image = Image::zero(1, 1);

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let expected = LargestInteriorRectangle {
            rectangles: HashSet::<Rectangle>::new(),
            mass: 0,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20010_largest_interior_rectangle() {
        // Arrange
        let input: Image = Image::empty();

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        let expected = LargestInteriorRectangle {
            rectangles: HashSet::<Rectangle>::new(),
            mass: 0,
        };
        assert_eq!(actual, expected);
    }
}
