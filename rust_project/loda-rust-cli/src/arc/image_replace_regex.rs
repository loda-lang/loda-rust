use super::Image;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct ImageReplaceRegexToColor {
    pub regex: Regex,
    pub color: u8,
}

pub trait ImageReplaceRegex {
    /// Substitute the center pixel using regex
    /// 
    /// The minimum size is 5x5.
    /// 
    /// Substitutions only occurs inside the center area with inset 2. 
    /// 
    /// Substitutions does not occurs in the 2px border area. 
    /// 
    /// Returns the number of substitutions performed.
    fn replace_5x5_regex(&mut self, replacements: &Vec<ImageReplaceRegexToColor>, max_iterations: usize, max_substitutions: usize) -> anyhow::Result<usize>;
}

impl ImageReplaceRegex for Image {
    fn replace_5x5_regex(&mut self, replacements: &Vec<ImageReplaceRegexToColor>, max_iterations: usize, max_substitutions: usize) -> anyhow::Result<usize> {
        let verbose = false;

        let width: u8 = self.width();
        let height: u8 = self.height();
        if width < 5 || height < 5 {
            return Err(anyhow::anyhow!("too small image, must be 5x5 or bigger"));
        }
        if replacements.is_empty() {
            return Err(anyhow::anyhow!("there must be 1 or more replacements"));
        }
        if max_iterations == 0 {
            return Err(anyhow::anyhow!("max_iterations must be 1 or greater"));
        }
        if max_substitutions == 0 {
            return Err(anyhow::anyhow!("max_substitutions must be 1 or greater"));
        }

        let mut conv: [u8; 25] = [0; 25];
        let mut count_substitution: usize = 0;
        for _ in 0..max_iterations {
            let mut stop = true;

            // Traverse all the pixels in this image
            for self_y in 0..height-4 {
                for self_x in 0..width-4 {
        
                    // Collect pixels for a 5x5 convolution
                    let mut index: usize = 0;
                    for conv_y in 0..5u8 {
                        for conv_x in 0..5u8 {
                            let get_x: i32 = (self_x as i32) + (conv_x as i32);
                            let get_y: i32 = (self_y as i32) + (conv_y as i32);
                            let value: u8 = self.get(get_x, get_y)
                                .ok_or_else(|| anyhow::anyhow!("self.get({},{}) returned None", get_x, get_y))?;
                            conv[index] = value;
                            index += 1;
                        }
                    }

                    let pixels_string: String = conv.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(",");

                    // Do replacement if there is a partial match
                    // Pick the earliest rule that is satisfied.
                    // for replacement in replacements {
                    for (replacement_index, replacement) in replacements.iter().enumerate() {
                        if replacement.regex.is_match(&pixels_string) {
                            let set_x: i32 = (self_x as i32) + 2;
                            let set_y: i32 = (self_y as i32) + 2;
                            _ = self.set(set_x, set_y, replacement.color);
                            if verbose {
                                println!("replacement {} set {},{}={}", replacement_index, set_x, set_y, replacement.color);
                            }
                            count_substitution += 1;
                            if count_substitution >= max_substitutions {
                                stop = true;
                            } else {
                                stop = false;
                            }
                            break;
                        }
                    }
                }
            }

            if stop {
                break;
            }
        }
        Ok(count_substitution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_one_pattern_a() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let pattern: &str = 
        "^1,1,\\d+,\\d+,\\d+,\
        1,1,\\d+,\\d+,\\d+,\
        \\d+,\\d+,0,\\d+,\\d+,\
        \\d+,\\d+,\\d+,\\d+,\\d+,\
        \\d+,\\d+,\\d+,\\d+,\\d+$";

        let replacements: Vec<ImageReplaceRegexToColor> = vec![
            ImageReplaceRegexToColor {
                regex: Regex::new(pattern).expect("regex"),
                color: 5,
            }
        ];

        // Act
        let mut output: Image = input.clone();
        let count: usize = output.replace_5x5_regex(&replacements, 1, 1).expect("ok");

        // Assert
        assert_eq!(count, 1);
        let expected_pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 5, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10001_one_pattern_b() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 5, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let pattern: &str = 
        "^1,\\d+,\\d+,\\d+,\\d+,\
        \\d+,5,\\d+,\\d+,\\d+,\
        \\d+,\\d+,0,\\d+,\\d+,\
        \\d+,\\d+,\\d+,\\d+,\\d+,\
        \\d+,\\d+,\\d+,\\d+,\\d+$";

        let replacements: Vec<ImageReplaceRegexToColor> = vec![
            ImageReplaceRegexToColor {
                regex: Regex::new(pattern).expect("regex"),
                color: 8,
            }
        ];

        // Act
        let mut output: Image = input.clone();
        let count: usize = output.replace_5x5_regex(&replacements, 1, 1).expect("ok");

        // Assert
        assert_eq!(count, 1);
        let expected_pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 5, 0, 0, 0,
            0, 0, 8, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_10002_two_patterns() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(9, 9, pixels).expect("image");

        let pattern0: &str = 
        "^1,1,\\d+,\\d+,\\d+,\
        1,1,\\d+,\\d+,\\d+,\
        \\d+,\\d+,0,\\d+,\\d+,\
        \\d+,\\d+,\\d+,\\d+,\\d+,\
        \\d+,\\d+,\\d+,\\d+,\\d+$";

        let pattern1: &str = 
        "^1,0,\\d+,\\d+,\\d+,\
        0,1,\\d+,\\d+,\\d+,\
        \\d+,\\d+,0,\\d+,\\d+,\
        \\d+,\\d+,\\d+,\\d+,\\d+,\
        \\d+,\\d+,\\d+,\\d+,\\d+$";

        let replacements: Vec<ImageReplaceRegexToColor> = vec![
            ImageReplaceRegexToColor {
                regex: Regex::new(pattern0).expect("regex"),
                color: 1,
            },
            ImageReplaceRegexToColor {
                regex: Regex::new(pattern1).expect("regex"),
                color: 1,
            }
        ];

        // Act
        let mut output: Image = input.clone();
        let count: usize = output.replace_5x5_regex(&replacements, 5, 5).expect("ok");

        // Assert
        assert_eq!(count, 3);
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(9, 9, expected_pixels).expect("image");
        assert_eq!(output, expected);
    }
}
