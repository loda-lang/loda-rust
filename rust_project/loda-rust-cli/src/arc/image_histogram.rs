use super::{Histogram, Image};

pub trait ImageHistogram {
    /// Histogram with all pixels in the image
    fn histogram_all(&self) -> Histogram;
    
    /// Histogram by traversing the border of the image
    fn histogram_border(&self) -> Histogram;

    /// Histogram by only traversing the pixels where `mask_pixel=1`
    fn histogram_with_mask(&self, mask: &Image) -> anyhow::Result<Histogram>;
    
    /// Histogram for every row
    fn histogram_rows(&self) -> Vec<Histogram>;
    
    /// Histogram for every column
    fn histogram_columns(&self) -> Vec<Histogram>;

    /// Detect horizontal lines.
    /// 
    /// return `Some(true)` if all rows contain just one color.
    /// 
    /// return `Some(false)` if one or more rows contain 2 or more colors.
    /// 
    /// return `None` when there is less than 2 columns.
    #[allow(dead_code)]
    fn is_repeated_column(&self) -> Option<bool>;

    /// Detect vertical lines.
    /// 
    /// return `Some(true)` if all columns contain just one color.
    /// 
    /// return `Some(false)` if one or more columns contain 2 or more colors.
    /// 
    /// return `None` when there is less than 2 rows.
    #[allow(dead_code)]
    fn is_repeated_row(&self) -> Option<bool>;
}

impl ImageHistogram for Image {
    fn histogram_all(&self) -> Histogram {
        let mut h = Histogram::new();
        for y in 0..self.height() {
            for x in 0..self.width() {
                h.increment_pixel(&self, x as i32, y as i32);
            }
        }
        h
    }

    fn histogram_border(&self) -> Histogram {
        let mut h = Histogram::new();
        if self.is_empty() {
            return h;
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
        h
    }

    fn histogram_with_mask(&self, mask: &Image) -> anyhow::Result<Histogram> {
        if self.size() != mask.size() {
            return Err(anyhow::anyhow!("Images must have same size. self: {:?} mask: {:?}", self.size(), mask.size()));
        }
        let mut h = Histogram::new();
        if self.is_empty() {
            return Ok(h);
        }
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let mask_value: u8 = mask.get(x, y).unwrap_or(255);
                if mask_value == 0 {
                    continue;
                }
                h.increment_pixel(&self, x, y);
            }
        }
        Ok(h)
    }

    fn histogram_rows(&self) -> Vec<Histogram> {
        if self.is_empty() {
            return vec!();
        }
        let mut rows = Vec::<Histogram>::with_capacity(self.height() as usize);
        for y in 0..self.height() {
            let mut h = Histogram::new();
            for x in 0..self.width() {
                h.increment_pixel(&self, x as i32, y as i32);
            }
            rows.push(h);
        }
        rows
    }

    fn histogram_columns(&self) -> Vec<Histogram> {
        if self.is_empty() {
            return vec!();
        }
        let mut columns = Vec::<Histogram>::with_capacity(self.width() as usize);
        for x in 0..self.width() {
            let mut h = Histogram::new();
            for y in 0..self.height() {
                h.increment_pixel(&self, x as i32, y as i32);
            }
            columns.push(h);
        }
        columns
    }

    fn is_repeated_column(&self) -> Option<bool> {
        if self.width() < 2 || self.height() < 1 {
            // There must be 2 or more columns
            return None;
        }
        let histogram_vec: Vec<Histogram> = self.histogram_rows();
        for histogram in histogram_vec {
            if histogram.number_of_counters_greater_than_zero() > 1 {
                return Some(false);
            }
        }
        Some(true)
    }

    fn is_repeated_row(&self) -> Option<bool> {
        if self.width() < 1 || self.height() < 2 {
            // There must be 2 or more rows
            return None;
        }
        let histogram_vec: Vec<Histogram> = self.histogram_columns();
        for histogram in histogram_vec {
            if histogram.number_of_counters_greater_than_zero() > 1 {
                return Some(false);
            }
        }
        Some(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_histogram_all() {
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
        let histogram: Histogram = input.histogram_all();
        let histogram_vec: Vec<u32> = histogram.to_vec();

        // Assert
        let mut expected: Vec<u32> = vec![0; 256];
        expected[0] = 8;
        expected[1] = 5;
        expected[2] = 6;
        expected[3] = 5;
        expected[9] = 1;
        assert_eq!(histogram_vec, expected);
    }

    #[test]
    fn test_20000_histogram_border() {
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
        let histogram: Histogram = input.histogram_border();
        let histogram_vec: Vec<u32> = histogram.to_vec();

        // Assert
        let mut expected: Vec<u32> = vec![0; 256];
        expected[0] = 0;
        expected[1] = 5;
        expected[2] = 6;
        expected[3] = 5;
        expected[9] = 0;
        assert_eq!(histogram_vec, expected);
    }

    #[test]
    fn test_30000_histogram_with_mask() {
        // Arrange
        let mask_pixels: Vec<u8> = vec![
            0, 1, 0, 1, 1,
            0, 1, 0, 1, 0,
            0, 1, 1, 1, 0,
            0, 1, 0, 1, 0,
            1, 1, 0, 1, 0,
        ];
        let mask: Image = Image::try_create(5, 5, mask_pixels).expect("image");
        let image_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            2, 2, 2, 2, 2,
            3, 3, 3, 3, 3,
            4, 4, 4, 4, 4,
        ];
        let input: Image = Image::try_create(5, 5, image_pixels).expect("image");

        // Act
        let histogram: Histogram = input.histogram_with_mask(&mask).expect("histogram");
        let histogram_vec: Vec<u32> = histogram.to_vec();

        // Assert
        let mut expected: Vec<u32> = vec![0; 256];
        expected[0] = 3;
        expected[1] = 2;
        expected[2] = 3;
        expected[3] = 2;
        expected[4] = 3;
        assert_eq!(histogram_vec, expected);
    }

    #[test]
    fn test_40000_histogram_rows() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            2, 0, 3, 0, 4,
            1, 1, 1, 1, 1,
            2, 0, 0, 0, 2,
            3, 3, 3, 3, 3,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let histograms: Vec<Histogram> = input.histogram_rows();

        // Assert
        let mut s = String::new();
        for h in histograms {
            let pairs = h.pairs_descending();
            for pair in pairs {
                s += &format!("{} {},", pair.0, pair.1);
            }
            s += "\n";
        }
        assert_eq!(s, "5 1,\n2 0,1 4,1 3,1 2,\n5 1,\n3 0,2 2,\n5 3,\n");
    }

    #[test]
    fn test_50000_histogram_columns() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1, 2, 3,
            1, 0, 1, 0, 3,
            1, 3, 1, 0, 3,
            1, 0, 1, 0, 3,
            1, 4, 1, 2, 3,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let histograms: Vec<Histogram> = input.histogram_columns();

        // Assert
        let mut s = String::new();
        for h in histograms {
            let pairs = h.pairs_descending();
            for pair in pairs {
                s += &format!("{} {},", pair.0, pair.1);
            }
            s += "\n";
        }
        assert_eq!(s, "5 1,\n2 0,1 4,1 3,1 2,\n5 1,\n3 0,2 2,\n5 3,\n");
    }

    #[test]
    fn test_60000_is_repeated_column() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5,
            3, 3, 3,
            3, 3, 3,
            9, 9, 9,
            5, 5, 5,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: Option<bool> = input.is_repeated_column();

        // Assert
        assert_eq!(actual, Some(true));
    }

    #[test]
    fn test_60001_is_repeated_column() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5,
            3, 3, 3,
            3, 0, 3,
            9, 9, 9,
            5, 5, 5,
        ];
        let input: Image = Image::try_create(3, 5, pixels).expect("image");

        // Act
        let actual: Option<bool> = input.is_repeated_column();

        // Assert
        assert_eq!(actual, Some(false));
    }

    #[test]
    fn test_60002_is_repeated_column() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5,
            3,
            2,
        ];
        let input: Image = Image::try_create(1, 3, pixels).expect("image");

        // Act
        let actual: Option<bool> = input.is_repeated_column();

        // Assert
        assert_eq!(actual, None);
    }

    #[test]
    fn test_70000_is_repeated_row() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 3, 3, 9, 5,
            5, 3, 3, 9, 5,
            5, 3, 3, 9, 5,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: Option<bool> = input.is_repeated_row();

        // Assert
        assert_eq!(actual, Some(true));
    }

    #[test]
    fn test_70001_is_repeated_row() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 3, 3, 9, 5,
            5, 3, 0, 9, 5,
            5, 3, 3, 9, 5,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        // Act
        let actual: Option<bool> = input.is_repeated_row();

        // Assert
        assert_eq!(actual, Some(false));
    }

    #[test]
    fn test_70002_is_repeated_row() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 3, 2,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let actual: Option<bool> = input.is_repeated_row();

        // Assert
        assert_eq!(actual, None);
    }
}
