use super::{ImageSize, Image, ImageMask};
use rand::Rng;
use rand::rngs::StdRng;
use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;

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
        let value: usize = rng.gen_range(0..count);
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

    /// Random pixel values between `min_color_value..=max_color_value`, example `0..=255`, or `0..=9`.
    /// 
    /// Returns an error if the `min_color_value` is equal or greater than the `max_color_value`.
    /// 
    /// Returns an error if the size is empty.
    #[allow(dead_code)]
    pub fn uniform_colors(rng: &mut StdRng, size: ImageSize, min_color_value: u8, max_color_value: u8) -> anyhow::Result<Image> {
        if min_color_value >= max_color_value {
            return Err(anyhow::anyhow!("The minimum color value must be less than the maximum color value. min_color_value={} max_color_value={}", min_color_value, max_color_value));
        }
        if size.is_empty() {
            return Err(anyhow::anyhow!("size is empty. Must be 1x1 or bigger."));
        }
        let range: Uniform<u8> = Uniform::from(min_color_value..=max_color_value);
        let mut image: Image = Image::zero(size.width, size.height);
        for y in 0..size.height {
            for x in 0..size.width {
                let value: u8 = range.sample(rng);
                _ = image.set(x as i32, y as i32, value);
            }
        }
        Ok(image)
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
        let expected = Image::try_create(3, 2, vec![5, 5, 5, 5, 9, 5]).expect("ok");
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

    #[test]
    fn test_40000_uniform_colors() {
        let actual: Image = RandomImage::uniform_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(3, 2), 0, 3).expect("ok");
        let expected = Image::try_create(3, 2, vec![3, 2, 2, 3, 3, 0]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40001_uniform_colors() {
        let actual: Image = RandomImage::uniform_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(3, 3), 0, 4).expect("ok");
        let expected = Image::try_create(3, 3, vec![4, 3, 2, 3, 4, 0, 3, 2, 4]).expect("ok");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40002_uniform_colors() {
        let actual: Image = RandomImage::uniform_colors(&mut StdRng::seed_from_u64(0), ImageSize::new(2, 4), 4, 6).expect("ok");
        let expected = Image::try_create(2, 4, vec![6, 6, 5, 6, 6, 4, 6, 5]).expect("ok");
        assert_eq!(actual, expected);
    }
}
