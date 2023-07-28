use super::{Histogram, Image, ImageHistogram, ImageMask};
use super::{ImageToHTML, ImageSize, TaskGraph};
use super::arc_work_model::{Task, PairType};
use std::collections::HashSet;

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
        let mut s = String::new();
        let mut is_first = true;
        for (_count, color) in histogram.pairs_ordered_by_color() {
            if is_first {
                is_first = false;
            } else {
                s += ",";
            }
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
            if item.color == 0 {
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
                let size: ImageSize = pair.input.image.size();
                let s0: String = LineSpan::serialize_rle(&pair.input.image)?;
                let s1: String = format!("input[{}] = \"width{}:height{}:{}\";", pair_index, size.width, size.height, s0);
                rows.push(s1);
            }

            {
                let size: ImageSize = pair.output.image.size();
                let s0: String = LineSpan::serialize_rle(&pair.output.image)?;
                let s1: String = format!("output[{}] = \"width{}:height{}:{}\";", pair_index, size.width, size.height, s0);
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
                let size: ImageSize = pair.input.image.size();
                let s0: String = LineSpan::serialize_rle(&pair.input.image)?;
                let s1: String = format!("input[{}] = \"width{}:height{}:{}\";", pair_index, size.width, size.height, s0);
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
        let expected = "ID0:1W1B3W 1B1W1B2W 2B1W2B 3B1W1B,ID1:2W3B 3W2B 5W 5W,ID7:1B4W 1W1B3W 2W1B2W 3W1B1W";
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
