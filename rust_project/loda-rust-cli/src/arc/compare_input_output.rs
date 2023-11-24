//! Heatmap of where change/nochange are likely to be present.
//! 
//! Makes a fingerprint of what colors are present in the rows/columns/diagonals that change/nochange between input and output.
//! 
//! Example when the Red color is present, then we know that the column doesn't change between input and output.
//! 
//! Example when the Blue or Yellow colors are present, then we know that DiagonalA changes between input and output.
//! 
//! Weakness: It considers the entire row/column/diagonal, so it's a very loose hint that something may be going on in this row/column/diagonal.
//! A more narrow hint would be to consider the only pixels BEFORE the Red color, or the pixels AFTER the Red colors.
//! That way it would be more narrow where the change/nochange is likely to be present.
use super::{Histogram, Image, ImageCompare, ImageSize, ImageHistogram, ImageSkew};
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

    /// Fingerprint of what colors impacts the rows.
    pub fn single_line_row(&self) -> (Histogram, Histogram) {
        let histogram_rows: Vec<Histogram> = self.input.histogram_rows();
        let mut histogram_change = Histogram::new();
        let mut histogram_nochange = Histogram::new();
        for y in 0..self.image_size.height {
            let mut is_same: bool = true;
            for x in 0..self.image_size.width {
                let mask: u8 = self.difference.get(x as i32, y as i32).unwrap_or(0);
                if mask > 0 {
                    is_same = false;
                    break;
                }
            }

            if let Some(histogram) = histogram_rows.get(y as usize) {
                if is_same {
                    histogram_nochange.add_histogram(histogram);
                } else {
                    histogram_change.add_histogram(histogram);
                }
            }
        }
        Self::remove_overlap(&histogram_change, &histogram_nochange)
    }

    /// Fingerprint of what colors impacts the columns.
    pub fn single_line_column(&self) -> (Histogram, Histogram) {
        let histogram_columns: Vec<Histogram> = self.input.histogram_columns();
        let mut histogram_change = Histogram::new();
        let mut histogram_nochange = Histogram::new();
        for x in 0..self.image_size.width {
            let mut is_same: bool = true;
            for y in 0..self.image_size.height {
                let mask: u8 = self.difference.get(x as i32, y as i32).unwrap_or(0);
                if mask > 0 {
                    is_same = false;
                    break;
                }
            }

            if let Some(histogram) = histogram_columns.get(x as usize) {
                if is_same {
                    histogram_nochange.add_histogram(histogram);
                } else {
                    histogram_change.add_histogram(histogram);
                }
            }
        }
        Self::remove_overlap(&histogram_change, &histogram_nochange)
    }

    /// Fingerprint of what colors impacts DiagonalA.
    #[allow(dead_code)]
    pub fn single_line_diagonal_a(&self) -> anyhow::Result<(Histogram, Histogram)> {
        self.single_line_diagonal(true)
    }
    
    /// Fingerprint of what colors impacts DiagonalB.
    #[allow(dead_code)]
    pub fn single_line_diagonal_b(&self) -> anyhow::Result<(Histogram, Histogram)> {
        self.single_line_diagonal(false)
    }

    /// Fingerprint of what colors impacts DiagonalA or DiagonalB.
    fn single_line_diagonal(&self, skew_reverse: bool) -> anyhow::Result<(Histogram, Histogram)> {
        let magic_color: u8 = 255;
        let difference_skewed: Image = self.difference.skew_x(magic_color, skew_reverse)?;
        let input_skewed: Image = self.input.skew_x(magic_color, skew_reverse)?;
        let mut histogram_vec: Vec<Histogram> = input_skewed.histogram_columns();
        for histogram in histogram_vec.iter_mut() {
            histogram.set_counter_to_zero(magic_color);
        }

        let image_size: ImageSize = input_skewed.size();
        if difference_skewed.size() != image_size || histogram_vec.len() != image_size.width as usize {
            anyhow::bail!("CompareInputOutput.single_line_diagonal, size inconsistent");
        }

        let mut histogram_change = Histogram::new();
        let mut histogram_nochange = Histogram::new();
        for x in 0..image_size.width {
            let mut is_same: bool = true;
            for y in 0..image_size.height {
                let mask: u8 = difference_skewed.get(x as i32, y as i32).unwrap_or(0);
                if mask == magic_color {
                    continue;
                }
                if mask > 0 {
                    is_same = false;
                    break;
                }
            }

            if let Some(histogram) = histogram_vec.get(x as usize) {
                if is_same {
                    histogram_nochange.add_histogram(histogram);
                } else {
                    histogram_change.add_histogram(histogram);
                }
            }
        }
        let (histogram0, histogram1) = Self::remove_overlap(&histogram_change, &histogram_nochange);
        Ok((histogram0, histogram1))
    }

    /// The colors that are in common between the two histograms are removed.
    fn remove_overlap(histogram_change: &Histogram, histogram_nochange: &Histogram) -> (Histogram, Histogram) {
        let histogram0: Histogram = histogram_change.subtract_clamp01(&histogram_nochange);
        let histogram1: Histogram = histogram_nochange.subtract_clamp01(&histogram_change);
        (histogram0, histogram1)
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
        let (actual_change, actual_nochange) = instance.single_line_row();

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(2);
        expected_change.increment(3);
        assert_eq!(actual_change, expected_change);
        let expected_nochange: Histogram = Histogram::new();
        assert_eq!(actual_nochange, expected_nochange);
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
        let (actual_change, actual_nochange) = instance.single_line_row();

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(3);
        assert_eq!(actual_change, expected_change);
        let mut expected_nochange: Histogram = Histogram::new();
        expected_nochange.increment(4);
        assert_eq!(actual_nochange, expected_nochange);
    }

    #[test]
    fn test_10002_single_line_row() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            5, 8, 8,
            8, 8, 8,
            8, 9, 8,
            9, 8, 8,
        ];
        let input: Image = Image::try_create(3, 4, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            5, 8, 8,
            8, 8, 8,
            8, 7, 7,
            9, 8, 8,
        ];
        let output: Image = Image::try_create(3, 4, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let (actual_change, actual_nochange) = instance.single_line_row();

        // Assert
        let expected_change: Histogram = Histogram::new();
        assert_eq!(actual_change, expected_change);
        let mut expected_nochange: Histogram = Histogram::new();
        expected_nochange.increment(5);
        assert_eq!(actual_nochange, expected_nochange);
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
        let (actual_change, actual_nochange) = instance.single_line_column();

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(2);
        expected_change.increment(3);
        assert_eq!(actual_change, expected_change);
        let expected_nochange: Histogram = Histogram::new();
        assert_eq!(actual_nochange, expected_nochange);
    }

    #[test]
    fn test_20001_single_line_column() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            7, 5, 9, 9, 9,
            9, 9, 9, 3, 2,
            9, 9, 9, 9, 2,
        ];
        let input: Image = Image::try_create(5, 3, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            7, 5, 9, 9, 9,
            9, 7, 9, 7, 2,
            9, 9, 9, 9, 2,
        ];
        let output: Image = Image::try_create(5, 3, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let (actual_change, actual_nochange) = instance.single_line_column();

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(3);
        expected_change.increment(5);
        assert_eq!(actual_change, expected_change);
        let mut expected_nochange: Histogram = Histogram::new();
        expected_nochange.increment(2);
        expected_nochange.increment(7);
        assert_eq!(actual_nochange, expected_nochange);
    }

    #[test]
    fn test_30000_single_line_diagonal_a() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            9, 9, 9, 6, 9,
            9, 4, 9, 9, 9,
            9, 9, 9, 9, 2,
        ];
        let input: Image = Image::try_create(5, 3, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9,
            9, 9, 9, 9, 9,
            4, 6, 9, 9, 2,
        ];
        let output: Image = Image::try_create(5, 3, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let (actual_change, actual_nochange) = instance.single_line_diagonal_a().expect("ok");

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(4);
        expected_change.increment(6);
        assert_eq!(actual_change, expected_change);
        let mut expected_nochange: Histogram = Histogram::new();
        expected_nochange.increment(2);
        assert_eq!(actual_nochange, expected_nochange);
    }

    #[test]
    fn test_30001_single_line_diagonal_a() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            9, 9, 3, 9, 9,
            3, 9, 9, 3, 9,
            9, 3, 9, 9, 3,
        ];
        let input: Image = Image::try_create(5, 3, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            9, 9, 3, 9, 9,
            3, 9, 9, 7, 9,
            9, 7, 9, 9, 3,
        ];
        let output: Image = Image::try_create(5, 3, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let (actual_change, actual_nochange) = instance.single_line_diagonal_a().expect("ok");

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(3);
        assert_eq!(actual_change, expected_change);
        let mut expected_nochange: Histogram = Histogram::new();
        expected_nochange.increment(9);
        assert_eq!(actual_nochange, expected_nochange);
    }

    #[test]
    fn test_30002_single_line_diagonal_a() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            7, 3, 7, 7, 7, 7,
            7, 7, 7, 3, 7, 7,
            7, 7, 3, 7, 7, 7,
            7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 9, 9,
        ];
        let input: Image = Image::try_create(6, 5, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            7, 5, 7, 7, 7, 7,
            5, 7, 7, 3, 7, 7,
            7, 7, 3, 7, 7, 7,
            7, 5, 7, 7, 7, 7,
            7, 7, 7, 7, 9, 9,
        ];
        let output: Image = Image::try_create(6, 5, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let (actual_change, actual_nochange) = instance.single_line_diagonal_a().expect("ok");

        // Assert
        let expected_change: Histogram = Histogram::new();
        assert_eq!(actual_change, expected_change);
        let expected_nochange: Histogram = Histogram::new();
        assert_eq!(actual_nochange, expected_nochange);
    }

    #[test]
    fn test_40000_single_line_diagonal_b() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            9, 9, 9, 6, 9,
            9, 4, 9, 9, 9,
            9, 9, 9, 9, 2,
        ];
        let input: Image = Image::try_create(5, 3, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            7, 9, 9, 9, 9,
            9, 9, 9, 9, 7,
            9, 9, 9, 9, 2,
        ];
        let output: Image = Image::try_create(5, 3, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let (actual_change, actual_nochange) = instance.single_line_diagonal_b().expect("ok");

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(4);
        expected_change.increment(6);
        assert_eq!(actual_change, expected_change);
        let mut expected_nochange: Histogram = Histogram::new();
        expected_nochange.increment(2);
        assert_eq!(actual_nochange, expected_nochange);
    }

    #[test]
    fn test_40001_single_line_diagonal_b() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            3, 7, 7, 7, 7, 7,
            7, 7, 7, 3, 7, 7,
            7, 7, 3, 7, 7, 6,
            7, 7, 7, 7, 6, 7,
            9, 9, 7, 6, 7, 6,
        ];
        let input: Image = Image::try_create(6, 5, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            5, 7, 5, 7, 7, 7,
            7, 5, 7, 3, 7, 7,
            7, 7, 5, 7, 5, 6,
            7, 7, 7, 5, 6, 5,
            9, 9, 7, 6, 5, 6,
        ];
        let output: Image = Image::try_create(6, 5, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let (actual_change, actual_nochange) = instance.single_line_diagonal_b().expect("ok");

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(3);
        assert_eq!(actual_change, expected_change);
        let mut expected_nochange: Histogram = Histogram::new();
        expected_nochange.increment(6);
        assert_eq!(actual_nochange, expected_nochange);
    }

    #[test]
    fn test_40002_single_line_diagonal_b() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            7, 7, 5, 7, 7, 4,
            7, 5, 7, 7, 4, 7,
            5, 7, 7, 4, 7, 7,
        ];
        let input: Image = Image::try_create(6, 3, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            7, 7, 9, 7, 7, 9,
            7, 9, 7, 7, 9, 7,
            9, 7, 7, 9, 7, 7,
        ];
        let output: Image = Image::try_create(6, 3, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let (actual_change, actual_nochange) = instance.single_line_diagonal_b().expect("ok");

        // Assert
        let mut expected_change: Histogram = Histogram::new();
        expected_change.increment(4);
        expected_change.increment(5);
        assert_eq!(actual_change, expected_change);
        let mut expected_nochange: Histogram = Histogram::new();
        expected_nochange.increment(7);
        assert_eq!(actual_nochange, expected_nochange);
    }
}
