use super::{Image, ImageCrop, Rectangle};

pub enum ImageSplitDirection {
    IntoColumns,
    IntoRows,
}

pub trait ImageSplit {
    fn split(&self, number_of_parts: u8, spacing: u8, direction: ImageSplitDirection) -> anyhow::Result<Vec<Image>>;
}

impl ImageSplit for Image {
    fn split(&self, number_of_parts: u8, spacing: u8, direction: ImageSplitDirection) -> anyhow::Result<Vec<Image>> {
        if number_of_parts < 2 {
            return Err(anyhow::anyhow!("number_of_parts must be 2 or greater"));
        }
        if self.is_empty() {
            return Err(anyhow::anyhow!("The image must not be empty"));
        }
        let mut result_images = Vec::<Image>::new();

        let size: i32 = match direction {
            ImageSplitDirection::IntoColumns => self.width() as i32,
            ImageSplitDirection::IntoRows => self.height() as i32,
        };

        // Determine the size of a part
        let spacing_sum: i32 = ((number_of_parts as i32) - 1) * (spacing as i32);
        let content_size_raw: i32 = size - spacing_sum;
        if content_size_raw < 0 {
            return Err(anyhow::anyhow!("Content size must be positive"));
        }
        if content_size_raw > (u8::MAX as i32) {
            return Err(anyhow::anyhow!("Integrity error. content size should not exceed u8::MAX"));
        }
        let content_size = content_size_raw as u8;
        let part_size: u8 = content_size / number_of_parts;
        let remaining: u8 = content_size % number_of_parts;
        if remaining != 0 {
            return Err(anyhow::anyhow!("Remainder must be 0. Cannot split {} pixels into {} parts with {} spacing", size, number_of_parts, spacing));
        }
        if part_size < 1 {
            return Err(anyhow::anyhow!("Part size must be 1 or greater"));
        }

        // Crop the parts
        for i in 0..number_of_parts {
            let position: u16 = (i as u16) * (part_size as u16 + spacing as u16);
            if position > (u8::MAX as u16) {
                return Err(anyhow::anyhow!("Integrity error. x should not exceed u8::MAX"));
            }
            let rect: Rectangle = match direction {
                ImageSplitDirection::IntoColumns => Rectangle::new(position as u8, 0, part_size, self.height()),
                ImageSplitDirection::IntoRows => Rectangle::new(0, position as u8, self.width(), part_size),
            };
            let image: Image = self.crop(rect)?;
            result_images.push(image);
        }

        Ok(result_images)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, ImageStack};

    #[test]
    fn test_10000_split_columns_spacing0() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 2, 2, 3, 3,
            1, 1, 2, 2, 3, 3,
            0, 1, 0, 2, 0, 3,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");

        // Act
        let actual_images: Vec<Image> = input.split(3, 0, ImageSplitDirection::IntoColumns).expect("vec");

        // Assert
        assert_eq!(actual_images.len(), 3);
        let actual: Image = Image::hstack(actual_images).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 2, 2, 3, 3,
            1, 1, 2, 2, 3, 3,
            0, 1, 0, 2, 0, 3,
        ];
        let expected: Image = Image::try_create(6, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_split_columns_spacing1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 7, 2, 7, 3, 7, 0,
            1, 7, 2, 7, 3, 7, 4,
            0, 7, 2, 7, 3, 7, 4,
        ];
        let input: Image = Image::try_create(7, 3, pixels).expect("image");

        // Act
        let actual_images: Vec<Image> = input.split(4, 1, ImageSplitDirection::IntoColumns).expect("vec");

        // Assert
        let actual: Image = Image::hstack(actual_images).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3, 0,
            1, 2, 3, 4,
            0, 2, 3, 4,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_split_rows_spacing1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0,
            7, 7, 7,
            2, 2, 2,
            7, 7, 7,
            3, 3, 3,
            7, 7, 7,
            0, 4, 4,
        ];
        let input: Image = Image::try_create(3, 7, pixels).expect("image");

        // Act
        let actual_images: Vec<Image> = input.split(4, 1, ImageSplitDirection::IntoRows).expect("vec");

        // Assert
        let actual: Image = Image::vstack(actual_images).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 0,
            2, 2, 2,
            3, 3, 3,
            0, 4, 4,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

}
