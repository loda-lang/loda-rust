use super::{Image, PixelConnectivity, convolution3x3};

#[allow(dead_code)]
fn conv3x3_count_nonzero_neighbours_connectivity8(image: &Image) -> anyhow::Result<u8> {
    if image.get(1, 1).unwrap_or(255) == 0 {
        return Ok(0);
    }
    let mut is_solid: bool = true;
    for pixel in image.pixels() {
        if *pixel == 0 {
            is_solid = false;
        }
    }
    if is_solid {
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
    if count4 == 0 && count_diagonal == 0 {
        // An isolated pixel without any neighbours.
        return Ok(1);
    }
    if count4 == 0 && count_diagonal == 4 {
        // 4 diagonals crossing in a single point. No neighbours.
        return Ok(4);
    }
    if count4 == 0 && count_diagonal == 3 {
        // 3 diagonals intersecting in a single point. No neighbours.
        return Ok(3);
    }
    if count4 == 1 && count_diagonal == 0 {
        // An pixel with exactly 1 neighbor, and no diagonal neighbours.
        return Ok(1);
    }
    if count4 == 1 && count_diagonal == 1 {
        // An pixel with exactly 1 neighbor, and 1 diagonal neighbour.
        return Ok(2);
    }

    if count4 == 3 && count_diagonal == 4 {
        // Inside a dent into a solid block.
        return Ok(7);
    }

    if count4 == 4 {
        // Inside a solid block, or inside a plus shape.
        return Ok(0);
    }    

    if count_total == 3 {
        let has_corner_top_left: bool = (top + top_left + left) > 0;
        let has_corner_top_right: bool = (top + top_right + right) > 0;
        let has_corner_bottom_left: bool = (bottom + bottom_left + left) > 0;
        let has_corner_bottom_right: bool = (bottom + bottom_right + right) > 0;
        if has_corner_top_left || has_corner_top_right || has_corner_bottom_left || has_corner_bottom_right {
            // This is a corner. It's not an edge.
            return Ok(3);
        }
    }

    let sum_top_bottom: u8 = top + bottom;
    let sum_left_right: u8 = left + right;
    let sum_top: u8 = top_left + top + top_right;
    let sum_bottom: u8 = bottom_left + bottom + bottom_right;
    let sum_left: u8 = top_left + left + bottom_left;
    let sum_right: u8 = top_right + right + bottom_right;

    if count4 == 2 && count_diagonal == 4 && sum_top_bottom == 2 {
        // H shape
        return Ok(6);
    }
    if count4 == 2 && count_diagonal == 4 && sum_left_right == 2 {
        // H shape
        return Ok(6);
    }

    // if count_diagonal
    // if count == 3 {
    //     // This is an edge. It's not a corner. 
    //     return Ok(0);
    // }
    Ok(0)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, SingleColorObject, ImagePadding};

    // #[test]
    fn test_10000_exterior_corners() {
        // Arrange
        let pixels: Vec<u8> = vec![
            6, 0, 0, 6, 6, 0, 6, 0, 6,
            0, 0, 6, 0, 6, 0, 6, 6, 6,
            0, 6, 0, 0, 6, 0, 6, 0, 6,
            0, 6, 6, 6, 6, 0, 0, 0, 0,
            0, 6, 6, 0, 6, 0, 0, 0, 0,
            0, 0, 0, 0, 6, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(9, 6, pixels).expect("image");

        // Act
        let objects: SingleColorObject = SingleColorObject::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual1: Image = objects.filled_holes_mask(6, PixelConnectivity::Connectivity4).expect("image");
        let actual2: Image = actual1.padding_with_color(1, 0).expect("image");
        let actual: Image = convolution3x3(&actual2, conv3x3_count_nonzero_neighbours_connectivity8).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 3, 3, 0, 2, 0, 2,
            0, 0, 0, 0, 0, 0, 0, 6, 0,
            0, 3, 0, 0, 0, 0, 2, 0, 2,
            0, 0, 0, 7, 0, 0, 0, 0, 0,
            0, 3, 3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0,
        ];
        let expected = Image::create_raw(9, 6, expected_pixels);
        assert_eq!(actual, expected);
    }
}
