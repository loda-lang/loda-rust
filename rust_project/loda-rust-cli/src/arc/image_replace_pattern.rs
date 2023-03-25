use super::{Image,ImageFind,ImageOverlay};
use std::collections::HashMap;

pub trait ImageReplacePattern {
    /// Do substitutions from the dictionary
    fn replace_pattern(&mut self, replacements: &HashMap::<Image, Image>, max_iterations: usize) -> anyhow::Result<()>;
}

impl ImageReplacePattern for Image {
    fn replace_pattern(&mut self, replacements: &HashMap::<Image, Image>, max_iterations: usize) -> anyhow::Result<()> {
        if self.is_empty() {
            return Ok(());
        }
        if replacements.is_empty() {
            return Ok(());
        }
        let mut result_image: Image = self.clone();

        for _ in 0..max_iterations {
            let mut stop = true;
            for (key, value) in replacements {
                let position = result_image.find_exact(key)?;
                if let Some((x, y)) = position {
                    result_image = result_image.overlay_with_position(value, x as i32, y as i32)?;
                    stop = false;
                }
            }
            if stop {
                break;
            }
        }
        self.set_image(result_image);
        Ok(())
    }
}

// TODO: test replace_pattern
