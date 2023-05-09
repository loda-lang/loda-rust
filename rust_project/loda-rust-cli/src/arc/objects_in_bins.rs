use crate::arc::{Image, Histogram, ImageHistogram, ImageMask, ImageMaskCount};

#[allow(dead_code)]
pub struct ObjectsInBins;

impl ObjectsInBins {
    /// Assign new N object ids to the existing objects.
    /// 
    /// Usecase: Group the objects into 3 bins based on mass.
    /// - The smallest objects with same mass gets assigned `id=1`.
    /// - The in between objects gets assigned `id=2`.
    /// - The biggest objects with same mass gets assigned `id=3`.
    /// 
    /// Usecase: Group the objects into 2 bins based on mass.
    /// - The objects with exactly `mass=6` gets assigned `id=1`.
    /// - All other objects gets assigned `id=2`.
    /// 
    /// The `image` and the `enumerated_objects` must have same size. And the size must be 1x1 or bigger.
    /// 
    /// Returns an image with the same size as the input image.
    /// The pixel value is for non-objects.
    #[allow(dead_code)]
    pub fn run(image: &Image, enumerated_objects: &Image, ignore_colors: Option<&Histogram>) -> anyhow::Result<Image> {
        if image.size() != enumerated_objects.size() {
            return Err(anyhow::anyhow!("ObjectsInBins: images must have same size"));
        }
        if image.is_empty() {
            return Err(anyhow::anyhow!("ObjectsInBins: image must be 1x1 or bigger"));
        }

        let mut items = Vec::<Item>::new();

        for color in 0..=255u8 {
            if let Some(other) = ignore_colors {
                if other.get(color) > 0 {
                    println!("ignoreing color: {}", color);
                    continue;
                }
            }
            let mask: Image = enumerated_objects.to_mask_where_color_is(color);
            let mass_of_object: u16 = mask.mask_count_one();
            if mass_of_object == 0 {
                continue;
            }
            let item = Item {
                object_id: color,
                mask,
                object_mass: mass_of_object,
            };
            items.push(item);
        }
        if items.is_empty() {
            return Err(anyhow::anyhow!("ObjectsInBins: found zero objects. There must be 1 or more objects"));
        }

        let mut smallest_mass: u16 = u16::MAX;
        let mut biggest_mass: u16 = 0;
        for item in &items {
            smallest_mass = smallest_mass.min(item.object_mass);
            biggest_mass = biggest_mass.max(item.object_mass);
        }
        println!("smallest_mass: {}", smallest_mass);
        println!("biggest_mass: {}", biggest_mass);
        if smallest_mass == biggest_mass {
            return Err(anyhow::anyhow!("ObjectsInBins: it's ambiguous in what bin to place the objects. biggest and smallest is the same mass"));
        }

        let mut result_image = Image::zero(image.width(), image.height());
        for item in &items {
            let mut set_color: u8 = 2;
            if item.object_mass == biggest_mass {
                set_color = 1;
            }
            if item.object_mass == smallest_mass {
                set_color = 3;
            }
            for y in 0..image.height() {
                for x in 0..image.width() {
                    let mask_value: u8 = item.mask.get(x as i32, y as i32).unwrap_or(0);
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

struct Item {
    object_id: u8,
    mask: Image,
    object_mass: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_three_groups_by_mass() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5,
            1, 2, 3, 4, 5,
            1, 2, 3, 4, 5,
            1, 2, 3, 4, 5,
            1, 2, 3, 4, 5,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let enumerated_object_pixels: Vec<u8> = vec![
            1, 0, 2, 0, 3,
            0, 0, 0, 0, 0,
            4, 4, 0, 5, 5,
            4, 0, 6, 0, 5,
            6, 6, 6, 6, 6, 
        ];
        let enumerated_objects: Image = Image::try_create(5, 5, enumerated_object_pixels).expect("image");
        let mut ignore_colors = Histogram::new();
        ignore_colors.increment(0);

        // Act
        let actual: Image = ObjectsInBins::run(&input, &enumerated_objects, Some(&ignore_colors)).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 0, 3, 0, 3,
            0, 0, 0, 0, 0,
            2, 2, 0, 2, 2,
            2, 0, 1, 0, 2,
            1, 1, 1, 1, 1, 
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
