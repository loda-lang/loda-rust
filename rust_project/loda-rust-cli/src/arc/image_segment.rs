use super::Image;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageSegmentItem {
    mask: Image,
    mass: u16,
    x: u8,
    y: u8,
}

impl ImageSegmentItem {
    #[allow(dead_code)]
    pub fn mask(&self) -> &Image {
        &self.mask
    }

    #[allow(dead_code)]
    pub fn mass(&self) -> u16 {
        self.mass
    }

    #[allow(dead_code)]
    pub fn x(&self) -> u8 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> u8 {
        self.y
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ImageSegmentAlgorithm {
    /// Visit 4 neighbors around a pixel.
    /// 
    /// Flood fill the top/bottom/left/right pixels.
    /// 
    /// Don't visit the corners.
    Neighbors,

    /// Visit all 8 neighbors around a pixel.
    /// 
    /// Flood fill diagonally via corners.
    All,
}

pub trait ImageSegment {
    /// Replace color with another color
    fn flood_fill_neighbors(&mut self, x: i32, y: i32, from_color: u8, to_color: u8);

    /// Build a mask of connected pixels that has the same color
    fn flood_fill_visit_neighbors(&mut self, image: &Image, x: i32, y: i32, color: u8);

    /// Build a mask of connected pixels that has the same color
    fn flood_fill_visit_all(&mut self, image: &Image, x: i32, y: i32, color: u8);

    /// Identify clusters of connected pixels with an `ignore_mask` of areas to be ignored
    /// 
    /// Each object is a mask, where it's 1 the object is present, where it's 0 there is no object.
    /// 
    /// Counts the number of pixels in each of the objects, so that this costly operation can be avoided.
    fn find_objects_with_ignore_mask_inner(&self, algorithm: ImageSegmentAlgorithm, ignore_mask: &Image) -> anyhow::Result<Vec<ImageSegmentItem>>;

    /// Identify clusters of connected pixels
    /// 
    /// Each object is a mask, where it's 1 the object is present, where it's 0 there is no object.
    fn find_objects(&self, algorithm: ImageSegmentAlgorithm) -> anyhow::Result<Vec<Image>>;
    
    /// Identify clusters of connected pixels with an `ignore_mask` of areas to be ignored
    /// 
    /// Each object is a mask, where it's 1 the object is present, where it's 0 there is no object.
    fn find_objects_with_ignore_mask(&self, algorithm: ImageSegmentAlgorithm, ignore_mask: &Image) -> anyhow::Result<Vec<Image>>;
}

impl ImageSegment for Image {
    fn flood_fill_neighbors(&mut self, x: i32, y: i32, from_color: u8, to_color: u8) {
        if x < 0 || y < 0 || x >= (self.width() as i32) || y >= (self.height() as i32) {
            return;
        }
        let value: u8 = self.get(x, y).unwrap_or(255);
        if value != from_color {
            return;
        }
        let _ = self.set(x, y, to_color);
        self.flood_fill_neighbors(x-1, y, from_color, to_color);
        self.flood_fill_neighbors(x+1, y, from_color, to_color);
        self.flood_fill_neighbors(x, y-1, from_color, to_color);
        self.flood_fill_neighbors(x, y+1, from_color, to_color);
    }

    fn flood_fill_visit_neighbors(&mut self, image: &Image, x: i32, y: i32, color: u8) {
        assert!(self.width() == image.width());
        assert!(self.height() == image.height());
        if x < 0 || y < 0 || x >= (self.width() as i32) || y >= (self.height() as i32) {
            return;
        }
        let mask_value: u8 = self.get(x, y).unwrap_or(255);
        if mask_value > 0 {
            // already visited
            return;
        }
        let value: u8 = image.get(x, y).unwrap_or(255);
        if value != color {
            return;
        }
        let _ = self.set(x, y, 1); // flag as visited
        self.flood_fill_visit_neighbors(image, x-1, y, color);
        self.flood_fill_visit_neighbors(image, x+1, y, color);
        self.flood_fill_visit_neighbors(image, x, y-1, color);
        self.flood_fill_visit_neighbors(image, x, y+1, color);
    }

    fn flood_fill_visit_all(&mut self, image: &Image, x: i32, y: i32, color: u8) {
        assert!(self.width() == image.width());
        assert!(self.height() == image.height());
        if x < 0 || y < 0 || x >= (self.width() as i32) || y >= (self.height() as i32) {
            return;
        }
        let mask_value: u8 = self.get(x, y).unwrap_or(255);
        if mask_value > 0 {
            // already visited
            return;
        }
        let value: u8 = image.get(x, y).unwrap_or(255);
        if value != color {
            return;
        }
        let _ = self.set(x, y, 1); // flag as visited
        self.flood_fill_visit_all(image, x-1, y-1, color);
        self.flood_fill_visit_all(image, x, y-1, color);
        self.flood_fill_visit_all(image, x+1, y-1, color);
        self.flood_fill_visit_all(image, x-1, y, color);
        self.flood_fill_visit_all(image, x+1, y, color);
        self.flood_fill_visit_all(image, x-1, y+1, color);
        self.flood_fill_visit_all(image, x, y+1, color);
        self.flood_fill_visit_all(image, x+1, y+1, color);
    }

    fn find_objects_with_ignore_mask_inner(&self, algorithm: ImageSegmentAlgorithm, ignore_mask: &Image) -> anyhow::Result<Vec<ImageSegmentItem>> {
        if ignore_mask.size() != self.size() {
            return Err(anyhow::anyhow!("The size of the ignore_mask must be the same, but is different"));
        }
        let mut object_mask_vec = Vec::<ImageSegmentItem>::new();
        let mut accumulated_mask: Image = ignore_mask.clone();
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                // Only visit pixels that have not yet been visited
                let mask_value: u8 = accumulated_mask.get(x, y).unwrap_or(255);
                if mask_value > 0 {
                    // This pixel has already been visited, ignore it
                    continue;
                }

                // Flood fill
                let color: u8 = self.get(x, y).unwrap_or(255);
                let mut object_mask = ignore_mask.clone();
                match algorithm {
                    ImageSegmentAlgorithm::Neighbors => {
                        object_mask.flood_fill_visit_neighbors(&self, x, y, color);
                    },
                    ImageSegmentAlgorithm::All => {
                        object_mask.flood_fill_visit_all(&self, x, y, color);
                    },
                }

                // Clear pixels that are in the original ignore_mask
                for yy in 0..(self.height() as i32) {
                    for xx in 0..(self.width() as i32) {
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
                for yy in 0..self.height() {
                    for xx in 0..self.width() {
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

                let item = ImageSegmentItem {
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

    fn find_objects(&self, algorithm: ImageSegmentAlgorithm) -> anyhow::Result<Vec<Image>> {
        let ignore_mask = Image::zero(self.width(), self.height());
        self.find_objects_with_ignore_mask(algorithm, &ignore_mask)
    }

    fn find_objects_with_ignore_mask(&self, algorithm: ImageSegmentAlgorithm, ignore_mask: &Image) -> anyhow::Result<Vec<Image>> {
        let items: Vec<ImageSegmentItem> = self.find_objects_with_ignore_mask_inner(algorithm, ignore_mask)?;
        let images: Vec<Image> = items.into_iter().map(|item| item.mask ).collect();
        Ok(images)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use crate::arc::ImageStack;

    #[test]
    fn test_10000_flood_fill_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill_neighbors(0, 0, 5, 3);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3,
            3, 8, 8, 3, 8,
            3, 8, 3, 3, 8,
            3, 3, 3, 3, 8,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10001_flood_fill_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill_neighbors(1, 1, 8, 1);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 1, 1, 5, 8,
            5, 1, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10002_flood_fill_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mut output: Image = input.clone();
        output.flood_fill_neighbors(4, 1, 8, 1);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 1,
            5, 8, 5, 5, 1,
            5, 5, 5, 5, 1,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20000_flood_fill_visit_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");
        let mut output = Image::zero(5, 4);
        let color: u8 = input.get(0, 0).unwrap_or(255);

        // Act
        output.flood_fill_visit_neighbors(&input, 0, 0, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 0, 1, 0,
            1, 0, 1, 1, 0,
            1, 1, 1, 1, 0,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20001_flood_fill_visit_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");
        let mut output = Image::zero(5, 4);
        let color: u8 = input.get(1, 1).unwrap_or(255);

        // Act
        output.flood_fill_visit_neighbors(&input, 1, 1, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20002_flood_fill_visit_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");
        let mut output = Image::zero(5, 4);
        let color: u8 = input.get(4, 1).unwrap_or(255);

        // Act
        output.flood_fill_visit_neighbors(&input, 4, 1, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_20003_flood_fill_visit_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let mut output = Image::zero(3, 3);
        let color: u8 = input.get(2, 0).unwrap_or(255);

        // Act
        output.flood_fill_visit_neighbors(&input, 2, 0, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 
            0, 0, 1, 
            0, 0, 0,
        ];
        let expected = Image::create_raw(3, 3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_30000_flood_fill_visit_all() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let mut output = Image::zero(3, 3);
        let color: u8 = input.get(2, 0).unwrap_or(255);

        // Act
        output.flood_fill_visit_all(&input, 2, 0, color);

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 1, 
            1, 0, 1, 
            1, 1, 0,
        ];
        let expected = Image::create_raw(3, 3, expected_pixels);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_40000_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 8, 8, 5, 8,
            5, 8, 5, 5, 8,
            5, 5, 5, 5, 8,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::Neighbors).expect("vec");

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
    fn test_40001_find_objects_neighbors() {
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
        let mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::Neighbors).expect("vec");

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
    fn test_40002_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::Neighbors).expect("vec");

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
    fn test_40003_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 
            0, 1, 0, 
            0, 0, 0, 
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::Neighbors).expect("vec");

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
    fn test_40004_find_objects_neighbors() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::Neighbors).expect("vec");

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
    fn test_50000_find_objects_with_ignore_mask_inner() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");
        let ignore_mask = Image::zero(input.width(), input.height());

        // Act
        let mask_vec: Vec<ImageSegmentItem> = input.find_objects_with_ignore_mask_inner(ImageSegmentAlgorithm::All, &ignore_mask).expect("vec");

        // Assert
        let mut expected = Vec::<ImageSegmentItem>::new();
        {
            let pixels: Vec<u8> = vec![
                1, 0, 0,
                0, 1, 0,
                0, 0, 1,
            ];
            let mask: Image = Image::try_create(3, 3, pixels).expect("image");
            let item = ImageSegmentItem {
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
            let item = ImageSegmentItem {
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
    fn test_50001_find_objects_with_ignore_mask_inner() {
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
        let mask_vec: Vec<ImageSegmentItem> = input.find_objects_with_ignore_mask_inner(ImageSegmentAlgorithm::All, &ignore_mask).expect("vec");

        // Assert
        let mut expected = Vec::<ImageSegmentItem>::new();
        {
            let pixels: Vec<u8> = vec![
                0, 0, 0, 0,
                0, 0, 1, 1,
                0, 0, 0, 0,
                0, 0, 0, 0,
            ];
            let mask: Image = Image::try_create(4, 4, pixels).expect("image");
            let item = ImageSegmentItem {
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
            let item = ImageSegmentItem {
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
    fn test_60000_find_objects_all() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 5, 5, 
            5, 9, 5, 
            5, 5, 9,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::All).expect("vec");

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
    fn test_70000_find_objects_with_ignore_mask() {
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
        let mask_vec: Vec<Image> = input.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &ignore_mask).expect("vec");

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
    fn test_70001_find_objects_with_ignore_mask() {
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
        let mask_vec: Vec<Image> = input.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &ignore_mask).expect("vec");

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
