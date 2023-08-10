//! Detect split views where a separator extends from edge to edge near the middle.
use super::{Histogram, Image, ImageHistogram, ImageRotate};

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

pub struct Split {
    x_candidate_vec: Vec<SplitCandidate>,
    y_candidate_vec: Vec<SplitCandidate>,
}

impl Split {
    pub fn analyze(input: &Image) -> anyhow::Result<Self> {
        let x_candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(input)?;
        let input_rotated: Image = input.rotate_cw()?;
        let y_candidate_vec: Vec<SplitCandidate> = SplitCandidate::find_candidates(&input_rotated)?;
        let instance = Split {
            x_candidate_vec,
            y_candidate_vec,
        };
        Ok(instance)
    }

    /// If there an even split with the same size on both sides, return it.
    /// 
    /// If there is a split but the separator isn't the same size on both sides, return `None`.
    /// 
    /// If there are no splits, return `None`.
    pub fn even_splitx(&self) -> Option<&SplitCandidate> {
        let candidate: &SplitCandidate = match self.x_candidate_vec.first() {
            Some(value) => value,
            None => return None,
        };
        if candidate.size_diff > 0 {
            return None;
        }
        Some(candidate)
    }

    /// If there an even split with the same size on both sides, return it.
    /// 
    /// If there is a split but the separator isn't the same size on both sides, return `None`.
    /// 
    /// If there are no splits, return `None`.
    pub fn even_splity(&self) -> Option<&SplitCandidate> {
        let candidate: &SplitCandidate = match self.y_candidate_vec.first() {
            Some(value) => value,
            None => return None,
        };
        if candidate.size_diff > 0 {
            return None;
        }
        Some(candidate)
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
    fn test_20000_splitx() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 6, 6, 0, 1,
            0, 1, 6, 6, 1, 0,
        ];
        let input: Image = Image::try_create(6, 2, pixels).expect("image");

        // Act
        let instance = Split::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.even_splity(), None);
        let candidate: &SplitCandidate = instance.even_splitx().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "2 2 2");
        assert_eq!(candidate.separator_color, 6);
    }

    #[test]
    fn test_20001_splity() {
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

        // Act
        let instance = Split::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.even_splitx(), None);
        let candidate: &SplitCandidate = instance.even_splity().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "2 2 2");
        assert_eq!(candidate.separator_color, 6);
    }
}
