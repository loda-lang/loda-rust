//! Rotate an image by 45 degrees.
use super::{Checkerboard, HtmlLog, Image, ImageMask, ImageRemoveRowColumn, ImageReplaceColor, ImageSymmetry, ImageTrim, Rectangle};
use bit_set::BitSet;

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
    if original.is_empty() {
        // No point in processing an empty image.
        return Ok(original.clone());
    }
    if original.width() == 1 && original.height() == 1 {
        // No point in processing an 1x1 image.
        return Ok(original.clone());
    }

    let combined_u16: u16 = original.width() as u16 + original.height() as u16 - 1;
    if combined_u16 > 255 {
        return Err(anyhow::anyhow!("Unable to rotate image. The combined width and height is too large: {}", combined_u16));
    }

    let mut image = Image::color(combined_u16 as u8, combined_u16 as u8, fill_color);

    // Copy pixels from the original image to the rotated image
    for get_y in 0..original.height() {
        for get_x in 0..original.width() {
            let pixel_value: u8 = original.get(get_x as i32, get_y as i32).unwrap_or(255);
            let set_x: i32 = get_x as i32 + get_y as i32;
            let set_y: i32 = get_x as i32 - get_y as i32 + (original.height() - 1) as i32;
            match image.set(set_x, set_y, pixel_value) {
                Some(()) => {},
                None => {
                    return Err(anyhow::anyhow!("Integrity error. Unable to set pixel ({}, {}) inside the result image", set_x, set_y));
                }
            }
        }
    }
    if is_clockwise {
        image = image.flip_diagonal_a()?;
    } else {
        image = image.flip_y()?;
    }
    Ok(image)
}

#[allow(dead_code)]
struct ReverseRotate45 {
    rotated_a: Image,
    rotated_b: Image,
}

impl ReverseRotate45 {
    #[allow(dead_code)]
    fn process(image: &Image, verbose: bool, triangle_color: u8, is_clockwise: bool) -> anyhow::Result<Self> {
        if verbose {
            HtmlLog::image(&image);
        }
        let rotated_a: Image = Self::extract_lattice(image, verbose, triangle_color, is_clockwise, false)?;
        let rotated_b: Image = Self::extract_lattice(image, verbose, triangle_color, is_clockwise, true)?;
        if verbose {
            HtmlLog::compare_images(vec![rotated_a.clone(), rotated_b.clone()]);
        }
        let instance = Self {
            rotated_a,
            rotated_b,
        };
        Ok(instance)
    }

    #[allow(dead_code)]
    fn extract_lattice(input: &Image, verbose: bool, triangle_color: u8, is_clockwise: bool, extract_second: bool) -> anyhow::Result<Image> {
        let space_color: u8 = 255;
        
        let color0: u8 = if extract_second { 0 } else { 1 };
        let color1: u8 = if extract_second { 1 } else { 0 };
        let mask: Image = Checkerboard::checkerboard(input.width(), input.height(), color0, color1);
        let masked_input: Image = mask.select_from_image_and_color(&input, space_color).expect("image");
        // if verbose {
        //     HtmlLog::image(&masked_input);
        // }

        // Rotate CW or CCW
        let rotated_image: Image = rotate_45(&masked_input, space_color, is_clockwise)?;
        // if verbose {
        //     HtmlLog::image(&rotated_image);
        // }

        // Bounding box
        let rect: Rectangle = rotated_image.outer_bounding_box_after_trim_with_color(space_color).expect("rectangle");

        // Determine where in the lattice the image is located
        let keep_x: u8 = rect.x() & 1;
        let keep_y: u8 = rect.y() & 1;

        // Keep every second row and column        
        let mut delete_row_indexes = BitSet::new();
        let mut delete_column_indexes = BitSet::new();
        for x in 0..rotated_image.width() {
            if x & 1 == keep_x {
                continue;
            }
            delete_column_indexes.insert(x as usize);
        }
        for y in 0..rotated_image.height() {
            if y & 1 == keep_y {
                continue;
            }
            delete_row_indexes.insert(y as usize);
        }

        // Remove rows and columns
        let actual1: Image = rotated_image.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");
        // if verbose {
        //     HtmlLog::image(&actual1);
        // }

        // Assign color to the corner triangles
        let actual2: Image = actual1.replace_color(space_color, triangle_color).expect("image");
        Ok(actual2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{checkerboard, Histogram, HtmlLog, ImageHistogram, ImageMask, ImageRemoveRowColumn, ImageReplaceColor, ImageTrim, ImageTryCreate, Rectangle};
    use bit_set::BitSet;
    use checkerboard::Checkerboard;
    use num_integer::Integer;

    #[test]
    fn test_10000_rotate_tiny_images() {
        {
            let actual: Image = Image::empty().rotate_cw_45(0).expect("image");
            assert_eq!(actual, Image::empty());
        }
        {
            let actual: Image = Image::color(1, 1, 9).rotate_cw_45(0).expect("image");
            assert_eq!(actual, Image::color(1, 1, 9));
        }
    }

    #[test]
    fn test_10001_rotate_ccw_square() {
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
    fn test_10002_rotate_ccw_landscape_onerow() {
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
    fn test_10003_rotate_ccw_landscape_tworows() {
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
    fn test_10004_rotate_ccw_portrait_onecolumn() {
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
    fn test_10005_rotate_ccw_portrait_twocolumns() {
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
    fn test_30001_reversable_rotate_keep_empty_lines_inside_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 0, 0, 4,
            3, 0, 0, 0,
            0, 6, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        let space_color: u8 = 0;
        
        // Act - part 1
        let actual0: Image = input.rotate_ccw_45(space_color).expect("image");
        let expected_pixels0: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 1, 0, 4, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 3, 0, 6, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
        ];
        let expected0: Image = Image::try_create(7, 7, expected_pixels0).expect("image");
        assert_eq!(actual0, expected0);

        // Act - part 2
        let rect: Rectangle = actual0.outer_bounding_box_after_trim_with_color(space_color).expect("rectangle");
        assert_eq!(rect, Rectangle::new(2, 1, 3, 5));

        // Keep every second row and column
        let mut keep_ys = BitSet::new();
        for y in 0..rect.height() {
            if y.is_even() {
                keep_ys.insert((y as usize) + rect.y() as usize);
            }
        }
        let mut keep_xs = BitSet::new();
        for x in 0..rect.width() {
            if x.is_even() {
                keep_xs.insert((x as usize) + rect.x() as usize);
            }
        }

        // Identify the rows and columns that can be removed
        let mut delete_row_indexes = BitSet::new();
        let mut delete_column_indexes = BitSet::new();
        for x in 0..actual0.width() {
            if keep_xs.contains(x as usize) {
                continue;
            }
            delete_column_indexes.insert(x as usize);
        }
        for y in 0..actual0.height() {
            if keep_ys.contains(y as usize) {
                continue;
            }
            delete_row_indexes.insert(y as usize);
        }

        // Remove the rows and columns
        let actual1: Image = actual0.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");

        // Assert
        let expected_pixels1: Vec<u8> = vec![
            1, 4,
            0, 0,
            3, 6,
        ];
        let expected1: Image = Image::try_create(2, 3, expected_pixels1).expect("image");
        assert_eq!(actual1, expected1);

        // Rotating again, should yield the input image
        let actual2: Image = actual1.rotate_cw_45(space_color).expect("image");
        assert_eq!(actual2, input);
    }

    #[test]
    fn test_30002_reversable_rotate_keep_empty_lines_inside_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 3,
            0, 2, 0,
            1, 0, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        let space_color: u8 = 0;
        
        // Act - part 1
        let actual0: Image = input.rotate_ccw_45(space_color).expect("image");
        let expected_pixels0: Vec<u8> = vec![
            0, 0, 3, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 2, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
        ];
        let expected0: Image = Image::try_create(5, 5, expected_pixels0).expect("image");
        assert_eq!(actual0, expected0);

        // Act - part 2
        let rect: Rectangle = actual0.outer_bounding_box_after_trim_with_color(space_color).expect("rectangle");
        assert_eq!(rect, Rectangle::new(2, 0, 1, 5));

        // Keep every second row and column
        let mut keep_ys = BitSet::new();
        for y in 0..rect.height() {
            if y.is_even() {
                keep_ys.insert((y as usize) + rect.y() as usize);
            }
        }
        let mut keep_xs = BitSet::new();
        for x in 0..rect.width() {
            if x.is_even() {
                keep_xs.insert((x as usize) + rect.x() as usize);
            }
        }

        // Identify the rows and columns that can be removed
        let mut delete_row_indexes = BitSet::new();
        let mut delete_column_indexes = BitSet::new();
        for x in 0..actual0.width() {
            if keep_xs.contains(x as usize) {
                continue;
            }
            delete_column_indexes.insert(x as usize);
        }
        for y in 0..actual0.height() {
            if keep_ys.contains(y as usize) {
                continue;
            }
            delete_row_indexes.insert(y as usize);
        }

        // Remove the rows and columns
        let actual1: Image = actual0.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");

        // Assert
        let expected_pixels1: Vec<u8> = vec![
            3,
            2, 
            1, 
        ];
        let expected1: Image = Image::try_create(1, 3, expected_pixels1).expect("image");
        assert_eq!(actual1, expected1);

        // Rotating again, should yield the input image
        let actual2: Image = actual1.rotate_cw_45(space_color).expect("image");
        assert_eq!(actual2, input);
    }

    #[test]
    fn test_30003_reversable_rotate_two_images_interleaved() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 3, 8,
            0, 2, 7, 8,
            1, 0, 5, 8,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        let space_color: u8 = 0;
        
        // Act - part 1
        let actual0: Image = input.rotate_ccw_45(space_color).expect("image");
        let expected_pixels0: Vec<u8> = vec![
            0, 0, 0, 8, 0, 0,
            0, 0, 3, 0, 8, 0,
            0, 0, 0, 7, 0, 8,
            0, 0, 2, 0, 5, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
        ];
        let expected0: Image = Image::try_create(6, 6, expected_pixels0).expect("image");
        assert_eq!(actual0, expected0);

        // Act - part 2
        let rect: Rectangle = actual0.outer_bounding_box_after_trim_with_color(space_color).expect("rectangle");
        assert_eq!(rect, Rectangle::new(2, 0, 4, 6));

        // Keep every second row and column
        let mut keep_ys_even = BitSet::new();
        let mut keep_ys_odd = BitSet::new();
        for y in 0..rect.height() {
            if y.is_even() {
                keep_ys_even.insert((y as usize) + rect.y() as usize);
            } else {
                keep_ys_odd.insert((y as usize) + rect.y() as usize);
            }
        }
        let mut keep_xs_even = BitSet::new();
        let mut keep_xs_odd = BitSet::new();
        for x in 0..rect.width() {
            if x.is_even() {
                keep_xs_even.insert((x as usize) + rect.x() as usize);
            } else {
                keep_xs_odd.insert((x as usize) + rect.x() as usize);
            }
        }

        let mut images = Vec::<Image>::new();
        for i in 0..=3u8 {
            let keep_ys: &BitSet = if (i & 1) == 1 { &keep_ys_even } else { &keep_ys_odd };
            let keep_xs: &BitSet = if (i & 2) == 2 { &keep_xs_even } else { &keep_xs_odd };

            // Identify the rows and columns that can be removed
            let mut delete_row_indexes = BitSet::new();
            let mut delete_column_indexes = BitSet::new();
            for x in 0..actual0.width() {
                if keep_xs.contains(x as usize) {
                    continue;
                }
                delete_column_indexes.insert(x as usize);
            }
            for y in 0..actual0.height() {
                if keep_ys.contains(y as usize) {
                    continue;
                }
                delete_row_indexes.insert(y as usize);
            }
    
            // Remove the rows and columns
            let image: Image = actual0.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");
            images.push(image);
        }

        // Assert
        {
            let expected_pixels: Vec<u8> = vec![
                0, 0,
                0, 0,
                0, 0,
            ];
            let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
            assert_eq!(images[0], expected);
        }
        {
            let expected_pixels: Vec<u8> = vec![
                8, 0,
                7, 8,
                0, 0,
            ];
            let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
            assert_eq!(images[1], expected);
        }
        {
            let expected_pixels: Vec<u8> = vec![
                3, 8,
                2, 5,
                1, 0,
            ];
            let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
            assert_eq!(images[2], expected);
        }
        {
            let expected_pixels: Vec<u8> = vec![
                0, 0,
                0, 0,
                0, 0,
            ];
            let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
            assert_eq!(images[3], expected);
        }
    }

    #[allow(dead_code)]
    // #[test]
    fn test_30004_reversable_rotate_two_images_interleaved() {
        // Arrange
        let input: Image = Image::color(7, 8, 1);

        let space_color: u8 = 0;
        
        // Act - part 1
        let actual0: Image = input.rotate_ccw_45(space_color).expect("image");

        // Act - part 2
        let rect: Rectangle = actual0.outer_bounding_box_after_trim_with_color(space_color).expect("rectangle");

        // Keep every second row and column
        let mut keep_ys_even = BitSet::new();
        let mut keep_ys_odd = BitSet::new();
        for y in 0..rect.height() {
            if y.is_even() {
                keep_ys_even.insert((y as usize) + rect.y() as usize);
            } else {
                keep_ys_odd.insert((y as usize) + rect.y() as usize);
            }
        }
        let mut keep_xs_even = BitSet::new();
        let mut keep_xs_odd = BitSet::new();
        for x in 0..rect.width() {
            if x.is_even() {
                keep_xs_even.insert((x as usize) + rect.x() as usize);
            } else {
                keep_xs_odd.insert((x as usize) + rect.x() as usize);
            }
        }

        let mut images = Vec::<Image>::new();
        for i in 0..=3u8 {
            let keep_ys: &BitSet = if (i & 1) == 1 { &keep_ys_even } else { &keep_ys_odd };
            let keep_xs: &BitSet = if (i & 2) == 2 { &keep_xs_even } else { &keep_xs_odd };

            // Identify the rows and columns that can be removed
            let mut delete_row_indexes = BitSet::new();
            let mut delete_column_indexes = BitSet::new();
            for x in 0..actual0.width() {
                if keep_xs.contains(x as usize) {
                    continue;
                }
                delete_column_indexes.insert(x as usize);
            }
            for y in 0..actual0.height() {
                if keep_ys.contains(y as usize) {
                    continue;
                }
                delete_row_indexes.insert(y as usize);
            }
    
            // Remove the rows and columns
            let image: Image = actual0.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");
            HtmlLog::text(&format!("Image {}", i));
            HtmlLog::image(&image);
            images.push(image);
        }
    }

    #[allow(dead_code)]
    // #[test]
    fn test_30005_reversable_rotate_two_images_interleaved_using_checkerboard() {
        // Arrange
        let space_color: u8 = 255;
        let staircase_color: u8 = 2;
        
        let input_raw: Image = Checkerboard::checkerboard(7, 8, 1, 3);
        // let input_raw: Image = Image::color(7, 8, 1);
        HtmlLog::image(&input_raw);

        let extract_second = false;

        let color0: u8 = if extract_second { 0 } else { 1 };
        let color1: u8 = if extract_second { 1 } else { 0 };
        let checkerboard: Image = Checkerboard::checkerboard(input_raw.width(), input_raw.height(), color0, color1);
        let input: Image = checkerboard.select_from_image_and_color(&input_raw, space_color).expect("image");
        HtmlLog::image(&input);

        // Act - part 1
        let actual0: Image = input.rotate_ccw_45(space_color).expect("image");
        HtmlLog::image(&actual0);

        // Act - part 2
        let rect: Rectangle = actual0.outer_bounding_box_after_trim_with_color(space_color).expect("rectangle");

        // Keep every second row and column        
        let keep_x: u8 = rect.x() & 1;
        let keep_y: u8 = rect.y() & 1;
        let mut delete_row_indexes = BitSet::new();
        let mut delete_column_indexes = BitSet::new();
        for x in 0..actual0.width() {
            if x & 1 == keep_x {
                continue;
            }
            delete_column_indexes.insert(x as usize);
        }
        for y in 0..actual0.height() {
            if y & 1 == keep_y {
                continue;
            }
            delete_row_indexes.insert(y as usize);
        }

        // Remove the rows and columns
        let actual1: Image = actual0.remove_rowcolumn(&delete_row_indexes, &delete_column_indexes).expect("image");
        HtmlLog::image(&actual1);

        // Replace the spacer color with the staircase color
        let actual2: Image = actual1.replace_color(space_color, staircase_color).expect("image");
        HtmlLog::image(&actual2);
    }

    #[test]
    fn test_30006_reversable_ccw() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 3, 0, 0,
            0, 2, 0, 6, 0,
            1, 0, 5, 0, 9,
            0, 4, 0, 8, 0,
            0, 0, 7, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let verbose = false;
        let is_clockwise = false;
        let triangle_color: u8 = 11;

        // Act
        let actual: ReverseRotate45 = ReverseRotate45::process(&input, verbose, triangle_color, is_clockwise).expect("reverse rotate");

        // Assert
        let expected_pixels0: Vec<u8> = vec![
            11, 0, 0, 11,
             0, 0, 0, 0,
             0, 0, 0, 0,
            11, 0, 0, 11,
        ];
        let expected0: Image = Image::try_create(4, 4, expected_pixels0).expect("image");
        assert_eq!(actual.rotated_a, expected0);

        let expected_pixels1: Vec<u8> = vec![
            11, 11,  0, 11, 11,
            11,  3,  6,  9, 11,
             0,  2,  5,  8,  0,
            11,  1,  4,  7, 11,
            11, 11,  0, 11, 11,
        ];
        let expected1: Image = Image::try_create(5, 5, expected_pixels1).expect("image");
        assert_eq!(actual.rotated_b, expected1);
    }

    #[allow(dead_code)]
    // #[test]
    fn test_30007_reversable_ccw() {
        let input: Image = Checkerboard::checkerboard(6, 3, 1, 3);

        let verbose = true;
        let triangle_color: u8 = 11;
        let is_clockwise = false;
        let actual: ReverseRotate45 = ReverseRotate45::process(&input, verbose, triangle_color, is_clockwise).expect("reverse rotate");
    }
}
