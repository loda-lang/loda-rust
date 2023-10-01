use super::{Image, RandomImage, ImageSize, ImageHistogram, ImageSort, ImageSortMode, ImageSymmetry, ImageStack, ImageOffset, ImageDenoise, ImageGravity, ImageRepairTrigram};
use super::HtmlLog;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng, Rng};

static MAX_VALID_PIXEL_VALUE: u8 = 35;

struct GenerateDataset;

impl GenerateDataset {
    /// Convert an Image to a text representation.
    /// 
    /// The text representation is a string with the `0-9a-z` format.
    /// 
    /// The values `0..=35` are converted to `0..=9` and `a..=z`.
    /// 
    /// An error is returned when encountering a value greater than `35`.
    fn image_to_text(image: &Image) -> anyhow::Result<String> {
        let max_value: u16 = 'z' as u16;
        let mut rows = Vec::<String>::new();
        for y in 0..image.height() {
            let mut row = String::new();
            for x in 0..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                if color < 10 {
                    // convert from 0-9 to 0-9
                    row.push_str(color.to_string().as_str());
                } else {
                    // convert from 10-35 to a-z
                    let value: u16 = ('a' as u16) + (color - 10) as u16;
                    if value > max_value {
                        return Err(anyhow::anyhow!("Cannot represent value as 0-9a-z representation. The value {} is greater than {}", value, max_value));
                    }
                    let c: char = match std::char::from_u32(value as u32) {
                        Some(value) => value,
                        None => {
                            return Err(anyhow::anyhow!("Cannot represent value as 0-9a-z representation. The value {} cannot be converted to a Char", value));
                        }
                    };
                    row.push(c);
                }
            }
            rows.push(row);
        }
        let mut result = String::new();
        result += "image='";
        result += &rows.join(",");
        result += "'";
        Ok(result)
    }

    /// Generate a noisy image with the specified size.
    fn simple_image(rng: &mut StdRng, size: ImageSize) -> anyhow::Result<Image> {
        let mut permutation: u32 = rng.gen();
        let image0: Image = match permutation % 2 {
            0 => {
                let max_color: u8 = rng.gen_range(3..=MAX_VALID_PIXEL_VALUE);
                RandomImage::uniform_colors(
                    rng,
                    size, 
                    max_color,
                )?
            },
            _ => {
                let color0: u8 = rng.gen_range(0..=9);
                let mut color1: u8 = rng.gen_range(0..=9);
                if color1 == color0 {
                    color1 = (color1 + 1) % 10;
                }                    
                let temperature: u8 = rng.gen_range(25..=75);
                RandomImage::two_colors(
                    rng, 
                    size, 
                    color0,
                    color1,
                    temperature
                )?
            }
        };
        permutation /= 2;

        let most_popular_color: Option<u8> = image0.histogram_all().most_popular_color_disallow_ambiguous();
        let mut image1: Image = image0.clone();
        match permutation % 12 {
            0 => {
                // do nothing, keep input image as it is.
            },
            1 => {
                if let Some(color) = most_popular_color {
                    image1 = image0.sort_by_mass(color, ImageSortMode::RowsAscending)?;
                }
            },
            2 => {
                if let Some(color) = most_popular_color {
                    image1 = image0.sort_by_mass(color, ImageSortMode::RowsDescending)?;
                }
            },
            3 => {
                if let Some(color) = most_popular_color {
                    image1 = image0.sort_by_mass(color, ImageSortMode::ColumnsAscending)?;
                }
            },
            4 => {
                if let Some(color) = most_popular_color {
                    image1 = image0.sort_by_mass(color, ImageSortMode::ColumnsDescending)?;
                }
            },
            5 => {
                image1 = image0.sort_by_pixel_value(ImageSortMode::RowsAscending)?;
            },
            6 => {
                image1 = image0.sort_by_pixel_value(ImageSortMode::RowsDescending)?;
            },
            7 => {
                image1 = image0.sort_by_pixel_value(ImageSortMode::ColumnsAscending)?;
            },
            8 => {
                image1 = image0.sort_by_pixel_value(ImageSortMode::ColumnsDescending)?;
            },
            9 => {
                if let Some(color) = most_popular_color {
                    image1 = image0.gravity(color, crate::arc::GravityDirection::Down)?;
                }
            },
            10 => {
                if let Some(color) = most_popular_color {
                    image1 = image0.gravity(color, crate::arc::GravityDirection::Up)?;
                }
            },
            11 => {
                if let Some(color) = most_popular_color {
                    image1 = image0.gravity(color, crate::arc::GravityDirection::Left)?;
                }
            },
            _ => {
                if let Some(color) = most_popular_color {
                    image1 = image0.gravity(color, crate::arc::GravityDirection::Right)?;
                }
            },
        }
        Ok(image1)
    }

    /// Generate a composition of two simple noisy images, or return a simple noisy image.
    fn medium_image(rng: &mut StdRng, size: ImageSize) -> anyhow::Result<Image> {
        let permutation: u32 = rng.gen();
        match permutation % 3 {
            0 => {
                // do nothing, use the simple image generator
            },
            1 => {
                let factor: u16 = rng.gen_range(1..=9);
                let width0: u8 = ((size.width as u16) * factor / 10).min(255) as u8;
                let width1: u8 = size.width - width0;
                if width0 > 0 && width1 > 0 {
                    let image0: Image = Self::simple_image(rng, ImageSize::new(width0, size.height))?;
                    let image1: Image = Self::simple_image(rng, ImageSize::new(width1, size.height))?;
                    let image: Image = image0.hjoin(image1)?;
                    return Ok(image);
                }
            },
            _ => {
                let factor: u16 = rng.gen_range(1..=9);
                let height0: u8 = ((size.height as u16) * factor / 10).min(255) as u8;
                let height1: u8 = size.height - height0;
                if height0 > 0 && height1 > 0 {
                    let image0: Image = Self::simple_image(rng, ImageSize::new(size.width, height0))?;
                    let image1: Image = Self::simple_image(rng, ImageSize::new(size.width, height1))?;
                    let image: Image = image0.vjoin(image1)?;
                    return Ok(image);
                }
            },
        };
        let image: Image = Self::simple_image(rng, size)?;
        Ok(image)
    }

    /// Postprocessing of the medium noisy image.
    fn advanced_image(rng: &mut StdRng, size: ImageSize) -> anyhow::Result<Image> {
        let image0: Image = Self::medium_image(rng, size)?;
        let mut image1: Image = image0.clone();
        let permutation: u32 = rng.gen();
        match permutation % 6 {
            0 => {
                // do nothing
            },
            1 => {
                // offset x
                let x: i32 = rng.gen_range(-5..=5);
                image1 = image0.offset_wrap(x, 0)?;
            },
            2 => {
                // offset y
                let y: i32 = rng.gen_range(-5..=5);
                image1 = image0.offset_wrap(0, y)?;
            },
            3 => {
                // repair a little
                image1 = image0.denoise_type3(10)?;
            },
            4 => {
                // repair a lot
                image1 = image0.denoise_type3(40)?;
            },
            _ => {
                // repair a lot
                if image0.width() >= 3 && image0.height() >= 3 {
                    for _ in 0..0 {
                        let most_popular_color: Option<u8> = image0.histogram_all().least_popular_color();
                        if let Some(color) = most_popular_color {
                            image1.repair_trigram_algorithm(color)?;
                        }
                    }
                }
            },
        }
        Ok(image1)
    }

    fn random_size(rng: &mut StdRng) -> ImageSize {
        let width: u8 = rng.gen_range(1..=30);
        let height: u8 = rng.gen_range(1..=30);
        ImageSize::new(width, height)
    }

    fn random_instruction_context(rng: &mut StdRng) -> &str {
        let texts = [
            "In context of SimonSolver.",
            "Using SimonSolver context.",
            "With the context SimonSolver.",
            "With SimonSolver.",
            "Use SimonSolver.",
            "Context=SimonSolver.",
        ];
        texts.choose(rng).unwrap()
    }

    fn example_flipy(number_of_rows: u32) -> anyhow::Result<()> {
        for i in 0..number_of_rows {
            Self::example_flipy_iteration(i)?;
        }
        Ok(())
    }

    fn example_flipy_iteration(iteration: u32) -> anyhow::Result<()> {
        let mut rng = StdRng::seed_from_u64(iteration as u64);

        let instruction_descriptions = [
            "Invoke flipy.",
            "Run flip-y.",
            "Apply flip_y.",
            "Compute flipy and return the output image.",
            "Perform image-flipy with the image",
            "Flip y with image",
            "Flip y",
            "With the provided image, do a flip-y",
            "Give me image-flip-y of this image",
        ];
        let instruction_description: &str = instruction_descriptions.choose(&mut rng).unwrap();

        let image_size: ImageSize = Self::random_size(&mut rng);
        let input_image: Image = Self::advanced_image(&mut rng, image_size)?;

        let output_image: Image = input_image.flip_y()?;
        let input: String = Self::image_to_text(&input_image)?;
        let output: String = Self::image_to_text(&output_image)?;

        let instruction_prefix: &str = Self::random_instruction_context(&mut rng);

        let instruction: String = format!("{} {}", instruction_prefix, instruction_description);
        let prompt = format!(r#"{{"create":"flipy","instruction":"{}","input":"{}","output":"{}"}}"#, instruction, input, output);
        println!("{}", prompt);
        HtmlLog::text(prompt);
        HtmlLog::compare_images(vec![input_image, output_image]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_image_to_text() {
        let input: Image = Image::color(2, 2, 0);
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='00,00'");
    }

    #[test]
    fn test_10001_image_to_text() {
        let input: Image = Image::color(1, 1, 9);
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='9'");
    }

    #[test]
    fn test_10002_image_to_text() {
        let input: Image = Image::color(1, 1, 10);
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='a'");
    }

    #[test]
    fn test_10003_image_to_text() {
        let input: Image = Image::color(1, 1, 35);
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='z'");
    }

    #[test]
    fn test_10004_image_to_text() {
        let input = Image::try_create(4, 1, vec![0, 9, 10, 35]).expect("ok");
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='09az'");
    }

    // #[test]
    fn test_20000_example_flipy() {
        GenerateDataset::example_flipy(20).expect("ok");
    }
}
