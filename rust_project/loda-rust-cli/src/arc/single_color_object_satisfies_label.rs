use super::{SingleColorObject, SingleColorObjectLabel};

pub trait SingleColorObjectSatisfiesLabel {
    fn satisfies_label(&self, label: &SingleColorObjectLabel) -> bool;
}

impl SingleColorObjectSatisfiesLabel for SingleColorObject {
    fn satisfies_label(&self, label: &SingleColorObjectLabel) -> bool {
        match label {
            SingleColorObjectLabel::SquareWithColor { color } => {
                self.color == *color && self.is_square == true
            },
            SingleColorObjectLabel::NonSquareWithColor { color } => {
                self.color == *color && self.is_square == false
            },
            SingleColorObjectLabel::RectangleWithColor { color } => {
                self.color == *color
            },
            SingleColorObjectLabel::SquareWithSomeColor => {
                self.is_square == true
            },
            SingleColorObjectLabel::NonSquareWithSomeColor => {
                self.is_square == false
            },
            SingleColorObjectLabel::RectangleWithSomeColor => {
                true
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, Image, Rectangle};

    #[test]
    fn test_10000_square_true() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            0, 1, 1, 0,
            0, 0, 0, 0,
        ];
        let mask: Image = Image::try_create(4, 3, pixels).expect("image");

        let object = SingleColorObject {
            color: 7,
            mask,
            bounding_box: Rectangle::new(1, 0, 2, 2),
            mass: 4,
            is_square: true,
        };
        
        let mut pairs = Vec::<(bool, SingleColorObjectLabel)>::new();
        pairs.push((true, SingleColorObjectLabel::SquareWithColor { color: 7 }));
        pairs.push((true, SingleColorObjectLabel::SquareWithSomeColor));
        pairs.push((true, SingleColorObjectLabel::RectangleWithColor { color: 7 }));
        pairs.push((true, SingleColorObjectLabel::RectangleWithSomeColor));
        pairs.push((false, SingleColorObjectLabel::SquareWithColor { color: 42 }));
        pairs.push((false, SingleColorObjectLabel::NonSquareWithColor { color: 7 }));
        pairs.push((false, SingleColorObjectLabel::NonSquareWithSomeColor));
        pairs.push((false, SingleColorObjectLabel::RectangleWithColor { color: 42 }));

        let labels: Vec<SingleColorObjectLabel> = pairs.iter().map(|(_expected,label)| label.clone()).collect();
        let expected: Vec<bool> = pairs.iter().map(|(expected,_label)| *expected).collect();

        // Act
        let actual: Vec<bool> = labels.iter().map(|label| object.satisfies_label(label)).collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_square_false() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 1, 0,
            0, 1, 1, 0,
            0, 1, 1, 0,
        ];
        let mask: Image = Image::try_create(4, 3, pixels).expect("image");

        let object = SingleColorObject {
            color: 7,
            mask,
            bounding_box: Rectangle::new(1, 0, 2, 3),
            mass: 6,
            is_square: false,
        };
        
        let mut pairs = Vec::<(bool, SingleColorObjectLabel)>::new();
        pairs.push((true, SingleColorObjectLabel::NonSquareWithColor { color: 7 }));
        pairs.push((true, SingleColorObjectLabel::NonSquareWithSomeColor));
        pairs.push((true, SingleColorObjectLabel::RectangleWithColor { color: 7 }));
        pairs.push((true, SingleColorObjectLabel::RectangleWithSomeColor));
        pairs.push((false, SingleColorObjectLabel::SquareWithColor { color: 7 }));
        pairs.push((false, SingleColorObjectLabel::SquareWithColor { color: 42 }));
        pairs.push((false, SingleColorObjectLabel::SquareWithSomeColor));
        pairs.push((false, SingleColorObjectLabel::RectangleWithColor { color: 42 }));

        let labels: Vec<SingleColorObjectLabel> = pairs.iter().map(|(_expected,label)| label.clone()).collect();
        let expected: Vec<bool> = pairs.iter().map(|(expected,_label)| *expected).collect();

        // Act
        let actual: Vec<bool> = labels.iter().map(|label| object.satisfies_label(label)).collect();

        // Assert
        assert_eq!(actual, expected);
    }
}
