use super::{ImageSize, Image, ImageMask};
use rand::rngs::StdRng;
use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;

// Ideas
// Noise pixel within a range (0..=N)
// Three colors with a noise temperature
#[allow(dead_code)]
pub struct RandomImage;

impl RandomImage {
    /// Draw just 1 pixel at a random position.
    /// 
    /// The image is filled with `background` color.
    /// 
    /// The pixel is drawn with `foreground` color.
    #[allow(dead_code)]
    pub fn one_dot(rng: &mut StdRng, size: ImageSize, background: u8, foreground: u8) -> anyhow::Result<Image> {
        if size.is_empty() {
            return Err(anyhow::anyhow!("size is empty. Must be 1x1 or bigger."));
        }
        let mut image: Image = Image::color(size.width, size.height, background);
        let count: usize = (size.width as usize) * (size.height as usize);
        let range: Uniform<usize> = Uniform::from(0..count);
        let value: usize = range.sample(rng);
        let x: usize = value % (size.width as usize);
        let y: usize = value / (size.width as usize);
        _ = image.set(x as i32, y as i32, foreground);
        Ok(image)
    }

    /// Noise with two colors and a noise temperature parameter.
    /// 
    /// When the temperature is 0, the image is filled with `color0`.
    /// 
    /// When the temperature is 50, the image is half `color0` and half `color1`.
    /// 
    /// When the temperature is 100, the image is filled with `color1`.
    #[allow(dead_code)]
    pub fn two_colors(rng: &mut StdRng, size: ImageSize, color0: u8, color1: u8, temperature: u8) -> anyhow::Result<Image> {
        if temperature > 100 {
            return Err(anyhow::anyhow!("temperature is greater than 100"));
        }
        if size.is_empty() {
            return Err(anyhow::anyhow!("size is empty. Must be 1x1 or bigger."));
        }
        if temperature == 0 {
            return Ok(Image::color(size.width, size.height, color0));
        }
        if temperature == 100 {
            return Ok(Image::color(size.width, size.height, color1));
        }

        let weight0: usize = (100 - temperature) as usize;
        let weight1: usize = temperature as usize;

        let color_weight_vec: Vec<(u8,usize)> = vec![
            (color0, weight0),
            (color1, weight1),
        ];
        let mut image: Image = Image::zero(size.width, size.height);
        for y in 0..size.height {
            for x in 0..size.width {
                let color: u8 = color_weight_vec.choose_weighted(rng, |item| item.1).unwrap().0;
                _ = image.set(x as i32, y as i32, color)
            }
        }
        Ok(image)
    }

    /// Draw random dots on an existing image.
    /// 
    /// If the temperature is 0, the original image is returned unchanged.
    /// 
    /// If the temperature is 50, then half of the pixels of the original image has been overdrawn with the specified color.
    /// 
    /// If the temperature is 100, then all pixels have been replaced with the specified color, and no colors from the original image is preserved.
    #[allow(dead_code)]
    pub fn draw_dots(rng: &mut StdRng, image: &Image, color: u8, temperature: u8) -> anyhow::Result<Image> {
        if temperature > 100 {
            return Err(anyhow::anyhow!("temperature is greater than 100"));
        }
        if image.is_empty() {
            return Err(anyhow::anyhow!("size is empty. Must be 1x1 or bigger."));
        }
        if temperature == 0 { 
            return Ok(image.clone());
        }
        let mask: Image = Self::two_colors(rng, image.size(), 0, 1, temperature)?;
        let result_image: Image = mask.select_from_image_and_color(&image, color)?;
        Ok(result_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_one_dot() {
        let actual: Image = RandomImage::one_dot(&mut StdRng::seed_from_u64(1), ImageSize::new(1, 1), 0, 5).expect("ok");
        let expected = Image::try_create(1, 1, vec![5]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_one_dot() {
        let actual: Image = RandomImage::one_dot(&mut StdRng::seed_from_u64(0), ImageSize::new(3, 2), 0, 1).expect("ok");
        let expected = Image::try_create(3, 2, vec![0, 0, 0, 0, 1, 0]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_one_dot() {
        let actual: Image = RandomImage::one_dot(&mut StdRng::seed_from_u64(1), ImageSize::new(3, 2), 5, 9).expect("ok");
        let expected = Image::try_create(3, 2, vec![5, 5, 5, 5, 5, 9]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_two_colors() {
        let actual: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(1, 1), 4, 5, 0).expect("ok");
        let expected = Image::try_create(1, 1, vec![4]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_two_colors() {
        let actual: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(1, 1), 4, 5, 100).expect("ok");
        let expected = Image::try_create(1, 1, vec![5]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20002_two_colors() {
        let actual: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(1, 1), 4, 5, 99).expect("ok");
        let expected = Image::try_create(1, 1, vec![5]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20003_two_colors() {
        let actual: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(1, 1), 4, 5, 1).expect("ok");
        let expected = Image::try_create(1, 1, vec![4]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20004_two_colors() {
        let actual: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(3, 2), 4, 5, 50).expect("ok");
        let expected = Image::try_create(3, 2, vec![5, 5, 4, 5, 4, 5]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20005_two_colors() {
        let actual: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(3, 2), 4, 5, 25).expect("ok");
        let expected = Image::try_create(3, 2, vec![4, 5, 4, 4, 4, 5]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20006_two_colors() {
        let actual: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(3, 2), 4, 5, 75).expect("ok");
        let expected = Image::try_create(3, 2, vec![5, 5, 4, 5, 5, 5]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_draw_dots() {
        let input: Image = Image::color(4, 2, 9);
        let actual: Image = RandomImage::draw_dots(&mut StdRng::seed_from_u64(0), &input, 5, 25).expect("ok");
        let expected = Image::try_create(4, 2, vec![9, 5, 9, 9, 9, 5, 9, 5]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30001_draw_dots() {
        let input: Image = Image::color(4, 2, 9);
        let actual: Image = RandomImage::draw_dots(&mut StdRng::seed_from_u64(0), &input, 5, 75).expect("ok");
        let expected = Image::try_create(4, 2, vec![5, 5, 9, 5, 5, 5, 9, 5]).expect("ok");
        assert_eq!(actual, expected);
    }
}
