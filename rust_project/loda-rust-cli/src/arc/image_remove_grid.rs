use super::{Image, ImageHistogram, ImageRemoveRowColumn, Histogram};
use bit_set::BitSet;

pub trait ImageRemoveGrid {
    fn remove_grid(&self) -> anyhow::Result<Image>;
}

impl ImageRemoveGrid for Image {
    fn remove_grid(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if self.width() == 1 || self.height() == 1 {
            return Ok(self.clone());
        }
        let histogram_rows: Vec<Histogram> = self.histogram_rows();
        let histogram_columns: Vec<Histogram> = self.histogram_columns();

        // the overlap between the two histograms is where the count is 1
        let mut delete_rows = BitSet::with_capacity(256);
        for (index, histogram) in histogram_rows.iter().enumerate() {
            if histogram.number_of_counters_greater_than_zero() == 1 {
                delete_rows.insert(index);
            }
        }
        let mut delete_columns = BitSet::with_capacity(256);
        for (index, histogram) in histogram_columns.iter().enumerate() {
            if histogram.number_of_counters_greater_than_zero() == 1 {
                delete_columns.insert(index);
            }
        }

        // Delete the rows/columns
        let result_image: Image = self.remove_rowcolumn(&delete_rows, &delete_columns)?;
        return Ok(result_image);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_remove_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 5, 0, 5, 2,
            5, 5, 5, 5, 5,
            0, 5, 0, 5, 0,
            5, 5, 5, 5, 5,
            3, 5, 0, 5, 4,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.remove_grid().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 2,
            0, 0, 0,
            3, 0, 4,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_remove_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 2, 0, 3,
            0, 4, 5, 0, 6,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: Image = input.remove_grid().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_remove_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 9, 0, 0,
            0, 1, 2, 0, 3,
            0, 4, 5, 0, 6,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: Image = input.remove_grid().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 9, 0,
            1, 2, 3,
            4, 5, 6,
        ];
        let expected: Image = Image::try_create(3, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_remove_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 1, 0,
            0, 0, 1, 0, 1,
            0, 0, 1, 1, 0,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: Image = input.remove_grid().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0,
            0, 1,
            1, 0,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_remove_grid_1px_no_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let actual: Image = input.remove_grid().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 2,
        ];
        let expected: Image = Image::try_create(3, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_remove_grid_1px_no_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 
            1, 
            2,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Image = input.remove_grid().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 
            1, 
            2,
        ];
        let expected: Image = Image::try_create(1, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
