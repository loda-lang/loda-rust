use super::{Image, ImageHistogram, Histogram, HistogramPair, ImagePadding, convolution3x3};
use std::cell::Cell;

pub trait ImageNoiseColor {
    fn noise_color_vec(&self, denoised_image: &Image) -> anyhow::Result<Vec<u8>>;

    /// Traverse all pixels in the 3x3 convolution and count how many have the same color as the center.
    /// 
    /// - Returns 1 when the center pixel is unique and have no duplicates.
    /// - Returns 2..9 depending on how many duplicates are found.
    fn count_duplicate_pixels_in_3x3(&self) -> anyhow::Result<Image>;

    /// Compare with the pixels above,below,left,right and count how many have the same color as the center.
    /// 
    /// - Returns 1 when the center pixel is unique and have no duplicates.
    /// - Returns 2..5 depending on how many duplicates are found.
    fn count_duplicate_pixels_in_neighbours(&self) -> anyhow::Result<Image>;

    fn one_pixel_noise_color_vec(&self) -> anyhow::Result<Vec<u8>>;

    /// Disregard counters that are lower than `min_count`.
    /// 
    /// When there are multiple ambiguous colors then pick the `color_ambiguous`.
    /// 
    /// When there are zero counters then pick the `color_none`.
    fn most_popular_color_in_3x3(&self, min_count: u8, color_none: u8, color_ambiguous: u8) -> anyhow::Result<Image>;
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

    fn count_duplicate_pixels_in_3x3(&self) -> anyhow::Result<Image> {
        // find an unused color for use as padding_color
        let histogram: Histogram = self.histogram_all();
        let padding_color: u8 = match histogram.unused_color() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("All colors are used in the histogram. Cannot pick a padding color"));
            }
        };
        let image_padded: Image = self.padding_with_color(1, padding_color)?;

        let image: Image = convolution3x3(&image_padded, |bm| {
            let center_color: u8 = bm.get(1, 1).unwrap_or(255);
            let mut count: u8 = 1;
            for y in 0..3i32 {
                for x in 0..3i32 {
                    if y == 1 && x == 1 {
                        continue;
                    }
                    let color: u8 = bm.get(x, y).unwrap_or(255);
                    if color == center_color {
                        count += 1;
                    }
                }
            }
            Ok(count)
        })?;
        Ok(image)
    }

    fn count_duplicate_pixels_in_neighbours(&self) -> anyhow::Result<Image> {
        // find an unused color for use as padding_color
        let histogram: Histogram = self.histogram_all();
        let padding_color: u8 = match histogram.unused_color() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("All colors are used in the histogram. Cannot pick a padding color"));
            }
        };
        let image_padded: Image = self.padding_with_color(1, padding_color)?;

        let image: Image = convolution3x3(&image_padded, |bm| {
            let center_color: u8 = bm.get(1, 1).unwrap_or(255);
            let mut count: u8 = 1;
            let pairs: [(u8,u8); 4] = [(1,0),(0,1),(2,1),(1,2)];
            for (x, y) in pairs {
                let color: u8 = bm.get(x as i32, y as i32).unwrap_or(255);
                if color == center_color {
                    count += 1;
                }
            }
            Ok(count)
        })?;
        Ok(image)
    }

    fn one_pixel_noise_color_vec(&self) -> anyhow::Result<Vec<u8>> {
        let count_image: Image = self.count_duplicate_pixels_in_3x3()?;

        // Histogram of all pixels where the count is just 1
        let mut histogram = Histogram::new();
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let count_value: u8 = count_image.get(x, y).unwrap_or(255);
                if count_value > 1 {
                    continue;
                }
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                histogram.increment(pixel_value);
            }
        }

        // The first element is the most frequently occuring noise
        // The last element is the least frequently occuring noise
        let pairs: Vec<(u32, u8)> = histogram.pairs_descending();
        let noise_colors: Vec<u8> = pairs.iter().map(|(_count,color)| *color).collect();
        Ok(noise_colors)
    }

    fn most_popular_color_in_3x3(&self, min_count: u8, color_none: u8, color_ambiguous: u8) -> anyhow::Result<Image> {
        // find an unused color for use as padding_color
        let histogram: Histogram = self.histogram_all();
        let padding_color: u8 = match histogram.unused_color() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("All colors are used in the histogram. Cannot pick a padding color"));
            }
        };
        let image_padded: Image = self.padding_with_color(1, padding_color)?;

        let padding_color_cell: Cell<u8> = Cell::new(padding_color);
        let image: Image = convolution3x3(&image_padded, |bm| {
            let center_color: u8 = bm.get(1, 1).unwrap_or(255);

            let mut histogram: Histogram = bm.histogram_all();

            // Ignore the padding color
            histogram.set_counter_to_zero(padding_color_cell.get());

            // Give a tiny boost to the color of the center pixel of the 3x3
            histogram.increment(center_color);

            // Ignore colors that occur fewer time than the limit
            histogram.set_counter_to_zero_where_count_is_below(min_count as u32);

            let the_color: u8;
            match histogram.most_popular() {
                HistogramPair::Item { count: _, color, ambiguous_score } => {
                    if ambiguous_score > 0 {
                        the_color = color_ambiguous;
                    } else {
                        the_color = color;
                    }
                },
                HistogramPair::None => {
                    the_color = color_none;
                }
            }

            Ok(the_color)
        })?;
        Ok(image)
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
        let actual: Vec<u8> = input_with_noise.noise_color_vec(&input_denoised).expect("vec");

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
        let actual: Vec<u8> = input_with_noise.noise_color_vec(&input_denoised).expect("vec");

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
        let actual: Vec<u8> = input_with_noise.noise_color_vec(&input_denoised).expect("vec");

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
        let actual: Vec<u8> = input_with_noise.noise_color_vec(&input_denoised).expect("vec");

        // Assert
        let expected: Vec<u8> = vec![5, 1];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_count_duplicate_pixels_in_3x3() {
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
        let actual: Image = input.count_duplicate_pixels_in_3x3().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 4, 1, 3, 2,
            4, 6, 5, 4, 4,
            3, 1, 4, 6, 6,
            4, 4, 3, 4, 4,
            4, 4, 1, 3, 2,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_count_duplicate_pixels_in_neighbours() {
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
        let actual: Image = input.count_duplicate_pixels_in_neighbours().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 1, 2, 2,
            3, 4, 3, 3, 3,
            2, 1, 3, 4, 4,
            3, 3, 2, 3, 3,
            3, 3, 1, 2, 2,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_one_pixel_noise_color_vec() {
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
        let actual: Vec<u8> = input.one_pixel_noise_color_vec().expect("vec");

        // Assert
        let expected: Vec<u8> = vec![5, 1];
        assert_eq!(actual, expected);
    }
}
