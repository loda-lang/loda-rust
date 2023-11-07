use super::{CellularAutomaton, cellular_automaton::rule};
use super::{Image, ImageSize, RandomImage};
use super::HtmlLog;
use bloomfilter::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::seq::SliceRandom;

struct GenerateDataset;

impl GenerateDataset {
    fn curriculum_easy() -> anyhow::Result<()> {
        let sizes: [u8; 4] = [
            3, 4, 5, 6
        ];
        let temperatures: [u8; 9] = [
            10, 20, 30, 40, 50, 60, 70, 80, 90
        ];

        let bloom_items_count = 1000;
        let false_positive_rate = 0.01;
        let mut bloom = Bloom::<Image>::new_for_fp_rate(bloom_items_count, false_positive_rate);

        for i in 0..100 {
            let mut rng = StdRng::seed_from_u64(i);
            let width: u8 = *sizes.choose(&mut rng).unwrap();
            let height: u8 = *sizes.choose(&mut rng).unwrap();
            let temperature: u8 = *temperatures.choose(&mut rng).unwrap();

            let size = ImageSize::new(width, height);
            let step0: Image = RandomImage::two_colors(&mut rng, size, 0, 1, temperature)?;

            if bloom.check(&step0) {
                debug!("skipping duplicate");
                continue;
            }
            bloom.set(&step0);

            let mut ca_nowrap: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&step0, Some(0));
            let images_nowrap: Vec<Image> = ca_nowrap.images_for_n_steps(5);

            let mut ca_wrap: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&step0, None);
            let images_wrap: Vec<Image> = ca_wrap.images_for_n_steps(5);

            if images_wrap == images_nowrap {
                HtmlLog::text("identical for wrap and nowrap");
                HtmlLog::compare_images(images_wrap);
                continue;
            }
            HtmlLog::text("nowrap");
            HtmlLog::compare_images(images_nowrap);
            HtmlLog::text("wrap");
            HtmlLog::compare_images(images_wrap);
        }
        Ok(())
    }

    fn image_to_string(image: &Image) -> String {
        let mut result = String::new();
        for y in 0..image.height() {
            if y > 0 {
                result.push('\n');
            }
            for x in 0..image.width() {
                let value: u8 = image.get(x as i32, y as i32).unwrap_or(0);
                let character: char = match value {
                    0 => '.',
                    1 => '*',
                    _ => '?',
                };
                result.push(character);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_image_to_string() {
        // Act
        let pixels: Vec<u8> = vec![
            1, 0, 0,
            0, 1, 1,
            1, 1, 255,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let actual: String = GenerateDataset::image_to_string(&input);
        
        // Assert
        assert_eq!(actual, "*..\n.**\n**?");
    }

    // #[test]
    fn test_20000_do_something() {
        for i in 0..10u64 {
            let size = ImageSize::new(6, 5);
            let step0: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(i), size, 0, 1, 25).expect("ok");
            let mut images = Vec::<Image>::new();
            images.push(step0.clone());
            let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&step0, None);
            for _ in 0..2 {
                ca.step_once();
                images.push(ca.image().clone());
            }
            HtmlLog::compare_images(images);
        }
    }

    // #[test]
    fn test_20001_do_something() {
        for i in 0..20u64 {
            let size = ImageSize::new(10, 8);
            let step0: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(i), size, 0, 1, 35).expect("ok");
            let step1: Image = RandomImage::draw_dots(&mut StdRng::seed_from_u64(i+5), &step0, 2, 5).expect("ok");
            let step2: Image = RandomImage::draw_dots(&mut StdRng::seed_from_u64(i+8), &step1, 3, 5).expect("ok");
            let mut images = Vec::<Image>::new();
            images.push(step2.clone());
            let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLifeExtra>::with_image(&step0, None);
            for _ in 0..4 {
                ca.step_once();
                images.push(ca.image().clone());
            }
            HtmlLog::compare_images(images);
        }
    }

    // #[test]
    fn test_20002_do_something() {
        GenerateDataset::curriculum_easy().expect("ok");
    }
}
