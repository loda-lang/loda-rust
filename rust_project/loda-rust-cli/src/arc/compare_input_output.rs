//! Fingerprint of what colors are present in the rows/columns that gets changes between input and output.
use super::{Histogram, Image, ImageCompare, ImageSize, ImageHistogram};
use anyhow::Context;

pub struct CompareInputOutput {
    image_size: ImageSize,
    input: Image,
    difference: Image,
}

impl CompareInputOutput {
    /// Compare input and output images.
    pub fn create(input: &Image, output: &Image) -> anyhow::Result<Self> {
        let image_size: ImageSize = input.size();
        if image_size != output.size() {
            anyhow::bail!("CompareInputOutput failed because input and output have different sizes");
        }
        if image_size.is_empty() {
            anyhow::bail!("CompareInputOutput failed because input and output are empty");
        }
        let difference: Image = input.diff(output)
            .context("CompareInputOutput failed to diff input and output")?;
        if image_size != difference.size() {
            anyhow::bail!("CompareInputOutput, difference.size inconsistent");
        }
        let instance = Self {
            image_size,
            input: input.clone(),
            difference,
        };
        Ok(instance)
    }

    /// Fingerprint of what colors are present in rows that changes between input and output.
    pub fn single_line_row(&self) -> Histogram {
        let histogram_rows: Vec<Histogram> = self.input.histogram_rows();
        let mut histogram_change = Histogram::new();
        let mut histogram_nochange = Histogram::new();
        for y in 0..self.image_size.height {
            let mut has_change_in_row: bool = false;
            for x in 0..self.image_size.width {
                let mask: u8 = self.difference.get(x as i32, y as i32).unwrap_or(0);
                if mask == 0 {
                    continue;
                }
                has_change_in_row = true;
                break;
            }

            if let Some(histogram) = histogram_rows.get(y as usize) {
                if has_change_in_row {
                    histogram_change.add_histogram(histogram);
                } else {
                    histogram_nochange.add_histogram(histogram);
                }
            }
        }
        let mut intersection_histogram: Histogram = histogram_change.clone();
        intersection_histogram.subtract_histogram(&histogram_nochange);
        intersection_histogram.clamp01();
        intersection_histogram
    }

    /// Fingerprint of what colors are present in columns that changes between input and output.
    pub fn single_line_column(&self) -> Histogram {
        let histogram_columns: Vec<Histogram> = self.input.histogram_columns();
        let mut histogram_change = Histogram::new();
        let mut histogram_nochange = Histogram::new();
        for x in 0..self.image_size.width {
            let mut has_change_in_column: bool = false;
            for y in 0..self.image_size.height {
                let mask: u8 = self.difference.get(x as i32, y as i32).unwrap_or(0);
                if mask == 0 {
                    continue;
                }
                has_change_in_column = true;
                break;
            }

            if let Some(histogram) = histogram_columns.get(x as usize) {
                if has_change_in_column {
                    histogram_change.add_histogram(histogram);
                } else {
                    histogram_nochange.add_histogram(histogram);
                }
            }
        }
        let mut intersection_histogram: Histogram = histogram_change.clone();
        intersection_histogram.subtract_histogram(&histogram_nochange);
        intersection_histogram.clamp01();
        intersection_histogram
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_single_line_row() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            9, 9, 9,
            9, 2, 9,
            9, 9, 9,
            3, 9, 9,
            9, 9, 9,
        ];
        let input: Image = Image::try_create(3, 5, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            9, 9, 9,
            9, 9, 2,
            9, 9, 9,
            9, 9, 3,
            9, 9, 9,
        ];
        let output: Image = Image::try_create(3, 5, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let actual: Histogram = instance.single_line_row();

        // Assert
        let mut expected: Histogram = Histogram::new();
        expected.increment(2);
        expected.increment(3);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_single_line_row() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            8, 3, 8, 8, 8,
            8, 8, 8, 8, 8,
            8, 8, 8, 8, 8,
            8, 8, 8, 4, 8,
            8, 3, 8, 3, 8,
            8, 8, 8, 8, 8,
        ];
        let input: Image = Image::try_create(5, 6, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            4, 8, 4, 4, 4,
            8, 8, 8, 8, 8,
            8, 8, 8, 8, 8,
            8, 8, 8, 4, 8,
            4, 8, 4, 8, 4,
            8, 8, 8, 8, 8,
        ];
        let output: Image = Image::try_create(5, 6, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let actual: Histogram = instance.single_line_row();

        // Assert
        let mut expected: Histogram = Histogram::new();
        expected.increment(3);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_single_line_column() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            9, 9, 9, 3, 9,
            9, 2, 9, 9, 9,
            9, 9, 9, 9, 9,
        ];
        let input: Image = Image::try_create(5, 3, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9,
            9, 9, 9, 9, 9,
            9, 2, 9, 3, 9,
        ];
        let output: Image = Image::try_create(5, 3, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let actual: Histogram = instance.single_line_column();

        // Assert
        let mut expected: Histogram = Histogram::new();
        expected.increment(2);
        expected.increment(3);
        assert_eq!(actual, expected);
    }
}
