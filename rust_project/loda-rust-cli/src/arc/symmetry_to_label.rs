use super::{Symmetry, SymmetryLabel};
use std::collections::HashSet;

pub trait SymmetryToLabel {
    fn to_symmetry_labels(&self) -> HashSet<SymmetryLabel>;
}

impl SymmetryToLabel for Symmetry {
    fn to_symmetry_labels(&self) -> HashSet<SymmetryLabel> {
        let mut result = HashSet::<SymmetryLabel>::new();

        if self.horizontal_found {
            if self.horizontal_mismatches == 0 {
                if self.horizontal_left == 0 && self.horizontal_right == 0 {
                    result.insert(SymmetryLabel::Horizontal);
                    result.insert(SymmetryLabel::HorizontalWithInset);
                    result.insert(SymmetryLabel::HorizontalWithMismatches);
                    result.insert(SymmetryLabel::HorizontalWithInsetAndMismatches);
                } else {
                    result.insert(SymmetryLabel::HorizontalWithInset);
                    result.insert(SymmetryLabel::HorizontalWithInsetAndMismatches);
                }
            } else {
                if self.horizontal_left == 0 && self.horizontal_right == 0 {
                    result.insert(SymmetryLabel::HorizontalWithMismatches);
                    result.insert(SymmetryLabel::HorizontalWithInsetAndMismatches);
                } else {
                    result.insert(SymmetryLabel::HorizontalWithInsetAndMismatches);
                }
            }
        }

        if self.vertical_found {
            if self.vertical_mismatches == 0 {
                if self.vertical_top == 0 && self.vertical_bottom == 0 {
                    result.insert(SymmetryLabel::Vertical);
                    result.insert(SymmetryLabel::VerticalWithInset);
                    result.insert(SymmetryLabel::VerticalWithMismatches);
                    result.insert(SymmetryLabel::VerticalWithInsetAndMismatches);
                } else {
                    result.insert(SymmetryLabel::VerticalWithInset);
                    result.insert(SymmetryLabel::VerticalWithInsetAndMismatches);
                }
            } else {
                if self.vertical_top == 0 && self.vertical_bottom == 0 {
                    result.insert(SymmetryLabel::VerticalWithMismatches);
                    result.insert(SymmetryLabel::VerticalWithInsetAndMismatches);
                } else {
                    result.insert(SymmetryLabel::VerticalWithInsetAndMismatches);
                }
            }
        }

        if self.diagonal_a_found {
            if self.diagonal_a_mismatches == 0 {
                result.insert(SymmetryLabel::DiagonalA);
                result.insert(SymmetryLabel::DiagonalAWithMismatches);
            } else {
                result.insert(SymmetryLabel::DiagonalAWithMismatches);
            }
        }

        if self.diagonal_b_found {
            if self.diagonal_b_mismatches == 0 {
                result.insert(SymmetryLabel::DiagonalB);
                result.insert(SymmetryLabel::DiagonalBWithMismatches);
            } else {
                result.insert(SymmetryLabel::DiagonalBWithMismatches);
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
    fn test_10000_symmetry_yes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0,
            1, 0, 1,
            0, 1, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let symmetry: Symmetry = Symmetry::analyze(&input).expect("ok");

        // Act
        let actual: HashSet<SymmetryLabel> = symmetry.to_symmetry_labels();

        // Assert
        assert_eq!(actual.contains(&SymmetryLabel::Horizontal), true);
        assert_eq!(actual.contains(&SymmetryLabel::Vertical), true);
        assert_eq!(actual.contains(&SymmetryLabel::DiagonalA), true);
        assert_eq!(actual.contains(&SymmetryLabel::DiagonalB), true);
    }

    #[test]
    fn test_10001_symmetry_no() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
            3, 4, 5,
            6, 7, 8,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let symmetry: Symmetry = Symmetry::analyze(&input).expect("ok");

        // Act
        let actual: HashSet<SymmetryLabel> = symmetry.to_symmetry_labels();

        // Assert
        assert_eq!(actual.contains(&SymmetryLabel::Horizontal), false);
        assert_eq!(actual.contains(&SymmetryLabel::Vertical), false);
        assert_eq!(actual.contains(&SymmetryLabel::DiagonalA), false);
        assert_eq!(actual.contains(&SymmetryLabel::DiagonalB), false);
    }
}
