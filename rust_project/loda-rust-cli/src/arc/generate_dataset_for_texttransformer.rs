use super::{ImageSize, Image};
use rand::rngs::StdRng;
use rand::distributions::{Distribution, Uniform};

// Ideas
// Noise pixel within a range (0..=N)
// Two colors with a noise temperature
// Three colors with a noise temperature
struct RandomImage;

impl RandomImage {
    fn image_with_one_pixel(rng: &mut StdRng, size: ImageSize, background: u8, foreground: u8) -> anyhow::Result<Image> {
        if size.is_empty() {
            return Err(anyhow::anyhow!("size is empty"));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_10000_image_with_one_pixel() {
        let actual: Image = RandomImage::image_with_one_pixel(&mut StdRng::seed_from_u64(1), ImageSize::new(1, 1), 0, 5).expect("ok");
        let expected = Image::create_raw(1, 1, vec![5]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_image_with_one_pixel() {
        let actual: Image = RandomImage::image_with_one_pixel(&mut StdRng::seed_from_u64(0), ImageSize::new(3, 2), 0, 1).expect("ok");
        let expected = Image::create_raw(3, 2, vec![0, 0, 0, 0, 1, 0]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_image_with_one_pixel() {
        let actual: Image = RandomImage::image_with_one_pixel(&mut StdRng::seed_from_u64(1), ImageSize::new(3, 2), 5, 9).expect("ok");
        let expected = Image::create_raw(3, 2, vec![5, 5, 5, 5, 5, 9]);
        assert_eq!(actual, expected);
    }
}
