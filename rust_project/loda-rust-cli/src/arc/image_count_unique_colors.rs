use super::{Image, ImageHistogram, ImageRotate, Histogram};

pub trait ImageCountUniqueColors {
    /// Returns an image that is 1 pixel wide and the same height as `self.height`.
    /// 
    /// Each row is the number of unique colors for that particular row.
    fn count_unique_colors_per_row(&self) -> anyhow::Result<Image>;
    
    /// Returns an image that is 1 pixel tall and the same width as `self.width`.
    /// 
    /// Each column is the number of unique colors for that particular column.
    fn count_unique_colors_per_column(&self) -> anyhow::Result<Image>;
}

impl ImageCountUniqueColors for Image {
    fn count_unique_colors_per_row(&self) -> anyhow::Result<Image> {
        let histograms: Vec<Histogram> = self.histogram_rows();
        let mut result_image = Image::zero(1, self.height());
        for (index, histogram) in histograms.iter().enumerate() {
            let count: u32 = histogram.number_of_counters_greater_than_zero();
            let clamped_count: u8 = if count <= (u8::MAX as u32) {
                count as u8
            } else {
                255
            };
            _ = result_image.set(0, index as i32, clamped_count);
        }
        Ok(result_image)
    }

    fn count_unique_colors_per_column(&self) -> anyhow::Result<Image> {
        let mut image: Image = self.rotate_cw()?;
        image = image.count_unique_colors_per_row()?;
        image = image.rotate_ccw()?;
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_count_unique_colors_per_row() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0, 0,
            5, 1, 5, 0, 1,
            3, 3, 3, 3, 3,
            5, 6, 7, 4, 3,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.count_unique_colors_per_row().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2,
            3,
            1,
            5
        ];
        let expected: Image = Image::try_create(1, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_count_unique_colors_per_column() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 0, 0, 0,
            5, 1, 5, 0, 1,
            3, 3, 3, 3, 3,
            5, 6, 7, 4, 3,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.count_unique_colors_per_column().expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            2, 4, 4, 3, 3,
        ];
        let expected: Image = Image::try_create(5, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
