use super::{Image, ImageMix, MixMode};

pub trait ImageMaskBoolean {
    /// Performs an `xor` operation between `self` and another `mask`.
    /// 
    /// Both images must have same size.
    fn mask_xor(&self, mask: &Image) -> anyhow::Result<Image>;

    /// Performs an `and` operation between `self` and another `mask`.
    /// 
    /// Both images must have same size.
    fn mask_and(&self, mask: &Image) -> anyhow::Result<Image>;

    /// Performs an `or` operation between `self` and another `mask`.
    /// 
    /// Both images must have same size.
    fn mask_or(&self, mask: &Image) -> anyhow::Result<Image>;
}

impl ImageMaskBoolean for Image {
    fn mask_xor(&self, mask: &Image) -> anyhow::Result<Image> {
        self.mix(mask, MixMode::BooleanXor)
    }

    fn mask_and(&self, mask: &Image) -> anyhow::Result<Image> {
        self.mix(mask, MixMode::BooleanAnd)
    }

    fn mask_or(&self, mask: &Image) -> anyhow::Result<Image> {
        self.mix(mask, MixMode::BooleanOr)
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

    #[test]
    fn test_20000_mask_and() {
        // Arrange
        let input0: Image = Image::try_create(4, 1, vec![0, 0, 1, 1]).expect("image");
        let input1: Image = Image::try_create(4, 1, vec![0, 1, 1, 0]).expect("image");

        // Act
        let actual: Image = input0.mask_and(&input1).expect("image");

        // Assert
        let expected: Image = Image::try_create(4, 1, vec![0, 0, 1, 0]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_mask_or() {
        // Arrange
        let input0: Image = Image::try_create(4, 1, vec![0, 0, 1, 1]).expect("image");
        let input1: Image = Image::try_create(4, 1, vec![0, 1, 1, 0]).expect("image");

        // Act
        let actual: Image = input0.mask_or(&input1).expect("image");

        // Assert
        let expected: Image = Image::try_create(4, 1, vec![0, 1, 1, 1]).expect("image");
        assert_eq!(actual, expected);
    }
}
