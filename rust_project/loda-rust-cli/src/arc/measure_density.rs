use super::{Image, ImagePadding, convolution3x3, ImageHistogram, Histogram};
use anyhow::bail;

pub struct MeasureDensity {
    pub density_any_direction: Image,
}

impl MeasureDensity {
    pub fn analyze(image: &Image) -> anyhow::Result<Self> {
        if image.is_empty() {
            bail!("image is empty. Must be 1x1 or larger");
        }
        let padding: Image = image.padding_with_color(1, 255)?;
        let density: Image = convolution3x3(&padding, conv3x3_measure_density_any_direction)?;
        let instance = Self {
            density_any_direction: density,
        };
        Ok(instance)
    }
}

fn conv3x3_measure_density_any_direction(image: &Image) -> anyhow::Result<u8> {
    let histogram: Histogram = image.histogram_all();
    let center: u8 = image.get(1, 1).unwrap_or(255);
    let count: u32 = histogram.get(center).min(9).max(1) - 1;
    Ok(count as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_density_any_direction() {
         // Arrange
         let pixels: Vec<u8> = vec![
            5, 3, 3, 3,
            3, 3, 3, 3,
            5, 5, 3, 3,
            3, 5, 3, 3,
        ];
        let input: Image = Image::try_create(4, 4, pixels).expect("image");

        // Act
        let actual: MeasureDensity = MeasureDensity::analyze(&input).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 4, 5, 3,
            2, 5, 7, 5,
            2, 2, 6, 5,
            0, 2, 3, 3,
        ];
        let expected: Image = Image::try_create(4, 4, expected_pixels).expect("image");
        assert_eq!(actual.density_any_direction, expected);
    }
}