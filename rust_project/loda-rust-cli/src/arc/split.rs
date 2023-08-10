//! Detect split views where a separator extends from edge to edge near the middle.
use super::{Histogram, Image, ImageHistogram};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct SplitCandidate {
    size_diff: u8,
    size0: u8,
    separator_size: u8,
    separator_color: u8,
    size1: u8,
}

impl SplitCandidate {
    #[cfg(test)]
    fn sizes_string(&self) -> String {
        format!("{} {} {}", self.size0, self.separator_size, self.size1)
    }
}

struct Split {
    x_candidate_vec: Vec<SplitCandidate>,
}

impl Split {
    pub fn analyze(input: &Image) -> anyhow::Result<Self> {
        let x_candidate_vec: Vec<SplitCandidate> = Self::obtain_candidate_vec(input)?;
        let instance = Split {
            x_candidate_vec,
        };
        Ok(instance)
    }

    fn obtain_candidate_vec(input: &Image) -> anyhow::Result<Vec<SplitCandidate>> {

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
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_split_separator_size1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 2, 3, 3,
        ];
        let input: Image = Image::try_create(5, 1, pixels).expect("image");

        // Act
        let instance = Split::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.x_candidate_vec.len(), 3);
        let candidate: &SplitCandidate = instance.x_candidate_vec.first().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "2 1 2");
        assert_eq!(candidate.separator_color, 2);
    }

    #[test]
    fn test_10001_split_separator_size2() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 5, 5, 2,
        ];
        let input: Image = Image::try_create(4, 1, pixels).expect("image");

        // Act
        let instance = Split::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.x_candidate_vec.len(), 3);
        let candidate: &SplitCandidate = instance.x_candidate_vec.first().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "1 2 1");
        assert_eq!(candidate.separator_color, 5);
    }

    #[test]
    fn test_10002_split_separator_size3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 5, 6, 6, 6, 1,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let instance = Split::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.x_candidate_vec.len(), 4);
        let candidate: &SplitCandidate = instance.x_candidate_vec.first().expect("SplitCandidate");
        assert_eq!(candidate.sizes_string(), "2 3 1");
        assert_eq!(candidate.separator_color, 6);
    }
}
