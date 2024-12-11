use super::Image;
use anyhow::Context;

/// Perform a 3x3 convolution the places where the mask is non-zero.
/// 
/// The `image` minimum size is 3x3.
/// 
/// The `mask` size is equal to the `image` size minus 2 pixels. 
/// - With `image.width = 8` then use `mask.width = 6`. 
/// - With `image.height = 42` then use `mask.height = 40`. 
/// 
/// The `color_for_unprocessed_pixels` is used the places where the mask is zero.
/// 
/// The `callback` is invoked where the mask is non-zero. The callback is provided with a 3x3 image.
/// 
/// This function returns an `Image` with the same size as the `mask` image.
#[allow(dead_code)]
pub fn convolution3x3_with_mask<F>(image: &Image, mask: &Image, color_for_unprocessed_pixels: u8, callback: F) -> anyhow::Result<Image>
    where F: Fn(&Image) -> anyhow::Result<u8>
{
    let width: u8 = image.width();
    let height: u8 = image.height();
    if width < 3 || height < 3 {
        return Err(anyhow::anyhow!("too small image, must be 3x3 or bigger"));
    }
    let invalid_width: bool = (width as u16) != (mask.width() as u16) + 2;
    let invalid_height: bool = (height as u16) != (mask.height() as u16) + 2;
    if invalid_width || invalid_height {
        return Err(anyhow::anyhow!("Size constraint not satisfied. Expected image.width == mask.width+2 AND image.height == mask.height+2"));
    }
    let mut result_image = Image::color(width - 2, height - 2, color_for_unprocessed_pixels);
    let mut conv_bitmap = Image::zero(3, 3);
    for self_y in 0..height-2 {
        for self_x in 0..width-2 {
            let mask_x: i32 = self_x as i32;
            let mask_y: i32 = self_y as i32;
            let mask_value = mask.get(mask_x, mask_y)
                .ok_or_else(|| anyhow::anyhow!("mask.get({},{}) returned None", mask_x, mask_y))?;
        
            // Ignore areas where the mask is zero
            if mask_value == 0 {
                continue;
            }

            // Perform 3x3 convolution
            for conv_y in 0..3u8 {
                for conv_x in 0..3u8 {
                    let get_x: i32 = (self_x as i32) + (conv_x as i32);
                    let get_y: i32 = (self_y as i32) + (conv_y as i32);
                    let pixel_value: u8 = image.get(get_x, get_y)
                        .ok_or_else(|| anyhow::anyhow!("image.get({},{}) returned None", get_x, get_y))?;
                    conv_bitmap.set(conv_x as i32, conv_y as i32, pixel_value)
                        .ok_or_else(|| anyhow::anyhow!("conv_bitmap.set({},{}) returned None", conv_x, conv_y))?;
                }
            }
            let computed_value: u8 = callback(&conv_bitmap)
                .with_context(|| format!("error in callback when computing ({},{})", self_x, self_y))?;
            result_image.set(self_x as i32, self_y as i32, computed_value)
                .ok_or_else(|| anyhow::anyhow!("result_image.set({},{}) returned None", self_x, self_y))?;
        }
    }
    Ok(result_image)

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_sunshine_scenario() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            6, 7, 8, 9, 10,
            11, 12, 13, 14, 15,
            16, 17, 18, 19, 20,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        let mask_pixels: Vec<u8> = vec![
            1, 0, 1,
            0, 1, 0,
        ];
        let input_mask: Image = Image::try_create(3, 2, mask_pixels).expect("image");

        // Act
        let output: Image = convolution3x3_with_mask(&input, &input_mask, 42, |bm| {
            let pixel_value: u8 = match bm.get(1, 1) {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("get pixel"));
                }
            };
            Ok(pixel_value)
        }).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7,  42,  9,
            42, 13, 42,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20000_error_size_constraint_not_satisfied() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            6, 7, 8, 9, 10,
            11, 12, 13, 14, 15,
            16, 17, 18, 19, 20,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        let mask_pixels: Vec<u8> = vec![
            1, 0,
            0, 1,
        ];
        let input_mask: Image = Image::try_create(2, 2, mask_pixels).expect("image");

        // Act
        let result = convolution3x3_with_mask(&input, &input_mask, 42, |bm| {
            let pixel_value: u8 = match bm.get(1, 1) {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("get pixel"));
                }
            };
            Ok(pixel_value)
        });

        // Assert
        let error = result.expect_err("is supposed to fail");
        let message: String = format!("{:?}", error);
        assert_eq!(message.contains("Size constraint not satisfied"), true);
    }

    #[test]
    fn test_20001_error_size_constraint_not_satisfied() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            6, 7, 8, 9, 10,
            11, 12, 13, 14, 15,
            16, 17, 18, 19, 20,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        let mask_pixels: Vec<u8> = vec![
            1, 0,
            0, 1,
            1, 0,
        ];
        let input_mask: Image = Image::try_create(2, 3, mask_pixels).expect("image");

        // Act
        let result = convolution3x3_with_mask(&input, &input_mask, 42, |bm| {
            let pixel_value: u8 = match bm.get(1, 1) {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("get pixel"));
                }
            };
            Ok(pixel_value)
        });

        // Assert
        let error = result.expect_err("is supposed to fail");
        let message: String = format!("{:?}", error);
        assert_eq!(message.contains("Size constraint not satisfied"), true);
    }
}
