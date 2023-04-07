use super::{Image, ImageCompare, ImageCrop, ImageMaskCount, ImageRotate, ImageSymmetry, Rectangle};
use std::fmt;

const MAX_INSET_VALUE: u8 = 5;

#[allow(dead_code)]
pub struct DetectSymmetry {
    left: u8,
    right: u8,
    horizontal_mismatches: u16,
    found_horizontal_symmetry: bool,

    top: u8,
    bottom: u8,
    vertical_mismatches: u16,
    found_vertical_symmetry: bool,

    // Idea for more
    // number of pixels that isn't symmetric
    // repair the damaged pixels
    // partial symmetry
    // full symmetry
    // if square area, identify if there is a diagonal symmetry
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
            top: u8::MAX,
            bottom: u8::MAX,
            found_vertical_symmetry: false,
            vertical_mismatches: u16::MAX,
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

    fn perform_analyze(&mut self, image: &Image) -> anyhow::Result<()> {
        self.analyze_horizontal_symmetry(image)?;
        self.analyze_vertical_symmetry(image)?;
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
        let part_of_width: u8 = (image.width() / 5) + 1;
        let max_inset: u8 = part_of_width.min(MAX_INSET_VALUE);
        for left in 0..max_inset {
            for right in 0..max_inset {
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
                let error0 = found_left * found_left + found_right * found_right;
                let error1 = left * left + right * right;
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

}

impl fmt::Debug for DetectSymmetry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.horizontal_to_string(), self.vertical_to_string())
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
        assert_eq!(instance.horizontal_mismatches, 0);
        assert_eq!(instance.vertical_mismatches, 0);
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
    }
}
