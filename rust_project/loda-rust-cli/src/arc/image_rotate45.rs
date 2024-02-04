//! Rotate an image by 45 degrees.
use super::Image;

pub trait ImageRotate45 {
    /// Rotate an image by 45 degrees. clockwise (CW)
    /// 
    /// Where rotate by 90 degrees is a simple operation, rotate by 45 degrees is a bit more complex.
    /// This yields gaps in the rotated image. Every pixel has 4 gaps surrounding it.
    fn rotate_cw_45(&self, fill_color: u8) -> anyhow::Result<Image>;

    /// Rotate an image by 45 degrees. counter clockwise (CCW)
    /// 
    /// Where rotate by 90 degrees is a simple operation, rotate by 45 degrees is a bit more complex.
    /// This yields gaps in the rotated image. Every pixel has 4 gaps surrounding it.
    fn rotate_ccw_45(&self, fill_color: u8) -> anyhow::Result<Image>;
}

impl ImageRotate45 for Image {
    fn rotate_cw_45(&self, fill_color: u8) -> anyhow::Result<Image> {
        rotate_45(&self, fill_color, true)
    }

    fn rotate_ccw_45(&self, fill_color: u8) -> anyhow::Result<Image> {
        rotate_45(&self, fill_color, false)
    }
}

fn rotate_45(original: &Image, fill_color: u8, is_clockwise: bool) -> anyhow::Result<Image> {
    if original.width() <= 1 && original.height() <= 1 {
        // No point in processing an empty image or a 1x1 image.
        return Ok(original.clone());
    }

    let combined_u16: u16 = original.width() as u16 + original.height() as u16 - 1;
    if combined_u16 > 255 {
        return Err(anyhow::anyhow!("Unable to rotate image. The combined width and height is too large: {}", combined_u16));
    }

    // Rotate by 45 degrees
    let rads_amount: f32 = std::f32::consts::FRAC_PI_4; // pi divided by 4
    let rads: f32 = if is_clockwise { -rads_amount } else { rads_amount };

    let source_center_x: f32 = ((original.width() - 1) as f32) / 2.0;
    let source_center_y: f32 = ((original.height() - 1) as f32) / 2.0;
    let dest_center_x: f32 = ((combined_u16 - 1) as f32) / 2.0;
    let dest_center_y: f32 = ((combined_u16 - 1) as f32) / 2.0;

    // Increase the spacing between the points in the grid from 1 to sqrt(2)
    let scale: f32 = std::f32::consts::SQRT_2;

    let mut image = Image::color(combined_u16 as u8, combined_u16 as u8, fill_color);
    for get_y in 0..original.height() {
        for get_x in 0..original.width() {
            let pixel_value: u8 = original.get(get_x as i32, get_y as i32).unwrap_or(255);

            let x = (get_x as f32) - source_center_x;
            let y = (get_y as f32) - source_center_y;

            let rotated_x: f32 = (rads.cos() * x + rads.sin() * y) * scale;
            let rotated_y: f32 = (rads.cos() * y - rads.sin() * x) * scale;
            
            let set_x: i32 = (dest_center_x + rotated_x).round() as i32;
            let set_y: i32 = (dest_center_y + rotated_y).round() as i32;
            match image.set(set_x, set_y, pixel_value) {
                Some(()) => {},
                None => {
                    return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result image", set_x, y));
                }
            }
        }
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{Histogram, ImageHistogram, ImageRemoveRowColumn, ImageTryCreate};
    use bit_set::BitSet;

    #[test]
    fn test_10000_rotate_ccw_square() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3, 0, 0,
            0, 2, 0, 6, 0,
            1, 0, 5, 0, 9,
            0, 4, 0, 8, 0,
            0, 0, 7, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_rotate_ccw_landscape_onerow() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3,
            0, 2, 0,
            1, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_rotate_ccw_landscape_tworows() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 3, 0,
            0, 2, 0, 6,
            1, 0, 5, 0,
            0, 4, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_rotate_ccw_portrait_onecolumn() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 
            2, 
            3,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 2, 0,
            0, 0, 3,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_rotate_ccw_portrait_twocolumns() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 4,
            2, 5,
            3, 6,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_ccw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 4, 0, 0,
            1, 0, 5, 0,
            0, 2, 0, 6,
            0, 0, 3, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_rotate_cw() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 4,
            2, 5,
            3, 6,
        ];
        let input: Image = Image::try_create(2, 3, pixels).expect("image");

        // Act
        let actual: Image = input.rotate_cw_45(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 2, 0, 4,
            3, 0, 5, 0,
            0, 6, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_reversable_rotate_cw() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 2, 0, 4,
            3, 0, 5, 0,
            0, 6, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act - part 1
        let actual0: Image = input.rotate_ccw_45(0).expect("image");
        let expected_pixels0: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 1, 0, 4, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 2, 0, 5, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 3, 0, 6, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
        ];
        let expected0: Image = Image::try_create(7, 7, expected_pixels0).expect("image");
        assert_eq!(actual0, expected0);

        // Act - part 2
        let histogram_columns: Vec<Histogram> = actual0.histogram_columns();
        let histogram_rows: Vec<Histogram> = actual0.histogram_rows();

        let space_color: u8 = 0;

        // Identify the rows and columns that can be removed
        let mut delete_row_indexes = BitSet::new();
        for (index, histogram) in histogram_rows.iter().enumerate() {
            if histogram.number_of_counters_greater_than_zero() > 1 {
                continue;
            }
            if histogram.most_popular_color_disallow_ambiguous() == Some(space_color) {
                delete_row_indexes.insert(index as usize);
            }
        }
        let mut delete_column_indexes = BitSet::new();
        for (index, histogram) in histogram_columns.iter().enumerate() {
            if histogram.number_of_counters_greater_than_zero() > 1 {
                continue;
            }
            if histogram.most_popular_color_disallow_ambiguous() == Some(space_color) {
                delete_column_indexes.insert(index as usize);
            }
        }

        // Remove the rows and columns
        let actual1: Image = actual0.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");

        // Assert
        let expected_pixels1: Vec<u8> = vec![
            1, 4,
            2, 5,
            3, 6,
        ];
        let expected1: Image = Image::try_create(2, 3, expected_pixels1).expect("image");
        assert_eq!(actual1, expected1);
    }
}
