use super::{Image, ImageHistogram, Histogram};

pub trait ImageNoiseColor {
    fn noise_color_vec(&self, denoised_image: &Image) -> anyhow::Result<Vec<u8>>;
}

impl ImageNoiseColor for Image {
    fn noise_color_vec(&self, denoised_image: &Image) -> anyhow::Result<Vec<u8>> {
        if self.is_empty() && denoised_image.is_empty() {
            return Ok(vec!());
        }
        if self.width() != denoised_image.width() {
            return Err(anyhow::anyhow!("both images must have same size, width is different"));
        }
        if self.height() != denoised_image.height() {
            return Err(anyhow::anyhow!("both images must have same size, height is different"));
        }
        let histogram_input_with_noise: Histogram = self.histogram_all();
        let histogram_denoised_image: Histogram = denoised_image.histogram_all();
        
        // Obtain histogram just for the noise
        let mut histogram: Histogram = histogram_input_with_noise.clone();
        for pair in histogram_denoised_image.pairs_descending() {
            let color: u8 = pair.1;
            histogram.set_counter_to_zero(color);
        }
        // The first element is the most frequently occuring noise
        // The last element is the least frequently occuring noise
        let pairs: Vec<(u32, u8)> = histogram.pairs_descending();
        let noise_colors: Vec<u8> = pairs.iter().map(|(_count,color)| *color).collect();
        Ok(noise_colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_no_difference_no_noise() {
        // Arrange
        let input_with_noise_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 1, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 1, 0, 0,
        ];
        let input_with_noise: Image = Image::try_create(5, 5, input_with_noise_pixels).expect("image");

        let input_denoised_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 1, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 1, 0, 0,
        ];
        let input_denoised: Image = Image::try_create(5, 5, input_denoised_pixels).expect("image");

        // Act
        let actual: Vec<u8> = input_with_noise.noise_color_vec(&input_denoised).expect("image");

        // Assert
        let expected: Vec<u8> = vec!();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_some_difference_some_noise() {
        // Arrange
        let input_with_noise_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 1, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 1, 0, 0,
        ];
        let input_with_noise: Image = Image::try_create(5, 5, input_with_noise_pixels).expect("image");

        let input_denoised_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 0, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 0, 0, 0,
        ];
        let input_denoised: Image = Image::try_create(5, 5, input_denoised_pixels).expect("image");

        // Act
        let actual: Vec<u8> = input_with_noise.noise_color_vec(&input_denoised).expect("image");

        // Assert
        let expected: Vec<u8> = vec![1];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_more_noise() {
        // Arrange
        let input_with_noise_pixels: Vec<u8> = vec![
            1, 0, 5, 0, 0,
            0, 0, 0, 3, 3,
            0, 1, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 1, 0, 0,
        ];
        let input_with_noise: Image = Image::try_create(5, 5, input_with_noise_pixels).expect("image");

        let input_denoised_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 0, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 0, 0, 0,
        ];
        let input_denoised: Image = Image::try_create(5, 5, input_denoised_pixels).expect("image");

        // Act
        let actual: Vec<u8> = input_with_noise.noise_color_vec(&input_denoised).expect("image");

        // Assert
        let expected: Vec<u8> = vec![1, 5];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10003_even_more_noise() {
        // Arrange
        let input_with_noise_pixels: Vec<u8> = vec![
            5, 0, 1, 0, 0,
            0, 0, 0, 3, 3,
            0, 5, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 5, 0, 0,
        ];
        let input_with_noise: Image = Image::try_create(5, 5, input_with_noise_pixels).expect("image");

        let input_denoised_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 3, 3,
            0, 0, 0, 3, 3,
            2, 2, 0, 3, 3,
            2, 2, 0, 0, 0,
        ];
        let input_denoised: Image = Image::try_create(5, 5, input_denoised_pixels).expect("image");

        // Act
        let actual: Vec<u8> = input_with_noise.noise_color_vec(&input_denoised).expect("image");

        // Assert
        let expected: Vec<u8> = vec![5, 1];
        assert_eq!(actual, expected);
    }
}
