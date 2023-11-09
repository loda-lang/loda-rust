//! Determine local directionality in an image.
use super::{Image, ImageCrop, ImageHistogram, Histogram, ImageRotate90, HtmlLog};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Classification {
    TrueStrong,
    TrueWeak,
    False,
}

#[allow(dead_code)]
struct AnalyzeDirection {
    direction_horizontal: Image,
}

impl AnalyzeDirection {
    #[allow(dead_code)]
    fn analyze(image: &Image) -> anyhow::Result<Self> {

        let mut direction_horizontal_image: Image = image.clone_zero();

        let outside_color: u8 = 255;
        for y in 0..image.height() {
            for x in 0..image.width() {

                let area: Image = image.crop_outside((x as i32) - 2, (y as i32) - 2, 5, 5, outside_color)?;
                // if x == 2 && y == 0 {
                //     HtmlLog::image(&area);
                // }

                let is_row: Classification = Self::classify_row(&area)?;

                let set_value: u8 = match is_row {
                    Classification::TrueStrong => 1,
                    Classification::TrueWeak => 1,
                    Classification::False => 0,
                };
                _ = direction_horizontal_image.set(x as i32, y as i32, set_value);
            }
        }

        let instance = Self {
            direction_horizontal: direction_horizontal_image,
        };
        Ok(instance)
    }

    fn classify_row(image: &Image) -> anyhow::Result<Classification> {
        let center: u8 = image.get(2, 2).unwrap_or(255);

        let histograms: Vec<Histogram> = image.histogram_rows();

        let mut number_of_times_center_color_detected_outside: usize = 0;
        let mut all_pixels_have_same_value: bool = false;
        let mut center_row_outside_count: u32 = 0;
        for (index, histogram) in histograms.iter().enumerate() {
            if index != 2 {
                number_of_times_center_color_detected_outside += histogram.get(center) as usize;
            }
            if index == 2 {
                center_row_outside_count = histogram.get(255);
                if histogram.get(center) == 5 {
                    all_pixels_have_same_value = true;
                }
                if histogram.get(center) == 4 && histogram.get(255) == 1 {
                    all_pixels_have_same_value = true;
                }
                if histogram.get(center) == 3 && histogram.get(255) == 2 {
                    all_pixels_have_same_value = true;
                }
            }
        }

        if all_pixels_have_same_value && number_of_times_center_color_detected_outside == 0 && center_row_outside_count == 0 {
            return Ok(Classification::TrueStrong);
        }

        if all_pixels_have_same_value && number_of_times_center_color_detected_outside == 0 && center_row_outside_count > 0 {
            return Ok(Classification::TrueWeak);
        }

        if all_pixels_have_same_value && number_of_times_center_color_detected_outside > 0 {
            return Ok(Classification::TrueWeak);
        }

        Ok(Classification::False)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_classify_row_truestrong() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 7, 7,
            7, 7, 7, 7, 7,
            3, 3, 3, 3, 3,
            7, 7, 7, 7, 7,
            7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::TrueStrong);
    }

    #[test]
    fn test_10001_classify_row_trueweak() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 7, 7,
            7, 7, 3, 7, 7,
            3, 3, 3, 3, 3,
            7, 7, 7, 7, 7,
            7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::TrueWeak);
    }

    #[test]
    fn test_10002_classify_row_false() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 3, 7, 7,
            7, 7, 3, 7, 7,
            7, 7, 3, 7, 7,
            7, 7, 3, 7, 7,
            7, 7, 3, 7, 7,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::False);
    }

    #[test]
    fn test_20000_direction_horizontal() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9,
            2, 3, 4, 9, 2, 3, 4,
            3, 4, 5, 9, 1, 2, 3,
            4, 5, 6, 9, 0, 1, 2,
            9, 9, 9, 9, 9, 9, 9,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");

        // Act
        let actual: AnalyzeDirection = AnalyzeDirection::analyze(&input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(7, 6, expected_pixels).expect("image");
        assert_eq!(actual.direction_horizontal, expected);
    }
}
