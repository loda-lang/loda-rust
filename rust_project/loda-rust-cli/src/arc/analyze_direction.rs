//! Determine local directionality in an image.
use super::{Image, ImageCrop, ImageHistogram, Histogram, ImageRotate90, ImageTrim};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Classification {
    TrueStrong,
    TrueWeak,
    TrueWeakStripeDot,
    TrueWeakStripeHole,
    FalseIsRepeatedRow,
    False,
}

#[allow(dead_code)]
struct AnalyzeDirection {
    direction_horizontal: Image,
    direction_vertical: Image,
}

impl AnalyzeDirection {
    #[allow(dead_code)]
    fn analyze(image: &Image) -> anyhow::Result<Self> {

        let direction_horizontal: Image = Self::process(image)?;
        
        let image_rotated: Image = image.rotate_cw()?;
        let direction_image: Image = Self::process(&image_rotated)?;
        let direction_vertical: Image = direction_image.rotate_ccw()?;

        let instance = Self {
            direction_horizontal,
            direction_vertical,
        };
        Ok(instance)
    }

    fn process(image: &Image) -> anyhow::Result<Image> {
        let mut direction_horizontal: Image = image.clone_zero();

        let outside_color: u8 = 255;
        for y in 0..image.height() {
            for x in 0..image.width() {

                let area: Image = image.crop_outside((x as i32) - 3, (y as i32) - 2, 7, 5, outside_color)?;
                // if x == 2 && y == 0 {
                //     HtmlLog::image(&area);
                // }

                let is_row: Classification = Self::classify_row(&area)?;

                let set_value: u8 = match is_row {
                    Classification::TrueStrong => 1,
                    Classification::TrueWeak => 1,
                    Classification::TrueWeakStripeDot => 1,
                    Classification::TrueWeakStripeHole => 1,
                    Classification::FalseIsRepeatedRow => 0,
                    Classification::False => 0,
                };
                _ = direction_horizontal.set(x as i32, y as i32, set_value);
            }
        }
        Ok(direction_horizontal)
    }

    fn classify_row(image: &Image) -> anyhow::Result<Classification> {
        let center_minus1: u8 = image.get(2, 2).unwrap_or(255);
        let center: u8 = image.get(3, 2).unwrap_or(255);
        let center_plus1: u8 = image.get(4, 2).unwrap_or(255);

        // The center row, histogram for the left side, and the right side.
        let mut histogram_left = Histogram::new();
        let mut histogram_right = Histogram::new();
        for i in 1..=3 {
            histogram_left.increment_pixel(image, 3 - i, 2);
            histogram_right.increment_pixel(image, 3 + i, 2);
        }

        let histograms: Vec<Histogram> = image.histogram_rows();
        assert!(histograms.len() == 5, "there are supposed to be 5 rows in the image and thus 5 histograms");

        // Compare the histograms the with the center row.
        let histogram_center: &Histogram = &histograms[2];
        let mut all_same_histograms: bool = true;
        for (index, histogram) in histograms.iter().enumerate() {
            if index == 2 {
                // don't compare center row with itself
                continue;
            }
            if histogram.get(255) == 7 {
                // skip rows that are outside the image
                continue;
            }
            if histogram != histogram_center {
                all_same_histograms = false;
                break;
            }
        }
        if all_same_histograms {
            let trimmed_image: Image = image.trim_color(255)?;
            if trimmed_image.is_repeated_row().unwrap_or(false) {
                // It's the same row that is repeated.
                // This is usually when it's vertical lines.
                return Ok(Classification::FalseIsRepeatedRow);
            }
        }

        let mut number_of_times_center_color_detected_outside: usize = 0;
        let mut all_pixels_have_same_value_as_center: bool = false;
        let mut center_row_outside_count: u32 = 0;
        let mut center_row_same_center_color_count: u32 = 0;
        for (index, histogram) in histograms.iter().enumerate() {
            if index != 2 {
                number_of_times_center_color_detected_outside += histogram.get(center) as usize;
            }
            if index == 2 {
                center_row_same_center_color_count = histogram.get(center);
                center_row_outside_count = histogram.get(255);
                if histogram.get(center) == 7 {
                    all_pixels_have_same_value_as_center = true;
                }
                if histogram.get(center) == 6 && histogram.get(255) == 1 {
                    all_pixels_have_same_value_as_center = true;
                }
                if histogram.get(center) == 5 && histogram.get(255) == 2 {
                    all_pixels_have_same_value_as_center = true;
                }
                if histogram.get(center) == 4 && histogram.get(255) == 3 {
                    all_pixels_have_same_value_as_center = true;
                }
            }
        }

        // println!("histograms: {:?}", histograms);
        // println!("all_pixels_have_same_value: {}", all_pixels_have_same_value);
        // println!("number_of_times_center_color_detected_outside: {}", number_of_times_center_color_detected_outside);
        // println!("center_row_outside_count: {}", center_row_outside_count);
        // println!("center_row_same_center_color_count: {}", center_row_same_center_color_count);

        if all_pixels_have_same_value_as_center && number_of_times_center_color_detected_outside == 0 && center_row_outside_count == 0 {
            return Ok(Classification::TrueStrong);
        }

        if all_pixels_have_same_value_as_center && number_of_times_center_color_detected_outside == 0 && center_row_outside_count > 0 {
            return Ok(Classification::TrueWeak);
        }

        if all_pixels_have_same_value_as_center {
            return Ok(Classification::TrueWeak);
        }

        if histogram_left.get(center) > 0 && histogram_right.get(center) > 0 && center_minus1 == center_plus1 {
            // The center color is present to the left side, at the center to the right side. The center color occurs 3 or more times.
            // This may be a striped line with holes in it.
            return Ok(Classification::TrueWeakStripeDot);
        }

        if center_row_same_center_color_count == 1 {
            let center_minus1_present_on_opposite_side: bool = histogram_right.get(center_minus1) >= 2;
            let center_plus1_present_on_opposite_side: bool = histogram_left.get(center_plus1) >= 2;
            if center_minus1_present_on_opposite_side || center_plus1_present_on_opposite_side {
                // The center color is present on the opposite side.
                // This may be a striped line with holes in it. Where the center pixel is missing.
                return Ok(Classification::TrueWeakStripeHole);
            }
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
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
            3, 3, 3, 3, 3, 3, 3,
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::TrueStrong);
    }

    #[test]
    fn test_10001_classify_row_trueweak() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
            3, 3, 3, 3, 3, 3, 3,
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::TrueWeak);
    }

    #[test]
    fn test_10002_classify_row_trueweak_corner_topleft() {
        // Arrange
        let pixels: Vec<u8> = vec![
            255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 3, 3, 3, 3,
            255, 255, 255, 7, 7, 7, 7,
            255, 255, 255, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::TrueWeak);
    }

    #[test]
    fn test_10003_classify_row_trueweak_stripedot() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
            7, 3, 7, 3, 7, 3, 7, // alternating 3 and 7
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::TrueWeakStripeDot);
    }

    #[test]
    fn test_10004_classify_row_trueweak_stripedot() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
            3, 7, 3, 7, 3, 7, 3, // alternating 3 and 7
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::TrueWeakStripeDot);
    }

    #[test]
    fn test_10005_classify_row_trueweak_stripehole() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
            3, 8, 3, 7, 3, 8, 3, // stripe where the center pixel with value 7, is missing from the stripe
            7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::TrueWeakStripeHole);
    }

    #[test]
    fn test_10006_classify_row_false_is_repeated_row() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::FalseIsRepeatedRow);
    }

    #[test]
    fn test_10007_classify_row_false_is_repeated_row() {
        // Arrange
        let pixels: Vec<u8> = vec![
            255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255,
            7, 7, 7, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let actual: Classification = AnalyzeDirection::classify_row(&input).expect("ok");

        // Assert
        assert_eq!(actual, Classification::FalseIsRepeatedRow);
    }

    #[test]
    fn test_10008_classify_row_false() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
            6, 6, 6, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
            7, 7, 7, 3, 7, 7, 7,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

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

    #[test]
    fn test_30000_direction_vertical() {
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
            0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(7, 6, expected_pixels).expect("image");
        assert_eq!(actual.direction_vertical, expected);
    }
}
