use super::{Image, RandomImage, ImageSize, ImageHistogram, ImageSort, ImageSortMode, ImageSymmetry, ImageStack, ImageOffset, ImageDenoise, ImageGravity, ImageRepairTrigram, ImageRotate90};
use super::HtmlLog;
use std::io::Write;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng, Rng};

#[allow(dead_code)]
trait ImageTo09azRepresentation {
    /// Convert an Image to a text representation.
    /// 
    /// The text representation is a string with the `0-9a-z` format.
    /// 
    /// The values `0..=35` are converted to `0..=9` and `a..=z`.
    /// 
    /// An error is returned when encountering a value greater than `35`.
    fn to_09az(&self) -> anyhow::Result<String>;
}

impl ImageTo09azRepresentation for Image {
    fn to_09az(&self) -> anyhow::Result<String> {
        let max_value: u16 = 'z' as u16;
        let mut rows = Vec::<String>::new();
        for y in 0..self.height() {
            let mut row = String::new();
            for x in 0..self.width() {
                let color: u8 = self.get(x as i32, y as i32).unwrap_or(255);
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
}

#[allow(dead_code)]
trait ImageToCompactJsonRepresentation {
    /// Convert an Image to a text representation similar to JSON without spaces.
    /// 
    /// Example of a 3x2 image: `[[1,2,3],[4,5,6]]`
    /// 
    /// Example of an empty image with size=0x0: `[[]]`
    /// 
    /// Example of a 1x1 image: `[[9]]`
    fn to_compactjson(&self) -> anyhow::Result<String>;
}

impl ImageToCompactJsonRepresentation for Image {
    fn to_compactjson(&self) -> anyhow::Result<String> {
        let mut rows = Vec::<String>::new();
        for y in 0..self.height() {
            let mut pixels = Vec::<String>::new();
            for x in 0..self.width() {
                let color: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                pixels.push(color.to_string());
            }
            rows.push(pixels.join(","));
        }
        let mut result = String::new();
        result += "[[";
        result += &rows.join("],[");
        result += "]]";
        Ok(result)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct GenerateRandomImage;

impl GenerateRandomImage {
    /// Generate a noisy image with the specified size.
    fn simple_image(rng: &mut StdRng, size: ImageSize) -> anyhow::Result<Image> {
        let mut permutation: u32 = rng.gen();
        let image0: Image = match permutation % 2 {
            0 => {
                // let max_color: u8 = rng.gen_range(3..=35); // 35 = REPR09AZ_MAX_VALID_PIXEL_VALUE
                let max_color: u8 = rng.gen_range(3..=9);
                RandomImage::uniform_colors(
                    rng,
                    size, 
                    0,
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

    #[allow(dead_code)]
    fn random_size_big(rng: &mut StdRng) -> ImageSize {
        let width: u8 = rng.gen_range(1..=30);
        let height: u8 = rng.gen_range(1..=30);
        ImageSize::new(width, height)
    }

    fn random_size_small(rng: &mut StdRng) -> ImageSize {
        let width: u8 = rng.gen_range(1..=10);
        let height: u8 = rng.gen_range(1..=10);
        ImageSize::new(width, height)
    }

    fn create(rng: &mut StdRng) -> anyhow::Result<Image> {
        // let image_size: ImageSize = Self::random_size_big(rng);
        let image_size: ImageSize = Self::random_size_small(rng);
        let image: Image = Self::advanced_image(rng, image_size)?;
        Ok(image)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum GenerateDataType {
    FlipX,
    FlipY,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl GenerateDataType {
    fn execute(&self, input: &Image) -> anyhow::Result<Image> {
        match self {
            GenerateDataType::FlipX => input.flip_x(),
            GenerateDataType::FlipY => input.flip_y(),
            GenerateDataType::Rotate90 => input.rotate_ccw(),
            GenerateDataType::Rotate180 => input.rotate(2),
            GenerateDataType::Rotate270 => input.rotate_cw(),
        }
    }

    fn generator_label(&self) -> &str {
        match self {
            GenerateDataType::FlipX => "flipx",
            GenerateDataType::FlipY => "flipy",
            GenerateDataType::Rotate90 => "rotate90",
            GenerateDataType::Rotate180 => "rotate180",
            GenerateDataType::Rotate270 => "rotate270",
        }
    }

    fn random_seed(&self) -> u64 {
        match self {
            GenerateDataType::FlipX => 0,
            GenerateDataType::FlipY => 10000,
            GenerateDataType::Rotate90 => 20000,
            GenerateDataType::Rotate180 => 30000,
            GenerateDataType::Rotate270 => 40000,
        }
    }

    fn instruction_description(&self, rng: &mut StdRng) -> String {
        match self {
            GenerateDataType::FlipX => Self::instruction_description_flip_x(rng).to_string(),
            GenerateDataType::FlipY => Self::instruction_description_flip_y(rng).to_string(),
            GenerateDataType::Rotate90 => Self::instruction_description_rotate90(rng).to_string(),
            GenerateDataType::Rotate180 => Self::instruction_description_rotate180(rng).to_string(),
            GenerateDataType::Rotate270 => Self::instruction_description_rotate270(rng).to_string(),
        }
    }

    fn instruction_description_flip_x(rng: &mut StdRng) -> &str {
        let texts = [
            "Invoke flipx.",
            "Run flip-x.",
            "Apply flip_x.",
            "Compute flipx and return the output image.",
            "Perform image-flipx with the image",
            "Flip X with image",
            "Flip x",
            "With the provided image, do a flip-x",
            "Give me image-flip-x of this image",
        ];
        texts.choose(rng).unwrap()
    }

    fn instruction_description_flip_y(rng: &mut StdRng) -> &str {
        let texts = [
            "Invoke flipy.",
            "Run flip-y.",
            "Apply flip_y.",
            "Compute flipy and return the output image.",
            "Perform image-flipy with the image",
            "Flip Y with image",
            "Flip y",
            "With the provided image, do a flip-y",
            "Give me image-flip-y of this image",
        ];
        texts.choose(rng).unwrap()
    }

    fn instruction_description_rotate90(rng: &mut StdRng) -> &str {
        let texts = [
            "Invoke rotate90.",
            "Run rotate 90.",
            "Apply rotate_90.",
            "Compute Rotate90 and return the output image.",
            "Perform image-rotate90 with the image",
            "Rotate90 with image",
            "Rotate 90 degrees",
            "With the provided image, do a rotate 90",
            "Give me image-rotate-90 of this image",
        ];
        texts.choose(rng).unwrap()
    }

    fn instruction_description_rotate180(rng: &mut StdRng) -> &str {
        let texts = [
            "Invoke rotate180.",
            "Run rotate 180.",
            "Apply rotate_180.",
            "Compute Rotate180 and return the output image.",
            "Perform image-rotate180 with the image",
            "Rotate180 with image",
            "Rotate 180 degrees",
            "With the provided image, do a rotate 180",
            "Give me image-rotate-180 of this image",
        ];
        texts.choose(rng).unwrap()
    }

    fn instruction_description_rotate270(rng: &mut StdRng) -> &str {
        let texts = [
            "Invoke rotate270.",
            "Run rotate 270.",
            "Apply rotate_270.",
            "Compute Rotate270 and return the output image.",
            "Perform image-rotate270 with the image",
            "Rotate270 with image",
            "Rotate 270 degrees",
            "With the provided image, do a rotate 270",
            "Give me image-rotate-270 of this image",
        ];
        texts.choose(rng).unwrap()
    }
}

#[allow(dead_code)]
struct GenerateDataset {
    dataset_items: Vec<String>,
}

impl GenerateDataset {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            dataset_items: vec!()
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn populate(&mut self, number_of_rows: u32, print_to_htmllog: bool) -> anyhow::Result<()> {
        let generator_types = [
            GenerateDataType::FlipX,
            GenerateDataType::FlipY,
            GenerateDataType::Rotate90,
            GenerateDataType::Rotate180,
            GenerateDataType::Rotate270,
        ];
        for generator_type in &generator_types {
            self.create_many_dataset_items(generator_type.clone(), number_of_rows, print_to_htmllog)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn create_many_dataset_items(&mut self, generator_type: GenerateDataType, number_of_rows: u32, print_to_htmllog: bool) -> anyhow::Result<()> {
        let mut dataset_items = Vec::<String>::new();
        for i in 0..number_of_rows {
            let dataset_item: String = match Self::create_dataset_item(i, generator_type.clone(), print_to_htmllog) {
                Ok(value) => value,
                Err(error) => {
                    error!("create_dataset_item. Unable to execute iteration {}: {}", i, error);
                    continue;
                }
            };
            // println!("{}", dataset_item);
            dataset_items.push(dataset_item);
        }
        // println!("dataset.len = {}", dataset_items.len());
        self.dataset_items.append(&mut dataset_items);
        Ok(())
    }

    #[allow(dead_code)]
    fn create_dataset_item(iteration: u32, generator_type: GenerateDataType, print_to_htmllog: bool) -> anyhow::Result<String> {
        let seed: u64 = generator_type.random_seed() ^ (iteration as u64);
        let mut rng = StdRng::seed_from_u64(seed);
        let input_image: Image = GenerateRandomImage::create(&mut rng)?;

        let instruction_description: String = generator_type.instruction_description(&mut rng);
        let generator_label: &str = generator_type.generator_label();

        let output_image: Image = generator_type.execute(&input_image)?;
        // let input: String = input_image.to_09az()?;
        // let output: String = output_image.to_09az()?;
        let input_compactjson: String = input_image.to_compactjson()?;
        let input: String = format!("image={}", input_compactjson);
        let output_compactjson: String = output_image.to_compactjson()?;
        let output: String = format!("image={}", output_compactjson);

        let instruction_prefix: &str = Self::random_instruction_context(&mut rng);

        let instruction: String = format!("{} {}", instruction_prefix, instruction_description);
        let dataset_item = format!(r#"{{"create":"{}","instruction":"{}","input":"{}","output":"{}"}}"#, generator_label, instruction, input, output);
        if print_to_htmllog {
            HtmlLog::text(dataset_item.clone());
            HtmlLog::compare_images(vec![input_image, output_image]);
        }
        Ok(dataset_item)
    }

    #[allow(dead_code)]
    fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let s: String = self.dataset_items.join("\n");
        let mut file = std::fs::File::create(path)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    use std::path::PathBuf;

    #[test]
    fn test_10000_to_09az() {
        let input: Image = Image::color(2, 2, 0);
        let actual: String = input.to_09az().expect("ok");
        assert_eq!(actual, "image='00,00'");
    }

    #[test]
    fn test_10001_to_09az() {
        let input: Image = Image::color(1, 1, 9);
        let actual: String = input.to_09az().expect("ok");
        assert_eq!(actual, "image='9'");
    }

    #[test]
    fn test_10002_to_09az() {
        let input: Image = Image::color(1, 1, 10);
        let actual: String = input.to_09az().expect("ok");
        assert_eq!(actual, "image='a'");
    }

    #[test]
    fn test_10003_to_09az() {
        let input: Image = Image::color(1, 1, 35);
        let actual: String = input.to_09az().expect("ok");
        assert_eq!(actual, "image='z'");
    }

    #[test]
    fn test_10004_to_09az() {
        let input = Image::try_create(4, 1, vec![0, 9, 10, 35]).expect("ok");
        let actual: String = input.to_09az().expect("ok");
        assert_eq!(actual, "image='09az'");
    }

    #[test]
    fn test_20000_to_compactjson() {
        let input: Image = Image::color(2, 2, 0);
        let actual: String = input.to_compactjson().expect("ok");
        assert_eq!(actual, "[[0,0],[0,0]]");
    }

    #[test]
    fn test_20001_to_compactjson() {
        let input: Image = Image::color(1, 1, 9);
        let actual: String = input.to_compactjson().expect("ok");
        assert_eq!(actual, "[[9]]");
    }

    #[test]
    fn test_20002_to_compactjson() {
        let input: Image = Image::empty();
        let actual: String = input.to_compactjson().expect("ok");
        assert_eq!(actual, "[[]]");
    }

    #[test]
    fn test_20003_to_compactjson() {
        let input = Image::try_create(5, 1, vec![0, 9, 10, 35, 255]).expect("ok");
        let actual: String = input.to_compactjson().expect("ok");
        assert_eq!(actual, "[[0,9,10,35,255]]");
    }

    #[allow(dead_code)]
    // #[test]
    fn test_20000_generate_dataset() {
        let path: PathBuf = PathBuf::from("/Users/neoneye/Downloads/texttransformer_output.jsonl");
        let mut generator = GenerateDataset::new();
        generator.populate(200, true).expect("ok");
        generator.save(&path).expect("ok");
    }
}
