use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// Extract the bounding box from strings like: `ignore_t3_l7_b7_r11_ignore`
    static ref EXTRACT_TLBR: Regex = Regex::new(
        "t(-?\\d+)_l(-?\\d+)_b(-?\\d+)_r(-?\\d+)"
    ).unwrap();

    /// Extract the `id` prefixed data from strings like: `ignore_idP3d53ef_ignore`
    static ref EXTRACT_ID: Regex = Regex::new(
        "id([A-Za-z0-9]{1,10})"
    ).unwrap();

    /// Extract the `shape` prefixed data from strings like: `ignore_shapeRectangle_ignore`
    static ref EXTRACT_SHAPE: Regex = Regex::new(
        "shape([A-Za-z0-9]{1,30})"
    ).unwrap();
}

/// XY coordinates for Top-Left corner and Bottom-Right corner. Aka. `TLBR`.
#[derive(Clone, Debug)]
struct TLBR {
    top: i8,
    left: i8,
    bottom: i8,
    right: i8,
}

impl TryFrom<&str> for TLBR {
    type Error = anyhow::Error;

    /// Extract the bounding box from strings like: `ignore_t3_l7_b7_r11_ignore`
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_TLBR;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract TLBR from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let capture2: &str = captures.get(2).map_or("", |m| m.as_str());
        let capture3: &str = captures.get(3).map_or("", |m| m.as_str());
        let capture4: &str = captures.get(4).map_or("", |m| m.as_str());
        let top = capture1.parse::<i8>()?;
        let left = capture2.parse::<i8>()?;
        let bottom = capture3.parse::<i8>()?;
        let right = capture4.parse::<i8>()?;
        let instance = Self {
            top,
            left,
            bottom,
            right,
        };
        Ok(instance)
    }
}

/// The `FieldID` holds the obfuscated color value.
///
/// The non-obfuscated color value didn't work with the language models I have tried. 
/// Often the language model would interpret the color as an integer value or RGB value.
/// In ARC the color is an opaque value that has no meaning other than being a symbol identifier,
/// that uniquely identifies each color.
#[derive(Clone, Debug)]
struct FieldId {
    id: String,
}

impl TryFrom<&str> for FieldId {
    type Error = anyhow::Error;

    /// Extract the `id` prefixed data from strings like: `ignore_idP3d53ef_ignore`
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_ID;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract ID from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let instance = Self {
            id: capture1.to_string(),
        };
        Ok(instance)
    }
}

/// The `FieldShape` holds the shape type.
#[derive(Clone, Debug)]
struct FieldShape {
    shape_name: String,
}

impl TryFrom<&str> for FieldShape {
    type Error = anyhow::Error;

    /// Extract the `shape` prefixed data from strings like: `ignore_shapeRectangle_ignore`
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_SHAPE;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract SHAPE from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let instance = Self {
            shape_name: capture1.to_string(),
        };
        Ok(instance)
    }
}

#[derive(Clone, Debug)]
struct ParseNaturalLanguage {
    lines: Vec<String>,
}

impl ParseNaturalLanguage {
    fn interpret_line(line_index: usize, line: &str) {
        println!("line: {}", line_index);
        if let Ok(id) = FieldId::try_from(line) {
            println!("id: {:?}", id);
        }
        if let Ok(tlbr) = TLBR::try_from(line) {
            println!("tlbr: {:?}", tlbr);
        }
        if let Ok(shape) = FieldShape::try_from(line) {
            println!("shape: {:?}", shape);
        }
    }

    fn interpret(&self) {
        for (line_index, line) in self.lines.iter().enumerate() {
            Self::interpret_line(line_index, line);
        }
    }
}

impl TryFrom<&str> for ParseNaturalLanguage {
    type Error = anyhow::Error;

    /// Extract the interesting parts from the prompt response.
    /// 
    /// The response is supposed to contain a markdown formatted text
    /// with three backticks to mark the beginning and end of a code block.
    /// The code block of interest starts with `prolog`.
    /// 
    /// Within the `prolog` code block, there is supposed to be
    /// a list of `object(input...` and `object(output...` lines.
    /// 
    /// It's the `object(output...` lines that are of interest,
    /// that gets extracted.
    fn try_from(multiline_text: &str) -> Result<Self, Self::Error> {
        let mut lines_with_prefix = Vec::<String>::new();
        let mut inside_code_block = false;
        let mut count_unrecognized_inside_code_block: usize = 0;
        let mut count_code_block: usize = 0;
        for line in multiline_text.split("\n") {
            let trimmed_line: &str = line.trim();
            if trimmed_line.contains("```prolog") {
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
            if trimmed_line.starts_with("%") {
                continue;
            }
            if trimmed_line.starts_with("object(input") {
                continue;
            }
            if trimmed_line.starts_with("object(output") {
                lines_with_prefix.push(line.to_string());
                continue;
            }
            count_unrecognized_inside_code_block += 1;
        }
        if count_code_block == 0 {
            anyhow::bail!("No code block found. Expected a code block starting with 3 backticks and prolog.");
        }
        if count_code_block >= 2 {
            anyhow::bail!("Multiple code blocks found. Expected just one code block starting with 3 backticks and prolog.");
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

#[cfg(test)]
mod tests {
    use super::*;

    const RESPONSE1: &str = r#"
From the examples given, it appears the transformation from input to output operates as follows:

1. Each object in the input, regardless of its original shape, mass, and transformation properties, is reduced to a 1x1 rectangle in the output.

2. The mass of each object in the output is always 1, irrespective of the mass of the input object.

3. The transformation applied to the output objects is always "all".

4. The bounding coordinates of the output objects are restructured such that the top, left, bottom, and right parameters describe a 1x1 rectangle. 

5. The ordering of the objects in the output seems to be determined by the top coordinate (t), from the lowest to the highest.

Given these rules, the predicted output for Example 4 should be as follows:

```prolog
% Example 4 input grid_width12_height12
object(input4_idP48kmo7_t11_l6_b11_r6_w1_h1_mass1_shapeRectangle_scalex1_scaley1, transform(all)).
object(input4_idP48kmo7_t9_l7_b12_r9_w3_h4_mass6_shapeUnclassified_scalex1_scaley1, transform(rot90_rot270)).
object(input4_idP33ffe7_t2_l2_b3_r3_w2_h2_mass3_shapeL_scalex1_scaley1, transform(rot90_flip)).
object(input4_idP3d53ef_t6_l4_b7_r6_w3_h2_mass5_shapeL_scaleUnknown, transform(rot90_flip)).

% Example 4 output grid_width1_height3
object(output4_idP33ffe7_t1_l1_b1_r1_w1_h1_mass1_shapeRectangle_scalex1_scaley1, transform(all)).
object(output4_idP3d53ef_t2_l1_b2_r1_w1_h1_mass1_shapeRectangle_scalex1_scaley1, transform(all)).
object(output4_idP48kmo7_t3_l1_b3_r1_w1_h1_mass1_shapeRectangle_scalex1_scaley1, transform(all)).
```

Note: Even though there are two objects with the id "idP48kmo7" in the input, only one of them is represented in the output. The one with the lower 't' value is represented following the sorting rule.
"#;

    #[test]
    fn test_10000_tlbr_positive_values() {
        // Act
        let actual: TLBR = TLBR::try_from("junk_t1_l2_b3_r4_junk").expect("ok");

        // Assert
        assert_eq!(actual.top, 1);
        assert_eq!(actual.left, 2);
        assert_eq!(actual.bottom, 3);
        assert_eq!(actual.right, 4);
    }

    #[test]
    fn test_10001_tlbr_negative_values() {
        // Act
        let actual: TLBR = TLBR::try_from("junk_t-1_l-2_b-3_r-4_junk").expect("ok");

        // Assert
        assert_eq!(actual.top, -1);
        assert_eq!(actual.left, -2);
        assert_eq!(actual.bottom, -3);
        assert_eq!(actual.right, -4);
    }

    #[test]
    fn test_20000_field_id() {
        // Act
        let actual: FieldId = FieldId::try_from("junk_idP33ffe7_junk").expect("ok");

        // Assert
        assert_eq!(actual.id, "P33ffe7");
    }

    #[test]
    fn test_30000_field_shape() {
        // Act
        let actual: FieldShape = FieldShape::try_from("junk_shapeUnclassified_junk").expect("ok");

        // Assert
        assert_eq!(actual.shape_name, "Unclassified");
    }

    #[test]
    fn test_30001_field_shape() {
        // Act
        let actual: FieldShape = FieldShape::try_from("junk_shapeRectangle_junk").expect("ok");

        // Assert
        assert_eq!(actual.shape_name, "Rectangle");
    }

    #[test]
    fn test_40000_parse_ok() {
        // Act
        let actual: ParseNaturalLanguage = ParseNaturalLanguage::try_from(RESPONSE1).expect("ok");
        // actual.interpret();

        // Assert
        assert_eq!(actual.lines.len(), 3);
    }

    #[test]
    fn test_40100_parse_error() {
        // Arrange
        let s = "Text without code block\n\njunk\nignore";

        // Act
        let error = ParseNaturalLanguage::try_from(s).expect_err("is supposed to fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "No code block found. Expected a code block starting with 3 backticks and prolog.");
    }

    #[test]
    fn test_40101_parse_unrecognized_stuff_inside_code_block() {
        // Arrange
        let s = r#"
```prolog
junk1.
junk2.
```
"#;

        // Act
        let error = ParseNaturalLanguage::try_from(s).expect_err("is supposed to fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "2 unrecognized lines inside the code block");
    }
}
