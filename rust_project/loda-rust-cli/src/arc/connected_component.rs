//! Connected component labeling/analysis
//! 
//! https://en.wikipedia.org/wiki/Connected-component_labeling
use super::{Image, ImageFill, PixelConnectivity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectedComponentItem {
    pub mask: Image,
    pub mass: u16,
    pub x: u8,
    pub y: u8,
}

pub struct ConnectedComponent;

impl ConnectedComponent {
    /// Identify clusters of connected pixels with an `ignore_mask` of areas to be ignored
    /// 
    /// Each object is a mask, where it's 1 the object is present, where it's 0 there is no object.
    /// 
    /// Counts the number of pixels in each of the objects, so that this costly operation can be avoided.
    pub fn find_objects_with_ignore_mask_inner(connectivity: PixelConnectivity, image: &Image, ignore_mask: &Image) -> anyhow::Result<Vec<ConnectedComponentItem>> {
        if ignore_mask.size() != image.size() {
            return Err(anyhow::anyhow!("The size of the ignore_mask must be the same, but is different"));
        }
        let mut object_mask_vec = Vec::<ConnectedComponentItem>::new();
        let mut accumulated_mask: Image = ignore_mask.clone();
        for y in 0..(image.height() as i32) {
            for x in 0..(image.width() as i32) {
                // Only visit pixels that have not yet been visited
                let mask_value: u8 = accumulated_mask.get(x, y).unwrap_or(255);
                if mask_value > 0 {
                    // This pixel has already been visited, ignore it
                    continue;
                }

                // Flood fill
                let color: u8 = image.get(x, y).unwrap_or(255);
                let mut object_mask = ignore_mask.clone();
                object_mask.mask_flood_fill(&image, x, y, color, connectivity)?;

                // Clear pixels that are in the original ignore_mask
                for yy in 0..(image.height() as i32) {
                    for xx in 0..(image.width() as i32) {
                        let mask_value: u8 = ignore_mask.get(xx, yy).unwrap_or(255);
                        if mask_value > 0 {
                            let _ = object_mask.set(xx, yy, 0);
                        }
                    }
                }

                // Copy the mask into the accumulated mask, so that the pixel doesn't get visited again
                //
                // Count the number of pixels in the mask that are non-zero.
                //
                // Determine the top/left coordinate of where the mask has a non-zero pixel.
                let mut mass: u32 = 0;
                let mut first_nonzero_pixel_x: u8 = 0;
                let mut first_nonzero_pixel_y: u8 = 0;
                for yy in 0..image.height() {
                    for xx in 0..image.width() {
                        let mask_value: u8 = object_mask.get(xx as i32, yy as i32).unwrap_or(255);
                        if mask_value == 0 {
                            continue;
                        }
                        let _ = accumulated_mask.set(xx as i32, yy as i32, 1);
                        if mass == 0 {
                            first_nonzero_pixel_x = xx;
                            first_nonzero_pixel_y = yy;
                        }
                        mass += 1;
                    }
                }
                let mass: u16 = mass.min(u16::MAX as u32) as u16;

                let item = ConnectedComponentItem {
                    mask: object_mask,
                    mass,
                    x: first_nonzero_pixel_x,
                    y: first_nonzero_pixel_y,
                };
                object_mask_vec.push(item);
            }
        }
        Ok(object_mask_vec)
    }

    /// Identify clusters of connected pixels
    /// 
    /// Each object is a mask, where it's 1 the object is present, where it's 0 there is no object.
    pub fn find_objects(connectivity: PixelConnectivity, image: &Image) -> anyhow::Result<Vec<Image>> {
        let ignore_mask = Image::zero(image.width(), image.height());
        Self::find_objects_with_ignore_mask(connectivity, image, &ignore_mask)
    }

    /// Identify clusters of connected pixels with an `ignore_mask` of areas to be ignored
    /// 
    /// Each object is a mask, where it's 1 the object is present, where it's 0 there is no object.
    pub fn find_objects_with_ignore_mask(connectivity: PixelConnectivity, image: &Image, ignore_mask: &Image) -> anyhow::Result<Vec<Image>> {
        let items: Vec<ConnectedComponentItem> = Self::find_objects_with_ignore_mask_inner(connectivity, image, ignore_mask)?;
        let images: Vec<Image> = items.into_iter().map(|item| item.mask ).collect();
        Ok(images)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageStack, ImageTryCreate};

    #[test]
    fn test_10000_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = ConnectedComponent::find_objects(PixelConnectivity::Connectivity4, &input).expect("vec");

        // Assert
        assert_eq!(mask_vec.len(), 3);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 0, 1, 0,
            1, 0, 1, 1, 0,
            1, 1, 1, 1, 0,

            0, 0, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 0, 0,

            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
        ];
        let expected = Image::create_raw(5, 4*3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10001_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 6, 6, 6, 5,
            5, 6, 5, 6, 5,
            5, 6, 6, 6, 5,
            5, 5, 5, 5, 5,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = ConnectedComponent::find_objects(PixelConnectivity::Connectivity4, &input).expect("vec");

        // Assert
        assert_eq!(mask_vec.len(), 3);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,

            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 1, 0, 1, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 0,
            
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected = Image::create_raw(5, 5*3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10002_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = ConnectedComponent::find_objects(PixelConnectivity::Connectivity4, &input).expect("vec");

        // Assert
        assert_eq!(mask_vec.len(), 5);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 0, 0,
            0, 0, 0,

            0, 1, 1,
            0, 0, 1,
            0, 0, 0,

            0, 0, 0,
            1, 0, 0,
            1, 1, 0,

            0, 0, 0,
            0, 1, 0,
            0, 0, 0,

            0, 0, 0,
            0, 0, 0,
            0, 0, 1,
        ];
        let expected = Image::create_raw(3, 3*5, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10003_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 
            0, 1, 0, 
            0, 0, 0, 
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = ConnectedComponent::find_objects(PixelConnectivity::Connectivity4, &input).expect("vec");

        // Assert
        assert_eq!(mask_vec.len(), 2);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,

            0, 0, 0, 
            0, 1, 0, 
            0, 0, 0, 
        ];
        let expected = Image::create_raw(3, 3*2, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10004_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = ConnectedComponent::find_objects(PixelConnectivity::Connectivity4, &input).expect("vec");

        // Assert
        assert_eq!(mask_vec.len(), 2);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,

            0, 0, 0, 
            0, 1, 0, 
            0, 0, 0, 
        ];
        let expected = Image::create_raw(3, 3*2, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20000_find_objects_with_ignore_mask_inner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let ignore_mask = Image::zero(input.width(), input.height());

        // Act
        let mask_vec: Vec<ConnectedComponentItem> = ConnectedComponent::find_objects_with_ignore_mask_inner(PixelConnectivity::Connectivity8, &input, &ignore_mask).expect("vec");

        // Assert
        let mut expected = Vec::<ConnectedComponentItem>::new();
        {
            let pixels: Vec<u8> = vec![
                1, 0, 0,
                0, 1, 0,
                0, 0, 1,
            ];
            let mask: Image = Image::try_create(3, 3, pixels).expect("image");
            let item = ConnectedComponentItem {
                mask,
                mass: 3,
                x: 0,
                y: 0,
            };
            expected.push(item);
        }
        {
            let pixels: Vec<u8> = vec![
                0, 1, 1,
                1, 0, 1,
                1, 1, 0,
            ];
            let mask: Image = Image::try_create(3, 3, pixels).expect("image");
            let item = ConnectedComponentItem {
                mask,
                mass: 6,
                x: 1,
                y: 0,
            };
            expected.push(item);
        }
        assert_eq!(mask_vec, expected);
    }

    #[test]
    fn test_20001_find_objects_with_ignore_mask_inner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5,
            5, 5, 9, 9,
            9, 5, 5, 5,
            9, 9, 5, 5,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let ignore_pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            1, 1, 0, 0,
            0, 1, 1, 1,
            0, 0, 1, 1,
        ];
        let ignore_mask: Image = Image::try_create(4, 4, ignore_pixels).expect("image");

        // Act
        let mask_vec: Vec<ConnectedComponentItem> = ConnectedComponent::find_objects_with_ignore_mask_inner(PixelConnectivity::Connectivity8, &input, &ignore_mask).expect("vec");

        // Assert
        let mut expected = Vec::<ConnectedComponentItem>::new();
        {
            let pixels: Vec<u8> = vec![
                0, 0, 0, 0,
                0, 0, 1, 1,
                0, 0, 0, 0,
                0, 0, 0, 0,
            ];
            let mask: Image = Image::try_create(4, 4, pixels).expect("image");
            let item = ConnectedComponentItem {
                mask,
                mass: 2,
                x: 2,
                y: 1,
            };
            expected.push(item);
        }
        {
            let pixels: Vec<u8> = vec![
                0, 0, 0, 0,
                0, 0, 0, 0,
                1, 0, 0, 0,
                1, 1, 0, 0,
            ];
            let mask: Image = Image::try_create(4, 4, pixels).expect("image");
            let item = ConnectedComponentItem {
                mask,
                mass: 3,
                x: 0,
                y: 2,
            };
            expected.push(item);
        }
        assert_eq!(mask_vec, expected);
    }

    #[test]
    fn test_30000_find_objects_all() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = ConnectedComponent::find_objects(PixelConnectivity::Connectivity8, &input).expect("vec");

        // Assert
        assert_eq!(mask_vec.len(), 2);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 1, 0,
            0, 0, 1,

            0, 1, 1,
            1, 0, 1,
            1, 1, 0,
        ];
        let expected = Image::create_raw(3, 3*2, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_40000_find_objects_with_ignore_mask() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let mask_pixels: Vec<u8> = vec![
            1, 1, 0,
            1, 1, 0,
            0, 0, 0,
        ];
        let ignore_mask: Image = Image::try_create(3, 3, mask_pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &input, &ignore_mask).expect("vec");

        // Assert
        assert_eq!(mask_vec.len(), 2);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1,
            0, 0, 1,
            1, 1, 0,

            0, 0, 0,
            0, 0, 0,
            0, 0, 1,
        ];
        let expected = Image::create_raw(3, 3*2, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_40001_find_objects_with_ignore_mask() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 6, 6, 
            5, 5, 6, 6,
            5, 5, 6, 6,
            5, 5, 6, 6,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");
        let mask_pixels: Vec<u8> = vec![
            0, 0, 0, 0,
            0, 1, 1, 0,
            0, 1, 1, 0,
            0, 0, 0, 0,
        ];
        let ignore_mask: Image = Image::try_create(4, 4, mask_pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &input, &ignore_mask).expect("vec");

        // Assert
        assert_eq!(mask_vec.len(), 2);
        let output: Image = Image::vstack(mask_vec).expect("image");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 0, 0, 
            1, 0, 0, 0,
            1, 0, 0, 0,
            1, 1, 0, 0,

            0, 0, 1, 1, 
            0, 0, 0, 1,
            0, 0, 0, 1,
            0, 0, 1, 1, 
        ];
        let expected = Image::create_raw(4, 4*2, expected_pixels);
        assert_eq!(output, expected);
    }
}
