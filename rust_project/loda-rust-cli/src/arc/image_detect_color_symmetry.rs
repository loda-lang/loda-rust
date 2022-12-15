use super::Image;

#[derive(Debug, PartialEq)]
pub enum ImageDetectColorSymmetryMode {
    /// Empty image.
    Empty,

    /// When the colors differ in some way that no symmetry is detected.
    NoSymmetryDetected,

    /// Symmetry. When all the pixels have the same value.
    Same,

    /// Horizontal symmetry. There is one color per row. And muliple colors are across the rows.
    Rows,

    /// Vertical symmetry. There is one color per column. And muliple colors are across the columns.
    Columns,
}

pub trait ImageDetectColorSymmetry {
    fn detect_color_symmetry(&self) -> ImageDetectColorSymmetryMode;
}

impl ImageDetectColorSymmetry for Image {
    fn detect_color_symmetry(&self) -> ImageDetectColorSymmetryMode {
        if self.is_empty() {
            return ImageDetectColorSymmetryMode::Empty;
        }
        let cols: bool = has_same_value_in_columns(&self);
        let rows: bool = has_same_value_in_rows(&self);
        match (cols, rows) {
            (false, false) => {
                return ImageDetectColorSymmetryMode::NoSymmetryDetected;
            },
            (false, true) => {
                return ImageDetectColorSymmetryMode::Columns;
            },
            (true, false) => {
                return ImageDetectColorSymmetryMode::Rows;
            },
            (true, true) => {
                return ImageDetectColorSymmetryMode::Same;
            },
        }
    }
}

fn has_same_value_in_columns(image: &Image) -> bool {
    for y in 0..(image.height() as i32) {
        let pixel_value0: u8 = image.get(0, y).unwrap_or(255);
        for x in 1..(image.width() as i32) {
            let pixel_value: u8 = image.get(x, y).unwrap_or(255);
            if pixel_value0 != pixel_value {
                return false;
            }
        }
    }
    true
}

fn has_same_value_in_rows(image: &Image) -> bool {
    for x in 0..(image.width() as i32) {
        let pixel_value0: u8 = image.get(x, 0).unwrap_or(255);
        for y in 1..(image.height() as i32) {
            let pixel_value: u8 = image.get(x, y).unwrap_or(255);
            if pixel_value0 != pixel_value {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_empty() {
        let actual: ImageDetectColorSymmetryMode = Image::empty().detect_color_symmetry();
        assert_eq!(actual, ImageDetectColorSymmetryMode::Empty);
    }

    #[test]
    fn test_20000_different() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
            0, 3, 4,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::NoSymmetryDetected);
    }

    #[test]
    fn test_20001_different() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
            0, 9, 2,
            0, 1, 2,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::NoSymmetryDetected);
    }

    #[test]
    fn test_20002_different() {
        // Arrange
        let pixels: Vec<u8> = vec![
            3, 3, 3,
            3, 3, 3,
            3, 3, 0,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::NoSymmetryDetected);
    }

    #[test]
    fn test_30000_same() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4,
        ];
        let input: Image = Image::try_create(1, 1, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::Same);
    }

    #[test]
    fn test_30001_same() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 4,
        ];
        let input: Image = Image::try_create(2, 1, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::Same);
    }

    #[test]
    fn test_30002_same() {
        // Arrange
        let pixels: Vec<u8> = vec![
            4, 
            4,
        ];
        let input: Image = Image::try_create(1, 2, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::Same);
    }

    #[test]
    fn test_30003_same() {
        // Arrange
        let pixels: Vec<u8> = vec![
            3, 3, 3,
            3, 3, 3,
            3, 3, 3,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::Same);
    }

    #[test]
    fn test_40000_rows() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            3, 3, 3, 3,
            5, 5, 5, 5,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::Rows);
    }

    #[test]
    fn test_50000_columns() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 8, 1,
            7, 8, 1,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: ImageDetectColorSymmetryMode = input.detect_color_symmetry();

        // Assert
        assert_eq!(actual, ImageDetectColorSymmetryMode::Columns);
    }
}
