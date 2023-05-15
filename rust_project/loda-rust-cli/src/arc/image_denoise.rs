use super::{Image, ImageHistogram, ImagePadding, ImageNoiseColor, Histogram, convolution3x3};

pub trait ImageDenoise {
    fn denoise_type1(&self, background_color: u8) -> anyhow::Result<Image>;
    fn denoise_type2(&self, noise_color: u8) -> anyhow::Result<Image>;
    fn denoise_type3(&self, repair_iterations: u8) -> anyhow::Result<Image>;
    fn denoise_type4(&self) -> anyhow::Result<Image>;
    fn denoise_type5(&self, noise_color: u8) -> anyhow::Result<Image>;
    fn denoise_type6(&self) -> anyhow::Result<Image>;
    fn blur(&self) -> anyhow::Result<Image>;
    fn denoise_type7(&self, noise_color: u8, background_color: u8) -> anyhow::Result<Image>;
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

    fn denoise_type2(&self, noise_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        
        // find an unused color for use as padding_color
        let histogram: Histogram = self.histogram_all();
        let padding_color: u8 = match histogram.unused_color() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("All colors are used in the histogram. Cannot pick a padding color"));
            }
        };

        let input_padded: Image = self.padding_with_color(1, padding_color)?;

        let denoised_image: Image = convolution3x3(&input_padded, |bm| {
            let value: u8 = bm.get(1, 1).unwrap_or(255);
            if value != noise_color {
                // not a noisy pixel
                return Ok(value);
            }
            // this is a noise pixel. Look at the surrounding pixels, and take the most popular
            let mut histogram: Vec<u8> = vec![0; 256];
            for y in 0..3i32 {
                for x in 0..3i32 {
                    if y == 1 && x == 1 {
                        continue;
                    }
                    let pixel_value: u8 = bm.get(x, y).unwrap_or(255);
                    if pixel_value == padding_color {
                        continue;
                    }
                    let original_count: u8 = match histogram.get(pixel_value as usize) {
                        Some(value) => *value,
                        None => {
                            return Err(anyhow::anyhow!("Integrity error. Counter in histogram out of bounds"));
                        }
                    };
                    let count: u8 = (original_count + 1) & 255;
                    histogram[pixel_value as usize] = count;
                }
            }
            let mut found_count: u8 = 0;
            let mut found_value: usize = 0;
            for (pixel_value, number_of_occurences) in histogram.iter().enumerate() {
                if *number_of_occurences > found_count {
                    found_count = *number_of_occurences;
                    found_value = pixel_value;
                }
            }
            let value: u8 = (found_value & 255) as u8;
            Ok(value)
        })?;

        Ok(denoised_image)
    }

    fn denoise_type3(&self, repair_iterations: u8) -> anyhow::Result<Image> {
        if repair_iterations == 0 {
            return Err(anyhow::anyhow!("The number of repair iterations must be 1 or greater"));
        }
        let mut noise_color_vec: Vec<u8> = self.one_pixel_noise_color_vec()?;
        noise_color_vec.reverse();

        let mut image: Image = self.clone();
        for (index, noise_color) in noise_color_vec.iter().enumerate() {
            if index >= (repair_iterations as usize) {
                break;
            }
            image = image.denoise_type2(*noise_color)?;
        }
        Ok(image)
    }

    fn denoise_type4(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = self.padding_with_color(1, 0)?;
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
            if cc > 0 {
                // not a noisy pixel
                return Ok(1);
            }
            if tc > 0 && cl > 0 {
                return Ok(1);
            }
            if tc > 0 && cr > 0 {
                return Ok(1);
            }
            if bc > 0 && cl > 0 {
                return Ok(1);
            }
            if bc > 0 && cr > 0 {
                return Ok(1);
            }
            Ok(0)
        })?;
        Ok(denoised_image)
    }

    fn denoise_type5(&self, noise_color: u8) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = self.padding_with_color(1, 0)?;
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
            if cc != noise_color {
                // not a noisy pixel
                return Ok(cc);
            }
            if tc > 0 && cl > 0 {
                return Ok(1);
            }
            if tc > 0 && cr > 0 {
                return Ok(1);
            }
            if bc > 0 && cl > 0 {
                return Ok(1);
            }
            if bc > 0 && cr > 0 {
                return Ok(1);
            }
            Ok(0)
        })?;
        Ok(denoised_image)
    }

    fn denoise_type6(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = self.padding_with_color(1, 0)?;
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
            if cc > 0 {
                // not a noisy pixel
                return Ok(cc);
            }
            // let h: Histogram = bm.histogram_all();
            // let colors = [tc, cl, cr, bc];
            // let mut count: u8 = 0;
            // for color in colors {
            //     if color == cc {
            //         count += 1;
            //     }
            // }
            // if count >= 3 {
            //     return Ok(1);
            // }

            if tc == bc && tc == 0 {
                // empty column
                return Ok(0);
            }
            if cl == cr && cl == 0 {
                // empty row
                return Ok(0);
            }

            let has_top_left: bool = tl == tc && tl == cl && tl > 0;
            let has_top_right: bool = tr == tc && tr == cr && tr > 0; 
            let has_bottom_left: bool = bl == bc && bl == cl && bl > 0;
            let has_bottom_right: bool = br == bc && br == cr && br > 0;

            if tc > 0 && tc == cl && tc == cr && tc == bc {
                // connect-4 crosshair
                return Ok(tc);
            }
            if has_top_left && has_top_right {
                return Ok(tc);
            }
            if has_top_left && has_bottom_left {
                return Ok(cl);
            }
            if has_top_right && has_bottom_right {
                return Ok(cr);
            }
            if has_bottom_left && has_bottom_right {
                return Ok(bc);
            }

            // if has_top_left && tc == tl && tc != tr && tc != cr {
            //     return Ok(tc);
            // }
            // if has_top_right && tc == tr && tc != tl {
            //     return Ok(tc);
            // }
            // if has_bottom_left && bc == bl && bc != br {
            //     return Ok(bc);
            // }
            // if has_bottom_right && bc == br && bc != bl {
            //     return Ok(bc);
            // }

            if has_top_left {
                return Ok(tc);
            }
            if has_top_right {
                return Ok(tc);
            }
            if has_bottom_left {
                return Ok(bc);
            }
            if has_bottom_right {
                return Ok(bc);
            }
            // if tc == cl && (tc == cr || tc == bc) {
            //     return Ok(tc);
            // }
            // if tc > 0 && cl > 0 {
            //     return Ok(1);
            // }
            // if tc > 0 && cr > 0 {
            //     return Ok(1);
            // }
            // if bc > 0 && cl > 0 {
            //     return Ok(1);
            // }
            // if bc > 0 && cr > 0 {
            //     return Ok(1);
            // }
            Ok(0)
        })?;
        Ok(denoised_image)
    }

    fn blur(&self) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = self.padding_with_color(1, 0)?;
        let denoised_image: Image = convolution3x3(&input_padded, |bm| {
            let cc: u8 = bm.get(1, 1).unwrap_or(255);
            if cc > 0 {
                // not a noisy pixel
                return Ok(cc);
            }
            let mut h: Histogram = bm.histogram_all();
            h.set_counter_to_zero(0);
            let color: u8 = h.most_popular_color_disallow_ambiguous().unwrap_or(0);
            Ok(color)
        })?;
        Ok(denoised_image)
    }

    fn denoise_type7(&self, noise_color: u8, background_color: u8) -> anyhow::Result<Image> {
        if noise_color == background_color {
            return Err(anyhow::anyhow!("noise color and background color must be different"));
        }
        if self.is_empty() {
            return Ok(Image::empty());
        }
        let input_padded: Image = self.padding_with_color(1, background_color)?;
        let output: Image = convolution3x3(&input_padded, |source| {
            let center: u8 = source.get(1, 1).unwrap_or(255);
            if center != noise_color {
                // not a noisy pixel
                return Ok(center);
            }
            let top_center: u8 = source.get(1, 0).unwrap_or(255);
            let center_left: u8 = source.get(0, 1).unwrap_or(255);
            let center_right: u8 = source.get(2, 1).unwrap_or(255);
            let bottom_center: u8 = source.get(1, 2).unwrap_or(255);
            let top_bottom_separator: bool = top_center == background_color && bottom_center == background_color;
            let left_right_separator: bool = center_left == background_color && center_right == background_color;
            if top_bottom_separator || left_right_separator {
                return Ok(background_color);
            }
            // if top_center

            // let mut h: Histogram = source.histogram_all();
            // h.set_counter_to_zero(0);
            // h.set_counter_to_zero(center);
            // let color: u8 = h.most_popular_color_disallow_ambiguous().unwrap_or(0);
            if top_center > 0 {
                return Ok(top_center);
            }
            if bottom_center > 0 {
                return Ok(bottom_center);
            }
            if center_left > 0 {
                return Ok(center_left);
            }
            if center_right > 0 {
                return Ok(center_right);
            }

            Ok(background_color)
        })?;
        Ok(output)
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

    #[test]
    fn test_10002_denoise_type1_diagonal_single_pixel() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            3, 0, 3, 0, 0,
            0, 0, 0, 3, 3,
            0, 3, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 3, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, input_pixels).expect("image");

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
    
    #[test]
    fn test_20000_denoise_type2_some_objects() {
        // Arrange
        let pixels: Vec<u8> = vec![
            3, 3, 3, 0, 0, 0, 8, 8, 8,
            3, 3, 3, 0, 0, 0, 8, 5, 8,
            3, 3, 3, 0, 0, 0, 8, 8, 8,
            0, 0, 0, 7, 5, 7, 0, 0, 0,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
            6, 6, 6, 0, 0, 5, 9, 9, 9,
            6, 6, 6, 0, 0, 0, 9, 9, 9,
            6, 5, 6, 0, 5, 0, 9, 9, 5,
        ];
        let input: Image = Image::try_create(9, 9, pixels).expect("image");

        // Act
        let actual: Image = input.denoise_type2(5).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 0, 0, 0, 8, 8, 8,
            3, 3, 3, 0, 0, 0, 8, 8, 8,
            3, 3, 3, 0, 0, 0, 8, 8, 8,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
            6, 6, 6, 0, 0, 0, 9, 9, 9,
            6, 6, 6, 0, 0, 0, 9, 9, 9,
            6, 6, 6, 0, 0, 0, 9, 9, 9,
        ];
        let expected: Image = Image::try_create(9, 9, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_denoise_type2_some_objects() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 2, 2, 2, 0, 0, 0,
            0, 5, 0, 2, 2, 2, 0, 0, 0,
            0, 0, 0, 2, 2, 2, 0, 0, 0,
            5, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 5, 0, 0, 0, 5, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 7, 7, 7, 0, 0, 0,
            0, 0, 0, 7, 7, 5, 0, 0, 0,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
        ];
        let input: Image = Image::try_create(9, 9, pixels).expect("image");

        // Act
        let actual: Image = input.denoise_type2(5).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 2, 2, 2, 0, 0, 0,
            0, 0, 0, 2, 2, 2, 0, 0, 0,
            0, 0, 0, 2, 2, 2, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
            0, 0, 0, 7, 7, 7, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(9, 9, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_denoise_type3_single_pixel_little_denoising() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            5, 0, 1, 0, 0,
            0, 0, 0, 3, 3,
            0, 5, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 5, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, input_pixels).expect("image");

        // Act
        let actual: Image = input.denoise_type3(1).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 5, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 5, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30001_denoise_type3_single_pixel_more_denoising() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            5, 0, 1, 0, 0,
            0, 0, 0, 3, 3,
            0, 5, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 5, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, input_pixels).expect("image");

        // Act
        let actual: Image = input.denoise_type3(2).expect("image");

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
