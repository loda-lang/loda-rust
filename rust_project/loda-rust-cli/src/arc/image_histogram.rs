use super::{Histogram, Image};

pub trait ImageHistogram {
    fn histogram_border(&self) -> anyhow::Result<Histogram>;
}

impl ImageHistogram for Image {
    /// Traverses the border of the image, and builds a histogram
    fn histogram_border(&self) -> anyhow::Result<Histogram> {
        let mut h = Histogram::new();
        if self.is_empty() {
            return Ok(h);
        }
        let x_max: i32 = (self.width() as i32) - 1;
        let y_max: i32 = (self.height() as i32) - 1;
        for y in 0..=y_max {
            for x in 0..=x_max {
                if x > 0 && x < x_max && y > 0 && y < y_max {
                    continue;
                }
                h.increment_pixel(&self, x, y);
            }
        }
        Ok(h)
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
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let histogram: Histogram = input.histogram_border().expect("histogram");
        let histogram_vec: Vec<u32> = histogram.to_vec();

        // Assert
        let mut expected: Vec<u32> = vec![0; 256];
        expected[1] = 5;
        expected[2] = 6;
        expected[3] = 5;
        assert_eq!(histogram_vec, expected);
    }
}
