//! Draw lines between the extreme positions of an image.
//! 
//! Usecase:
//! Fill the area between the left-most pixel and the right-most pixel in each row.
//! 
//! Usecase:
//! Fill the area between the top-most pixel and the bottom-most pixel in each column.
//! 
//! Usecase:
//! Fill the area before the left-most pixel in each row.
//! 
//! Usecase:
//! Fill the area after the right-most pixel in each row.
use super::{Image, ImageSize, ImageRotate, ImageMaskBoolean};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum LineSpanDirection {
    Horizontal { mode: LineSpanMode, draw_mass: bool },
    Vertical { mode: LineSpanMode, draw_mass: bool },
    HorizontalFillOrVerticalFill,

    // Future experiments
    // DiagonalA,
    // DiagonalB,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum LineSpanMode {
    Fill,
    Before,
    After,
}

#[allow(dead_code)]
pub struct LineSpan;

impl LineSpan {
    #[allow(dead_code)]
    pub fn draw(image: &Image, direction: &LineSpanDirection) -> anyhow::Result<Image> {
        if image.is_empty() {
            return Err(anyhow::anyhow!("image must be 1x1 or bigger"));
        }
        match direction {
            LineSpanDirection::Horizontal { mode, draw_mass } => {
                Self::draw_horizontal_linespans(image, mode, *draw_mass)
            },
            LineSpanDirection::Vertical { mode, draw_mass } => {
                Self::draw_vertical_linespans(image, mode, *draw_mass)
            },
            LineSpanDirection::HorizontalFillOrVerticalFill => {
                let mode = LineSpanMode::Fill;
                let image0: Image = Self::draw_horizontal_linespans(image, &mode, false)?;
                let image1: Image = Self::draw_vertical_linespans(image, &mode, false)?;
                let image2: Image = image0.mask_or(&image1)?;
                return Ok(image2);
            }
        }
    }

    fn draw_vertical_linespans(image: &Image, mode: &LineSpanMode, draw_mass: bool) -> anyhow::Result<Image> {
        let image2: Image = image.rotate_ccw()?;
        let image3: Image = Self::draw_horizontal_linespans(&image2, mode, draw_mass)?;
        let image4: Image = image3.rotate_cw()?;
        Ok(image4)
    }

    fn draw_horizontal_linespans(image: &Image, mode: &LineSpanMode, draw_mass: bool) -> anyhow::Result<Image> {
        let image_size: ImageSize = image.size();

        let mut result_image: Image = image.clone_zero();

        for y in 0..image_size.height {
            let mut min_x: i32 = i32::MAX;
            let mut max_x: i32 = i32::MIN;
            // Identify the left-most pixel, and the right-most pixel.
            for x in 0..image_size.width {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(0);
                if color == 0 {
                    continue;
                }
                min_x = min_x.min(x as i32);
                max_x = max_x.max(x as i32);
            }

            if max_x < min_x {
                continue;
            }

            match mode {
                LineSpanMode::Fill => {
                    // Draw line in the identified line span
                },
                LineSpanMode::Before => {
                    // Draw line *before* the identified line span
                    max_x = min_x - 1;
                    min_x = 0;
                },
                LineSpanMode::After => {
                    // Draw line *after* the identified line span
                    min_x = max_x + 1;
                    max_x = (image_size.width as i32) - 1;
                },
            }

            if max_x < min_x {
                continue;
            }

            // Draw line
            let mass_i32: i32 = max_x - min_x + 1;
            if mass_i32 < 0 || mass_i32 >= 255 {
                continue;
            }
            let mass: u8 = mass_i32 as u8;
            let color: u8 = if draw_mass { mass } else { 1 };
            for x in min_x..=max_x {
                _ = result_image.set(x, y as i32, color);
            }
        }

        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_horizontal_fill_without_mass() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1,
        ];
        let mask: Image = Image::try_create(6, 5, pixels).expect("image");
        
        let direction: LineSpanDirection = LineSpanDirection::Horizontal { 
            mode: LineSpanMode::Fill,
            draw_mass: false,
        };
        
        // Act
        let actual: Image = LineSpan::draw(&mask, &direction).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 1, 1, 1, 0,
            0, 0, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_horizontal_fill_with_mass() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1,
        ];
        let mask: Image = Image::try_create(6, 5, pixels).expect("image");
        
        let direction: LineSpanDirection = LineSpanDirection::Horizontal { 
            mode: LineSpanMode::Fill,
            draw_mass: true,
        };
        
        // Act
        let actual: Image = LineSpan::draw(&mask, &direction).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            6, 6, 6, 6, 6, 6,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 2, 2, 0,
            0, 4, 4, 4, 4, 0,
            0, 0, 4, 4, 4, 4,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_horizontal_before() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1,
        ];
        let mask: Image = Image::try_create(6, 5, pixels).expect("image");
        
        let direction: LineSpanDirection = LineSpanDirection::Horizontal { 
            mode: LineSpanMode::Before,
            draw_mass: false,
        };
        
        // Act
        let actual: Image = LineSpan::draw(&mask, &direction).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            1, 1, 1, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
            1, 1, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_horizontal_after() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1,
        ];
        let mask: Image = Image::try_create(6, 5, pixels).expect("image");
        
        let direction: LineSpanDirection = LineSpanDirection::Horizontal { 
            mode: LineSpanMode::After,
            draw_mass: false,
        };
        
        // Act
        let actual: Image = LineSpan::draw(&mask, &direction).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_vertical_fill() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 1, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1,
        ];
        let mask: Image = Image::try_create(6, 5, pixels).expect("image");
        
        let direction: LineSpanDirection = LineSpanDirection::Vertical { 
            mode: LineSpanMode::Fill,
            draw_mass: false,
        };

        // Act
        let actual: Image = LineSpan::draw(&mask, &direction).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 1, 0, 0, 1, 1,
            0, 1, 0, 1, 1, 1,
            0, 1, 0, 1, 1, 1,
            0, 0, 1, 1, 0, 1,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_vertical_before() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 1, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1,
        ];
        let mask: Image = Image::try_create(6, 5, pixels).expect("image");
        
        let direction: LineSpanDirection = LineSpanDirection::Vertical { 
            mode: LineSpanMode::Before,
            draw_mass: false,
        };

        // Act
        let actual: Image = LineSpan::draw(&mask, &direction).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 1, 0, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_vertical_after() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 1, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1,
        ];
        let mask: Image = Image::try_create(6, 5, pixels).expect("image");
        
        let direction: LineSpanDirection = LineSpanDirection::Vertical { 
            mode: LineSpanMode::After,
            draw_mass: false,
        };

        // Act
        let actual: Image = LineSpan::draw(&mask, &direction).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
            1, 1, 0, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_horizontal_fill_or_vertical_fill() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1,
        ];
        let mask: Image = Image::try_create(6, 5, pixels).expect("image");
        
        let direction: LineSpanDirection = LineSpanDirection::HorizontalFillOrVerticalFill;
        
        // Act
        let actual: Image = LineSpan::draw(&mask, &direction).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1,
            0, 0, 0, 1, 1, 1,
            0, 1, 1, 1, 1, 1,
            0, 0, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(6, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
