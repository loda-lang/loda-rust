use super::{Split, SplitLabel};
use std::collections::HashSet;

pub trait SplitToLabel {
    fn to_split_labels(&self) -> HashSet<SplitLabel>;
}

impl SplitToLabel for Split {
    fn to_split_labels(&self) -> HashSet<SplitLabel> {
        let mut result = HashSet::<SplitLabel>::new();

        let mut separator_size_x: Option<u8> = None;
        let mut separator_size_y: Option<u8> = None;
        let mut part_size_x: Option<u8> = None;
        let mut part_size_y: Option<u8> = None;

        if let Some(split) = self.x_container.maximize_even_splits() {
            separator_size_x = Some(split.separator_size);
            part_size_x = Some(split.part_size);
            {
                let label = SplitLabel::SplitWithSomeColor;
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitColor { color: split.separator_color };
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
            {
                let label = SplitLabel::SplitSeparatorSizeX { size: split.separator_size };
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitSeparatorCountX { count: split.separator_count };
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitPartSizeX { size: split.part_size };
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitPartCountX { count: split.part_count };
                result.insert(label);
            }
        }

        if let Some(split) = self.y_container.maximize_even_splits() {
            separator_size_y = Some(split.separator_size);
            part_size_y = Some(split.part_size);
            {
                let label = SplitLabel::SplitWithSomeColor;
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitColor { color: split.separator_color };
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
            {
                let label = SplitLabel::SplitSeparatorSizeY { size: split.separator_size };
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitSeparatorCountY { count: split.separator_count };
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitPartSizeY { size: split.part_size };
                result.insert(label);
            }
            {
                let label = SplitLabel::SplitPartCountY { count: split.part_count };
                result.insert(label);
            }
        }

        if let Some(size_x) = separator_size_x {
            if separator_size_x == separator_size_y {
                let label = SplitLabel::SplitSeparatorSizeXY { size: size_x };
                result.insert(label);
            }
        }

        if let Some(size_x) = part_size_x {
            if part_size_x == part_size_y {
                let label = SplitLabel::SplitPartSizeXY { size: size_x };
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
        assert_eq!(actual.contains(&SplitLabel::SplitSeparatorCountX { count: 1 }), true);
        assert_eq!(actual.contains(&SplitLabel::SplitSeparatorSizeXY { size: 1 }), true);
        assert_eq!(actual.contains(&SplitLabel::SplitPartSizeXY { size: 1 }), true);
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
