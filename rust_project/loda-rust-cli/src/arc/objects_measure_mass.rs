use crate::arc::{Image, Histogram, ImageHistogram, ImageMask};

#[allow(dead_code)]
pub struct ObjectsMeasureMass;

impl ObjectsMeasureMass {
    /// Count the number of pixels in each object.
    /// 
    /// The `image` and the `enumerated_objects` must have same size. And the size must be 1x1 or bigger.
    /// 
    /// Returns an image with the same size as the input image.
    /// Where every pixel of the object is set the the mass count of that object.
    #[allow(dead_code)]
    pub fn run(image: &Image, enumerated_objects: &Image, ignore_colors: Option<&Histogram>) -> anyhow::Result<Image> {
        if image.size() != enumerated_objects.size() {
            return Err(anyhow::anyhow!("ObjectsMeasureMass: image must have same size"));
        }
        if image.is_empty() {
            return Err(anyhow::anyhow!("ObjectsMeasureMass: image must be 1x1 or bigger"));
        }
        let mut result_image = Image::zero(image.width(), image.height());
        for color in 0..=255u8 {
            let mask: Image = enumerated_objects.to_mask_where_color_is(color);
            let mut histogram: Histogram = image.histogram_with_mask(&mask)?;
            if let Some(other) = ignore_colors {
                histogram.subtract_histogram(other);
            }
            let mass_of_object: u32 = histogram.sum();
            let set_color: u8 = mass_of_object.min(255) as u8;
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
    fn test_10000_two_objects() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 5, 5, 5,
            7, 7, 5, 5, 5,
            5, 5, 7, 7, 7,
            5, 5, 7, 7, 7,
            5, 5, 7, 7, 7,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let enumerated_object_pixels: Vec<u8> = vec![
            1, 1, 2, 2, 2,
            1, 1, 2, 2, 2,
            2, 2, 1, 1, 1,
            2, 2, 1, 1, 1,
            2, 2, 1, 1, 1,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectsMeasureMass::run(&input, &enumerated_objects, None).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            13, 13, 12, 12, 12,
            13, 13, 12, 12, 12,
            12, 12, 13, 13, 13,
            12, 12, 13, 13, 13,
            12, 12, 13, 13, 13,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_three_objects() {
        // Arrange
        let input: Image = Image::zero(5, 5);

        let enumerated_object_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 2, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectsMeasureMass::run(&input, &enumerated_objects, None).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            16, 16, 16, 16, 16,
            16,  8,  8,  8, 16,
            16,  8,  1,  8, 16,
            16,  8,  8,  8, 16,
            16, 16, 16, 16, 16,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
