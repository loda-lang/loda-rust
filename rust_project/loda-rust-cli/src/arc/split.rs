//! Detect split views where a separator extends from edge to edge near the middle.
use super::{Histogram, Image, ImageHistogram, ImageRotate};
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct SplitCandidate {
    pub size_diff: u8,
    pub size0: u8,
    pub separator_size: u8,
    pub separator_color: u8,
    pub size1: u8,
}

impl SplitCandidate {
    fn find_candidates(input: &Image) -> anyhow::Result<Vec<SplitCandidate>> {
        // Loop over all the columns and check if a column uses a single color.
        let mut candidates = Vec::<SplitCandidate>::new();
        let histogram_vec: Vec<Histogram> = input.histogram_columns();
        let mut last_separator_color: Option<u8> = None;
        for (index, histogram) in histogram_vec.iter().enumerate() {
            // Ignore columns with 2 or more colors.
            if histogram.number_of_counters_greater_than_zero() > 1 {
                last_separator_color = None;
                continue;
            }
            // Obtain the separator color of the split.
            let separator_color: u8 = match histogram.most_popular_color_disallow_ambiguous() {
                Some(color) => color,
                None => {
                    last_separator_color = None;
                    continue;
                },
            };
            if Some(separator_color) == last_separator_color {
                // Extend the current candidate by 1 column.
                if let Some(candidate) = candidates.last_mut() {
                    candidate.separator_size += 1;
                }
            } else {
                // New candidate for a split.
                let candidate = SplitCandidate {
                    size_diff: 255,
                    size0: index.min(255) as u8,
                    separator_size: 1,
                    separator_color,
                    size1: 0,
                };
                candidates.push(candidate);
                last_separator_color = Some(separator_color);
            }
        }

        let mut candidates2 = Vec::<SplitCandidate>::new();

        for candidate in candidates {
            // Determine size of the opposite half.
            let size1: i32 = (histogram_vec.len() as i32) - (candidate.size0 as i32) - (candidate.separator_size as i32);
            if size1 < 0 {
                continue;
            }
            let size1_u8: u8 = size1.min(255) as u8;
            // Measure the difference between the two halves. If it's evenly split, the difference is 0.
            let size_diff: u8 = (candidate.size0 as i32 - size1_u8 as i32).abs().min(255) as u8;
            let mut c = candidate.clone();
            c.size1 = size1_u8;
            c.size_diff = size_diff;
            candidates2.push(c);
        }

        // Move the most even split to the front of the vector.
        candidates2.sort();

        Ok(candidates2)
    }

    #[cfg(test)]
    fn sizes_string(&self) -> String {
        format!("{} {} {}", self.size0, self.separator_size, self.size1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvenSplit {
    pub part_size: u8,
    pub part_count: u8,
    pub separator_size: u8,
    pub separator_color: u8,
    pub separator_count: u8,
}

impl EvenSplit {
    fn sum(&self) -> u32 {
        self.part_size as u32 * self.part_count as u32 + self.separator_size as u32 * self.separator_count as u32
    }
}

impl fmt::Display for EvenSplit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}.join({}, color:{})", self.part_size, self.part_count, self.separator_size, self.separator_color)
    }
}

#[derive(Clone, Debug)]
pub struct SplitCandidateContainer {
    #[allow(dead_code)]
    candidate_vec: Vec<SplitCandidate>,
    position_to_candidate: HashMap<u8, SplitCandidate>,
    total_size: u8,
}

impl SplitCandidateContainer {
    fn analyze(input: &Image) -> anyhow::Result<Self> {
        let candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(input)?;

        let mut position_to_candidate = HashMap::<u8, SplitCandidate>::new();
        for candidate in &candidate_vec {
            for index in 0..candidate.separator_size {
                let position: u16 = (index as u16) + (candidate.size0 as u16);
                if position >= 255 {
                    continue;
                }
                position_to_candidate.insert(position as u8, candidate.clone());
            }
        }

        let instance = SplitCandidateContainer {
            candidate_vec,
            position_to_candidate,
            total_size: input.width(),
        };
        Ok(instance)
    }

    /// Split the image into `n` parts.
    /// 
    /// Determines if the image has `separator lines` spaced evenly across the image.
    fn even_split(&self, n: u8) -> anyhow::Result<EvenSplit> {
        if n < 2 {
            return Err(anyhow::anyhow!("Expected 2 or more. Cannot split into {} parts", n));
        }
        let mut separator_color: u8 = 255;
        let mut separator_size: u8 = 255;
        let mut part_size: u8 = 255;
        let mut used_size: u16 = 0;
        let mut separator_count: u8 = 0;
        for i in 1..n {
            let position_u32: u32 = ((self.total_size as u32) * (i as u32)) / (n as u32);
            if position_u32 >= 255 {
                return Err(anyhow::anyhow!("Position {} is out of bounds", position_u32));
            }
            let position: u8 = position_u32 as u8;
            let candidate: &SplitCandidate = match self.position_to_candidate.get(&position) {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("No candidate found for position {}", position));
                }
            };
            separator_count += 1;
            if i == 1 {
                separator_color = candidate.separator_color;
                separator_size = candidate.separator_size;
                part_size = candidate.size0;
                used_size += part_size as u16 + separator_size as u16;
                continue;
            }
            if candidate.separator_color != separator_color {
                return Err(anyhow::anyhow!("Separator color mismatch for position {}", position));
            }
            if candidate.separator_size != separator_size {
                return Err(anyhow::anyhow!("Separator size mismatch for position {}", position));
            }
            let expected_size0: u16 = used_size + part_size as u16;

            // Same gap size between the between all the separators
            if candidate.size0 as u16 != expected_size0 {
                return Err(anyhow::anyhow!("Separator is not evenly positioned. position {}", position));
            }
            used_size += part_size as u16 + separator_size as u16;
        }

        // The last part must have the same size as all the other parts
        if (self.total_size as u16) != (used_size + part_size as u16) {
            return Err(anyhow::anyhow!("Total size mismatch"));
        }
        if n as u16 != (separator_count as u16) + 1 {
            return Err(anyhow::anyhow!("Incorrect number of separators found"));
        }

        let instance = EvenSplit {
            part_size,
            part_count: n,
            separator_size,
            separator_color,
            separator_count,
        };
        Ok(instance)
    }

    /// Split the image into as many parts as possible. 
    /// Where the parts have the same size, and the separators have the same size and color.
    /// 
    /// If the splitting lines are inconsistently positioned, return `None`.
    /// 
    /// If there are no splits, return `None`.
    pub fn maximize_even_splits(&self) -> Option<EvenSplit> {
        let size: u8 = self.total_size;
        for n in (2..size).rev() {
            let split: EvenSplit = match self.even_split(n) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if split.separator_size < 1 || split.part_size < 1 {
                // Non-sense scenario - A separator line must be at least be 1px wide
                // Non-sense scenario - A part must be at least be 1px wide
                continue;
            }
            if split.part_count < 2 || split.separator_count < 1 {
                // Non-sense scenario - An image without any splits
                continue;
            }
            if split.sum() != size as u32 {
                // Non-sense scenario - The sum of the parts and separators is not the same as the image size
                continue;
            }
            return Some(split);
        }
        // No split found
        None
    }
}

#[derive(Clone, Debug)]
pub struct Split {
    pub x_container: SplitCandidateContainer,
    pub y_container: SplitCandidateContainer,
}

impl Split {
    pub fn analyze(input: &Image) -> anyhow::Result<Self> {
        let x_container = SplitCandidateContainer::analyze(input)?;
        let input_rotated: Image = input.rotate_cw()?;
        let y_container = SplitCandidateContainer::analyze(&input_rotated)?;
        let instance = Split {
            x_container,
            y_container,
        };
        Ok(instance)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_split_empty() {
        // Arrange
        let input: Image = Image::empty();

        // Act
        let candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(&input).expect("ok");

        // Assert
        assert_eq!(candidate_vec.len(), 0);
    }

    #[test]
    fn test_10001_split_separator_size1_tiny_image() {
        // Arrange
        let input: Image = Image::color(1, 1, 9);

        // Act
        let candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(&input).expect("ok");

        // Assert
        assert_eq!(candidate_vec.len(), 1);
        let candidate: &SplitCandidate = candidate_vec.first().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "0 1 0");
        assert_eq!(candidate.separator_color, 9);
    }

    #[test]
    fn test_10002_split_separator_size1_singleline() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 2, 3, 3,
        ];
        let input: Image = Image::try_create(5, 1, pixels).expect("image");

        // Act
        let candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(&input).expect("ok");

        // Assert
        assert_eq!(candidate_vec.len(), 3);
        let candidate: &SplitCandidate = candidate_vec.first().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "2 1 2");
        assert_eq!(candidate.separator_color, 2);
    }

    #[test]
    fn test_10002_split_separator_size1_multiline() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 2, 1, 0,
            0, 1, 2, 0, 1,
        ];
        let input: Image = Image::try_create(5, 2, pixels).expect("image");

        // Act
        let candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(&input).expect("ok");

        // Assert
        assert_eq!(candidate_vec.len(), 1);
        let candidate: &SplitCandidate = candidate_vec.first().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "2 1 2");
        assert_eq!(candidate.separator_color, 2);
    }

    #[test]
    fn test_10003_split_separator_size2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 5, 5, 2,
        ];
        let input: Image = Image::try_create(4, 1, pixels).expect("image");

        // Act
        let candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(&input).expect("ok");

        // Assert
        assert_eq!(candidate_vec.len(), 3);
        let candidate: &SplitCandidate = candidate_vec.first().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "1 2 1");
        assert_eq!(candidate.separator_color, 5);
    }

    #[test]
    fn test_10004_split_separator_size3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 5, 6, 6, 6, 1,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(&input).expect("ok");

        // Assert
        assert_eq!(candidate_vec.len(), 4);
        let candidate: &SplitCandidate = candidate_vec.first().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "2 3 1");
        assert_eq!(candidate.separator_color, 6);
    }

    #[test]
    fn test_20000_even_split_3parts() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 6, 1, 0, 6, 1, 0,
            0, 1, 6, 0, 1, 6, 0, 1,
        ];
        let input: Image = Image::try_create(8, 2, pixels).expect("image");

        // Act
        let instance = Split::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.even_splitx(), None);
        assert_eq!(instance.even_splity(), None);

        let actual: EvenSplit = instance.x_container.even_split(3).expect("ok");
        assert_eq!(actual.to_string(), "2x3.join(1, color:6)");
    }

    #[test]
    fn test_20001_even_split_3parts() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 7, 7, 0, 7, 7, 1,
            0, 7, 7, 1, 7, 7, 0,
        ];
        let input: Image = Image::try_create(7, 2, pixels).expect("image");

        // Act
        let instance = Split::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.even_splitx(), None);
        assert_eq!(instance.even_splity(), None);

        let actual: EvenSplit = instance.x_container.even_split(3).expect("ok");
        assert_eq!(actual.to_string(), "1x3.join(2, color:7)");
    }

    #[test]
    fn test_20002_even_split_5parts() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 7, 7, 0, 7, 7, 1, 7, 7, 0, 7, 7, 1,
            0, 7, 7, 1, 7, 7, 0, 7, 7, 1, 7, 7, 0,
        ];
        let input: Image = Image::try_create(13, 2, pixels).expect("image");

        // Act
        let instance = Split::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.even_splitx(), None);
        assert_eq!(instance.even_splity(), None);

        let actual: EvenSplit = instance.x_container.even_split(5).expect("ok");
        assert_eq!(actual.to_string(), "1x5.join(2, color:7)");
    }

    #[test]
    fn test_30000_maximize_even_splits() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 7, 0, 7, 1, 7, 0, 7, 1, 7, 0,
            0, 7, 1, 7, 0, 7, 1, 7, 0, 7, 1,
        ];
        let input: Image = Image::try_create(11, 2, pixels).expect("image");
        let split: Split = Split::analyze(&input).expect("ok");

        // Act
        let actualx: EvenSplit = split.x_container.maximize_even_splits().expect("some");
        let actualy: Option<EvenSplit> = split.y_container.maximize_even_splits();

        // Assert
        assert_eq!(actualx.to_string(), "1x6.join(1, color:7)");
        assert_eq!(actualy, None);
    }

    #[test]
    fn test_30001_maximize_even_splits() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 6, 6, 0, 1,
            0, 1, 6, 6, 1, 0,
        ];
        let input: Image = Image::try_create(6, 2, pixels).expect("image");
        let split: Split = Split::analyze(&input).expect("ok");

        // Act
        let actualx: EvenSplit = split.x_container.maximize_even_splits().expect("some");
        let actualy: Option<EvenSplit> = split.y_container.maximize_even_splits();

        // Assert
        assert_eq!(actualx.to_string(), "2x2.join(2, color:6)");
        assert_eq!(actualy, None);
    }

    #[test]
    fn test_30002_maximize_even_splits() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 
            0, 1, 
            6, 6, 
            6, 6, 
            0, 1,
            1, 0,
        ];
        let input: Image = Image::try_create(2, 6, pixels).expect("image");
        let split: Split = Split::analyze(&input).expect("ok");

        // Act
        let actualx: Option<EvenSplit> = split.x_container.maximize_even_splits();
        let actualy: EvenSplit = split.y_container.maximize_even_splits().expect("some");

        // Assert
        assert_eq!(actualx, None);
        assert_eq!(actualy.to_string(), "2x2.join(2, color:6)");
    }

    #[test]
    fn test_30003_maximize_even_splits_ignore_ambiguous() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 1, 2,
            1, 2, 1, 2,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");
        let split: Split = Split::analyze(&input).expect("ok");

        // Act
        let actualx: Option<EvenSplit> = split.x_container.maximize_even_splits();
        let actualy: Option<EvenSplit> = split.y_container.maximize_even_splits();

        // Assert
        assert_eq!(actualx, None);
        assert_eq!(actualy, None);
    }
}
