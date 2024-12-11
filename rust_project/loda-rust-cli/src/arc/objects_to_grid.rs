use crate::arc::{Image, Histogram, ImageHistogram, ImageMask};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ObjectsToGridMode {
    MostPopularColor,
    LeastPopularColor,
}

#[allow(dead_code)]
pub struct ObjectsToGrid;

impl ObjectsToGrid {
    /// Layout objects in a grid. Transforms each object into a single pixel.
    /// 
    /// The `image` and the `enumerated_objects` must have same size. And the size must be 1x1 or bigger.
    /// 
    /// The number of objects must match with `grid_width * grid_height`.
    /// 
    /// Returns an image with the size: `width=grid_width` and `height=grid_height`.
    #[allow(dead_code)]
    pub fn run(image: &Image, enumerated_objects: &Image, grid_width: u8, grid_height: u8, mode: ObjectsToGridMode) -> anyhow::Result<Image> {
        if image.size() != enumerated_objects.size() {
            return Err(anyhow::anyhow!("ObjectsMeasureMass: images must have same size"));
        }
        if image.is_empty() {
            return Err(anyhow::anyhow!("ObjectsMeasureMass: image must be 1x1 or bigger"));
        }
        if grid_width < 1 || grid_height < 1 {
            return Err(anyhow::anyhow!("Too small grid. Must be 1x1 or bigger"));
        }

        let mut histogram_all: Histogram = enumerated_objects.histogram_all();
        // Ignore `object id=0` which is the background
        histogram_all.set_counter_to_zero(0);

        let object_count: u16 = histogram_all.number_of_counters_greater_than_zero();
        let expected_count: u16 = (grid_width as u16) * (grid_height as u16);
        if object_count != expected_count {
            return Err(anyhow::anyhow!("One object for one cell. Expected {} objects, but got {} objects", expected_count, object_count));
        }

        let mut result_image: Image = Image::zero(grid_width, grid_height);

        let mut grid_index: u16 = 0;
        for object_index in 1..=255u8 {
            let count: u32 = histogram_all.get(object_index);
            if count == 0 {
                continue;
            }
            let current_grid_index: u16 = grid_index;
            grid_index += 1;

            let mask: Image = enumerated_objects.to_mask_where_color_is(object_index);
            let histogram: Histogram = image.histogram_with_mask(&mask)?;
            // println!("object index: {} mask: {:?} histogram: {:?}", object_index, mask, histogram);

            // Transform the object into a single pixel
            let set_color: u8;
            match mode {
                ObjectsToGridMode::MostPopularColor => {
                    set_color = match histogram.most_popular_color_disallow_ambiguous() {
                        Some(value) => value,
                        None => {
                            return Err(anyhow::anyhow!("Cannot decide what color is the most popular. ambiguous"));
                        }
                    };  
                },
                ObjectsToGridMode::LeastPopularColor => {
                    set_color = match histogram.least_popular_color_disallow_ambiguous() {
                        Some(value) => value,
                        None => {
                            return Err(anyhow::anyhow!("Cannot decide what color is the least popular. ambiguous"));
                        }
                    };  
                }
            }

            let y_u16: u16 = current_grid_index / (grid_width as u16);
            let x_u16: u16 = current_grid_index % (grid_width as u16);
            let x: u8 = x_u16.min(255) as u8;
            let y: u8 = y_u16.min(255) as u8;
            _ = result_image.set(x as i32, y as i32, set_color);
        }

        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_most_popular_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 5, 5, 5,
            7, 7, 5, 5, 5,
            5, 5, 7, 7, 7,
            5, 5, 7, 7, 7,
            4, 5, 7, 7, 8,
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
        let actual: Image = ObjectsToGrid::run(
            &input, 
            &enumerated_objects, 
            2, 
            1,
            ObjectsToGridMode::MostPopularColor
        ).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 5,
        ];
        let expected: Image = Image::try_create(2, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_least_popular_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 5, 5, 5,
            7, 7, 5, 5, 5,
            5, 5, 7, 7, 7,
            5, 5, 7, 7, 7,
            4, 5, 7, 7, 8,
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
        let actual: Image = ObjectsToGrid::run(
            &input, 
            &enumerated_objects, 
            2, 
            1,
            ObjectsToGridMode::LeastPopularColor
        ).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            8, 4,
        ];
        let expected: Image = Image::try_create(2, 1, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
