use super::{ShapeIdentification, SingleColorObject, PixelConnectivity, Image, ImageHistogram, Histogram, ImageMask, ImageSize};

pub struct ColorAndShape {
    pub color: u8,
    pub shape_identification: ShapeIdentification,
    pub position_x: u8,
    pub position_y: u8,
    pub position_x_reverse: Option<u8>,
    pub position_y_reverse: Option<u8>,
}

pub struct ShapeIdentificationFromSingleColorObject {
    pub color_and_shape_vec: Vec<ColorAndShape>,
}

impl ShapeIdentificationFromSingleColorObject {
    pub fn find_shapes(sco: &SingleColorObject, connectivity: PixelConnectivity) -> anyhow::Result<Self> {
        let image_size: ImageSize = sco.image_size;
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

                let position_x: u8 = shape_id.rect.x();
                let position_y: u8 = shape_id.rect.y();

                let x_reverse_i32: i32 = (image_size.width as i32) - 1 - shape_id.rect.max_x();
                let position_x_reverse: Option<u8> = if x_reverse_i32 >= 0 { Some(x_reverse_i32 as u8) } else { None };

                let y_reverse_i32: i32 = (image_size.height as i32) - 1 - shape_id.rect.max_y();
                let position_y_reverse: Option<u8> = if y_reverse_i32 >= 0 { Some(y_reverse_i32 as u8) } else { None };
    
                let color_and_shape = ColorAndShape {
                    color,
                    shape_identification: shape_id,
                    position_x,
                    position_y,
                    position_x_reverse,
                    position_y_reverse,
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
            assert_eq!(record.position_x, 3);
            assert_eq!(record.position_y, 0);
            assert_eq!(record.position_x_reverse, Some(0));
            assert_eq!(record.position_y_reverse, Some(0));
        }
        {
            let record: &ColorAndShape = instance.color_and_shape_vec.iter().find(|record| record.color == 7 ).expect("some");
            assert_eq!(record.shape_identification.shape_type, ShapeType::UpTack);
            assert_eq!(record.position_x, 0);
            assert_eq!(record.position_y, 2);
            assert_eq!(record.position_x_reverse, Some(2));
            assert_eq!(record.position_y_reverse, Some(0));
        }
    }
}
