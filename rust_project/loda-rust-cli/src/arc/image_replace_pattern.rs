use super::{Image,ImageFind,ImageOverlay};
use std::collections::HashMap;

pub trait ImageReplacePattern {
    /// Do substitutions from a dictionary.
    /// 
    /// Returns the number of substitutions that was performed.
    fn replace_pattern(&mut self, replacements: &HashMap::<Image, Image>, max_iterations: usize, max_substitutions: u16) -> anyhow::Result<u16>;
}

impl ImageReplacePattern for Image {
    fn replace_pattern(&mut self, replacements: &HashMap::<Image, Image>, max_iterations: usize, max_substitutions: u16) -> anyhow::Result<u16> {
        if self.is_empty() {
            return Err(anyhow::anyhow!("must be 1x1 or bigger"));
        }
        if replacements.is_empty() {
            return Err(anyhow::anyhow!("there must be 1 or more replacements"));
        }
        if max_iterations == 0 {
            return Err(anyhow::anyhow!("max_iterations must be 1 or more"));
        }
        if max_substitutions == 0 {
            return Err(anyhow::anyhow!("max_substitutions must be 1 or more"));
        }
        let mut result_image: Image = self.clone();
        let mut count_substitutions: u16 = 0;
        'outer: for _ in 0..max_iterations {
            let mut stop = true;
            for (key, value) in replacements {
                let position = result_image.find_exact(key)?;
                if let Some((x, y)) = position {
                    result_image = result_image.overlay_with_position(value, x as i32, y as i32)?;
                    stop = false;
                    count_substitutions += 1;
                    if count_substitutions >= max_substitutions {
                        break 'outer;
                    }
                }
            }
            if stop {
                // reached a point where no more substitutions are being performed
                break;
            }
        }
        self.set_image(result_image);
        Ok(count_substitutions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_sunshine_scenario() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 0,
            8, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 5, pixels).expect("image");

        let mut replacements = HashMap::<Image, Image>::new();
        {
            let source: Image = Image::try_create(2, 2, vec![1, 0, 0, 0]).expect("image");
            let target: Image = Image::try_create(2, 2, vec![2, 2, 2, 0]).expect("image");
            replacements.insert(source, target);
        }
        {
            let source: Image = Image::try_create(1, 1, vec![8]).expect("image");
            let target: Image = Image::try_create(1, 1, vec![3]).expect("image");
            replacements.insert(source, target);
        }

        let mut actual: Image = input.clone();

        // Act
        let count: u16 = actual.replace_pattern(&replacements, 10, 5).expect("count");

        // Assert
        assert_eq!(count, 3);
        let expected_pixels: Vec<u8> = vec![
            2, 2, 0, 0,
            2, 0, 0, 0,
            0, 0, 2, 2,
            0, 0, 2, 0,
            3, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(4, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
