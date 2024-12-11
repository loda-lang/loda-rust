use super::{Histogram, Image, ImageHistogram};

pub trait ImageColorProfile {
    /// Identify the most popular color.
    /// 
    /// Returns `None` when it's ambiguous which color to pick.
    fn most_popular_color(&self) -> Option<u8>;

    /// Identify the least popular color.
    /// 
    /// Returns `None` when it's ambiguous which color to pick.
    fn least_popular_color(&self) -> Option<u8>;
}

impl ImageColorProfile for Image {
    fn most_popular_color(&self) -> Option<u8> {
        let histogram: Histogram = self.histogram_all();
        histogram.most_popular().color_disallow_ambiguous()
    }

    fn least_popular_color(&self) -> Option<u8> {
        let histogram: Histogram = self.histogram_all();
        histogram.least_popular().color_disallow_ambiguous()
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
    fn test_10001_most_popular_color_some() {
        // Arrange
        let input: Image = Image::color(1, 1, 42);

        // Act
        let color: Option<u8> = input.most_popular_color();

        // Assert
        assert_eq!(color, Some(42));
    }

    #[test]
    fn test_10002_most_popular_color_ambiguous() {
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

    #[test]
    fn test_10003_most_popular_color_ambiguous() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let color: Option<u8> = input.most_popular_color();

        // Assert
        assert_eq!(color, None);
    }

    #[test]
    fn test_20000_least_popular_color_some() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 2, 0,
            0, 2, 2, 0,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let color: Option<u8> = input.least_popular_color();

        // Assert
        assert_eq!(color, Some(1));
    }

    #[test]
    fn test_20001_least_popular_color_some() {
        // Arrange
        let input: Image = Image::color(1, 1, 42);

        // Act
        let color: Option<u8> = input.least_popular_color();

        // Assert
        assert_eq!(color, Some(42));
    }

    #[test]
    fn test_20002_least_popular_color_ambiguous() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let color: Option<u8> = input.least_popular_color();

        // Assert
        assert_eq!(color, None);
    }

    #[test]
    fn test_20003_least_popular_color_ambiguous() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
        ];
        let input: Image = Image::try_create(3, 1, pixels).expect("image");

        // Act
        let color: Option<u8> = input.least_popular_color();

        // Assert
        assert_eq!(color, None);
    }
}
