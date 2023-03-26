use super::Image;
use anyhow::Context;

/// Perform a 5x5 convolution the places where the mask is non-zero.
/// 
/// The `source` minimum size is 5x5. It must be the same size as `target`.
/// 
/// The `target` minimum size is 5x5. It must be the same size as `source`.
/// 
/// The `mask` size is equal to the `source` size minus 4 pixels. 
/// - With `source.width = 8` then use `mask.width = 4`. 
/// - With `source.height = 42` then use `mask.height = 38`. 
/// 
/// The `color_for_unprocessed_pixels` is used the places where the mask is zero.
/// 
/// The `callback` is invoked where the mask is non-zero. 
/// The callback is provided with two 5x5 images. One image for the `source` and another image for the `target`.
/// 
/// This function returns an `Image` with the same size as the `mask` image.
#[allow(dead_code)]
pub fn convolution5x5_special<F>(source: &Image, target: &Image, mask: &Image, color_for_unprocessed_pixels: u8, callback: F) -> anyhow::Result<Image>
    where F: Fn(&Image, &Image) -> anyhow::Result<u8>
{
    let width: u8 = source.width();
    let height: u8 = source.height();
    if width < 5 || height < 5 {
        return Err(anyhow::anyhow!("too small image, must be 5x5 or bigger"));
    }
    if source.size() != target.size() {
        return Err(anyhow::anyhow!("source.size and target.size must be the same"));
    }
    let invalid_width: bool = (width as u16) != (mask.width() as u16) + 4;
    let invalid_height: bool = (height as u16) != (mask.height() as u16) + 4;
    if invalid_width || invalid_height {
        return Err(anyhow::anyhow!("Size constraint not satisfied. Expected image.width == mask.width+4 AND image.height == mask.height+4"));
    }
    let mut result_image = Image::color(width - 4, height - 4, color_for_unprocessed_pixels);
    let mut conv_source = Image::zero(5, 5);
    let mut conv_target = Image::zero(5, 5);
    for self_y in 0..height-4 {
        for self_x in 0..width-4 {
            let mask_x: i32 = self_x as i32;
            let mask_y: i32 = self_y as i32;
            let mask_value = mask.get(mask_x, mask_y)
                .ok_or_else(|| anyhow::anyhow!("mask.get({},{}) returned None", mask_x, mask_y))?;
        
            // Ignore areas where the mask is zero
            if mask_value == 0 {
                continue;
            }

            // Perform 5x5 convolution
            for conv_y in 0..5u8 {
                for conv_x in 0..5u8 {
                    let get_x: i32 = (self_x as i32) + (conv_x as i32);
                    let get_y: i32 = (self_y as i32) + (conv_y as i32);
                    {
                        let source_value: u8 = source.get(get_x, get_y)
                            .ok_or_else(|| anyhow::anyhow!("source.get({},{}) returned None", get_x, get_y))?;
                        conv_source.set(conv_x as i32, conv_y as i32, source_value)
                            .ok_or_else(|| anyhow::anyhow!("conv_source.set({},{}) returned None", conv_x, conv_y))?;
                    }
                    {
                        let target_value: u8 = target.get(get_x, get_y)
                            .ok_or_else(|| anyhow::anyhow!("target.get({},{}) returned None", get_x, get_y))?;
                        conv_target.set(conv_x as i32, conv_y as i32, target_value)
                            .ok_or_else(|| anyhow::anyhow!("conv_target.set({},{}) returned None", conv_x, conv_y))?;
                    }
                }
            }
            let computed_value: u8 = callback(&conv_source, &conv_target)
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

    fn conv5x5_doit(source: &Image, _target: &Image) -> anyhow::Result<u8> {
        let center_pixel: u8 = match source.get(2, 2) {
            Some(value) => value,
            None => {
                return Ok(255);
            }
        };
        Ok(center_pixel)
    }
    
    #[test]
    fn test_10000_sunshine_scenario() {
        // Arrange
        let source_pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            6, 7, 8, 9, 10,
            11, 12, 13, 14, 15,
            16, 17, 18, 19, 20,
            21, 22, 23, 24, 25,
        ];
        let input_source: Image = Image::try_create(5, 5, source_pixels).expect("image");

        let input_target = Image::zero(5, 5);

        let mask_pixels: Vec<u8> = vec![
            1,
        ];
        let input_mask: Image = Image::try_create(1, 1, mask_pixels).expect("image");

        // Act
        let output: Image = convolution5x5_special(&input_source, &input_target, &input_mask, 42, conv5x5_doit).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            13,
        ];
        let expected: Image = Image::try_create(1, 1, expected_pixels).expect("image");
        assert_eq!(output, expected);
    }
}
