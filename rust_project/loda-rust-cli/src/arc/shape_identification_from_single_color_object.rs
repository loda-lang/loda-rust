use super::{ShapeIdentification, SingleColorObject, PixelConnectivity, Image, ImageHistogram, Histogram, ImageMask};

pub struct ColorAndShape {
    pub color: u8,
    pub shape_identification: ShapeIdentification,
}

pub struct ShapeIdentificationFromSingleColorObject {
    pub color_and_shape_vec: Vec<ColorAndShape>,
}

impl ShapeIdentificationFromSingleColorObject {
    pub fn find_shapes(sco: &SingleColorObject, connectivity: PixelConnectivity) -> anyhow::Result<Self> {
        let mut color_and_shape_vec = Vec::<ColorAndShape>::new();
        for color in 0..=9 {
            let enumerated_objects: Image = match sco.enumerate_clusters(color, connectivity) {
                Ok(value) => value,
                Err(_error) => {
                    // println!("error: {:?}", error);
                    continue;
                }
            };
            let histogram: Histogram = enumerated_objects.histogram_all();
            for (count, object_id) in histogram.pairs_ordered_by_color() {
                if count == 0 || object_id == 0 {
                    continue;
                }
                let mask: Image = enumerated_objects.to_mask_where_color_is(object_id);
                let shape_id: ShapeIdentification = match ShapeIdentification::compute(&mask) {
                    Ok(value) => value,
                    Err(_error) => {
                        // println!("unable to find shape. error: {:?}", error);
                        continue;
                    }
                };
                let color_and_shape = ColorAndShape {
                    color,
                    shape_identification: shape_id,
                };
                color_and_shape_vec.push(color_and_shape);
            }
        }
        let instance = Self {
            color_and_shape_vec,
        };
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, ShapeType};

    #[test]
    fn test_10000_find_shapes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 5, 5,
            0, 0, 0, 5, 5,
            0, 7, 0, 0, 5,
            7, 7, 7, 0, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");
        let sco = SingleColorObject::find_objects(&input).expect("SingleColorObject");

        // Act
        let instance = ShapeIdentificationFromSingleColorObject::find_shapes(&sco, PixelConnectivity::Connectivity4).expect("ok");

        // Assert
        {
            let record: &ColorAndShape = instance.color_and_shape_vec.iter().find(|record| record.color == 5 ).expect("some");
            assert_eq!(record.shape_identification.shape_type, ShapeType::L);
        }
        {
            let record: &ColorAndShape = instance.color_and_shape_vec.iter().find(|record| record.color == 7 ).expect("some");
            assert_eq!(record.shape_identification.shape_type, ShapeType::UpTack);
        }
    }
}
