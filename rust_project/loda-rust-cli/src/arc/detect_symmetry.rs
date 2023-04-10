use super::{Histogram, Image, ImageCompare, ImageCrop, ImageHistogram, ImageMaskCount, ImageRotate, ImageSymmetry, Rectangle, ImageMask};
use std::fmt;

const MAX_INSET_VALUE: u8 = 5;

#[allow(dead_code)]
pub struct DetectSymmetry {
    pub left: u8,
    pub right: u8,
    pub horizontal_mismatches: u16,
    pub found_horizontal_symmetry: bool,
    pub horizontal_rect: Option<Rectangle>,

    pub top: u8,
    pub bottom: u8,
    pub vertical_mismatches: u16,
    pub found_vertical_symmetry: bool,
    pub vertical_rect: Option<Rectangle>,

    pub diagonal_a_mismatches: u16,
    pub diagonal_a_is_symmetric: bool,

    pub diagonal_b_mismatches: u16,
    pub diagonal_b_is_symmetric: bool,

    // TODO: "diagonal_a_rect" and "diagonal_b_rect"
    // Idea for more
    // Identify the repair color
    // repair plan for the damaged pixels
}

impl DetectSymmetry {
    #[allow(dead_code)]
    pub fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut instance = Self::new();
        instance.perform_analyze(image)?;
        Ok(instance)
    }

    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            left: u8::MAX,
            right: u8::MAX,
            found_horizontal_symmetry: false,
            horizontal_mismatches: u16::MAX,
            horizontal_rect: None,
            top: u8::MAX,
            bottom: u8::MAX,
            found_vertical_symmetry: false,
            vertical_mismatches: u16::MAX,
            vertical_rect: None,
            diagonal_a_mismatches: u16::MAX,
            diagonal_a_is_symmetric: false,
            diagonal_b_mismatches: u16::MAX,
            diagonal_b_is_symmetric: false,
        }
    }

    #[allow(dead_code)]
    fn horizontal_to_string(&self) -> String {
        if !self.found_horizontal_symmetry {
            return "no horizontal symmetry".to_string();
        }
        if self.horizontal_mismatches == 0 {
            return format!("horizontal symmetry, left: {} right: {}", self.left, self.right);
        }
        format!("partial horizontal symmetry, left: {} right: {} mismatches: {}", self.left, self.right, self.horizontal_mismatches)
    }

    #[allow(dead_code)]
    fn vertical_to_string(&self) -> String {
        if !self.found_vertical_symmetry {
            return "no vertical symmetry".to_string();
        }
        if self.vertical_mismatches == 0 {
            return format!("vertical symmetry, top: {} bottom: {}", self.top, self.bottom);
        }
        format!("partial vertical symmetry, top: {} bottom: {} mismatches: {}", self.top, self.bottom, self.vertical_mismatches)
    }

    #[allow(dead_code)]
    fn diagonal_a_to_string(&self) -> String {
        if !self.diagonal_a_is_symmetric {
            return "no diagonal-a symmetry".to_string();
        }
        if self.diagonal_a_mismatches == 0 {
            return "diagonal-a symmetry".to_string();
        }
        format!("partial diagonal-a symmetry, mismatches: {}", self.diagonal_a_mismatches)
    }

    #[allow(dead_code)]
    fn diagonal_b_to_string(&self) -> String {
        if !self.diagonal_b_is_symmetric {
            return "no diagonal-b symmetry".to_string();
        }
        if self.diagonal_b_mismatches == 0 {
            return "diagonal-b symmetry".to_string();
        }
        format!("partial diagonal-b symmetry, mismatches: {}", self.diagonal_b_mismatches)
    }

    fn perform_analyze(&mut self, image: &Image) -> anyhow::Result<()> {
        self.analyze_horizontal_symmetry(image)?;
        self.analyze_vertical_symmetry(image)?;
        self.suppress_false_positive(image)?;
        self.analyze_diagonal_symmetry(image)?;
        self.update_horizontal_rect(image)?;
        self.update_vertical_rect(image)?;
        Ok(())
    }

    fn update_horizontal_rect(&mut self, image: &Image) -> anyhow::Result<()> {
        if !self.found_horizontal_symmetry {
            return Ok(());
        }
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let mut x0: i32 = r.min_x();
        let y0: i32 = r.min_y();
        let mut x1: i32 = r.max_x();
        let y1: i32 = r.max_y();
        if self.found_horizontal_symmetry {
            x0 += self.left as i32;
            x1 -= self.right as i32;
        }
        self.horizontal_rect = Rectangle::span(x0, y0, x1, y1);
        Ok(())
    }

    fn update_vertical_rect(&mut self, image: &Image) -> anyhow::Result<()> {
        if !self.found_vertical_symmetry {
            return Ok(());
        }
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let x0: i32 = r.min_x();
        let mut y0: i32 = r.min_y();
        let x1: i32 = r.max_x();
        let mut y1: i32 = r.max_y();
        if self.found_vertical_symmetry {
            y0 += self.top as i32;
            y1 -= self.bottom as i32;
        }
        self.vertical_rect = Rectangle::span(x0, y0, x1, y1);
        Ok(())
    }

    fn analyze_diagonal_symmetry(&mut self, image: &Image) -> anyhow::Result<()> {
        let two_way_symmetric: bool = self.found_horizontal_symmetry && self.found_vertical_symmetry;
        if !two_way_symmetric {
            return Ok(());
        }
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let x0: i32 = r.min_x() + self.left as i32;
        let y0: i32 = r.min_y() + self.top as i32;
        let x1: i32 = r.max_x() - self.right as i32;
        let y1: i32 = r.max_y() - self.bottom as i32;
        let crop_rect: Rectangle = match Rectangle::span(x0, y0, x1, y1) {
            Some(value) => value,
            None => {
                return Ok(());
            }
        };
        if crop_rect.width() != crop_rect.height() {
            return Ok(());
        }
        let image_cropped: Image = image.crop(crop_rect)?;
        let area: u16 = (image_cropped.width() as u16) * (image_cropped.height() as u16);

        {
            let flipped_image: Image = image_cropped.flip_diagonal_a()?;
            let diff: Image = flipped_image.diff(&image_cropped)?;
            let mismatch_count: u16 = diff.mask_count_one();
            if mismatch_count <= (area / 2) {
                self.diagonal_a_is_symmetric = true;
                self.diagonal_a_mismatches = mismatch_count;
            }
        }
        {
            let flipped_image: Image = image_cropped.flip_diagonal_b()?;
            let diff: Image = flipped_image.diff(&image_cropped)?;
            let mismatch_count: u16 = diff.mask_count_one();
            if mismatch_count <= (area / 2) {
                self.diagonal_b_is_symmetric = true;
                self.diagonal_b_mismatches = mismatch_count;
            }
        }
        Ok(())
    }

    fn suppress_false_positive(&mut self, image: &Image) -> anyhow::Result<()> {
        let two_way_symmetric: bool = self.found_horizontal_symmetry && self.found_vertical_symmetry;
        if !two_way_symmetric {
            return Ok(());
        }
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let x0: i32 = r.min_x() + self.left as i32;
        let y0: i32 = r.min_y() + self.top as i32;
        let x1: i32 = r.max_x() - self.right as i32;
        let y1: i32 = r.max_y() - self.bottom as i32;
        let crop_rect: Rectangle = match Rectangle::span(x0, y0, x1, y1) {
            Some(value) => value,
            None => {
                return Ok(());
            }
        };
        let cropped_image: Image = image.crop(crop_rect)?;
        let histogram: Histogram = cropped_image.histogram_all();
        if histogram.number_of_counters_greater_than_zero() >= 2 {
            return Ok(());
        }

        // The cropped area is a single color, all the content has been trimmed off.
        // This is symmetric in a terrible way, not useful. Let's ignore this kind of symmetry.
        self.found_horizontal_symmetry = false;
        self.found_vertical_symmetry = false;
        Ok(())
    }

    #[allow(dead_code)]
    fn analyze_horizontal_symmetry(&mut self, image: &Image) -> anyhow::Result<()> {
        self.analyze_horizontal_symmetry_inner(image, false)
    }

    #[allow(dead_code)]
    fn analyze_vertical_symmetry(&mut self, image: &Image) -> anyhow::Result<()> {
        let image_rotated: Image = image.rotate_ccw()?;
        self.analyze_horizontal_symmetry_inner(&image_rotated, true)
    }

    fn analyze_horizontal_symmetry_inner(&mut self, image: &Image, should_update_vertical_data: bool) -> anyhow::Result<()> {
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let mut found: bool = false;
        let mut found_left: u8 = u8::MAX;
        let mut found_right: u8 = u8::MAX;
        let mut found_mismatches: u16 = u16::MAX;
        let part_of_width: u8 = (image.width() / 3) + 1;
        let max_inset: u8 = part_of_width.min(MAX_INSET_VALUE);
        for j in 0..max_inset {
            for i in 0..2 {
                // Only once try out the left=0 and right=0. Second time ignore this combo.
                if i == 1 && j == 0 {
                    continue;
                }

                // Alternate between inset left and inset right
                let left: u8;
                let right: u8;
                if i == 0 {
                    left = 0;
                    right = j;
                } else {
                    left = j;
                    right = 0;
                }

                let x0: i32 = r.min_x() + (left as i32);
                let x1: i32 = r.max_x() - (right as i32);
                if x0 > x1 {
                    continue;
                }
                if x0 < 0 || x0 > (u8::MAX as i32) {
                    continue;
                }
                if x1 < 0 || x1 > (u8::MAX as i32) {
                    continue;
                }
                let width: i32 = x1 - x0 + 1;
                if width <= 0 {
                    continue;
                }
                let x0_u8: u8 = x0 as u8;
                let width_u8: u8 = width as u8;
                let rect = Rectangle::new(x0_u8, 0, width_u8, image.height());
                let image_cropped: Image = match image.crop(rect) {
                    Ok(value) => value,
                    Err(_) => {
                        continue;
                    }
                };
                let area: u16 = (image_cropped.width() as u16) * (image_cropped.height() as u16);
                let image: Image = image_cropped.flip_x()?;
                let diff: Image = image.diff(&image_cropped)?;
                let histogram_mask: Image = diff.to_mask_where_color_is(0);
                let histogram: Histogram = image.histogram_with_mask(&histogram_mask)?;
                if histogram.number_of_counters_greater_than_zero() < 2 {
                    continue;
                }
                let mismatch_count: u16 = diff.mask_count_one();
                if mismatch_count > (area / 2) {
                    continue;
                }
                if !found {
                    found_left = left;
                    found_right = right;
                    found_mismatches = mismatch_count;
                    found = true;
                    continue;
                }
                let error0: u64 = Self::compute_error(found_left, found_right, found_mismatches);
                let error1: u64 = Self::compute_error(left, right, mismatch_count);
                if error1 >= error0 {
                    continue;
                }
                found_left = left;
                found_right = right;
                found_mismatches = mismatch_count;
            }
        }

        if should_update_vertical_data {
            if found {
                self.found_vertical_symmetry = true;
                self.top = found_left;
                self.bottom = found_right;
                self.vertical_mismatches = found_mismatches;
            } else {
                self.found_vertical_symmetry = false;
                self.top = u8::MAX;
                self.bottom = u8::MAX;
                self.vertical_mismatches = u16::MAX;
            }
        } else {
            if found {
                self.found_horizontal_symmetry = true;
                self.left = found_left;
                self.right = found_right;
                self.horizontal_mismatches = found_mismatches;
            } else {
                self.found_horizontal_symmetry = false;
                self.left = u8::MAX;
                self.right = u8::MAX;
                self.horizontal_mismatches = u16::MAX;
            }
        }

        Ok(())
    }

    fn compute_error(left: u8, right: u8, mismatches: u16) -> u64 {
        let left_squared: u16 = (left as u16) * (left as u16);
        let right_squared: u16 = (right as u16) * (right as u16);
        let mismatches_squared: u32 = (mismatches as u32) * (mismatches as u32);
        let sum: u64 = (left_squared as u64) + (right_squared as u64) + (mismatches_squared as u64);
        sum
    }

}

impl fmt::Debug for DetectSymmetry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "{}, {}, {}, {}", 
            self.horizontal_to_string(), 
            self.vertical_to_string(),
            self.diagonal_a_to_string(),
            self.diagonal_b_to_string(),
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_horizontal_symmetry_perfect() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 3, 2, 1, 0,
        ];
        let input: Image = Image::try_create(7, 1, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
    }

    #[test]
    fn test_10001_horizontal_symmetry_left1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 3, 2, 1,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 1 right: 0");
    }

    #[test]
    fn test_10002_horizontal_symmetry_right1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 2, 1, 0,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
    }

    #[test]
    fn test_10003_horizontal_symmetry_left1junk_right1junk() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 1, 2, 3, 2, 1, 8,
        ];
        let input: Image = Image::try_create(7, 1, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 2");
    }

    #[test]
    fn test_10004_horizontal_symmetry_none() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6,
        ];
        let input: Image = Image::try_create(7, 1, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "no horizontal symmetry");
    }

    #[test]
    fn test_10005_horizontal_symmetry_alternating() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0, 1,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
    }

    #[test]
    fn test_20000_vertical_symmetry_perfect() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0,
            1,
            2,
            3,
            2,
            1,
            0,
        ];
        let input: Image = Image::try_create(1, 7, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 0");
    }

    #[test]
    fn test_20001_vertical_symmetry_top1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0,
            1,
            2,
            3,
            2,
            1,
        ];
        let input: Image = Image::try_create(1, 6, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 1 bottom: 0");
    }

    #[test]
    fn test_20002_vertical_symmetry_bottom1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,
            2,
            3,
            2,
            1,
            0,
        ];
        let input: Image = Image::try_create(1, 6, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 1");
    }

    #[test]
    fn test_20003_vertical_symmetry_top1junk_bottom1junk() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9,
            1,
            2,
            3,
            2,
            1,
            8,
        ];
        let input: Image = Image::try_create(1, 7, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 0 mismatches: 2");
    }

    #[test]
    fn test_20000_vertical_symmetry_none() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0,
            1,
            2,
            3,
            4,
            5,
            6,
        ];
        let input: Image = Image::try_create(1, 7, pixels).expect("image");

        // Act
        let mut instance = DetectSymmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "no vertical symmetry");
    }

    #[test]
    fn test_30000_analyze_checkerboard() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 1, 0,
            0, 1, 0, 1, 0, 1,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 0");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30001_analyze_checkerboard_with_one_junk_pixel() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 1,
            0, 0, 1, 9, 0, 0,
            0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 2");
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 0 mismatches: 2");
        assert_eq!(instance.diagonal_a_to_string(), "diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "partial diagonal-b symmetry, mismatches: 2");
    }

    #[test]
    fn test_30002_analyze_nosymmetry() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5, 6,
            1, 2, 3, 4, 5, 6,
            8, 8, 8, 8, 8, 8,
            8, 1, 8, 1, 8, 1,
            0, 0, 1, 1, 2, 2,
            0, 0, 1, 1, 2, 2,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "no horizontal symmetry");
        assert_eq!(instance.vertical_to_string(), "no vertical symmetry");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30003_analyze_one_junk_pixel() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 1,
            0, 0, 1, 9, 0, 0,
            1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(6, 5, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 2");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 0");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30004_analyze_lines() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            5, 5, 5, 5, 5, 5,
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 7, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 2 bottom: 0");
        assert_eq!(instance.diagonal_a_to_string(), "partial diagonal-a symmetry, mismatches: 8");
        assert_eq!(instance.diagonal_b_to_string(), "partial diagonal-b symmetry, mismatches: 8");
    }

    #[test]
    fn test_30005_analyze_boxes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1, 1, 0,
            1, 1, 0, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0,
            1, 1, 0, 0, 1, 1, 0,
            1, 1, 0, 0, 1, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(7, 8, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 2");
        assert_eq!(instance.diagonal_a_to_string(), "diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "diagonal-b symmetry");
    }

    #[test]
    fn test_30006_analyze_border_pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "no horizontal symmetry");
        assert_eq!(instance.vertical_to_string(), "no vertical symmetry");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30007_analyze_border_pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            0, 1, 1, 1, 0, 1, 0,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "no horizontal symmetry");
        assert_eq!(instance.vertical_to_string(), "no vertical symmetry");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30008_analyze_diagonal_symmetry() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 1, 0,
            1, 8, 8, 8, 1,
            2, 8, 8, 8, 2,
            1, 8, 8, 8, 1,
            0, 1, 2, 1, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let instance = DetectSymmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 0");
        assert_eq!(instance.diagonal_a_to_string(), "diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "diagonal-b symmetry");
    }
}
