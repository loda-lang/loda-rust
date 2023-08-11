use super::{Image, ImageMaskBoolean};

#[derive(Debug, Clone)]
pub enum Operation {
    MaskAnd,
    MaskOr,
    MaskXor,
}

impl Operation {
    pub fn execute(&self, image0: &Image, image1: &Image) -> anyhow::Result<Image> {
        match self {
            Self::MaskAnd => {
                image0.mask_and(image1)
            },
            Self::MaskOr => {
                image0.mask_or(image1)
            },
            Self::MaskXor => {
                image0.mask_xor(image1)
            },
        }
    }

    pub fn execute_with_images(&self, images: &Vec<Image>) -> anyhow::Result<Image> {
        let mut work_image = match images.first() {
            Some(value) => value.clone(),
            None => {
                return Err(anyhow::anyhow!("Cannot apply operation to empty list of images"));
            }
        };
        for (image_index, image) in images.iter().enumerate() {
            if image_index == 0 {
                continue;
            }
            work_image = self.execute(&work_image, image)?;
        }
        Ok(work_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_or() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0,
            1, 1, 1, 1,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            1, 1, 1, 1,
        ];
        let input2: Image = Image::try_create(4, 3, pixels2).expect("image");

        let images = vec![input0, input1, input2];

        // Act
        let actual: Image = Operation::MaskOr.execute_with_images(&images).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
