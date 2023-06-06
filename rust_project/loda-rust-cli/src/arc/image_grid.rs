use super::{Image, ImageHistogram, ImageRemoveRowColumn, Histogram, ImageOverlay};
use bit_set::BitSet;

pub trait ImageGrid {
    fn remove_grid(&self) -> anyhow::Result<Image>;

    /// Returns a mask. Where the cells are the value is 1 and where the grid lines are the value is 0.
    /// 
    /// The grid color is optional. If it is specified, the only rows/columns with a 1 unique color
    /// that is the specified color are considered grid lines.
    /// 
    /// If the grid color is not specified, the grid lines are the places where the rows/columns has a single unique color,
    /// there may be multiple different colors in the image, that takes up an entire row/column.
    /// 
    /// The spacing between the cells is not considered.
    /// 
    /// The thickness of the grid lines is not considered.
    fn mask_for_gridcells(&self, grid_color: Option<u8>) -> anyhow::Result<Image>;
}

impl ImageGrid for Image {
    fn remove_grid(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if self.width() == 1 || self.height() == 1 {
            return Ok(self.clone());
        }
        let histogram_rows: Vec<Histogram> = self.histogram_rows();
        let histogram_columns: Vec<Histogram> = self.histogram_columns();

        // the grid lines is the places where there is overlap between the two histograms, where the count is 1
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

    fn mask_for_gridcells(&self, grid_color: Option<u8>) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        if self.width() == 1 || self.height() == 1 {
            let image = Image::color(self.width(), self.height(), 1);
            return Ok(image);
        }
        let histogram_rows: Vec<Histogram> = self.histogram_rows();
        let histogram_columns: Vec<Histogram> = self.histogram_columns();
        let dontcare_about_grid_color: bool = grid_color.is_none();

        let mut result_image = Image::color(self.width(), self.height(), 1);

        // Draw horizontal lines where there is grid
        let row = Image::zero(self.width(), 1);
        for (index, histogram) in histogram_rows.iter().enumerate() {
            if index >= (u8::MAX as usize) {
                break;
            }
            let y: u8 = index as u8;
            if histogram.number_of_counters_greater_than_zero() != 1 {
                continue;
            }
            if dontcare_about_grid_color || histogram.most_popular_color() == grid_color {
                result_image = result_image.overlay_with_position(&row, 0, y as i32)?;
            }
        }

        // Draw vertical lines where there is grid
        let column = Image::zero(1, self.height());
        for (index, histogram) in histogram_columns.iter().enumerate() {
            if index >= (u8::MAX as usize) {
                break;
            }
            let x: u8 = index as u8;
            if histogram.number_of_counters_greater_than_zero() != 1 {
                continue;
            }
            if dontcare_about_grid_color || histogram.most_popular_color() == grid_color {
                result_image = result_image.overlay_with_position(&column, x as i32, 0)?;
            }
        }

        Ok(result_image)
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

    #[test]
    fn test_30000_mask_for_gridcells() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1, 0, 3,
            3, 4, 1, 0, 1,
            5, 6, 1, 0, 3,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: Image = input.mask_for_gridcells(Some(1)).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 0, 1, 1,
            1, 1, 0, 1, 1,
            1, 1, 0, 1, 1,
        ];
        let expected: Image = Image::try_create(5, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30001_mask_for_gridcells() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 0, 2, 0, 2,
            2, 2, 2, 2, 2,
            2, 0, 2, 0, 2,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: Image = input.mask_for_gridcells(Some(2)).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(5, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30002_mask_for_gridcells() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5,
            3, 2, 3,
            5, 5, 5,
            1, 1, 1,
            3, 2, 3,
            5, 5, 5,
        ];
        let input: Image = Image::try_create(3, 6, pixels).expect("image");

        // Act
        let actual: Image = input.mask_for_gridcells(None).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0,
            1, 1, 1,
            0, 0, 0,
            0, 0, 0,
            1, 1, 1,
            0, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
