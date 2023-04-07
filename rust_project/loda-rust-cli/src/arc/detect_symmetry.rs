use super::{Image, ImageCompare, ImageCrop, ImageMaskCount, ImageSymmetry, Rectangle};

pub struct DetectSymmetry {
    left: u8,
    right: u8,
    found_horizontal_symmetry: bool,
    top: u8,
    bottom: u8,
    found_vertical_symmetry: bool,
}

impl DetectSymmetry {
    pub fn new() -> Self {
        Self {
            left: u8::MAX,
            right: u8::MAX,
            found_horizontal_symmetry: false,
            top: u8::MAX,
            bottom: u8::MAX,
            found_vertical_symmetry: false,
        }
    }

    fn horizontal_to_string(&self) -> String {
        if self.found_horizontal_symmetry {
            return format!("horizontal symmetry, left: {} right: {}", self.left, self.right);
        } else {
            return "no horizontal symmetry".to_string();
        }
    }

    fn vertical_to_string(&self) -> String {
        if self.found_vertical_symmetry {
            return format!("vertical symmetry, top: {} bottom: {}", self.top, self.bottom);
        } else {
            return "no vertical symmetry".to_string();
        }
    }

    fn analyze_horizontal_symmetry(&mut self, image: &Image) -> anyhow::Result<()> {
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let mut found: bool = false;
        let mut found_left: u8 = u8::MAX;
        let mut found_right: u8 = u8::MAX;
        for left in 0..5u8 {
            for right in 0..5u8 {
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
                let agree_count: u32 = diff.mask_count_one();
                // println!("pair: {} left: {} right: {} agree: {}", data.index, left, right, agree_count);
                if (agree_count * 2) > (area as u32) {
                    continue;
                }
                // if agree_count > (attention_mass as u32) * 2 {
                //     continue;
                // }
                // HtmlLog::text(format!("left: {} right: {} agree: {}", left, right, agree_count));
                // HtmlLog::image(&diff);
                // println!("left: {} right: {} agree: {}", left, right, agree_count);
                if !found {
                    found_left = left;
                    found_right = right;
                    found = true;
                    continue;
                }
                let error0 = found_left * found_left + found_right * found_right;
                let error1 = left * left + right * right;
                if error1 >= error0 {
                    // println!("skip bigger errors");
                    continue;
                }
                found_left = left;
                found_right = right;
            }
        }

        // if !found {
        //     // return Err(anyhow::anyhow!("Unable to find symmetry"));
        //     HtmlLog::text(format!("pair: {} no symmetry", data.index));
        // } else {
        //     HtmlLog::text(format!("pair: {} left: {} right: {}  symmetry", data.index, found_left, found_right));
        // }

        if found {
            self.found_horizontal_symmetry = true;
            self.left = found_left;
            self.right = found_right;
        } else {
            self.found_horizontal_symmetry = false;
            self.left = u8::MAX;
            self.right = u8::MIN;
        }

        Ok(())
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
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
    }
}
