use super::{Image, convolution3x3};

#[allow(dead_code)]
fn conv3x3_count_nonzero_neighbours_connectivity8(image: &Image) -> anyhow::Result<u8> {
    if image.get(1, 1).unwrap_or(0) == 0 {
        return Ok(0);
    }

    let top_left: u8 = image.get(0, 0).unwrap_or(0).min(1);
    let top: u8 = image.get(1, 0).unwrap_or(0).min(1);
    let top_right: u8 = image.get(2, 0).unwrap_or(0).min(1);
    let left: u8 = image.get(0, 1).unwrap_or(0).min(1);
    let right: u8 = image.get(2, 1).unwrap_or(0).min(1);
    let bottom_left: u8 = image.get(0, 2).unwrap_or(0).min(1);
    let bottom: u8 = image.get(1, 2).unwrap_or(0).min(1);
    let bottom_right: u8 = image.get(2, 2).unwrap_or(0).min(1);

    let count4: u8 = top + bottom + left + right;
    let count_diagonal: u8 = top_left + top_right + bottom_left + bottom_right;
    let count_total: u8 = count4 + count_diagonal;

    let solid_corner_top_left: bool = (top + top_left + left) == 3;
    let solid_corner_top_right: bool = (top + top_right + right) == 3;
    let solid_corner_bottom_left: bool = (bottom + bottom_left + left) == 3;
    let solid_corner_bottom_right: bool = (bottom + bottom_right + right) == 3;

    let missing_corner_top_left: bool = (top + top_left + left) == 0;
    let missing_corner_top_right: bool = (top + top_right + right) == 0;
    let missing_corner_bottom_left: bool = (bottom + bottom_left + left) == 0;
    let missing_corner_bottom_right: bool = (bottom + bottom_right + right) == 0;

    let sum_top_bottom: u8 = top + bottom;
    let sum_left_right: u8 = left + right;
    let sum_top: u8 = top_left + top + top_right;
    let sum_bottom: u8 = bottom_left + bottom + bottom_right;
    let sum_left: u8 = top_left + left + bottom_left;
    let sum_right: u8 = top_right + right + bottom_right;

    let empty_top: bool = sum_top == 0;
    let empty_bottom: bool = sum_bottom == 0;
    let empty_left: bool = sum_left == 0;
    let empty_right: bool = sum_right == 0;

    if count4 == 4 && count_diagonal == 4 {
        // Inside a solid block. It's not a corner.
        // 1, 1, 1
        // 1, 1, 1
        // 1, 1, 1
        return Ok(0);
    }    

    if count4 == 0 && count_diagonal == 0 {
        // An isolated pixel without any neighbours. This is a corner.
        // 0, 0, 0
        // 0, 1, 0
        // 0, 0, 0
        return Ok(1);
    }
    if count4 == 0 && count_diagonal == 4 {
        // 4 diagonals crossing in a single point. No neighbours. This is a corner.
        // 1, 0, 1
        // 0, 1, 0
        // 1, 0, 1
        return Ok(4);
    }
    if count4 == 0 && count_diagonal == 3 {
        // 3 diagonals intersecting in a single point. No neighbours. This is a corner.
        // 1, 0, 0
        // 0, 1, 0
        // 1, 0, 1
        return Ok(3);
    }
    if count4 == 1 && count_diagonal == 0 {
        // An pixel with exactly 1 neighbor, and no diagonal neighbours. This is a corner.
        // 0, 0, 0
        // 0, 1, 0
        // 0, 1, 0
        return Ok(1);
    }
    if count4 == 0 && count_diagonal == 1 {
        // An pixel with exactly 1 diagonal neighbor, and no neighbours. This is a corner.
        // 0, 0, 0
        // 0, 1, 0
        // 1, 0, 0
        return Ok(1);
    }
    if count4 == 1 && count_diagonal == 1 {
        // An pixel with exactly 1 neighbor, and 1 diagonal neighbour. This is a corner.
        // 0, 0, 0
        // 0, 1, 0
        // 1, 1, 0
        return Ok(2);
    }

    if count4 == 3 && count_diagonal == 4 {
        // Inside a dent into a solid block. This is a corner.
        return Ok(7);
    }

    if count_total == 3 
    {
        if solid_corner_top_left || solid_corner_top_right || solid_corner_bottom_left || solid_corner_bottom_right {
            // This is a corner.
            // 0, 0, 0
            // 1, 1, 0
            // 1, 1, 0
            return Ok(3);
        }
    }

    if count_total == 5
    {
        if missing_corner_top_left || missing_corner_top_right || missing_corner_bottom_left || missing_corner_bottom_right {
            // This is a diagonal edge, with one half solid, and the other half empty. It's not a corner.
            // 1, 0, 0
            // 1, 1, 0
            // 1, 1, 1
            return Ok(0);
        }
    }

    if count_total == 5 && (empty_top || empty_bottom || empty_left || empty_right) {
        // This is an edge. It's not a corner. Example:
        // 1, 1, 0
        // 1, 1, 0
        // 1, 1, 0
        return Ok(0);
    }

    if count4 == 2 && count_diagonal == 4 && sum_top_bottom == 2 {
        // H shape
        // 1, 1, 1
        // 0, 1, 0
        // 1, 1, 1
        return Ok(6);
    }
    if count4 == 2 && count_diagonal == 4 && sum_left_right == 2 {
        // H shape
        // 1, 0, 1
        // 1, 1, 1
        // 1, 0, 1
        return Ok(6);
    }

    if count4 == 3 && count_diagonal == 0 {
        // T shape, this is not a corner
        // 0, 1, 0
        // 1, 1, 1
        // 0, 0, 0
        return Ok(0);
    }

    if count4 == 2 && count_diagonal == 1 {
        if sum_top_bottom == 2 || sum_left_right == 2 {
            // L shape, this is not a corner
            // 1, 0, 0
            // 1, 1, 1
            // 0, 0, 0
            return Ok(0);
        }
    }

    if count4 == 3 && count_diagonal == 1 {
        if solid_corner_top_left || solid_corner_top_right || solid_corner_bottom_left || solid_corner_bottom_right {
            // Stetched L shape, this is not a corner
            // 1, 1, 0
            // 1, 1, 1
            // 0, 0, 0
            return Ok(0);
        }
    }

    if count4 == 2 && count_diagonal == 2 {
        if solid_corner_top_left || solid_corner_top_right || solid_corner_bottom_left || solid_corner_bottom_right {
            if empty_top || empty_bottom || empty_left || empty_right {
                // Stetched L shape, this is a corner
                // 0, 0, 0
                // 1, 1, 0
                // 1, 1, 1
                return Ok(4);
            }
        }
    }

    if count4 == 2 && count_diagonal == 1 {
        if empty_top || empty_bottom || empty_left || empty_right {
            // Tetris shape, this is a corner
            // 0, 0, 0
            // 1, 1, 0
            // 0, 1, 1
            return Ok(3);
        }
    }

    if count4 == 1 && count_diagonal == 2 {
        if empty_top || empty_bottom || empty_left || empty_right {
            // Tetris shape, this is a corner
            // 0, 0, 0
            // 0, 1, 0
            // 1, 1, 1
            return Ok(3);
        }
    }

    Ok(0)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, ImagePadding};

    #[test]
    fn test_10000_detect_corners() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 1, 1, 0, 1, 0, 1,
            0, 0, 1, 1, 1, 0, 1, 1, 1,
            0, 1, 1, 1, 1, 0, 1, 0, 1,
            0, 1, 1, 1, 1, 0, 0, 0, 0,
            0, 1, 1, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(9, 6, pixels).expect("image");

        // Act
        let actual1: Image = input.padding_with_color(1, 0).expect("image");
        let actual: Image = convolution3x3(&actual1, conv3x3_count_nonzero_neighbours_connectivity8).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 4, 3, 0, 2, 0, 2,
            0, 0, 0, 0, 0, 0, 0, 6, 0,
            0, 4, 0, 0, 0, 0, 2, 0, 2,
            0, 0, 0, 7, 0, 0, 0, 0, 0,
            0, 3, 4, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0,
        ];
        let expected = Image::create_raw(9, 6, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_detect_corners() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 1, 0, 0, 0, 1, 1, 0, 0,
            0, 0, 1, 0, 0, 0, 1, 1, 0,
            1, 0, 0, 1, 0, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 0, 0, 0, 1,
            1, 1, 1, 0, 0, 1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(9, 6, pixels).expect("image");

        // Act
        let actual1: Image = input.padding_with_color(1, 0).expect("image");
        let actual: Image = convolution3x3(&actual1, conv3x3_count_nonzero_neighbours_connectivity8).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 2, 3, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            2, 0, 0, 0, 0, 0, 0, 0, 3,
            0, 0, 0, 0, 0, 0, 0, 0, 2,
            3, 0, 2, 0, 0, 1, 0, 0, 0,
        ];
        let expected = Image::create_raw(9, 6, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_detect_corners() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0, 0, 1, 0, 0,
            1, 1, 1, 0, 0, 0, 1, 0, 0,
            0, 1, 0, 0, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 1, 1, 1, 1,
            1, 1, 0, 0, 0, 0, 1, 0, 0,
            1, 1, 0, 0, 0, 0, 1, 0, 0,
        ];
        let input: Image = Image::try_create(9, 6, pixels).expect("image");

        // Act
        let actual1: Image = input.padding_with_color(1, 0).expect("image");
        let actual: Image = convolution3x3(&actual1, conv3x3_count_nonzero_neighbours_connectivity8).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 3, 0, 0, 0, 0, 1, 0, 0,
            3, 0, 3, 0, 0, 0, 0, 0, 0,
            0, 3, 0, 0, 3, 0, 0, 0, 3,
            0, 0, 0, 0, 3, 0, 0, 0, 3,
            3, 3, 0, 0, 0, 0, 0, 0, 0,
            3, 3, 0, 0, 0, 0, 1, 0, 0,
        ];
        let expected = Image::create_raw(9, 6, expected_pixels);
        assert_eq!(actual, expected);
    }
}
