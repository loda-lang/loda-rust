use crate::arc::{Image, Histogram, ImageHistogram, ImageMask};

#[allow(dead_code)]
pub struct ObjectsUniqueColorCount;

impl ObjectsUniqueColorCount {
    /// Determine the number of colors in each object.
    /// 
    /// The `image` and the `enumerated_objects` must have same size. And the size must be 1x1 or bigger.
    /// 
    /// Returns an image with the same size as the input image.
    /// Where every pixel of the object is set the number of unique colors in that object.
    /// 
    /// The number of unique colors is clamped at 255, so it fits into 8 bits.
    #[allow(dead_code)]
    pub fn run(image: &Image, enumerated_objects: &Image, ignore_colors: Option<&Histogram>) -> anyhow::Result<Image> {
        if image.size() != enumerated_objects.size() {
            return Err(anyhow::anyhow!("ObjectsUniqueColorCount: images must have same size"));
        }
        if image.is_empty() {
            return Err(anyhow::anyhow!("ObjectsUniqueColorCount: image must be 1x1 or bigger"));
        }
        let mut result_image = Image::zero(image.width(), image.height());
        for color in 0..=255u8 {
            let mask: Image = enumerated_objects.to_mask_where_color_is(color);
            let mut histogram: Histogram = image.histogram_with_mask(&mask)?;
            if let Some(other) = ignore_colors {
                histogram.subtract_histogram(other);
            }
            let unique_color_count: u16 = histogram.number_of_counters_greater_than_zero();
            let set_color: u8 = unique_color_count.min(255) as u8;
            for y in 0..image.height() {
                for x in 0..image.width() {
                    let mask_value: u8 = mask.get(x as i32, y as i32).unwrap_or(0);
                    if mask_value == 0 {
                        continue;
                    }
                    _ = result_image.set(x as i32, y as i32, set_color);
                }
            }
        }
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_three_objects() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            1, 0, 1, 0, 1,
            8, 8, 8, 8, 8,
        ];
        let input: Image = Image::try_create(5, 3, pixels).expect("image");

        let enumerated_object_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            2, 2, 2, 2, 2,
            3, 3, 3, 3, 3,
        ];
        let enumerated_objects: Image = Image::try_create(5, 3, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectsUniqueColorCount::run(&input, &enumerated_objects, None).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            2, 2, 2, 2, 2,
            1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(5, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
