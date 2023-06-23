use super::{Image, Histogram, ImageMask, ImageSymmetry, ImageRotate};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ObjectsAndPositionMode {
    Top,
    Bottom,
    Left,
    Right,
}

#[allow(dead_code)]
pub struct ObjectsAndPosition;

impl ObjectsAndPosition {
    /// Mask of the object in the `enumerated_objects` that is at the top, bottom, left or right.
    /// 
    /// The `enumerated_objects` must be 1x1 or bigger.
    /// 
    /// Returns an error when there are zero objects in the `enumerated_objects`.
    /// 
    /// Returns an error when it's ambiguous which object to pick.
    /// 
    /// Returns an mask with the same size as the `enumerated_objects` image.
    #[allow(dead_code)]
    pub fn run(enumerated_objects: &Image, mode: ObjectsAndPositionMode) -> anyhow::Result<Image> {
        match mode {
            ObjectsAndPositionMode::Top => {
                return Self::top_most_object(enumerated_objects);
            },
            ObjectsAndPositionMode::Bottom => {
                let input_transformed: Image = enumerated_objects.flip_y()?;
                let output_transformed: Image = Self::top_most_object(&input_transformed)?;
                return output_transformed.flip_y();
            },
            ObjectsAndPositionMode::Left => {
                let input_transformed: Image = enumerated_objects.rotate_cw()?;
                let output_transformed: Image = Self::top_most_object(&input_transformed)?;
                return output_transformed.rotate_ccw();
            },
            ObjectsAndPositionMode::Right => {
                let input_transformed: Image = enumerated_objects.rotate_ccw()?;
                let output_transformed: Image = Self::top_most_object(&input_transformed)?;
                return output_transformed.rotate_cw();
            },
        }
    }

    /// Mask of the top most object in the `enumerated_objects`.
    /// 
    /// The `enumerated_objects` must be 1x1 or bigger.
    /// 
    /// Returns an error when there are zero objects in the `enumerated_objects`.
    /// 
    /// Returns an error when it's ambiguous which object to pick.
    /// 
    /// Returns an mask with the same size as the `enumerated_objects` image.
    fn top_most_object(enumerated_objects: &Image) -> anyhow::Result<Image> {
        if enumerated_objects.is_empty() {
            return Err(anyhow::anyhow!("ObjectsAndPosition: image must be 1x1 or bigger"));
        }

        // Start with the top row, and go down until we find an object.
        for y in 0..enumerated_objects.height() {
            let mut objects_found = Histogram::new();
            for x in 0..enumerated_objects.width() {
                let pixel_value: u8 = enumerated_objects.get(x as i32, y as i32).unwrap_or(0);
                if pixel_value == 0 {
                    continue;
                }
                objects_found.increment(pixel_value);
            }
            if objects_found.number_of_counters_greater_than_zero() >= 2 {
                return Err(anyhow::anyhow!("ObjectsAndPosition: ambiguous, two or more objects found"));
            }
            let object_index: u8 = match objects_found.most_popular_color_disallow_ambiguous() {
                Some(value) => value,
                None => {
                    // didn't find anything on this row, try next row
                    continue;
                }
            };

            // Found an object, create a mask and return it.
            let mask: Image = enumerated_objects.to_mask_where_color_is(object_index);
            return Ok(mask);
        }

        // Looped over all the rows, but didn't find any object.
        Err(anyhow::anyhow!("ObjectsAndPosition: didn't find any object"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_top() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            0, 0, 4, 0, 4,
            0, 0, 4, 4, 4,
            2, 2, 3, 4, 3,
            2, 2, 3, 3, 3,
            2, 2, 3, 3, 3,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectsAndPosition::run(&enumerated_objects, ObjectsAndPositionMode::Top).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0, 1,
            0, 0, 1, 1, 1,
            0, 0, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_top() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 4, 0, 4,
            0, 0, 4, 4, 4,
            2, 2, 3, 4, 3,
            2, 2, 3, 3, 3,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectsAndPosition::run(&enumerated_objects, ObjectsAndPositionMode::Top).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 1,
            0, 0, 1, 1, 1,
            0, 0, 0, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_top_two_or_more_objects_ambiguous_which_object_to_pick() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            7, 0, 4, 4, 0,
            0, 0, 4, 4, 0,
            2, 2, 3, 4, 3,
            2, 2, 3, 3, 3,
            2, 2, 3, 3, 3,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let error = ObjectsAndPosition::run(&enumerated_objects, ObjectsAndPositionMode::Top).expect_err("should fail");

        // Assert
        let message: String = format!("{}", error);
        assert_eq!(message.contains("ambiguous, two or more objects found"), true);
    }

    #[test]
    fn test_10003_top_no_objects() {
        // Arrange
        let enumerated_objects: Image = Image::zero(5, 5);

        // Act
        let error = ObjectsAndPosition::run(&enumerated_objects, ObjectsAndPositionMode::Top).expect_err("should fail");

        // Assert
        let message: String = format!("{}", error);
        assert_eq!(message.contains("didn't find any object"), true);
    }

    #[test]
    fn test_20000_bottom() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            0, 0, 4, 3, 4,
            0, 0, 4, 3, 4,
            3, 3, 3, 4, 3,
            0, 0, 0, 4, 0,
            0, 0, 0, 0, 0,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectsAndPosition::run(&enumerated_objects, ObjectsAndPositionMode::Bottom).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0, 1,
            0, 0, 1, 0, 1,
            0, 0, 0, 1, 0,
            0, 0, 0, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_left() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            0, 0, 4, 3, 4,
            0, 0, 4, 3, 4,
            3, 3, 3, 4, 3,
            0, 0, 0, 4, 0,
            0, 0, 0, 0, 0,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectsAndPosition::run(&enumerated_objects, ObjectsAndPositionMode::Left).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 1, 0,
            0, 0, 0, 1, 0,
            1, 1, 1, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_right() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            0, 0, 4, 3, 4,
            0, 0, 4, 3, 4,
            3, 3, 3, 4, 0,
            0, 0, 0, 4, 0,
            0, 0, 0, 3, 0,
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");

        // Act
        let actual: Image = ObjectsAndPosition::run(&enumerated_objects, ObjectsAndPositionMode::Right).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0, 1,
            0, 0, 1, 0, 1,
            0, 0, 0, 1, 0,
            0, 0, 0, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

}
