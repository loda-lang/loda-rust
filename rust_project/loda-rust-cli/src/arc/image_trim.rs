use super::{Histogram, Image, ImageCrop, ImageHistogram, Rectangle};

pub trait ImageTrim {
    /// Determines the most popular border color and removes the area.
    fn trim(&self) -> anyhow::Result<Image>;

    /// Remove border with the specified color.
    fn trim_color(&self, color_to_be_trimmed: u8) -> anyhow::Result<Image>;

    /// Find the outer bounding box.
    /// 
    /// Bounding box of what remains after trimming with a specific color.
    fn bounding_box_trim_color(&self, color_to_be_trimmed: u8) -> anyhow::Result<Rectangle>;

    /// Find the inner bounding box.
    /// 
    /// Shrink the bounding box further until the majority of border pixels are no longer the trim color.
    fn shrink_bounding_box(&self, color_to_be_trimmed: u8, rect: Rectangle) -> anyhow::Result<Rectangle>;

    /// Remove border with the specified color. Shrink further until the majority of border pixels are not the trim color.
    fn trim_shrink_color(&self, color_to_be_trimmed: u8) -> anyhow::Result<Image>;

    // Idea for future experiment
    // Trim as much as possible around the mask, and remove the same area from the the image
    // fn trim_mask_and_image(mask: &Image, image: &Image) -> anyhow::Result<(Image, Image)>;
}

impl ImageTrim for Image {
    fn trim(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        
        // Determine what is the most popular pixel value
        // traverses the border of the bitmap, and builds a histogram
        let histogram: Histogram = self.histogram_border();
        let popular_border_pixel_value: u8 = match histogram.most_popular_color() {
            Some(value) => value,
            None => {
                return Ok(Image::empty());
            }
        };
        self.trim_color(popular_border_pixel_value)
    }

    fn trim_color(&self, color_to_be_trimmed: u8) -> anyhow::Result<Image> {
        let rect: Rectangle = self.bounding_box_trim_color(color_to_be_trimmed)?;
        if rect.is_empty() {
            return Ok(Image::empty());
        }
        let image: Image = self.crop(rect)?;
        Ok(image)
    }

    fn bounding_box_trim_color(&self, color_to_be_trimmed: u8) -> anyhow::Result<Rectangle> {
        if self.is_empty() {
            return Ok(Rectangle::empty());
        }

        // Find bounding box
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;
        let mut found_x0: i32 = x_max;
        let mut found_x1: i32 = 0;
        let mut found_y0: i32 = y_max;
        let mut found_y1: i32 = 0;
        for y in 0..=y_max {
            for x in 0..=x_max {
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                if pixel_value == color_to_be_trimmed {
                    continue;
                }

                // grow the bounding box
                found_x0 = i32::min(found_x0, x);
                found_x1 = i32::max(found_x1, x);
                found_y0 = i32::min(found_y0, y);
                found_y1 = i32::max(found_y1, y);
            }
        }

        if found_x0 > found_x1 || found_y0 > found_y1 {
            return Ok(Rectangle::empty());
        }

        // Left position
        if found_x0 < 0 || found_x0 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. found_x0: {}", found_x0));
        }
        let x: u8 = found_x0 as u8;

        // Top position
        if found_y0 < 0 || found_y0 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. found_y0: {}", found_y0));
        }
        let y: u8 = found_y0 as u8;

        // Width
        let new_width_i32: i32 = found_x1 - found_x0 + 1;
        if new_width_i32 < 1 || new_width_i32 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. new_width_i32: {}", new_width_i32));
        }
        let width: u8 = new_width_i32 as u8;

        // Height
        let new_height_i32: i32 = found_y1 - found_y0 + 1;
        if new_height_i32 < 1 || new_height_i32 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. new_height_i32: {}", new_height_i32));
        }
        let height: u8 = new_height_i32 as u8;

        Ok(Rectangle::new(x, y, width, height))
    }

    fn shrink_bounding_box(&self, color_to_be_trimmed: u8, rect: Rectangle) -> anyhow::Result<Rectangle> {
        if rect.is_empty() {
            return Ok(Rectangle::empty());
        }

        let mut found_x0: i32 = rect.min_x();
        let mut found_x1: i32 = rect.max_x();
        let mut found_y0: i32 = rect.min_y();
        let mut found_y1: i32 = rect.max_y();

        // TODO: shrink bounding box, to get rid of junk
        let limit: usize = 3;
        for _ in 0..2 {
            if true {
                // Shrink left
                let mut count: usize = 0;
                for y in found_y0..=found_y1 {
                    let pixel_value: u8 = self.get(found_x0, y).unwrap_or(255);
                    if pixel_value == color_to_be_trimmed {
                        count += 1;
                    }
                }
                // if the majority of pixels are the color to be trimmed, then trim it.
                // if the majority of pixels are the object, then don't trim it.
                // TODO: ignore if count < height*0.5
                if count > limit {
                    found_x0 += 1;
                }
            }
            if true {
                // Shrink right
                let mut count: usize = 0;
                for y in found_y0..=found_y1 {
                    let pixel_value: u8 = self.get(found_x1, y).unwrap_or(255);
                    if pixel_value == color_to_be_trimmed {
                        count += 1;
                    }
                }
                // TODO: ignore if count < height*0.5
                if count > limit {
                    found_x1 -= 1;
                }
            }
            if true {
                // Shrink top
                let mut count: usize = 0;
                for x in found_x0..=found_x1 {
                    let pixel_value: u8 = self.get(x, found_y0).unwrap_or(255);
                    if pixel_value == color_to_be_trimmed {
                        count += 1;
                    }
                }
                // TODO: ignore if count < width*0.5
                if count > limit {
                    found_y0 += 1;
                }
            }
            if true {
                // Shrink bottom
                let mut count: usize = 0;
                for x in found_x0..=found_x1 {
                    let pixel_value: u8 = self.get(x, found_y1).unwrap_or(255);
                    if pixel_value == color_to_be_trimmed {
                        count += 1;
                    }
                }
                // TODO: ignore if count < width*0.5
                if count > limit {
                    found_y1 -= 1;
                }
            }
        }

        if found_x0 > found_x1 || found_y0 > found_y1 {
            return Ok(Rectangle::empty());
        }

        // Left position
        if found_x0 < 0 || found_x0 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. found_x0: {}", found_x0));
        }
        let x: u8 = found_x0 as u8;

        // Top position
        if found_y0 < 0 || found_y0 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. found_y0: {}", found_y0));
        }
        let y: u8 = found_y0 as u8;

        // Width
        let new_width_i32: i32 = found_x1 - found_x0 + 1;
        if new_width_i32 < 1 || new_width_i32 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. new_width_i32: {}", new_width_i32));
        }
        let width: u8 = new_width_i32 as u8;

        // Height
        let new_height_i32: i32 = found_y1 - found_y0 + 1;
        if new_height_i32 < 1 || new_height_i32 > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. Bounding box coordinates are messed up. new_height_i32: {}", new_height_i32));
        }
        let height: u8 = new_height_i32 as u8;

        Ok(Rectangle::new(x, y, width, height))
    }

    fn trim_shrink_color(&self, color_to_be_trimmed: u8) -> anyhow::Result<Image> {
        let rect: Rectangle = self.bounding_box_trim_color(color_to_be_trimmed)?;
        let rect2: Rectangle = self.shrink_bounding_box(color_to_be_trimmed, rect)?;
        if rect2.is_empty() {
            return Ok(Image::empty());
        }
        let image: Image = self.crop(rect2)?;
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_bounding_box_trim_color_sunshine_scenario() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: Rectangle = input.bounding_box_trim_color(0).expect("rectangle");

        // Assert
        assert_eq!(actual, Rectangle::new(1, 1, 1, 1));
    }

    #[test]
    fn test_10001_bounding_box_trim_color_empty() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: Rectangle = input.bounding_box_trim_color(0).expect("rectangle");

        // Assert
        assert_eq!(actual, Rectangle::empty());
    }

    #[test]
    fn test_10002_bounding_box_trim_color_left() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0, 0,
            5, 1, 2, 0, 0,
            5, 3, 4, 0, 0,
            5, 5, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Rectangle = input.bounding_box_trim_color(5).expect("rectangle");

        // Assert
        assert_eq!(actual, Rectangle::new(1, 0, 4, 4));
    }

    #[test]
    fn test_10003_bounding_box_trim_color_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0, 0,
            5, 1, 2, 0, 0,
            5, 3, 4, 0, 0,
            5, 5, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Rectangle = input.bounding_box_trim_color(0).expect("rectangle");

        // Assert
        assert_eq!(actual, Rectangle::new(0, 0, 3, 4));
    }

    #[test]
    fn test_20000_trim_color_left() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim_color(5).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 0, 0,
            1, 2, 0,
            3, 4, 0,
            5, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_trim_color_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0,
            5, 1, 2, 0,
            5, 3, 4, 0,
            5, 5, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim_color(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 0,
            5, 1, 2,
            5, 3, 4,
            5, 5, 0,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_trim_border_with_zeroes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 2, 0,
            0, 3, 4, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30001_trim_all_10s() {
        // Arrange
        let pixels: Vec<u8> = vec![
            10, 10, 10, 10, 10,
            10,  1,  2, 10, 10,
            10,  3,  4, 10, 10,
            10, 10, 10, 10, 10,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![1, 2, 3, 4]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30002_trim_top_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            5, 5, 1, 1,
            5, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(2, 2, vec![5, 5, 5, 1]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30003_trim_left_right_bottom() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0,
            0, 0, 1, 0,
            0, 0, 1, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(1, 3, vec![1, 1, 1]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30003_trim_no_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::empty();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30004_trim_1pixel() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = Image::try_create(1, 1, vec![5]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30005_trim_2pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 5,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.trim().expect("image");

        // Assert
        let expected: Image = input.clone();
        assert_eq!(actual, expected);
    }
}
