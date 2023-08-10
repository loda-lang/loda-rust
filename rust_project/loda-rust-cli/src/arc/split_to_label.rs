use super::{Split, SplitLabel};
use std::collections::HashSet;

pub trait SplitToLabel {
    fn to_split_labels(&self) -> HashSet<SplitLabel>;
}

impl SplitToLabel for Split {
    fn to_split_labels(&self) -> HashSet<SplitLabel> {
        let mut result = HashSet::<SplitLabel>::new();

        if let Some(candidate) = self.even_splitx() {
            {
                let label = SplitLabel::SplitWithSomeColor;
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitColor { color: candidate.separator_color };
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitDirectionX;
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitDirectionSome;
                result.insert(label);
            }
        }

        if let Some(candidate) = self.even_splity() {
            {
                let label = SplitLabel::SplitWithSomeColor;
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitColor { color: candidate.separator_color };
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitDirectionY;
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitDirectionSome;
                result.insert(label);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{Image, ImageTryCreate};

    #[test]
    fn test_10000_split_yes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 5, 0,
            5, 5, 5,
            0, 5, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let split: Split = Split::analyze(&input).expect("ok");
        
        // Act
        let actual: HashSet<SplitLabel> = split.to_split_labels();

        // Assert
        assert_eq!(actual.contains(&SplitLabel::SplitWithSomeColor), true);
        assert_eq!(actual.contains(&SplitLabel::SplitColor { color: 5 }), true);
        assert_eq!(actual.contains(&SplitLabel::SplitDirectionX), true);
        assert_eq!(actual.contains(&SplitLabel::SplitDirectionY), true);
        assert_eq!(actual.contains(&SplitLabel::SplitDirectionSome), true);
    }

    #[test]
    fn test_10001_split_none() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
            3, 4, 5,
            6, 7, 8,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let split: Split = Split::analyze(&input).expect("ok");
        
        // Act
        let actual: HashSet<SplitLabel> = split.to_split_labels();

        // Assert
        assert_eq!(actual.contains(&SplitLabel::SplitDirectionSome), false);
        assert_eq!(actual.contains(&SplitLabel::SplitWithSomeColor), false);
    }
}
