use crate::arc::ImageRotate90;

use super::{Image, convolution2x2, HtmlLog, ImageMaskCount};

pub struct LargestInteriorRectangle;

impl LargestInteriorRectangle {
    fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut current_layer: Image = image.clone();
        loop {
            let layer: Image = convolution2x2(&current_layer, conv2x2_is_full)?;
            let count: u16 = layer.mask_count_nonzero();
            if count == 0 {
                break;
            }
            HtmlLog::image(&layer);

            let slices_x: LongestHorizontalSlices = LongestHorizontalSlices::analyze(&layer)?;
            println!("slices_x: {:?}", slices_x);

            let layer90: Image = layer.rotate_cw()?;
            let slices_y: LongestHorizontalSlices = LongestHorizontalSlices::analyze(&layer90)?;
            println!("slices_y: {:?}", slices_y);

            // if slices_x.mass == slices_y.mass {

            // }

            current_layer = layer;
        }
        Ok(Self)
    }
}

#[derive(Clone, Debug)]
struct LongestHorizontalSlices {
    positions: Vec<(u8, u8)>,
    mass: u8,
}

impl LongestHorizontalSlices {
    fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut candidates: Vec<(u8, u8)> = vec!();
        let mut biggest_mass: u8 = 0;
        for y in 0..image.height() {
            let mut mass: u8 = 0;
            let mut start_x: u8 = 0;
            for x in 0..image.width() {
                let pixel: u8 = image.get(x as i32, y as i32).unwrap_or(0);
                if pixel > 0 {
                    if mass == 0 {
                        start_x = x;
                    }
                    mass += 1;
                } else {
                    if mass > 0 {
                        if mass > biggest_mass {
                            biggest_mass = mass;
                            candidates.clear();
                        }
                        if mass == biggest_mass {
                            candidates.push((start_x, y));
                        }
                    }
                    mass = 0;
                }
            }
            if mass > 0 {
                if mass > biggest_mass {
                    biggest_mass = mass;
                    candidates.clear();
                }
                if mass == biggest_mass {
                    candidates.push((start_x, y));
                }
            }
        }
        let instance = Self {
            positions: candidates,
            mass: biggest_mass,
        };
        Ok(instance)
    }
}

fn conv2x2_is_full(image: &Image) -> anyhow::Result<u8> {
    for pixel in image.pixels() {
        if *pixel == 0 {
            return Ok(0);
        }
    }
    Ok(1)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_analyze() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 0, 1,
            1, 1, 0, 1, 1, 0,
            1, 1, 1, 1, 1, 1,
            0, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 0, 
            1, 1, 0, 1, 1, 1,
        ];
        let input: Image = Image::try_create(6, 6, input_pixels).expect("image");

        // Act
        let actual: LargestInteriorRectangle = LargestInteriorRectangle::analyze(&input).expect("ok");

        // Assert
        // let expected_pixels: Vec<u8> = vec![
        //     1, 2, 3, 4, 5,
        //     6, 0, 0, 0, 10,
        //     11, 12, 13, 14, 15,
        // ];
        // let expected: Image = Image::try_create(5, 3, expected_pixels).expect("image");
        // assert_eq!(actual, expected);
    }
}
