use super::{Image, Histogram};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ColorMap {
    pub source_target_count: HashMap<(u8, u8), u16>,
}

impl ColorMap {
    pub fn empty() -> Self {
        Self {
            source_target_count: HashMap::<(u8, u8), u16>::new(),
        }
    }

    pub fn analyze(source: &Image, target: &Image) -> anyhow::Result<Self> {
        if source.size() != target.size() {
            anyhow::bail!("image0.size() != image1.size()");
        }
        if source.is_empty() {
            anyhow::bail!("The images must be 1x1 or bigger");
        }
        let width: u8 = source.width();
        let height: u8 = source.height();

        let mut source_target_count = HashMap::<(u8, u8), u16>::new();
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let source_color: u8 = source.get(x, y).unwrap_or(255);
                let target_color: u8 = target.get(x, y).unwrap_or(255);

                let key = (source_color, target_color);
                source_target_count
                    .entry(key)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
        let instance = Self {
            source_target_count
        };
        Ok(instance)
    }

    pub fn is_empty(&self) -> bool {
        self.source_target_count.is_empty()
    }

    pub fn to_vec(&self) -> Vec<(u8, u8, u16)> {
        let mut items = Vec::<(u8, u8, u16)>::new();
        for ((source_color, target_color), count) in &self.source_target_count {
            items.push((*source_color, *target_color, *count));
        }
        items.sort();
        items
    }

    /// Check if the mapping is good or bad.
    /// 
    /// Returns `true` when a source color have multiple different target colors.
    /// This is unwanted.
    /// 
    /// Returns `false` when a source color only has one target color.
    /// This is wanted.
    pub fn is_ambiguous(&self) -> bool {
        let mut histogram = Histogram::new();
        for ((source_color, _target_color), _count) in &self.source_target_count {
            histogram.increment(*source_color);
        }
        if let Some(count) = histogram.most_popular_count() {
            if count >= 2 {
                // Two or more mappings from a single source color.
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_unambiguous() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            5, 5, 4, 4,
            5, 5, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 1, 2, 2,
            1, 1, 0, 0,
            0, 0, 0, 0,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        // Act
        let actual: ColorMap = ColorMap::analyze(&input0, &input1).expect("ok");

        // Assert
        let expected: Vec<(u8, u8, u16)> = vec![
            (0, 0, 6),
            (4, 2, 2),
            (5, 1, 4),
        ];
        assert_eq!(actual.to_vec(), expected);
        assert_eq!(actual.is_ambiguous(), false);
    }

    #[test]
    fn test_10001_ambiguous() {
        // Arrange
        let pixels0: Vec<u8> = vec![
            5, 5, 4, 4,
            5, 5, 0, 0,
            0, 0, 0, 0,
        ];
        let input0: Image = Image::try_create(4, 3, pixels0).expect("image");

        let pixels1: Vec<u8> = vec![
            1, 1, 2, 2,
            1, 1, 0, 0,
            0, 0, 0, 9,
        ];
        let input1: Image = Image::try_create(4, 3, pixels1).expect("image");

        // Act
        let actual: ColorMap = ColorMap::analyze(&input0, &input1).expect("ok");

        // Assert
        let expected: Vec<(u8, u8, u16)> = vec![
            (0, 0, 5),
            (0, 9, 1),
            (4, 2, 2),
            (5, 1, 4),
        ];
        assert_eq!(actual.to_vec(), expected);
        assert_eq!(actual.is_ambiguous(), true);
    }
}
