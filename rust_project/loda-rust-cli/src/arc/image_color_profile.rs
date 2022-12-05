use super::{Histogram, Image, ImageHistogram};

pub trait ImageColorProfile {
    fn most_popular_color(&self) -> Option<u8>;
}

impl ImageColorProfile for Image {
    fn most_popular_color(&self) -> Option<u8> {
        let histogram: Histogram = self.histogram_all();
        let pairs = histogram.sorted_pairs();
        if pairs.len() == 1 {
            let pair: &(u32, u8) = pairs.get(0).unwrap();
            return Some(pair.1);
        }
        if pairs.len() >= 2 {
            let pair0: &(u32, u8) = pairs.get(0).unwrap();
            let pair1: &(u32, u8) = pairs.get(1).unwrap();
            if pair0.0 == pair1.0 {
                // Two popular colors with same popularity. It's ambiguous which one to pick. Pick none.
                return None;
            }
            return Some(pair0.1);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_most_popular_color_some() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 2, 0,
            0, 3, 4, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let color: Option<u8> = input.most_popular_color();

        // Assert
        assert_eq!(color, Some(0));
    }

    #[test]
    fn test_10001_most_popular_color_ambiguous() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let color: Option<u8> = input.most_popular_color();

        // Assert
        assert_eq!(color, None);
    }
}
