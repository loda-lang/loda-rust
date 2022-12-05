use super::Image;

pub trait ImageStack {
    // Horizontal stack - place images side by side
    fn hstack(images: Vec<Image>) -> anyhow::Result<Image>;

    // Vertical stack - place images on top of each other
    fn vstack(images: Vec<Image>) -> anyhow::Result<Image>;
}

impl ImageStack for Image {
    fn hstack(images: Vec<Image>) -> anyhow::Result<Image> {
        let first_image: &Image = match images.first() {
            Some(image) => image,
            None => {
                return Ok(Image::empty());
            }
        };
        let height = first_image.height();

        // Compute width of all the images combined
        // Verify that the height is the same of all the images
        let mut width_usize: usize = 0;
        for image in &images {
            if image.height() != height {
                return Err(anyhow::anyhow!("hstack: Cannot horizontal stack images with different heights"));
            }
            width_usize += image.width() as usize;
        }
        if width_usize > 255 {
            return Err(anyhow::anyhow!("hstack: The resulting image must be max 255, but got width: {}", width_usize));
        }
        let width: u8 = width_usize as u8;
        if width == 0 {
            return Ok(Image::empty());
        }

        // Transfer pixels
        let mut result_image = Image::zero(width, height);
        let mut current_x: i32 = 0;
        for image in images {
            for y in 0..image.height() {
                for x in 0..image.width() {
                    let pixel_value: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                    let set_x = current_x + (x as i32);
                    match result_image.set(set_x, y as i32, pixel_value) { 
                        Some(()) => {},
                        None => {
                            return Err(anyhow::anyhow!("hstack. Unable to set pixel ({}, {}) inside the result bitmap", set_x, y));
                        }
                    }
                }
            }
            current_x += image.width() as i32;
        }

        Ok(result_image)
    }

    fn vstack(images: Vec<Image>) -> anyhow::Result<Image> {
        let first_image: &Image = match images.first() {
            Some(image) => image,
            None => {
                return Ok(Image::empty());
            }
        };
        let width = first_image.width();

        // Compute height of all the images combined
        // Verify that the width is the same of all the images
        let mut height_usize: usize = 0;
        for image in &images {
            if image.width() != width {
                return Err(anyhow::anyhow!("vstack: Cannot vertical stack images with different widths"));
            }
            height_usize += image.height() as usize;
        }
        if height_usize > 255 {
            return Err(anyhow::anyhow!("vstack: The resulting image must be max 255, but got height: {}", height_usize));
        }
        let height: u8 = height_usize as u8;
        if height == 0 {
            return Ok(Image::empty());
        }

        // Transfer pixels
        let mut result_image = Image::zero(width, height);
        let mut current_y: i32 = 0;
        for image in images {
            for y in 0..image.height() {
                for x in 0..image.width() {
                    let pixel_value: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                    let set_y = current_y + (y as i32);
                    match result_image.set(x as i32, set_y, pixel_value) { 
                        Some(()) => {},
                        None => {
                            return Err(anyhow::anyhow!("vstack. Unable to set pixel ({}, {}) inside the result bitmap", x, set_y));
                        }
                    }
                }
            }
            current_y += image.height() as i32;
        }

        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_hstack_small() {
        // Arrange
        let image0: Image = Image::color(1, 1, 0);
        let image1: Image = Image::color(1, 1, 1);
        let image2: Image = Image::color(1, 1, 2);

        // Act
        let actual: Image = Image::hstack(vec![image0, image1, image2]).expect("image");

        // Assert
        let expected: Image = Image::try_create(3, 1, vec![0, 1, 2]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_hstack_big() {
        // Arrange
        let image0: Image = Image::color(2, 2, 0);
        let image1: Image = Image::color(3, 2, 1);

        // Act
        let actual: Image = Image::hstack(vec![image0, image1]).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 1, 1,
            0, 0, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(5, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_vstack_small() {
        // Arrange
        let image0: Image = Image::color(1, 1, 0);
        let image1: Image = Image::color(1, 1, 1);
        let image2: Image = Image::color(1, 1, 2);

        // Act
        let actual: Image = Image::vstack(vec![image0, image1, image2]).expect("image");

        // Assert
        let expected: Image = Image::try_create(1, 3, vec![0, 1, 2]).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_vstack_big() {
        // Arrange
        let image0: Image = Image::color(2, 2, 0);
        let image1: Image = Image::color(2, 3, 1);

        // Act
        let actual: Image = Image::vstack(vec![image0, image1]).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0,
            0, 0,
            1, 1,
            1, 1,
            1, 1,
        ];
        let expected: Image = Image::try_create(2, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
