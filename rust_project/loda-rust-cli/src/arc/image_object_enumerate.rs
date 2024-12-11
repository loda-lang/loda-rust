use super::Image;

pub trait ImageObjectEnumerate {
    /// Assign a unique value to each object.
    /// 
    /// The pixels for the background is assigned `value 0`.
    /// 
    /// The pixels for `objects[0]` is assigned `value 1`.
    /// 
    /// The pixels for `objects[1]` is assigned `value 2`.
    /// 
    /// The pixels for the `Nth` object is assigned `value N-1`.
    /// 
    /// There can minimum be 1 object. If zero objects are provided then an `Err` is returned.
    /// 
    /// There can maximum be 255 objects. If more objects are provided then an `Err` is returned.
    /// 
    /// The objects doesn't have to cover the entire area. The areas not covered by any object is assigned the `value 0`.
    /// 
    /// Each object is a mask, where it's 1 the object is present, where it's 0 there is no object.
    /// If the object mask contains values that isn't 0 or 1, then an `Err` is returned.
    /// 
    /// The objects are supposed to "not overlap" with each other. If they do overlap then an `Err` is returned.
    /// 
    /// All the objects are supposed to have the same `width x height`, otherwise an `Err` is returned.
    /// The size of the output image is `width x height`.
    fn object_enumerate(objects: &Vec<Image>) -> anyhow::Result<Image>;
}

impl ImageObjectEnumerate for Image {
    fn object_enumerate(objects: &Vec<Image>) -> anyhow::Result<Image> {
        if objects.len() > 255 {
            return Err(anyhow::anyhow!("object_enumerate: Expected maximum 255 objects"));
        }

        // Determine the size of the result image
        let width: u8;
        let height: u8;
        match objects.first() {
            Some(object) => {
                width = object.width();
                height = object.height();
            },
            None => {
                return Err(anyhow::anyhow!("object_enumerate: Expected minimum 1 object"));
            }
        }

        // Verify that all objects have the same size
        for object in objects {
            if object.width() != width || object.height() != height {
                return Err(anyhow::anyhow!("object_enumerate: Expected all objects to have same size"));
            }
        }

        // The size must not be empty
        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!("object_enumerate: The size of the objects must be 1x1 or bigger"));
        }

        // Enumerate the objects
        let mut result_image: Image = Image::zero(width, height);
        for (index, object) in objects.iter().enumerate() {
            let object_id: u8 = (index+1).min(255) as u8; 

            // Draw the object with the object_id
            for y in 0..height {
                for x in 0..width {
                    let object_pixel_value: u8 = object.get(x as i32, y as i32).unwrap_or(255); 
                    match object_pixel_value {
                        0 => continue,
                        1 => {},
                        _ => {
                            return Err(anyhow::anyhow!("object_enumerate: Invalid mask, the object mask is supposed to be values in the range 0..1"));
                        }
                    }
                    let existing_pixel_value: u8 = result_image.get(x as i32, y as i32).unwrap_or(255); 
                    if existing_pixel_value > 0 {
                        return Err(anyhow::anyhow!("object_enumerate: Detected overlap between objects"));
                    }
                    match result_image.set(x as i32, y as i32, object_id) {
                        Some(()) => {},
                        None => {
                            return Err(anyhow::anyhow!("object_enumerate: Unable to set pixel inside the result image"));
                        }
                    }
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
    fn test_10000_object_enumerate_ok() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(5, 4, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 1, 1, 0,
            0, 0, 1, 1, 0,
            0, 0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(5, 4, pixels1).expect("image");

        let pixels2: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
        ];
        let input2: Image = Image::try_create(5, 4, pixels2).expect("image");

        let input_objects: Vec<Image> = vec![input0, input1, input2];

        // Act
        let output: Image = Image::object_enumerate(&input_objects).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            1, 1, 2, 2, 0,
            1, 1, 2, 2, 0,
            0, 0, 0, 0, 3,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10001_object_enumerate_exceed_maximum() {
        // Arrange
        let mut input_objects: Vec<Image> = vec!();
        for _ in 0..256 {
            let small_image = Image::zero(1, 1);
            input_objects.push(small_image);
        }

        // Act
        let error = Image::object_enumerate(&input_objects).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("maximum"), true);
    }

    #[test]
    fn test_10002_object_enumerate_different_sizes() {
        // Arrange
        let mut input_objects: Vec<Image> = vec!();
        input_objects.push(Image::zero(1, 2));
        input_objects.push(Image::zero(2, 1));

        // Act
        let error = Image::object_enumerate(&input_objects).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("same size"), true);
    }

    #[test]
    fn test_10003_object_enumerate_too_small() {
        // Arrange
        let input_objects: Vec<Image> = vec![Image::empty()];

        // Act
        let error = Image::object_enumerate(&input_objects).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("1x1 or bigger"), true);
    }

    #[test]
    fn test_10004_object_enumerate_invalid_mask_color() {
        // Arrange
        let input_objects: Vec<Image> = vec![Image::color(1, 1, 5)];

        // Act
        let error = Image::object_enumerate(&input_objects).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("Invalid mask"), true);
    }

    #[test]
    fn test_10005_object_enumerate_overlap_detected() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            1, 1, 1, 0, 0,
            1, 1, 1, 0, 0,
            1, 1, 1, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(5, 4, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 1, 1, 1,
            0, 0, 1, 1, 1,
            0, 0, 1, 1, 1,
        ];
        let input1: Image = Image::try_create(5, 4, pixels1).expect("image");

        let input_objects: Vec<Image> = vec![input0, input1];

        // Act
        let error = Image::object_enumerate(&input_objects).expect_err("is supposed to fail");

        // Assert
        let s = format!("{:?}", error);
        assert_eq!(s.contains("Detected overlap"), true);
    }
}
