use super::{Image, TaskGraph, ImageRotate};
use super::prompt::{PromptSerialize, PromptDeserialize};
use super::arc_work_model::{Task, PairType};
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{Result, Context};

lazy_static! {
    /// Remove prefix and suffix, so only the image data remains.
    static ref EXTRACT_IMAGE_DATA: Regex = Regex::new(r"(width\d+,height\d+(?:,(:?\d+))+)").unwrap();

    /// Extract string, value from a string like: `width29`
    static ref EXTRACT_STRING_VALUE: Regex = Regex::new(r"([a-z]+)(\d+)").unwrap();

    /// Determine if it's all digits in the range 0..=9
    static ref ALL_DIGITS: Regex = Regex::new(r"^\d+$").unwrap();
}

const MOCK_REPLY1: &str = r#"
```python
output[19] = 'width5,height3,00008,88888,00008'
```
"#;

#[derive(Clone, Debug)]
pub struct PromptCompactDeserializer {
    pub lines: Vec<String>,
}

impl PromptCompactDeserializer {
    #[allow(dead_code)]
    pub fn reply_example1() -> String {
        MOCK_REPLY1.to_string()
    }

    fn interpret_line_and_draw(_line_index: usize, line: &str, image: &mut Image) -> anyhow::Result<()> {
        let (output_image, status) = TextToImage::convert(line)?;
        if let Some(status) = status {
            println!("PromptCompactDeserializer. Problems: {}", status);
        }
        image.set_image(output_image);        
        Ok(())
    }

    fn interpret_and_draw(&self, image: &mut Image) {
        for (line_index, line) in self.lines.iter().enumerate() {
            match Self::interpret_line_and_draw(line_index, line, image) {
                Ok(_) => {},
                Err(error) => {
                    println!("Error: {}", error);
                }
            }
        }
    }
}

impl PromptDeserialize for PromptCompactDeserializer {
    fn image(&self) -> anyhow::Result<Image> {
        let mut image = Image::zero(30, 30);
        self.interpret_and_draw(&mut image);
        Ok(image)
    }

    fn status(&self) -> Option<String> {
        None
    }
}

impl TryFrom<&str> for PromptCompactDeserializer {
    type Error = anyhow::Error;

    fn try_from(multiline_text: &str) -> Result<Self, Self::Error> {
        let mut lines_with_prefix = Vec::<String>::new();
        let mut inside_code_block = false;
        let mut count_unrecognized_inside_code_block: usize = 0;
        let mut count_code_block: usize = 0;
        for line in multiline_text.split("\n") {
            let trimmed_line: &str = line.trim();
            if trimmed_line.contains("```python") {
                if count_code_block == 0 {
                    inside_code_block = true;
                }
                count_code_block += 1;
                continue;
            }
            if !inside_code_block {
                continue;
            }
            if trimmed_line == "```" {
                inside_code_block = false;
                continue;
            }
            if trimmed_line.is_empty() {
                continue;
            }
            if trimmed_line.contains("#") {
                continue;
            }
            if trimmed_line.starts_with("output[") {
                lines_with_prefix.push(trimmed_line.to_string());
                continue;
            }
            count_unrecognized_inside_code_block += 1;
        }
        if count_code_block == 0 {
            anyhow::bail!("No code block found. Expected a code block starting with 3 backticks and python.");
        }
        if count_code_block >= 2 {
            anyhow::bail!("Multiple code blocks found. Expected just one code block starting with 3 backticks and python.");
        }
        if count_unrecognized_inside_code_block > 0 {
            anyhow::bail!("{} unrecognized lines inside the code block", count_unrecognized_inside_code_block);
        }
        let instance = Self {
            lines: lines_with_prefix,
        };
        Ok(instance)
    }
}

struct TextToImage;

impl TextToImage {
    /// Decode a compact string representation into an ARC image.
    fn convert(input: &str) -> anyhow::Result<(Image, Option<String>)> {
        // Remove prefix and suffix
        let capture = match EXTRACT_IMAGE_DATA.captures(input) {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("no image data found"));
            }
        };
        let input_trimmed: &str = capture.get(1).map_or("", |m| m.as_str());

        // Extract parameters for: `width`, `height`.
        let mut found_width: Option<u8> = None;
        let mut found_height: Option<u8> = None;
        for capture in EXTRACT_STRING_VALUE.captures_iter(input_trimmed) {
            let capture1: &str = capture.get(1).map_or("", |m| m.as_str());
            let capture2: &str = capture.get(2).map_or("", |m| m.as_str());
            match capture1 {
                "width" => {
                    let value: u8 = capture2.parse::<u8>().context("width value")?;
                    found_width = Some(value);
                },
                "height" => {
                    let value: u8 = capture2.parse::<u8>().context("height value")?;
                    found_height = Some(value);
                },
                _ => {}
            }
        }
        let field_width: u8 = found_width.context("width")?;
        let field_height: u8 = found_height.context("height")?;

        // Extract only strings with pixel values
        let mut rows = Vec::<String>::new();
        let mut width_max: usize = usize::MIN;
        let mut width_min: usize = usize::MAX;
        for item in input_trimmed.split(",") {
            if !ALL_DIGITS.is_match(item) {
                continue;
            }
            width_min = width_min.min(item.len());
            width_max = width_max.max(item.len());
            rows.push(item.to_string());
        }
        let pixeldata_height: usize = rows.len();

        // Checks if there is consensus about the width and the height and the pixeldata
        let same_width: bool = (width_max == width_min) && (width_max == field_width as usize);
        let same_height: bool = pixeldata_height == (field_height as usize);
        let same_size: bool = same_width && same_height;

        // Pick the biggest size of the size parameters, so no pixel data is outside the visible area.
        let width: u8 = (field_width as usize).max(width_max).min(40) as u8;
        let height: u8 = (field_height as usize).max(pixeldata_height).min(40) as u8;

        // Create empty image with 255 color to indicate that it has not been assigned a color yet.
        let fill_color: u8 = 255;
        let mut image: Image = Image::color(width, height, fill_color);

        // Assign pixel values
        for (row_index, row) in rows.iter().enumerate() {
            for (column_index, item) in row.chars().enumerate() {
                let x: i32 = column_index as i32;
                let y: i32 = row_index as i32;
                let color: u8 = item.to_digit(10).unwrap_or(255) as u8;
                _ = image.set(x, y, color);
            }
        }

        let mut problems = Vec::<String>::new();
        if width_min != width_max {
            let s: String = format!("Inconsistent width of pixeldata rows width_min: {} width_max: {}. They are supposed to be the same.", width_min, width_max);
            problems.push(s);
        }
        if !same_size {
            let s: String = format!("There is a mismatch between size of the image, and the pixel data. size: {}x{}, pixel data: {}x{}", field_width, field_height, width_max, pixeldata_height);
            problems.push(s);
        }
        let status: Option<String> = if problems.is_empty() {
            None
        } else {
            Some(problems.join(", "))
        };

        Ok((image, status))
    }
}

struct ImageToText;

impl ImageToText {
    /// Creates a compact string representation of an ARC image.
    /// 
    /// If `include_size` is false, then there is no width and height info in the dictionary.
    /// Returns a string like `008000700,008000700,888888288,008000700,008000700,008000700,772777777,008000700,008000700`
    /// 
    /// If `include_size` is true, then it will include the width and height of the image, like this
    /// `width9,height9,008000700,008000700,888888288,008000700,008000700,008000700,772777777,008000700,008000700`
    fn convert(image: &Image, include_size: bool) -> anyhow::Result<String> {
        let mut items = Vec::<String>::new();
        if include_size {
            items.push(format!("width{}", image.width()));
            items.push(format!("height{}", image.height()));
        }
        for y in 0..image.height() {
            let mut s = String::new();
            for x in 0..image.width() {
                let pixel = image.get(x as i32, y as i32).unwrap_or(255);
                s += &format!("{}", pixel);
            }
            items.push(s);
        }
        Ok(items.join(","))
    }
}

#[derive(Clone, Debug)]
pub struct PromptCompactSerializer;

impl PromptSerialize for PromptCompactSerializer {
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String> {
        let task: &Task = match &task_graph.task() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("graph is not initialized with a task"));
            }
        };

        let include_size: bool = true;
        let include_rotated: bool = false;

        let mut rows = Vec::<String>::new();

        rows.push("You answer questions about the logic puzzles.".to_string());

        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("Use the below article on a logic puzzle to answer the subsequent question. If the answer cannot be found, write \"I don't know.\"".to_string());

        rows.push("".to_string());
        rows.push("Focus on the `MISSING`, this is what I want you to fill in.".to_string());

        rows.push("".to_string());
        rows.push("Grids".to_string());
        rows.push("- The `input` and `output` dictionaries contain key-value pairs where keys are group numbers and values are strings that represent 2D grids.".to_string());
        rows.push("- The first two fields in each string of both input and output dictionaries specify the grid width and height. Example: `width30,height19` specifies a grid with 30 columns and 19 rows.".to_string());
        rows.push("- The remaining part of the string is a one-dimensional representation of a 2D grid.".to_string());

        rows.push("".to_string());
        rows.push("Article start marker".to_string());
        rows.push("```python".to_string());
        rows.push("input = {}".to_string());
        if include_rotated {
            rows.push("input_rotate90 = {}".to_string());
        }
        rows.push("output = {}".to_string());
        if include_rotated {
            rows.push("output_rotate90 = {}".to_string());
        }
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            rows.push(format!("# Group{}", pair_index));

            {
                let s0: String = ImageToText::convert(&pair.input.image, include_size)?;
                let s1: String = format!("input[{}] = '{}'", pair_index, s0);
                rows.push(s1);
            }
            if include_rotated {
                let image: Image = pair.input.image.rotate_cw()?;
                let s0: String = ImageToText::convert(&image, include_size)?;
                let s1: String = format!("input_rotate90[{}] = '{}'", pair_index, s0);
                rows.push(s1);
            }

            match pair.pair_type {
                PairType::Train => {
                    let s0: String = ImageToText::convert(&pair.output.image, include_size)?;
                    let s1: String = format!("output[{}] = '{}'", pair_index, s0);
                    rows.push(s1);

                    if include_rotated {
                        let image: Image = pair.output.image.rotate_cw()?;
                        let s0: String = ImageToText::convert(&image, include_size)?;
                        let s1: String = format!("output_rotate90[{}] = '{}'", pair_index, s0);
                        rows.push(s1);
                    }
                },
                PairType::Test => {
                    let s1: String = format!("output[{}] = 'MISSING'", pair_index);
                    rows.push(s1);

                    if include_rotated {
                        let s2: String = format!("output_rotate90[{}] = 'MISSING'", pair_index);
                        rows.push(s2);
                    }
                }
            }
        }
        rows.push("```".to_string());
        rows.push("Article end marker".to_string());

        // rows.push("Question: Is this a repair job. Are there masked out parts in the input image, that are present in the output. If so then the task is to repair these pixels.".to_string());

        rows.push("".to_string());
        rows.push("Question: Write 10 bullet points with observations about input and output.".to_string());
        rows.push("".to_string());

        let mut output_pair_index: usize = 0;
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type == PairType::Test {
                output_pair_index = pair_index;
                break;
            }
        }

        rows.push("Question: Fill in the `MISSING` piece into this python code block.".to_string());
        rows.push("```python".to_string());
        rows.push(format!("output[{}] = 'MISSING'", output_pair_index));
        rows.push("```".to_string());
        
        Ok(rows.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, ImageSize};

    #[test]
    fn test_10000_image_to_text_without_size() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: String = ImageToText::convert(&input, false).expect("ok");

        // Assert
        let expected = "779,879";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_image_to_text_with_size() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
            0, 1, 2,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: String = ImageToText::convert(&input, true).expect("ok");

        // Assert
        let expected = "width3,height2,012,012";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_text_to_image() {
        // Arrange
        let input: &str = "width2,height3,12,34,56";

        // Act
        let actual = TextToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_20001_text_to_image_inconsistent_width() {
        // Arrange
        let input: &str = "width2,height3,12,3499,56";

        // Act
        let actual = TextToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2, 255, 255,
            3, 4, 9, 9,
            5, 6, 255, 255,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        let message: String = actual.1.expect("error message");
        assert_eq!(message.contains("width_min: 2 width_max: 4"), true);
    }

    #[test]
    fn test_20002_text_to_image_remove_prefix_and_suffix() {
        // Arrange
        let input: &str = "junk`output[8] = 'width2,height3,12,34,56'`junk";

        // Act
        let actual = TextToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
            5, 6,
        ];
        let expected: Image = Image::try_create(2, 3, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_20003_text_to_image_width_2digits() {
        // Arrange
        let input: &str = "width12,height1,012345678901";

        // Act
        let actual = TextToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
        ];
        let expected: Image = Image::try_create(12, 1, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_20004_text_to_image_height_2digits() {
        // Arrange
        let input: &str = "width1,height12,0,1,2,3,4,5,6,7,8,9,0,1";

        // Act
        let actual = TextToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
        ];
        let expected: Image = Image::try_create(1, 12, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_30000_deserialize_ok() {
        // Arrange
        let s: String = PromptCompactDeserializer::reply_example1();
        let s1: &str = &s;

        // Act
        let actual: PromptCompactDeserializer = PromptCompactDeserializer::try_from(s1).expect("ok");

        // Assert
        assert_eq!(actual.lines.len(), 1);
        let image: Image = actual.image().expect("ok");
        assert_eq!(image.size(), ImageSize::new(5, 3));
    }
}
