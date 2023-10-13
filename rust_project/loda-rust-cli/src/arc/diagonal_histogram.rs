use super::{Image, ImageHistogram, Histogram, ImageSkew, ImageSize, ImageSymmetry};

#[allow(dead_code)]
pub struct DiagonalHistogram {
    reverse_x: bool,
    image_size: ImageSize,
    histogram_vec: Vec<Histogram>,
}

impl DiagonalHistogram {
    /// The `diagonal A` is going from `top-left` to `bottom-right`
    #[allow(dead_code)]
    pub fn diagonal_a(image: &Image) -> anyhow::Result<DiagonalHistogram> {
        let flipped: Image = image.flip_x()?;
        Self::create(&flipped, true, image.size())
    }

    /// The `diagonal B` is going from `top-right` to `bottom-left`
    #[allow(dead_code)]
    pub fn diagonal_b(image: &Image) -> anyhow::Result<DiagonalHistogram> {
        Self::create(image, false, image.size())
    }

    fn create(image: &Image, reverse_x: bool, image_size: ImageSize) -> anyhow::Result<DiagonalHistogram> {
        if image.is_empty() {
            anyhow::bail!("image is empty. Must be 1x1 or bigger.");
        }
        let magic_color: u8 = 255;
        let image_skewed: Image = image.skew_x(magic_color, false)?;
        let mut histogram_vec: Vec<Histogram> = image_skewed.histogram_columns();
        for histogram in histogram_vec.iter_mut() {
            histogram.set_counter_to_zero(magic_color);
        }
        Ok(DiagonalHistogram { 
            reverse_x,
            image_size,
            histogram_vec
        })
    }

    #[allow(dead_code)]
    pub fn max_number_of_unique_colors(&self) -> u16 {
        let mut found_value: u16 = 0;
        for histogram in &self.histogram_vec {
            let count: u16 = histogram.number_of_counters_greater_than_zero();
            if count > found_value {
                found_value = count;
            }
        }
        found_value
    }

    #[allow(dead_code)]
    pub fn min_number_of_unique_colors(&self) -> u16 {
        let mut found_value: u16 = u16::MAX;
        for histogram in &self.histogram_vec {
            let count: u16 = histogram.number_of_counters_greater_than_zero();
            if found_value > count {
                found_value = count;
            }
        }
        found_value
    }

    #[allow(dead_code)]
    pub fn get(&self, x: i32, y: i32) -> Option<&Histogram> {
        let x: i32 = if self.reverse_x {
            (self.image_size.width as i32) - 1 - x
        } else {
            x
        };
        if x < 0 || y < 0 {
            return None;
        }
        if x >= (self.image_size.width as i32) || y >= (self.image_size.height as i32) {
            return None;
        }
        let column_index: usize = (x as usize) + (y as usize);
        self.histogram_vec.get(column_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_diagonal_a() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            0, 1, 0, 0,
            2, 0, 1, 0,
            0, 9, 0, 1,
            0, 0, 2, 0,
            3, 0, 0, 2,
        ];
        let input: Image = Image::try_create(4, 6, pixels).expect("image");

        // Act
        let dh: DiagonalHistogram = DiagonalHistogram::diagonal_a(&input).expect("ok");

        // Assert
        {
            let histogram: &Histogram = dh.get(0, 0).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(1), 4);
        }
        {
            let histogram: &Histogram = dh.get(3, 3).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(1), 4);
        }
        {
            let histogram: &Histogram = dh.get(1, 3).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 2);
            assert_eq!(histogram.get(2), 3);
            assert_eq!(histogram.get(9), 1);
        }
        {
            let histogram: &Histogram = dh.get(0, 5).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(3), 1);
        }
    }

    #[test]
    fn test_10001_diagonal_a() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 3, 0, 0, 8,
            0, 1, 0, 3, 0, 0,
            0, 0, 1, 0, 7, 0,
            9, 0, 0, 1, 0, 3,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let dh: DiagonalHistogram = DiagonalHistogram::diagonal_a(&input).expect("ok");

        // Assert
        {
            let histogram: &Histogram = dh.get(0, 0).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(1), 4);
        }
        {
            let histogram: &Histogram = dh.get(2, 2).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(1), 4);
        }
        {
            let histogram: &Histogram = dh.get(2, 0).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 2);
            assert_eq!(histogram.get(3), 3);
            assert_eq!(histogram.get(7), 1);
        }
        {
            let histogram: &Histogram = dh.get(4, 2).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 2);
            assert_eq!(histogram.get(3), 3);
            assert_eq!(histogram.get(7), 1);
        }
        {
            let histogram: &Histogram = dh.get(5, 0).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(8), 1);
        }
    }

    #[test]
    fn test_20000_diagonal_b() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 1,
            0, 0, 1, 0,
            0, 1, 0, 2,
            1, 0, 9, 0,
            0, 2, 0, 0,
            2, 0, 0, 3,
        ];
        let input: Image = Image::try_create(4, 6, pixels).expect("image");

        // Act
        let dh: DiagonalHistogram = DiagonalHistogram::diagonal_b(&input).expect("ok");

        // Assert
        {
            let histogram: &Histogram = dh.get(3, 0).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(1), 4);
        }
        {
            let histogram: &Histogram = dh.get(1, 2).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(1), 4);
        }
        {
            let histogram: &Histogram = dh.get(2, 3).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 2);
            assert_eq!(histogram.get(2), 3);
            assert_eq!(histogram.get(9), 1);
        }
        {
            let histogram: &Histogram = dh.get(3, 5).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(3), 1);
        }
    }

    #[test]
    fn test_20001_diagonal_b() {
        // Arrange
        let pixels: Vec<u8> = vec![
            8, 0, 0, 1, 0, 9,
            0, 0, 1, 0, 9, 0,
            0, 1, 0, 2, 0, 0,
            1, 0, 9, 0, 0, 5,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let dh: DiagonalHistogram = DiagonalHistogram::diagonal_b(&input).expect("ok");

        // Assert
        {
            let histogram: &Histogram = dh.get(0, 0).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(8), 1);
        }
        {
            let histogram: &Histogram = dh.get(3, 0).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(1), 4);
        }
        {
            let histogram: &Histogram = dh.get(5, 0).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 2);
            assert_eq!(histogram.get(2), 1);
            assert_eq!(histogram.get(9), 3);
        }
        {
            let histogram: &Histogram = dh.get(5, 3).expect("some");
            assert_eq!(histogram.number_of_counters_greater_than_zero(), 1);
            assert_eq!(histogram.get(5), 1);
        }
    }
}
