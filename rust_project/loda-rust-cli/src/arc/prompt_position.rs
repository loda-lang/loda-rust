use super::{Image, TaskGraph};
use super::prompt::{PromptSerialize, PromptDeserialize};
use super::arc_work_model::{Task, PairType};
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{Result, Context};

lazy_static! {
    /// Extract string, value from a string like: `'width':3`
    static ref EXTRACT_STRING_VALUE: Regex = Regex::new(r"'(\w+)'\s*:\s*(\d+)").unwrap();

    /// Extract x, y, color from strings like: `(3,4):5`
    static ref EXTRACT_X_Y_COLOR: Regex = Regex::new(r"[(]\s*(\d+)\s*,\s*(\d+)\s*[)]\s*:\s*(\d+)").unwrap();
}

const MOCK_REPLY1: &str = r#"
# Task A
Each input seems to represent a 2D grid, with values at each coordinate. The transformation from input to output appears to involve shifting values in certain regions or blocks, and adding additional values in the region around non-zero elements. Non-zero values are typically clustered, indicating some form of a shape or pattern. 

# Task B
| Observation Name | Observation Values | Comments |
|------------------|--------------------|----------|
| Layer Mass       | Variable           | The mass of a layer, measured by summing the values, varies across inputs. |
| Connected Clusters| Variable          | Each layer appears to have one or more clusters of non-zero values. |
| Mass and Sorting | No Clear Relation  | It's not obvious if mass relates to the sorting of layers. |
| Horizontal Lines | Yes                | Some clusters form horizontal lines, but they don't typically extend edge to edge. |
| Vertical Lines   | Yes                | Some clusters form vertical lines, but they don't typically extend edge to edge. |
| Recognizable Shapes | No              | The clusters don't appear to resemble known shapes like L, H, E, etc. |
| Color Change     | Yes                | Non-zero values can change, possibly based on their position or nearby values. |
| Relative Movement | Yes               | Non-zero values appear to move vertically or horizontally. |
| Boolean Operations | No Clear Evidence | It's unclear if any boolean operations are involved in the transformation. |

# Task C
The operation seems to involve shifting non-zero values to the nearest top or left zero values. Additional non-zero values are added around existing ones, and the new values seem to be a copy of the values from the block that got shifted.

# Task D
Given the observed rules, we can anticipate the output. The non-zero values will shift towards top or left and new values will appear around existing ones.

```python
output[4] = {'width':9,'height':9',
(0,0):0,(1,0):0,(2,0):0,(3,0):0,(4,0):0,(5,0):0,(6,0):0,(7,0):0,(8,0):0,
(0,1):0,(1,1):0,(2,1):0,(3,1):0,(4,1):0,(5,1):0,(6,1):0,(7,1):0,(8,1):0,
(0,2):0,(1,2):0,(2,2):0,(3,2):0,(4,2):0,(5,2):0,(6,2):0,(7,2):0,(8,2):0,
(0,3):0,(1,3):0,(2,3):0,(3,3):0,(4,3):0,(5,3):0,(6,3):0,(7,3):0,(8,3):0,
(0,4):0,(1,4):0,(2,4):0,(3,4):0,(4,4):0,(5,4):0,(6,4):0,(7,4):0,(8,4):0,
(0,5):0,(1,5):0,(2,5):2,(3,5):2,(4,5):2,(5,5):0,(6,5):0,(7,5):0,(8,5):0,
(0,6):2,(1,6):2,(2,6):8,(3,6):8,(4,6):8,(5,6):2,(6,6):2,(7,6):0,(8,6):0,
(0,7):2,(1,7):2,(2,7):8,(3,7):2,(4,7):2,(5,7):2,(6,7):2,(7,7):2,(8,7):2,
(0,8):8,(1,8):8,(2,8):8,(3,8):2,(4,8):2,(5,8):2,(6,8):8,(7,8):8,(8,8):8}
```
This output is based on the pattern of shifting and expanding the clusters of non-zero values observed in previous input-output pairs.
"#;

struct ImageToDictionary;

impl ImageToDictionary {
    /// Creates a python dictionary with x, y coordinates as keys and colors as values.
    /// 
    /// The `background_color` parameter is optional.
    /// This is for omitting the most popular color, which is typically the background color.
    /// But only do so if all the training pairs agree on the same color.
    /// This can reduce the amount of text outputted.
    /// 
    /// If `include_size` is false, then there is no width and height info in the dictionary.
    /// Returns a string like `{(0,0):7,(1,0):7,(2,0):9,(0,1):8,(1,1):7,(2,1):9}`
    /// 
    /// If `include_size` is true, then it will include the width and height of the image, like this
    /// `{'width':3,'height':2,(0,0):0,(1,0):1,(2,0):2,(0,1):0,(1,1):1,(2,1):2}`
    fn convert(image: &Image, include_size: bool, background_color: Option<u8>) -> anyhow::Result<String> {
        let mut items = Vec::<String>::new();
        if include_size {
            items.push(format!("'width':{}", image.width()));
            items.push(format!("'height':{}", image.height()));
        }
        if let Some(color) = background_color {
            items.push(format!("'background':{}", color));
        }
        for y in 0..image.height() {
            for x in 0..image.width() {
                let pixel = image.get(x as i32, y as i32).unwrap_or(255);
                if Some(pixel) == background_color {
                    continue;
                }
                items.push(format!("({},{}):{}", x, y, pixel));
            }
        }
        let mut s = String::from("{");
        s += &items.join(",");
        s += "}";
        Ok(s)
    }
}

struct DictionaryToImage;

impl DictionaryToImage {
    fn convert(input: &str) -> anyhow::Result<(Image, Option<String>)> {
        // Extract parameters for: `width`, `height`, `background`.
        let mut found_width: Option<u8> = None;
        let mut found_height: Option<u8> = None;
        let mut found_background: Option<u8> = None;
        for capture in EXTRACT_STRING_VALUE.captures_iter(input) {
            let capture1: &str = capture.get(1).map_or("", |m| m.as_str());
            let capture2: &str = capture.get(2).map_or("", |m| m.as_str());
            let value: u8 = capture2.parse::<u8>().context("value")?;
            match capture1 {
                "width" => {
                    found_width = Some(value);
                },
                "height" => {
                    found_height = Some(value);
                },
                "background" => {
                    found_background = Some(value);
                },
                _ => {}
            }
        }

        // Create empty image with 255 color to indicate that it has not been assigned a color yet.
        let width: u8 = found_width.context("width")?;
        let height: u8 = found_height.context("height")?;
        let fill_color: u8 = found_background.unwrap_or(255);
        let mut image: Image = Image::color(width, height, fill_color);

        // Assign pixel values
        let mut count_outside: usize = 0;
        for capture in EXTRACT_X_Y_COLOR.captures_iter(input) {
            let capture1: &str = capture.get(1).map_or("", |m| m.as_str());
            let capture2: &str = capture.get(2).map_or("", |m| m.as_str());
            let capture3: &str = capture.get(3).map_or("", |m| m.as_str());
            let x: u8 = capture1.parse::<u8>().context("x")?;
            let y: u8 = capture2.parse::<u8>().context("y")?;
            let color: u8 = capture3.parse::<u8>().context("color")?;
            if image.set(x as i32, y as i32, color).is_none() {
                count_outside += 1;
            }
        }
        let mut count_unassigned: usize = 0;
        for y in 0..image.height() {
            for x in 0..image.width() {
                let pixel: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                if pixel == 255 {
                    count_unassigned += 1;
                }
            }
        }

        let mut problems = Vec::<String>::new();
        if count_outside > 0 {
            let s: String = format!("{} pixels outside", count_outside);
            problems.push(s);
        }
        if count_unassigned > 0 {
            let s: String = format!("{} unassigned pixels", count_unassigned);
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

#[derive(Clone, Debug)]
pub struct PromptPositionDeserializer {
    pub lines: Vec<String>,
}

impl PromptPositionDeserializer {
    #[allow(dead_code)]
    pub fn reply_example1() -> String {
        MOCK_REPLY1.to_string()
    }

    fn interpret_line_and_draw(_line_index: usize, line: &str, image: &mut Image) -> anyhow::Result<()> {
        let (output_image, status) = DictionaryToImage::convert(line)?;
        if let Some(status) = status {
            println!("PromptPositionDeserializer. Problems: {}", status);
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

impl PromptDeserialize for PromptPositionDeserializer {
    fn image(&self) -> anyhow::Result<Image> {
        let mut image = Image::zero(30, 30);
        self.interpret_and_draw(&mut image);
        Ok(image)
    }

    fn status(&self) -> Option<String> {
        None
    }
}

impl TryFrom<&str> for PromptPositionDeserializer {
    type Error = anyhow::Error;

    fn try_from(multiline_text: &str) -> Result<Self, Self::Error> {
        let mut lines_with_prefix = Vec::<String>::new();
        let mut inside_code_block = false;
        let mut count_unrecognized_inside_code_block: usize = 0;
        let mut count_code_block: usize = 0;
        let mut current_line = String::new();
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
                if !current_line.is_empty() {
                    lines_with_prefix.push(current_line.to_string());
                    current_line.truncate(0);
                }
                current_line = trimmed_line.to_string();
                // lines_with_prefix.push(line.to_string());
                continue;
            }
            let has_position_symbols: bool = trimmed_line.contains("(") && trimmed_line.contains(")") && trimmed_line.contains(":");
            let has_keyword: bool = trimmed_line.contains("'width'") || trimmed_line.contains("'height'") || trimmed_line.contains("'background'");
            let has_end_of_dictionary: bool = trimmed_line.contains("}");
            if has_position_symbols || has_keyword || has_end_of_dictionary {
                if !current_line.is_empty() {
                    current_line += trimmed_line;
                    if has_end_of_dictionary {
                        lines_with_prefix.push(current_line.to_string());
                        current_line.truncate(0);
                    }
                }
                continue;
            }
            count_unrecognized_inside_code_block += 1;
        }
        if !current_line.is_empty() {
            lines_with_prefix.push(current_line.to_string());
            current_line.truncate(0);
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

#[derive(Clone, Debug)]
pub struct PromptPositionSerializer;

impl PromptSerialize for PromptPositionSerializer {
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String> {
        let task: &Task = match &task_graph.task() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("graph is not initialized with a task"));
            }
        };

        let include_size: bool = true;
        let background_color: Option<u8> = task.input_output_most_popular_color_unambiguous();

        let mut rows = Vec::<String>::new();

        rows.push("I'm doing Python experiments.\n\n".to_string());

        rows.push("These are images.".to_string());

        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("```python".to_string());
        rows.push("input = {}".to_string());
        rows.push("output = {}".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type == PairType::Test {
                continue;
            }
            if pair_index > 0 {
                rows.push("".to_string());
            }

            {
                let s0: String = ImageToDictionary::convert(&pair.input.image, include_size, background_color)?;
                let s1: String = format!("input[{}] = {}", pair_index, s0);
                rows.push(s1);
            }

            {
                let s0: String = ImageToDictionary::convert(&pair.output.image, include_size, background_color)?;
                let s1: String = format!("output[{}] = {}", pair_index, s0);
                rows.push(s1);
            }
        }
        rows.push("```".to_string());
        rows.push("".to_string());
        
        rows.push("# Task A".to_string());
        rows.push("Use at most 100 words.".to_string());
        rows.push("Think step by step.".to_string());
        rows.push("- Write notes about what shapes and patterns you observe.".to_string());
        rows.push("- The output is never the same as the input.".to_string());
        rows.push("- Is the output a cropped out area from the input.".to_string());
        rows.push("- Is the output similar to the input rotated.".to_string());
        rows.push("- Is the output similar to the input flipped.".to_string());
        rows.push("- Is the output similar to the input diagonally flipped.".to_string());
        rows.push("\n\n# Task B".to_string());
        rows.push("Use at most 300 words.".to_string());
        rows.push("Include a markdown formatted table with the most important observations about input and output images.".to_string());
        rows.push("The table has three columns: observation name, observation values, comments about the observation.".to_string());
        rows.push("The `observation values` column can contain: integers, Yes, No, Absent, IDs, shape names, decreasing order, and so on.".to_string());
        // rows.push("The integers values can be: relative movement in the x y direction, size of shape, distance from edge, spacing, and so on.".to_string());
        rows.push("Think step by step.".to_string());
        rows.push("- Count the mass of each layer.".to_string());
        rows.push("- Count how many strongly connected clusters there are inside each layer.".to_string());
        rows.push("- Is mass related to the sorting of layers.".to_string());
        rows.push("- Isolated pixels without an adjacent pixel of same layer, sometimes they change layer.".to_string());
        rows.push("- Are there horizontal lines, do they extend edge to edge.".to_string());
        rows.push("- Are there vertical lines, do they extend edge to edge.".to_string());
        rows.push("- Are there stripes with evenly spaced lines.".to_string());
        rows.push("- Are there filled rectangles.".to_string());
        rows.push("- Are there hollow boxes.".to_string());
        rows.push("- Are there L-shapes.".to_string());
        rows.push("- Are there T-shapes.".to_string());
        rows.push("- Are there H-shapes.".to_string());
        rows.push("- Are there E-shapes.".to_string());
        rows.push("- Are there Plus-shapes.".to_string());
        rows.push("- Are there Tetris-shapes.".to_string());
        rows.push("- Are there other shapes.".to_string());
        // rows.push("- Does the output have the same overall shape as the input, but with minor differences, maybe fractal.".to_string());
        // rows.push("- Is it a fractal with the input shape being reused in the output as a repeating tile.".to_string());
        // rows.push("- Does the output contain a tile that is being used a few times. Is that tile identical to an area in the input image. If so then use that input area as the tile in the output.".to_string());
        // rows.push("- Does the output contain a tile that is being used a few times. Is that tile identical to the input image.".to_string());
        // rows.push("- Does the tile come from the input.".to_string());
        rows.push("- Does the output contain a tile that is being used a few times. Is that tile identical to the input image.".to_string());
        rows.push("- What shapes are there with holes, such as boxes, where the hole is separated from the outside.".to_string());
        rows.push("- What shapes are there where the border has small holes, such as a box with 1 pixel missing in the border, so it's no longer a hole.".to_string());
        rows.push("- Is there a line connecting two landmarks, is it the shortest path.".to_string());
        rows.push("- Does shape change layer, but preserves their shape, and what may be triggering it.".to_string());
        rows.push("- Does shape move relative x,y.".to_string());
        // rows.push("- What shapes move in the x direction.".to_string());
        // rows.push("- What shapes move in the y direction.".to_string());
        rows.push("- Boolean operations may happen: xor, and, or.".to_string());
        rows.push("- Does a tiny object change layer, because it's nearest to a bigger object in that layer.".to_string());
        rows.push("\n\n# Task C".to_string());
        rows.push("Use at most 100 words.".to_string());
        rows.push("Think step by step.".to_string());
        rows.push("What are the actions that converts input to output.".to_string());
        rows.push("\n\n# Task D".to_string());
        rows.push("With the following example input, I want you to predict what the output should be.".to_string());
        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("```python".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type == PairType::Train {
                continue;
            }

            {
                let s0: String = ImageToDictionary::convert(&pair.input.image, include_size, background_color)?;
                let s1: String = format!("input[{}] = {}", pair_index, s0);
                rows.push(s1);
            }
            rows.push("```".to_string());

            rows.push("Print your reasoning before printing the code.".to_string());    
            rows.push("Don't print other markdown code blocks than the code block containing your predictions.".to_string());
            rows.push("\n\nFill your predictions into the following template and replace PREDICT with your predictions.".to_string());
            rows.push("```python".to_string());
            {
                let s1: String = format!("output[{}] = PREDICT", pair_index);
                rows.push(s1);
            }

            // Future experiment:
            // process all the test pairs. Currently it's only 1 test pair.
            break;
        }
        rows.push("```".to_string());

        Ok(rows.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::{ImageTryCreate, ImageSize};

    #[test]
    fn test_10000_image_to_dictionary_without_size() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: String = ImageToDictionary::convert(&input, false, None).expect("ok");

        // Assert
        let expected = "{(0,0):7,(1,0):7,(2,0):9,(0,1):8,(1,1):7,(2,1):9}";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_image_to_dictionary_with_size() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2,
            0, 1, 2,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: String = ImageToDictionary::convert(&input, true, None).expect("ok");

        // Assert
        let expected = "{'width':3,'height':2,(0,0):0,(1,0):1,(2,0):2,(0,1):0,(1,1):1,(2,1):2}";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_image_to_dictionary_with_background_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: String = ImageToDictionary::convert(&input, false, Some(7)).expect("ok");

        // Assert
        let expected = "{'background':7,(2,0):9,(0,1):8,(2,1):9}";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_decode_image_in_order() {
        // Arrange
        let input: &str = "{'width':3,'height':2,(0,0):7,(1,0):7,(2,0):9,(0,1):8,(1,1):7,(2,1):9}";

        // Act
        let actual = DictionaryToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_20001_decode_image_out_of_order() {
        // Arrange
        let input: &str = "{(0,1):8,(1,1):7,(2,1):9,(0,0):7,(1,0):7,(2,0):9,'height':2,'width':3}";

        // Act
        let actual = DictionaryToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_20002_decode_image_with_newlines() {
        // Arrange
        let input: &str = "{'width':3,'height':2,\n(0,0):7,(1,0):7,(2,0):9,(0,1):8,(1,1):7,\n(2,1):9}";

        // Act
        let actual = DictionaryToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_20003_decode_image_with_spaces() {
        // Arrange
        let input: &str = "{'width':3,'height': 2,\n(0, 0): 7, (1 ,0):7,(2,0):9,(0,1):8,(1,1):7,\n(2,1):9}";

        // Act
        let actual = DictionaryToImage::convert(input).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, None);
    }

    #[test]
    fn test_30000_deserialize_ok() {
        // Arrange
        let s: String = PromptPositionDeserializer::reply_example1();
        let s1: &str = &s;

        // Act
        let actual: PromptPositionDeserializer = PromptPositionDeserializer::try_from(s1).expect("ok");

        // Assert
        assert_eq!(actual.lines.len(), 1);
        let image: Image = actual.image().expect("ok");
        assert_eq!(image.size(), ImageSize::new(9, 9));
    }

    #[test]
    fn test_30001_deserialize_ok() {
        // Arrange
        let s = r#"
```python
output[4] = {'width': 6, 'height': 5, 'background': 0, (0, 0): 4, (5, 0): 4, 
             (0, 1): 2, (5, 1): 8, (0, 2): 2, (5, 2): 8, (0, 3): 2, (5, 3): 8, 
             (0, 4): 4, (5, 4): 4, (1, 3): 8, (2, 3): 8, (3, 3): 2,
             (1, 4): 8, (2, 4): 8, (3, 4): 2, (4, 4): 2}
```
        "#;

        // Act
        let actual: PromptPositionDeserializer = PromptPositionDeserializer::try_from(s).expect("ok");

        // Assert
        assert_eq!(actual.lines.len(), 1);
        let image: Image = actual.image().expect("ok");
        assert_eq!(image.size(), ImageSize::new(6, 5));
    }

    #[test]
    fn test_30002_deserialize_ok() {
        // Arrange
        let s = r#"
```python
output[4] = {
    'width': 11,
    'height': 22,
    'background': 0,
    (1, 1): 5,
    (5, 1): 5,
    (7, 9): 5,
    (9, 9): 5,
}
```
"#;

        // Act
        let actual: PromptPositionDeserializer = PromptPositionDeserializer::try_from(s).expect("ok");

        // Assert
        assert_eq!(actual.lines.len(), 1);
        let image: Image = actual.image().expect("ok");
        assert_eq!(image.size(), ImageSize::new(11, 22));
    }
}
