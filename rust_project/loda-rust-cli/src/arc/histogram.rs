use super::Image;

pub trait BitmapHistogram {
    fn histogram_border(&self) -> anyhow::Result<Vec<u32>>;
}

impl BitmapHistogram for Image {
    /// Traverses the border of the bitmap, and builds a histogram with 256 counters
    fn histogram_border(&self) -> anyhow::Result<Vec<u32>> {
        let mut histogram: Vec<u32> = vec![0; 256];
        if self.is_empty() {
            return Ok(histogram);
        }
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;
        for y in 0..=y_max {
            for x in 0..=x_max {
                if x > 0 && x < x_max && y > 0 && y < y_max {
                    continue;
                }
                let pixel_value: u8 = self.get(x, y).unwrap_or(255);
                let original_count: u32 = match histogram.get(pixel_value as usize) {
                    Some(value) => *value,
                    None => {
                        return Err(anyhow::anyhow!("Integrity error. Counter in histogram out of bounds"));
                    }
                };
                let count: u32 = original_count + 1;
                histogram[pixel_value as usize] = count;
            }
        }
        Ok(histogram)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_histogram_border() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            2, 0, 0, 0, 2,
            2, 0, 9, 0, 2,
            2, 0, 0, 0, 2,
            3, 3, 3, 3, 3,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("bitmap");

        // Act
        let actual: Vec<u32> = input.histogram_border().expect("bitmap");

        // Assert
        let mut expected: Vec<u32> = vec![0; 256];
        expected[1] = 5;
        expected[2] = 6;
        expected[3] = 5;
        assert_eq!(actual, expected);
    }
}
