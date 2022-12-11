use super::{Image, ImageRemoveRowColumn};
use bit_set::BitSet;

pub trait ImageGetRowColumn {
    /// Take N rows from the top of the image.
    fn top_rows(&self, row_count: u8) -> anyhow::Result<Image>;

    /// Take N rows from the bottom of the image.
    fn bottom_rows(&self, row_count: u8) -> anyhow::Result<Image>;

    /// Take N columns from the left of the image.
    fn left_columns(&self, column_count: u8) -> anyhow::Result<Image>;

    /// Take N columns from the right of the image.
    fn right_columns(&self, column_count: u8) -> anyhow::Result<Image>;

    /// Remove N rows from the top of the image, and return the remaining part of the image.
    fn remove_top_rows(&self, row_count: u8) -> anyhow::Result<Image>;

    /// Remove N rows from the bottom of the image, and return the remaining part of the image.
    fn remove_bottom_rows(&self, row_count: u8) -> anyhow::Result<Image>;

    /// Remove N columns from the left of the image, and return the remaining part of the image.
    fn remove_left_columns(&self, column_count: u8) -> anyhow::Result<Image>;

    /// Remove N columns from the right of the image, and return the remaining part of the image.
    fn remove_right_columns(&self, column_count: u8) -> anyhow::Result<Image>;
}

impl ImageGetRowColumn for Image {
    fn top_rows(&self, row_count: u8) -> anyhow::Result<Image> {
        extract_top_bottom(&self, row_count, 0)
    }

    fn bottom_rows(&self, row_count: u8) -> anyhow::Result<Image> {
        extract_top_bottom(&self, 0, row_count)
    }

    fn left_columns(&self, column_count: u8) -> anyhow::Result<Image> {
        extract_left_right(&self, column_count, 0)
    }

    fn right_columns(&self, column_count: u8) -> anyhow::Result<Image> {
        extract_left_right(&self, 0, column_count)
    }

    fn remove_top_rows(&self, row_count: u8) -> anyhow::Result<Image> {
        let keep_count = (self.height() as i32) - (row_count as i32);
        if keep_count < 0 {
            return Err(anyhow::anyhow!("remove_top_rows: More rows are scheduled for deletion than the height of the image."));
        }
        extract_top_bottom(&self, 0, keep_count as u8)
    }

    fn remove_bottom_rows(&self, row_count: u8) -> anyhow::Result<Image> {
        let keep_count = (self.height() as i32) - (row_count as i32);
        if keep_count < 0 {
            return Err(anyhow::anyhow!("remove_bottom_rows: More rows are scheduled for deletion than the height of the image."));
        }
        extract_top_bottom(&self, keep_count as u8, 0)
    }

    fn remove_left_columns(&self, column_count: u8) -> anyhow::Result<Image> {
        let keep_count = (self.width() as i32) - (column_count as i32);
        if keep_count < 0 {
            return Err(anyhow::anyhow!("remove_left_columns: More columns are scheduled for deletion than the width of the image."));
        }
        extract_left_right(&self, 0, keep_count as u8)
    }

    fn remove_right_columns(&self, column_count: u8) -> anyhow::Result<Image> {
        let keep_count = (self.width() as i32) - (column_count as i32);
        if keep_count < 0 {
            return Err(anyhow::anyhow!("remove_right_columns: More columns are scheduled for deletion than the width of the image."));
        }
        extract_left_right(&self, keep_count as u8, 0)
    }
}

fn extract_top_bottom(image: &Image, top_count: u8, bottom_count: u8) -> anyhow::Result<Image> {
    let row_count: usize = (top_count as usize) + (bottom_count as usize);
    if image.is_empty() {
        return Ok(Image::empty());
    }
    if row_count > (image.height() as usize) {
        return Err(anyhow::anyhow!("extract_top_bottom: More rows are scheduled for extraction than the height of the image."));
    }
    if row_count == (image.height() as usize) {
        return Ok(image.clone());
    }
    let y0: i32 = (top_count as i32) - 1;
    let y1: i32 = (image.height() as i32) - (bottom_count as i32);
    if y1 < 0 {
        return Err(anyhow::anyhow!("extract_top_bottom: y1 is not supposed to be negative."));
    }

    let mut delete_rows = BitSet::new();
    for y in 0..(image.height() as usize) {
        if (y as i32) > y0 && (y as i32) < y1 {
            delete_rows.insert(y);
        }
    }
    let delete_columns = BitSet::new();
    image.remove_rowcolumn(&delete_rows, &delete_columns)
}

fn extract_left_right(image: &Image, left_count: u8, right_count: u8) -> anyhow::Result<Image> {
    let column_count: usize = (left_count as usize) + (right_count as usize);
    if image.is_empty() {
        return Ok(Image::empty());
    }
    if column_count > (image.width() as usize) {
        return Err(anyhow::anyhow!("extract_left_right: More columns are scheduled for extraction than the width of the image."));
    }
    if column_count == (image.width() as usize) {
        return Ok(image.clone());
    }
    let x0: i32 = (left_count as i32) - 1;
    let x1: i32 = (image.width() as i32) - (right_count as i32);
    if x1 < 0 {
        return Err(anyhow::anyhow!("extract_left_right: x1 is not supposed to be negative."));
    }

    let delete_rows = BitSet::new();
    let mut delete_columns = BitSet::new();
    for x in 0..(image.width() as usize) {
        if (x as i32) > x0 && (x as i32) < x1 {
            delete_columns.insert(x);
        }
    }
    image.remove_rowcolumn(&delete_rows, &delete_columns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    fn mock_image_portrait() -> Image {
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
        ];
        Image::try_create(2, 3, pixels).expect("image")
    }

    fn mock_image_landscape() -> Image {
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        Image::try_create(3, 2, pixels).expect("image")
    }

    fn mock_image_square() -> Image {
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9,
        ];
        Image::try_create(3, 3, pixels).expect("image")
    }

    #[test]
    fn test_10000_top_rows0() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.top_rows(0).expect("image");

        // Assert
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_10001_top_rows1() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.top_rows(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
        ];
        let expected: Image = Image::try_create(2, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_top_rows2() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.top_rows(2).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_top_rows3() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.top_rows(3).expect("image");

        // Assert
        let expected: Image = mock_image_portrait();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_top_rows_error_get_too_many() {
        let input: Image = mock_image_portrait();
        input.top_rows(4).expect_err("too many requested");
    }

    #[test]
    fn test_20000_bottom_rows0() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.bottom_rows(0).expect("image");

        // Assert
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_20001_bottom_rows1() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.bottom_rows(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_bottom_rows2() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.bottom_rows(2).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 4,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_bottom_rows3() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.bottom_rows(3).expect("image");

        // Assert
        let expected: Image = mock_image_portrait();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20004_bottom_rows_error_get_too_many() {
        let input: Image = mock_image_portrait();
        input.bottom_rows(4).expect_err("too many requested");
    }

    #[test]
    fn test_30000_left_columns0() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.left_columns(0).expect("image");

        // Assert
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_30001_left_columns1() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.left_columns(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 
            4,
        ];
        let expected: Image = Image::try_create(1, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30002_left_columns2() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.left_columns(2).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            4, 5,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30003_left_columns3() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.left_columns(3).expect("image");

        // Assert
        let expected: Image = mock_image_landscape();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30004_left_columns_error_get_too_many() {
        let input: Image = mock_image_landscape();
        input.left_columns(4).expect_err("too many requested");
    }

    #[test]
    fn test_40000_right_columns0() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.right_columns(0).expect("image");

        // Assert
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_40001_right_columns1() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.right_columns(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 
            6,
        ];
        let expected: Image = Image::try_create(1, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40002_right_columns2() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.right_columns(2).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2, 3,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40003_right_columns3() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.right_columns(3).expect("image");

        // Assert
        let expected: Image = mock_image_landscape();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40004_right_columns_error_get_too_many() {
        let input: Image = mock_image_landscape();
        input.right_columns(4).expect_err("too many requested");
    }

    #[test]
    fn test_50000_extract_top_bottom() {
        // Arrange
        let input: Image = mock_image_square();

        // Act
        let actual: Image = extract_top_bottom(&input, 1, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 3,
            7, 8, 9,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_60000_extract_left_right() {
        // Arrange
        let input: Image = mock_image_square();

        // Act
        let actual: Image = extract_left_right(&input, 1, 1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 3,
            4, 6,
            7, 9,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70000_remove_top_rows0() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.remove_top_rows(0).expect("image");

        // Assert
        assert_eq!(actual, input);
    }

    #[test]
    fn test_70001_remove_top_rows1() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.remove_top_rows(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 4,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70002_remove_top_rows2() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.remove_top_rows(2).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70003_remove_top_rows3() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.remove_top_rows(3).expect("image");

        // Assert
        assert_eq!(actual, Image::empty());
    }

    #[test]
    fn test_70004_remove_top_rows4() {
        let input: Image = mock_image_portrait();
        input.remove_top_rows(4).expect_err("is supposed to fail");
    }

    #[test]
    fn test_70101_remove_bottom_rows1() {
        // Arrange
        let input: Image = mock_image_portrait();

        // Act
        let actual: Image = input.remove_bottom_rows(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70201_remove_left_columns1() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.remove_left_columns(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2, 3,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70301_remove_right_columns1() {
        // Arrange
        let input: Image = mock_image_landscape();

        // Act
        let actual: Image = input.remove_right_columns(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            4, 5,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
