use super::{Histogram, Image};

pub trait ImageHistogram {
    fn histogram_all(&self) -> Histogram;
    fn histogram_border(&self) -> Histogram;
    fn histogram_rows(&self) -> Vec<Histogram>;
    fn histogram_columns(&self) -> Vec<Histogram>;
}

impl ImageHistogram for Image {
    /// Histogram with all pixels in the image
    fn histogram_all(&self) -> Histogram {
        let mut h = Histogram::new();
        for y in 0..self.height() {
            for x in 0..self.width() {
                h.increment_pixel(&self, x as i32, y as i32);
            }
        }
        h
    }

    /// Histogram by traversing the border of the image
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

    /// Histogram for every row
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
        return rows;
    }

    /// Histogram for every column
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
        return columns;
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
    fn test_30000_histogram_rows() {
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
    fn test_40000_histogram_columns() {
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
}
