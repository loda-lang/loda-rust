use super::{Image, ImageRotate90};
use num_integer::Integer;

pub trait ImageCenterIndicator {
    /// Draw indicators for where the centers are in the image.
    /// 
    /// The indicators are drawn in the horizontal direction.
    /// 
    /// The value `0` is for the area before the center pixels.
    /// 
    /// The value `1` is for the center pixel. When the line segment is an odd number of pixels, there is only 1 center pixel.
    /// 
    /// The value `2` and `3` is for the center pixels. When the line segment is an even number of pixels, there are 2 center pixels.
    /// Value `2` is for the left-most pixel. Value `3` is for the right-most pixel.
    ///
    /// The value `4` is for the area after the center pixels.
    fn center_indicator_x(&self) -> anyhow::Result<Image>;

    /// Draw indicators for where the centers are in the image.
    /// 
    /// The indicators are drawn in the vertical direction.
    /// 
    /// The value `0` is for the area before the center pixels.
    /// 
    /// The value `1` is for the center pixel. When the line segment is an odd number of pixels, there is only 1 center pixel.
    /// 
    /// The value `2` and `3` is for the center pixels. When the line segment is an even number of pixels, there are 2 center pixels.
    /// Value `2` is for the top-most pixel. Value `3` is for the bottom-most pixel.
    ///
    /// The value `4` is for the area after the center pixels.
    fn center_indicator_y(&self) -> anyhow::Result<Image>;
}

impl ImageCenterIndicator for Image {
    fn center_indicator_x(&self) -> anyhow::Result<Image> {
        draw_center_indicator_x(self)
    }

    fn center_indicator_y(&self) -> anyhow::Result<Image> {
        let image0: Image = self.rotate_ccw()?;
        let image1: Image = draw_center_indicator_x(&image0)?;
        let image2: Image = image1.rotate_cw()?;
        Ok(image2)
    }
}

fn draw_center_indicator_x(input: &Image) -> anyhow::Result<Image> {
    if input.is_empty() {
        return Ok(Image::empty());
    }
    let mut result_image: Image = input.clone_color(4);

    for y in 0..input.height() {
        // Measure how many times the same color gets repeated
        let mut current_length: u8 = 0;
        let mut last_color: Option<u8> = None;
        let mut length_vec = Vec::<u8>::new();
        for x in 0..input.width() {
            let color: u8 = input.get(x as i32, y as i32).unwrap_or(255);
            if last_color == Some(color) {
                current_length += 1;
            } else {
                if current_length > 0 {
                    length_vec.push(current_length);
                }
                current_length = 1;
                last_color = Some(color);
            }
        }
        if current_length > 0 {
            length_vec.push(current_length);
        }

        // Draw center indicators
        let mut x: u8 = 0;
        for length in length_vec {
            let center_x: u8;
            if length.is_odd() {
                // Odd number of pixels, so there is only 1 center pixel.
                // Set a pixel at half length
                let set_x0: u8 = x + length / 2;
                _ = result_image.set(set_x0 as i32, y as i32, 1);
                center_x = set_x0;
            } else {
                // Even number of pixels. There is not a single center pixel. Instead there are 2 pixels adjacent to the center.
                // Set both pixels around the center pixel
                let set_x0: u8 = x + (length - 1) / 2;
                let set_x1: u8 = set_x0 + 1;
                _ = result_image.set(set_x0 as i32, y as i32, 2);
                _ = result_image.set(set_x1 as i32, y as i32, 3);
                center_x = set_x0;
            }

            // Fill the area before the center pixels
            for xx in x..center_x {
                _ = result_image.set(xx as i32, y as i32, 0);
            }

            x += length;
        }
    }
    Ok(result_image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_center_indicator_x() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
            1, 1, 0, 0, 0, 0,
            1, 1, 1, 0, 0, 0,
            1, 1, 1, 1, 0, 0,
            1, 1, 1, 1, 1, 0,
            1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 7, pixels).expect("image");

        // Act
        let actual: Image = input.center_indicator_x().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 2, 3, 4, 4,
            1, 0, 0, 1, 4, 4,
            2, 3, 0, 2, 3, 4,
            0, 1, 4, 0, 1, 4,
            0, 2, 3, 4, 2, 3,
            0, 0, 1, 4, 4, 1,
            0, 0, 2, 3, 4, 4,
        ];
        let expected: Image = Image::try_create(6, 7, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_center_indicator_y() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
            1, 1, 0, 0, 0, 0,
            1, 1, 1, 0, 0, 0,
            1, 1, 1, 1, 0, 0,
            1, 1, 1, 1, 1, 0,
            1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 7, pixels).expect("image");

        // Act
        let actual: Image = input.center_indicator_y().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 0, 0, 0, 0,
            0, 3, 1, 2, 0, 0,
            0, 0, 4, 3, 1, 2,
            2, 0, 0, 4, 4, 3,
            3, 1, 2, 0, 4, 4,
            4, 4, 3, 1, 2, 4,
            4, 4, 4, 4, 3, 1,
        ];
        let expected: Image = Image::try_create(6, 7, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
