use super::{SingleColorObjectRectangle, SingleColorObjectRectangleLabel};

pub trait SingleColorObjectSatisfiesLabel {
    fn satisfies_label(&self, label: &SingleColorObjectRectangleLabel) -> bool;
}

impl SingleColorObjectSatisfiesLabel for SingleColorObjectRectangle {
    fn satisfies_label(&self, label: &SingleColorObjectRectangleLabel) -> bool {
        match label {
            SingleColorObjectRectangleLabel::SquareWithColor { color } => {
                self.color == *color && self.is_square == true
            },
            SingleColorObjectRectangleLabel::NonSquareWithColor { color } => {
                self.color == *color && self.is_square == false
            },
            SingleColorObjectRectangleLabel::RectangleWithColor { color } => {
                self.color == *color
            },
            SingleColorObjectRectangleLabel::SquareWithSomeColor => {
                self.is_square == true
            },
            SingleColorObjectRectangleLabel::NonSquareWithSomeColor => {
                self.is_square == false
            },
            SingleColorObjectRectangleLabel::RectangleWithSomeColor => {
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

        let object = SingleColorObjectRectangle {
            color: 7,
            mask,
            bounding_box: Rectangle::new(1, 0, 2, 2),
            mass: 4,
            is_square: true,
        };
        
        let mut pairs = Vec::<(bool, SingleColorObjectRectangleLabel)>::new();
        pairs.push((true, SingleColorObjectRectangleLabel::SquareWithColor { color: 7 }));
        pairs.push((true, SingleColorObjectRectangleLabel::SquareWithSomeColor));
        pairs.push((true, SingleColorObjectRectangleLabel::RectangleWithColor { color: 7 }));
        pairs.push((true, SingleColorObjectRectangleLabel::RectangleWithSomeColor));
        pairs.push((false, SingleColorObjectRectangleLabel::SquareWithColor { color: 42 }));
        pairs.push((false, SingleColorObjectRectangleLabel::NonSquareWithColor { color: 7 }));
        pairs.push((false, SingleColorObjectRectangleLabel::NonSquareWithSomeColor));
        pairs.push((false, SingleColorObjectRectangleLabel::RectangleWithColor { color: 42 }));

        let labels: Vec<SingleColorObjectRectangleLabel> = pairs.iter().map(|(_expected,label)| label.clone()).collect();
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

        let object = SingleColorObjectRectangle {
            color: 7,
            mask,
            bounding_box: Rectangle::new(1, 0, 2, 3),
            mass: 6,
            is_square: false,
        };
        
        let mut pairs = Vec::<(bool, SingleColorObjectRectangleLabel)>::new();
        pairs.push((true, SingleColorObjectRectangleLabel::NonSquareWithColor { color: 7 }));
        pairs.push((true, SingleColorObjectRectangleLabel::NonSquareWithSomeColor));
        pairs.push((true, SingleColorObjectRectangleLabel::RectangleWithColor { color: 7 }));
        pairs.push((true, SingleColorObjectRectangleLabel::RectangleWithSomeColor));
        pairs.push((false, SingleColorObjectRectangleLabel::SquareWithColor { color: 7 }));
        pairs.push((false, SingleColorObjectRectangleLabel::SquareWithColor { color: 42 }));
        pairs.push((false, SingleColorObjectRectangleLabel::SquareWithSomeColor));
        pairs.push((false, SingleColorObjectRectangleLabel::RectangleWithColor { color: 42 }));

        let labels: Vec<SingleColorObjectRectangleLabel> = pairs.iter().map(|(_expected,label)| label.clone()).collect();
        let expected: Vec<bool> = pairs.iter().map(|(expected,_label)| *expected).collect();

        // Act
        let actual: Vec<bool> = labels.iter().map(|label| object.satisfies_label(label)).collect();

        // Assert
        assert_eq!(actual, expected);
    }
}
