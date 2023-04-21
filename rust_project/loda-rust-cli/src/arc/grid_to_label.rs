use super::{Grid, GridLabel};
use std::collections::HashSet;

pub trait GridToLabel {
    fn to_grid_labels(&self) -> HashSet<GridLabel>;
}

impl GridToLabel for Grid {
    fn to_grid_labels(&self) -> HashSet<GridLabel> {
        let mut result = HashSet::<GridLabel>::new();

        if self.grid_found() {
            {
                let label = GridLabel::GridWithSomeColor;
                result.insert(label);
            }
            {
                let label = GridLabel::GridColor { color: self.grid_color() };
                result.insert(label);
            }
            {
                let label = GridLabel::GridWithMismatchesAndColor { color: self.grid_color() };
                result.insert(label);
            }
        }

        if self.grid_with_mismatches_found() {
            for pattern in self.patterns() {
                let color: u8 = pattern.color;
                {
                    let label = GridLabel::GridWithMismatchesAndColor { color };
                    result.insert(label);
                }
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
    fn test_10000_grid_yes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 5, 0,
            5, 5, 5,
            0, 5, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let grid: Grid = Grid::analyze(&input).expect("ok");
        
        // Act
        let actual: HashSet<GridLabel> = grid.to_grid_labels();

        // Assert
        assert_eq!(actual.contains(&GridLabel::GridWithSomeColor), true);
        assert_eq!(actual.contains(&GridLabel::GridColor { color: 5 }), true);
    }

    #[test]
    fn test_10001_grid_no() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
            3, 4, 5,
            6, 7, 8,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let grid: Grid = Grid::analyze(&input).expect("ok");
        
        // Act
        let actual: HashSet<GridLabel> = grid.to_grid_labels();

        // Assert
        assert_eq!(actual.contains(&GridLabel::GridWithSomeColor), false);
    }
}
