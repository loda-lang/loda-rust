use regex::Regex;
use lazy_static::lazy_static;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ExtractOffsetError {
    InvalidSyntax(usize),
}

impl fmt::Display for ExtractOffsetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidSyntax(line_number) => 
                write!(f, "Invalid #offset syntax in line {}", line_number),
        }
    }
}

/// Remove rows like `#offset -123` from a LODA assembler program, and return the remaining string with the offset value if present.
pub fn extract_offset(input: &str, line_number: usize) -> Result<(String, Option<i32>), ExtractOffsetError> {
    lazy_static! {
        static ref EXTRACT_OFFSET_RE: Regex = Regex::new(
            "^#offset\\s+(0|-?[1-9]\\d*)$"
        ).unwrap();
    }

    if input.starts_with("#offset") == false {
        return Ok((String::from(input), None));
    }

    let captures = match EXTRACT_OFFSET_RE.captures(input) {
        Some(value) => value,
        None => {
            return Err(ExtractOffsetError::InvalidSyntax(line_number));
        }
    };

    let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
    let offset = match capture1.parse::<i32>() {
        Ok(value) => value,
        Err(_) => {
            return Err(ExtractOffsetError::InvalidSyntax(line_number));
        }
    };

    return Ok((String::from(capture1), Some(offset)));
}

#[cfg(test)]
mod tests {
    use super::*;

    static INPUT: &'static [&'static str] = &[
        // valid offsets
        "#offset 0",
        "#offset     1",
        "#offset\t4",
        "#offset -123",
        // invalid offsets
        "#offset -00000123",
        "#offset -0",
        "#offset 007",
        "#offset 4.5",
        "#offset",
        "#offset3",
        // not offsets
        "mov $0,1 ; comment",
        "#junk 123",
    ];

    static OUTPUT: &'static [&'static str] = &[
        // valid offsets
        "<valid-offset 0>",
        "<valid-offset 1>",
        "<valid-offset 4>",
        "<valid-offset -123>",
        // invalid offsets
        "InvalidSyntax(4)",
        "InvalidSyntax(5)",
        "InvalidSyntax(6)",
        "InvalidSyntax(7)",
        "InvalidSyntax(8)",
        "InvalidSyntax(9)",
        // not offsets
        "mov $0,1 ; comment",
        "#junk 123",
    ];

    fn process<S: AsRef<str>>(input: S, line_number: usize) -> String {
        let input = input.as_ref();
        let (remaining, offset) = match extract_offset(input, line_number) {
            Ok(value) => value,
            Err(error) => {
                return format!("{:?}", error);
            }
        };
        if let Some(offset) = offset {
            return format!("<valid-offset {}>", offset);
        }
        remaining
    }

    #[test]
    fn it_works() {
        for (index, input) in INPUT.iter().enumerate() {
            assert_eq!(process(input, index), OUTPUT[index]);
        }
    }
}
