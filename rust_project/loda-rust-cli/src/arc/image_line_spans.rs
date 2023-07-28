use super::{Histogram, Image, ImageHistogram, ImageMask};
use super::{ImageToHTML, ImageSize, TaskGraph};
use super::arc_work_model::{Task, PairType};
use std::collections::HashSet;
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{Result, Context};

lazy_static! {
    /// Extract one SpanItem from a run length encoded string like: `5B1W8B1W5B`
    static ref EXTRACT_SPANITEM: Regex = Regex::new(
        r"(\d+)([BW])"
    ).unwrap();

    /// Extract one key=value from strings like: `width18` or `height16` or `ID0`
    static ref EXTRACT_KEY_VALUE: Regex = Regex::new(
        r"(width|height|ID)(\d+)"
    ).unwrap();
}

const MOCK_REPLY1: &str = r#"
The transformation appears to occur as follows:

- For sections that are filled entirely with black or white, the first 4 and last 4 cells in the rows 1 to 4 and 12 to 15 switch to the opposite color (B -> W or W -> B). In case the width is greater than 8, an extra cell in the middle also changes color.
- The rest of the rows (5 to 11) are left untouched except if a pattern of a different color in the middle exists. In that case, this pattern is split into two with a cell of the initial color in between.
- If there's a pattern of the form 1X10Y1Z in the middle, it is transformed to 1X5Y1Z1Y5Z.

So, for the given input[3], the transformation would be as follows:

```cpp
output[3] = "width18:height19:ID0:5B1W8B1W5B 5B1W8B1W5B 5B1W8B1W5B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 1B14W3B 5B1W8B1W5B 5B1W8B1W5B 5B1W8B1W5B,ID1:5W1B8W1W5W 5W1B8W1W5W 5W1B8W1W5W 1W14B3W 1W5B1W3B1W4B3W 1W2B5W1B2B3W 1W2B5W1B2B3W 1W2B5W1B2B3W 1W2B5W1B2B3W 1W14B3W 1W2B5W1B2B3W 1W2B5W1B2B3W 1W2B5W1B2B3W 1W2B5W1B2B3W 1W14B3W 1W14B3W 5W1B8W1W5W 5W1B8W1W5W 5W1B8W1W5W,ID8:5W1B8W1W5W 5W1B8W1W5W 5W1B8W1W5W 18W 6W1B3W1B7W 3W5B1W5B5W 3W5B1W5B5W 3W5B1W5B5W 3W5B1W5B5W 2W11B5W 3W5B1W5B5W 3W5B1W5B5W 3W5B1W5B5W 3W5B1W5B5W 18W 18W 5W1B8W1W5W 5W1B8W1W5W 5W1B8W1W5W";
```
Note: The actual transformation rule may be different, this is just a prediction based on the provided examples. The number '1' in the patterns is assumed to stay as '1'. If this is part of a larger system, other rules might apply.
"#;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct SpanItem {
    color: u8,
    x: u8,
    y: u8,
    length: u8,
}

#[derive(Debug)]
struct LineSpan {
    items: Vec<SpanItem>,
}

impl LineSpan {
    fn scan(image: &Image) -> anyhow::Result<Self> {
        let mut items: Vec<SpanItem> = Vec::new();
        for y in 0..image.height() {
            // Run length encoding
            let mut found_color: u8 = image.get(0, y as i32).unwrap_or(255);
            let mut found_x: u8 = 0;
            let mut found_length: u8 = 1;
            for x in 1..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                if color == found_color {
                    found_length += 1;
                    continue;
                }
                items.push(SpanItem { color: found_color, x: found_x, y, length: found_length });
                // Save data for next span
                found_x = x;
                found_length = 1;
                found_color = color;
            }
            if found_length > 0 {
                items.push(SpanItem { color: found_color, x: found_x, y, length: found_length });
            }
        }
        let instance = Self {
            items
        };
        Ok(instance)
    }

    /// Run-length encode an image.
    /// https://en.wikipedia.org/wiki/Run-length_encoding
    /// 
    /// I have experimented with GPT4 and it's able to decode this RLE format.
    /// I had to add a space character between the lines to make it work.
    /// I had to use B (Black) and W (White), so it's the same as on wikipedia.
    fn serialize_rle(image: &Image) -> anyhow::Result<String> {
        let histogram: Histogram = image.histogram_all();
        let mut s = format!("width{}:height{}", image.width(), image.height());
        for (_count, color) in histogram.pairs_ordered_by_color() {
            s += ":";
            s += &format!("ID{}:", color);
            s += &Self::serialize_rle_color(image, color)?;
        }
        Ok(s)
    }

    fn serialize_rle_color(image: &Image, color: u8) -> anyhow::Result<String> {
        let mask: Image = image.to_mask_where_color_is(color);
        let line_span = LineSpan::scan(&mask)?;
        let mut s = String::new();
        for item in &line_span.items {
            if item.x == 0 && item.y > 0 {
                s += " ";
            }
            s += &format!("{}", item.length);
            if item.color == 1 {
                s += "W";
            } else {
                s += "B";
            }
        }
        Ok(s)
    }
}

#[derive(Clone, Debug)]
pub struct PromptRLEDeserializer {
    pub lines: Vec<String>,
}

impl PromptRLEDeserializer {
    #[allow(dead_code)]
    pub fn reply_example1() -> String {
        MOCK_REPLY1.to_string()
    }

    fn decode_rle_string(s: &str) -> anyhow::Result<Vec<u8>> {
        let mut values = Vec::<u8>::new();
        for captures in EXTRACT_SPANITEM.captures_iter(s) {
            let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
            let capture2: &str = captures.get(2).map_or("", |m| m.as_str());
            let count: usize = capture1.parse()?;
            let color_name: char = capture2.chars().next().context("no color name")?;
            let value: u8 = match color_name {
                'B' => 0,
                'W' => 1,
                _ => anyhow::bail!("invalid color"),
            };
            for _ in 0..count {
                values.push(value);
            }
        }
        Ok(values)
    }

    fn decode_key_value(input: &str, expected_key: &str) -> anyhow::Result<u8> {
        let captures = match EXTRACT_KEY_VALUE.captures(input) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract key value from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let capture2: &str = captures.get(2).map_or("", |m| m.as_str());
        let integer_value: u8 = capture2.parse()?;
        if capture1 != expected_key {
            anyhow::bail!("Unknown key. Expected {}, but got {}", expected_key, capture1);
        }
        Ok(integer_value)
    }

    fn decode_image(input: &str) -> anyhow::Result<Image> {
        let parts: Vec<&str> = input.split(":").collect();
        if parts.len() < 3 {
            anyhow::bail!("Too few parts in the image format");
        }
        let width: u8 = Self::decode_key_value(parts[0], "width")?;
        let height: u8 = Self::decode_key_value(parts[1], "height")?;

        let mut result_image = Image::zero(width, height);

        let mut current_color: u8 = 0;
        for (index, part) in parts.iter().enumerate() {
            if index == 0 || index == 1 {
                continue;
            }
            if part.starts_with("ID") {
                current_color = Self::decode_key_value(part, "ID")?;
                continue;
            }
            for (y, rle_string) in part.split(" ").enumerate() {
                let values: Vec<u8> = Self::decode_rle_string(rle_string)?;
                for (x, value) in values.iter().enumerate() {
                    if *value > 0 {
                        result_image.set(x as i32, y as i32, current_color);
                    }
                }
            }
        }
        Ok(result_image)
    }

    fn interpret_line_and_draw(_line_index: usize, line: &str, image: &mut Image) -> anyhow::Result<()> {
        // Color from obfuscated color name
        // let id = FieldId::try_from(line)?;
        // let color: u8 = id.value;

        // // Coordinates for bounding box
        // let tlbr = TLBR::try_from(line)?;
        // // println!("tlbr: {:?}", tlbr);

        // let object_x: i32 = tlbr.left as i32 - 1;
        // let object_y: i32 = tlbr.top as i32 - 1;
        // let object_width: i32 = tlbr.right as i32 - tlbr.left as i32 + 1;
        // let object_height: i32 = tlbr.bottom as i32 - tlbr.top as i32 + 1;

        // if object_width < 0 || object_height < 0 {
        //     anyhow::bail!("Invalid width or height");
        // }

        // let mut _count_draw: usize = 0;
        // for y in 0..image.height() {
        //     for x in 0..image.width() {
        //         let xx: i32 = x as i32;
        //         let yy: i32 = y as i32;

        //         if xx >= object_x && xx < object_x + object_width && yy >= object_y && yy < object_y + object_height {
        //             image.set(xx, yy, color);
        //             _count_draw += 1;
        //         }
        //     }
        // }
        // println!("count_draw: {}", count_draw);
        
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

    pub fn to_html(&self) -> String {
        let mut image = Image::zero(30, 30);
        // if let Some(width_height) = &self.width_height {
        //     image = Image::zero(width_height.width, width_height.height);
        // }

        self.interpret_and_draw(&mut image);

        let mut s = String::new();
        s += &image.to_html();
        s
    }
}

impl TryFrom<&str> for PromptRLEDeserializer {
    type Error = anyhow::Error;

    fn try_from(multiline_text: &str) -> Result<Self, Self::Error> {
        let mut lines_with_prefix = Vec::<String>::new();
        let mut inside_code_block = false;
        let mut count_unrecognized_inside_code_block: usize = 0;
        let mut count_code_block: usize = 0;
        for line in multiline_text.split("\n") {
            let trimmed_line: &str = line.trim();
            if trimmed_line.contains("```cpp") {
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
            if trimmed_line.starts_with("output[") {
                lines_with_prefix.push(line.to_string());
                continue;
            }
            count_unrecognized_inside_code_block += 1;
        }
        if count_code_block == 0 {
            anyhow::bail!("No code block found. Expected a code block starting with 3 backticks and cpp.");
        }
        if count_code_block >= 2 {
            anyhow::bail!("Multiple code blocks found. Expected just one code block starting with 3 backticks and cpp.");
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

#[derive(Clone, Debug)]
pub struct PromptRLESerializer;

impl PromptRLESerializer {
    /// Convert the `TaskGraph` into a prompt for a language model to solve.
    /// 
    /// Known problem: It can only ask prompt about the first `test` pair.
    /// The tasks that have more than one `test` pair, will not create prompts for the remaining `test` pairs.
    pub fn to_prompt(task_graph: &TaskGraph) -> anyhow::Result<String> {
        let task: &Task = match &task_graph.task() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("graph is not initialized with a task"));
            }
        };

        let mut rows = Vec::<String>::new();

        rows.push("I'm doing CPP experiments.\n\n".to_string());

        rows.push("These are run-length encoded images.".to_string());
        
        rows.push("The RLE used here only uses black and white. There are no other colors than black or white. Example `B3W7B2` becomes `3 black, 7 white, 2 black`.".to_string());
        rows.push("".to_string());

        rows.push("The ID indicates the layer number. Multiple images can be stacked and black can be considered transparent.".to_string());

        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("```cpp".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type == PairType::Test {
                continue;
            }
            if pair_index > 0 {
                rows.push("".to_string());
            }

            {
                let s0: String = LineSpan::serialize_rle(&pair.input.image)?;
                let s1: String = format!("input[{}] = \"{}\";", pair_index, s0);
                rows.push(s1);
            }

            {
                let s0: String = LineSpan::serialize_rle(&pair.output.image)?;
                let s1: String = format!("output[{}] = \"{}\";", pair_index, s0);
                rows.push(s1);
            }
        }
        rows.push("```".to_string());
        rows.push("".to_string());

        rows.push("\n\n# Task".to_string());
        rows.push("With the following example, I want you to predict what the output should be. Print your reasoning before printing the code.\n\n".to_string());
        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("```cpp".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type == PairType::Train {
                continue;
            }

            {
                let s0: String = LineSpan::serialize_rle(&pair.input.image)?;
                let s1: String = format!("input[{}] = \"{}\";", pair_index, s0);
                rows.push(s1);
            }

            {
                let grid_size: String = match task.predict_output_size_for_pair(pair) {
                    Ok(size) => {
                        format!("width{}:height{}", size.width, size.height)
                    },
                    Err(_) => {
                        format!("widthPREDICT:heightPREDICT")
                    }
                };
                let s1: String = format!("output[{}] = \"{}:PREDICT\";", pair_index, grid_size);
                rows.push(s1);
            }

            // Future experiment:
            // process all the test pairs. Currently it's only 1 test pair.
            break;
        }
        rows.push("```".to_string());
        rows.push("Repeat the previous CPP code, with PREDICT replaced with your predictions.".to_string());

        Ok(rows.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_line_spans() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 0, 1, 1, 1,
            0, 7, 0, 1, 1,
            0, 0, 7, 0, 0,
            0, 0, 0, 7, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual = LineSpan::scan(&input).expect("ok");

        // Assert
        let mut expected_items = Vec::<SpanItem>::new();
        // y=0
        expected_items.push(SpanItem { color: 7, x: 0, y: 0, length: 1 });
        expected_items.push(SpanItem { color: 0, x: 1, y: 0, length: 1 });
        expected_items.push(SpanItem { color: 1, x: 2, y: 0, length: 3 });
        // y=1
        expected_items.push(SpanItem { color: 0, x: 0, y: 1, length: 1 });
        expected_items.push(SpanItem { color: 7, x: 1, y: 1, length: 1 });
        expected_items.push(SpanItem { color: 0, x: 2, y: 1, length: 1 });
        expected_items.push(SpanItem { color: 1, x: 3, y: 1, length: 2 });
        // y=2
        expected_items.push(SpanItem { color: 0, x: 0, y: 2, length: 2 });
        expected_items.push(SpanItem { color: 7, x: 2, y: 2, length: 1 });
        expected_items.push(SpanItem { color: 0, x: 3, y: 2, length: 2 });
        // y=3
        expected_items.push(SpanItem { color: 0, x: 0, y: 3, length: 3 });
        expected_items.push(SpanItem { color: 7, x: 3, y: 3, length: 1 });
        expected_items.push(SpanItem { color: 0, x: 4, y: 3, length: 1 });
        assert_eq!(actual.items, expected_items);
    }

    #[test]
    fn test_20000_run_length_encoding() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 0, 1, 1, 1,
            0, 7, 0, 1, 1,
            0, 0, 7, 0, 0,
            0, 0, 0, 7, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: String = LineSpan::serialize_rle(&input).expect("ok");

        // Assert
        let expected = "width5:height4:ID0:1B1W3B 1W1B1W2B 2W1B2W 3W1B1W:ID1:2B3W 3B2W 5B 5B:ID7:1W4B 1B1W3B 2B1W2B 3B1W1B";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_60000_decode_rle_string() {
        // Arrange
        let input: &str = "1B10W1B2W";

        // Act
        let actual = PromptRLEDeserializer::decode_rle_string(input).expect("ok");

        // Assert
        assert_eq!(actual, Vec::<u8>::from([0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1]));
    }

    #[test]
    fn test_60001_decode_image() {
        // Arrange
        let input: &str = "width5:height4:ID0:1B1W3B 1W1B1W2B 2W1B2W 3W1B1W:ID1:2B3W 3B2W 5B 5B:ID7:1W4B 1B1W3B 2B1W2B 3B1W1B";

        // Act
        let actual = PromptRLEDeserializer::decode_image(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 0, 1, 1, 1,
            0, 7, 0, 1, 1,
            0, 0, 7, 0, 0,
            0, 0, 0, 7, 0,
        ];
        let expected: Image = Image::try_create(5, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70000_deserialize_ok() {
        // Arrange
        let s: String = PromptRLEDeserializer::reply_example1();
        let s1: &str = &s;

        // Act
        let actual: PromptRLEDeserializer = PromptRLEDeserializer::try_from(s1).expect("ok");

        // Assert
        assert_eq!(actual.lines.len(), 1);
    }
}
