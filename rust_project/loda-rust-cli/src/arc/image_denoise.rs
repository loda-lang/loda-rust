use super::{Image, ImagePadding, convolution3x3};

pub trait ImageDenoise {
    fn denoise_type1(&self, background_color: u8) -> anyhow::Result<Image>;
}

impl ImageDenoise for Image {
    fn denoise_type1(&self, background_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = self.padding_with_color(1, background_color)?;
        let denoised_image: Image = convolution3x3(&input_padded, |bm| {
            let tl: u8 = bm.get(0, 0).unwrap_or(255);
            let tc: u8 = bm.get(1, 0).unwrap_or(255);
            let tr: u8 = bm.get(2, 0).unwrap_or(255);
            let cl: u8 = bm.get(0, 1).unwrap_or(255);
            let cc: u8 = bm.get(1, 1).unwrap_or(255);
            let cr: u8 = bm.get(2, 1).unwrap_or(255);
            let bl: u8 = bm.get(0, 2).unwrap_or(255);
            let bc: u8 = bm.get(1, 2).unwrap_or(255);
            let br: u8 = bm.get(2, 2).unwrap_or(255);
            let is_top_left: bool = tl == tc && cl == cc && tc == cc;
            let is_top_right: bool = tr == tc && cr == cc && tc == cc;
            let is_bottom_left: bool = bl == bc && cl == cc && bc == cc;
            let is_bottom_right: bool = br == bc && cr == cc && bc == cc;
            if is_top_left || is_top_right || is_bottom_left || is_bottom_right {
                return Ok(cc);
            }
            Ok(background_color)
        })?;
        Ok(denoised_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_denoise_type1_empty() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            0, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 1, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: Image = input.denoise_type1(0).expect("image");

        // Assert
        let expected: Image = Image::zero(4, 4);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_denoise_type1_some_objects() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 1, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 1, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Image = input.denoise_type1(0).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 0, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
