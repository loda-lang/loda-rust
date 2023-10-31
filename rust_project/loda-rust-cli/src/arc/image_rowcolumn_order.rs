use super::{Image, ImageSymmetry};
use std::collections::HashMap;

pub trait ImageRowColumnOrder {
    /// Compare rows two images.
    /// 
    /// Returns `true` if row content is preserved. Ignoring the ordering of the rows.
    /// 
    /// Returns `false` when the images have different sizes.
    /// 
    /// Otherwise returns `false`.
    fn is_same_rows_ignoring_order(&self, other: &Image) -> bool;

    /// Compare columns of two images.
    /// 
    /// Returns `true` if column content is preserved. Ignoring the ordering of the columns.
    /// 
    /// Returns `false` when the images have different sizes.
    /// 
    /// Otherwise returns `false`.
    fn is_same_columns_ignoring_order(&self, other: &Image) -> bool;
}

impl ImageRowColumnOrder for Image {
    fn is_same_rows_ignoring_order(&self, other: &Image) -> bool {
        if self.size() != other.size() {
            return false;
        }
        if self.is_empty() {
            return true;
        }
        if self.width() == 1 && self.height() == 1 {
            return true;
        }
        let mut row_count0 = HashMap::<Vec<u8>, usize>::new();
        let mut row_count1 = HashMap::<Vec<u8>, usize>::new();
        for y in 0..self.height() {
            let mut key0 = Vec::<u8>::with_capacity(self.width() as usize);
            let mut key1 = Vec::<u8>::with_capacity(self.width() as usize);
            for x in 0..self.width() {
                key0.push(self.get(x as i32, y as i32).unwrap_or(0));
                key1.push(other.get(x as i32, y as i32).unwrap_or(0));
            }
            *row_count0.entry(key0).or_insert(0) += 1;
            *row_count1.entry(key1).or_insert(0) += 1;
        }
        row_count0 == row_count1
    }

    fn is_same_columns_ignoring_order(&self, other: &Image) -> bool {
        let image0: Image = match self.flip_diagonal_a() {
            Ok(value) => value,
            Err(_error) => {
                return false;
            }
        };
        let image1: Image = match other.flip_diagonal_a() {
            Ok(value) => value,
            Err(_error) => {
                return false;
            }
        };
        image0.is_same_rows_ignoring_order(&image1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_empty() {
        let input = Image::empty();
        assert_eq!(input.is_same_columns_ignoring_order(&input), true);
        assert_eq!(input.is_same_rows_ignoring_order(&input), true);
    }

    #[test]
    fn test_10001_size1x1() {
        let input = Image::color(1, 1, 9);
        assert_eq!(input.is_same_columns_ignoring_order(&input), true);
        assert_eq!(input.is_same_rows_ignoring_order(&input), true);
    }

    #[test]
    fn test_20000_is_same_rows_but_ignoring_column_order() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            2, 1, 4, 3,
            6, 5, 8, 7,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 2, 3, 4,
            5, 6, 7, 8,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        // Act+Assert
        assert_eq!(input0.is_same_columns_ignoring_order(&input1), true);
        assert_eq!(input1.is_same_columns_ignoring_order(&input0), true);
        assert_eq!(input0.is_same_columns_ignoring_order(&input0), true);
        assert_eq!(input1.is_same_columns_ignoring_order(&input1), true);
        assert_eq!(input0.is_same_rows_ignoring_order(&input1), false);
        assert_eq!(input1.is_same_rows_ignoring_order(&input0), false);
    }

    #[test]
    fn test_20001_is_same_rows_but_ignoring_column_order() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            2, 6, 0,
            1, 5, 0,
            4, 8, 0,
            3, 7, 0,
        ];
        let input0: Image = Image::try_create(3, 4, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 5, 0,
            2, 6, 0,
            3, 7, 0,
            4, 8, 0,
        ];
        let input1: Image = Image::try_create(3, 4, pixels1).expect("image");

        // Act+Assert
        assert_eq!(input0.is_same_rows_ignoring_order(&input1), true);
        assert_eq!(input1.is_same_rows_ignoring_order(&input0), true);
        assert_eq!(input0.is_same_rows_ignoring_order(&input0), true);
        assert_eq!(input1.is_same_rows_ignoring_order(&input1), true);
        assert_eq!(input0.is_same_columns_ignoring_order(&input1), false);
        assert_eq!(input1.is_same_columns_ignoring_order(&input0), false);
    }

    #[test]
    fn test_20002_same_row_with_different_column_order() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            2, 1, 4, 3, 5, 5,
        ];
        let input0: Image = Image::try_create(6, 1, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 2, 3, 4, 5, 5,
        ];
        let input1: Image = Image::try_create(6, 1, pixels1).expect("image");

        // Act+Assert
        assert_eq!(input0.is_same_columns_ignoring_order(&input1), true);
    }

    #[test]
    fn test_20003_same_row_with_different_column_order_counterexample() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            2, 1, 4, 3, 5, 9,
        ];
        let input0: Image = Image::try_create(6, 1, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 2, 3, 4, 5, 5,
        ];
        let input1: Image = Image::try_create(6, 1, pixels1).expect("image");

        // Act+Assert
        assert_eq!(input0.is_same_columns_ignoring_order(&input1), false);
    }

    #[test]
    fn test_20004_is_same_rows_but_ignoring_column_order() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 3, 3, 0, 0, 0, 0, 0,
            5, 2, 3, 0, 0, 0, 0, 0,
            5, 5, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 7, 7, 0,
            0, 0, 0, 0, 0, 7, 7, 0,
        ];
        let input0: Image = Image::try_create(8, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 1, 3, 3, 0, 0,
            0, 0, 0, 5, 2, 3, 0, 0,
            0, 0, 0, 5, 5, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 7, 7,
            0, 0, 0, 0, 0, 0, 7, 7,
        ];
        let input1: Image = Image::try_create(8, 5, pixels1).expect("image");

        // Act+Assert
        assert_eq!(input0.is_same_columns_ignoring_order(&input1), true);
        assert_eq!(input1.is_same_columns_ignoring_order(&input0), true);
        assert_eq!(input0.is_same_rows_ignoring_order(&input1), false);
        assert_eq!(input1.is_same_rows_ignoring_order(&input0), false);
    }

    #[test]
    fn test_20005_is_same_rows_but_ignoring_column_order_counterexample() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 3, 3, 0, 0, 0, 0, 0,
            5, 2, 3, 0, 0, 0, 0, 0,
            5, 5, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 7, 7, 0,
            0, 0, 0, 0, 0, 7, 7, 0,
        ];
        let input0: Image = Image::try_create(8, 5, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0, 0, 2, 3, 3, 
            0, 0, 0, 0, 0, 5, 2, 3,
            0, 0, 0, 0, 0, 5, 5, 1,
            0, 0, 0, 0, 0, 0, 7, 7,
            0, 0, 0, 0, 0, 0, 7, 7,
        ];
        let input1: Image = Image::try_create(8, 5, pixels1).expect("image");

        // Act+Assert
        assert_eq!(input0.is_same_columns_ignoring_order(&input1), false);
        assert_eq!(input1.is_same_columns_ignoring_order(&input0), false);
        assert_eq!(input0.is_same_rows_ignoring_order(&input1), false);
        assert_eq!(input1.is_same_rows_ignoring_order(&input0), false);
    }
}
