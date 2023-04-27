use super::{Image, ImagePadding, convolution3x3};

pub trait ImageCorner {
    /// Detect corners
    /// 
    /// Compare the center pixel with the pixels above,below,left,right.
    /// Ignore lines with the same color.
    /// 
    /// Only detects when the same color is used for a corner, a `T` shape or a `+`.
    /// 
    /// - Returns 0 when it's not a corner.
    /// - Returns 1 when it's a `L` shaped corner. One corner.
    /// - Returns 2 when it's a `T` shape. Two corners.
    /// - Returns 3 when it's a `+` shape, where one of the corners is filled. 3 corners.
    /// - Returns 4 when it's a `+` shape. 4 corners.
    /// 
    /// Uses `255` as the padding color.
    fn corners(&self) -> anyhow::Result<Image>;
}

impl ImageCorner for Image {
    fn corners(&self) -> anyhow::Result<Image> {
        let padding_color: u8 = u8::MAX;
        let image_padded: Image = self.padding_with_color(1, padding_color)?;
        let image: Image = convolution3x3(&image_padded, |bm| {
            let top_left: u8 = bm.get(0, 0).unwrap_or(255);
            let top: u8 = bm.get(1, 0).unwrap_or(255);
            let top_right: u8 = bm.get(2, 0).unwrap_or(255);
            let left: u8 = bm.get(0, 1).unwrap_or(255);
            let center: u8 = bm.get(1, 1).unwrap_or(255);
            let right: u8 = bm.get(2, 1).unwrap_or(255);
            let bottom_left: u8 = bm.get(0, 2).unwrap_or(255);
            let bottom: u8 = bm.get(1, 2).unwrap_or(255);
            let bottom_right: u8 = bm.get(2, 2).unwrap_or(255);
            let is_corner_top_left: bool = top_left != center && top == center && left == center;
            let is_corner_top_right: bool = top_right != center && top == center && right == center;
            let is_corner_bottom_left: bool = bottom_left != center && bottom == center && left == center;
            let is_corner_bottom_right: bool = bottom_right != center && bottom == center && right == center;

            let corners = [
                is_corner_top_left,
                is_corner_top_right,
                is_corner_bottom_left,
                is_corner_bottom_right,
            ];
            let mut corner_count: u8 = 0;
            for is_corner in corners {
                if is_corner {
                    corner_count += 1;
                }
            }
            Ok(corner_count)
        })?;
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_corner_count1_top_left() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 0, 6, 0,
            0, 6, 6, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.corners().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 1,
            1, 0, 0, 0,
            0, 0, 1, 0,
            1, 0, 0, 1,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_corner_count1_top_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 6, 0, 0,
            0, 6, 6, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.corners().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 1, 0,
            0, 0, 0, 1,
            0, 1, 0, 0,
            1, 0, 0, 1,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_corner_count1_bottom_left() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 6, 6, 0,
            0, 0, 6, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.corners().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 1,
            0, 0, 1, 0,
            1, 0, 0, 0,
            0, 1, 0, 1,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_corner_count1_bottom_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 6, 6, 0,
            0, 6, 0, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.corners().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 1,
            0, 1, 0, 0,
            0, 0, 0, 1,
            1, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10004_corner_count2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 6, 0, 0,
            0, 6, 6, 6, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.corners().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            1, 0, 0, 0, 1,
            0, 0, 2, 0, 0,
            1, 0, 0, 0, 1,
        ];
        let expected: Image = Image::try_create(5, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10005_corner_count3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 6, 0, 0,
            0, 6, 6, 6, 0,
            0, 0, 6, 6, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.corners().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            1, 0, 0, 0, 1,
            0, 0, 3, 0, 0,
            1, 0, 0, 0, 0,
            0, 1, 0, 0, 1,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10006_corner_count4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 6, 0, 0,
            0, 6, 6, 6, 0,
            0, 0, 6, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.corners().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0,
            1, 0, 0, 0, 1,
            0, 0, 4, 0, 0,
            1, 0, 0, 0, 1,
            0, 1, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_everything() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 6, 6, 0, 7, 7, 7, 
            0, 6, 0, 0, 0, 7, 0,
            0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 
            0, 8, 0, 0, 0, 9, 0,
            8, 8, 8, 0, 9, 9, 9, 
            8, 8, 0, 0, 0, 9, 0, 
        ];
        let input: Image = Image::try_create(7, 8, pixels).expect("image");

        // Act
        let actual: Image = input.corners().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 2, 0, 0, 0,
            0, 1, 0, 0, 0, 2, 0,
            0, 0, 0, 2, 0, 0, 0,
            1, 0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 1, 0, 1,
            0, 0, 0, 2, 0, 0, 0,
            0, 3, 0, 0, 0, 4, 0,
            0, 0, 0, 2, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(7, 8, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
