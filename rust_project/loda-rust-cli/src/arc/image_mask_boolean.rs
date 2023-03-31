use super::Image;

pub trait ImageMaskBoolean {
    /// Performs an `xor` operation between `self` and another `mask`.
    /// 
    /// Both images must have same size.
    fn mask_xor(&self, mask: &Image) -> anyhow::Result<Image>;
}

impl ImageMaskBoolean for Image {
    fn mask_xor(&self, mask: &Image) -> anyhow::Result<Image> {
        let self_width: u8 = self.width();
        let self_height: u8 = self.height();
        if self_width != mask.width() || self_height != mask.height() {
            return Err(anyhow::anyhow!("Both images must have same size. mask: {}x{} image: {}x{}", self_width, self_height, mask.width(), mask.height()));
        }
        let mut image = Image::zero(self_width, self_height);
        for y in 0..self_height as i32 {
            for x in 0..self_width as i32 {
                let color0: u8 = self.get(x, y).unwrap_or(255);
                let color1: u8 = mask.get(x, y).unwrap_or(255);
                let value0: bool = color0 > 0;
                let value1: bool = color1 > 0;
                let set_color: u8 = match value0 ^ value1 {
                    false => 0,
                    true => 1,
                };
                let _ = image.set(x, y, set_color);
            }
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_mask_xor() {
        // Arrange
        let input0: Image = Image::try_create(4, 1, vec![0, 0, 1, 1]).expect("image");
        let input1: Image = Image::try_create(4, 1, vec![0, 1, 1, 0]).expect("image");

        // Act
        let actual: Image = input0.mask_xor(&input1).expect("image");

        // Assert
        let expected: Image = Image::try_create(4, 1, vec![0, 1, 0, 1]).expect("image");
        assert_eq!(actual, expected);
    }
}
