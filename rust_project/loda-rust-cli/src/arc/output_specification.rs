use super::{Image, ImageHistogram, ImageSize};

/// Future experiment:
/// In an ARC task. Give each test pair a `Specification` with what the output is supposed to be.
/// If the `Specification` is not satisfied, the prediction can be rejected.
#[derive(Clone, Debug)]
pub enum OutputSpecification {
    #[allow(dead_code)]
    ImageSize { size: ImageSize },

    #[allow(dead_code)]
    TwoOrMoreUniqueColors,
}

impl OutputSpecification {
    /// Returns either `true` or `false` if the specification is satisfied.
    ///
    /// The specification may return an error when encountering something unexpected.
    #[allow(dead_code)]
    pub fn is_satisfied_by(&self, image: &Image) -> anyhow::Result<bool> {
        match self {
            OutputSpecification::ImageSize { size } => {
                Ok(image.size() == *size)
            },
            OutputSpecification::TwoOrMoreUniqueColors => {
                let count: u16 = image.histogram_all().number_of_counters_greater_than_zero();
                Ok(count >= 2)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_two_or_more_unique_colors() {
        {
            // Arrange
            let input: Image = Image::color(3, 3, 7);
    
            // Act
            let actual: bool = OutputSpecification::TwoOrMoreUniqueColors.is_satisfied_by(&input).expect("ok");
    
            // Assert
            assert_eq!(actual, false);
        }
        {
            // Arrange
            let input: Image = Image::try_create(1, 2, vec![5, 7]).expect("image");

            // Act
            let actual: bool = OutputSpecification::TwoOrMoreUniqueColors.is_satisfied_by(&input).expect("ok");

            // Assert
            assert_eq!(actual, true);
        }
    }

    #[test]
    fn test_20000_image_size() {
        {
            // Arrange
            let input: Image = Image::color(4, 3, 7);
    
            // Act
            let actual: bool = OutputSpecification::ImageSize { size: ImageSize { width: 3, height: 4 } }.is_satisfied_by(&input).expect("ok");
    
            // Assert
            assert_eq!(actual, false);
        }
        {
            // Arrange
            let input: Image = Image::color(3, 4, 7);

            // Act
            let actual: bool = OutputSpecification::ImageSize { size: ImageSize { width: 3, height: 4 } }.is_satisfied_by(&input).expect("ok");

            // Assert
            assert_eq!(actual, true);
        }
    }
}
