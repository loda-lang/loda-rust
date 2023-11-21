//! Fingerprint of what colors are present in the rows/columns/diagonals that gets changes between input and output.
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

    /// Fingerprint of what colors are present in DiagonalA that changes between input and output.
    #[allow(dead_code)]
    pub fn single_line_diagonal_a(&self) -> anyhow::Result<Histogram> {
        self.single_line_diagonal(false)
    }

    /// Fingerprint of what colors are present in DiagonalB that changes between input and output.
    #[allow(dead_code)]
    pub fn single_line_diagonal_b(&self) -> anyhow::Result<Histogram> {
        self.single_line_diagonal(true)
    }

    /// Fingerprint of what colors are present in DiagonalA or DiagonalB that changes between input and output.
    fn single_line_diagonal(&self, skew_reverse: bool) -> anyhow::Result<Histogram> {
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
            let mut has_change: bool = false;
            for y in 0..image_size.height {
                let mask: u8 = difference_skewed.get(x as i32, y as i32).unwrap_or(0);
                if mask == 0 || mask == magic_color {
                    continue;
                }
                has_change = true;
                break;
            }

            if let Some(histogram) = histogram_vec.get(x as usize) {
                if has_change {
                    histogram_change.add_histogram(histogram);
                } else {
                    histogram_nochange.add_histogram(histogram);
                }
            }
        }
        let mut intersection_histogram: Histogram = histogram_change.clone();
        intersection_histogram.subtract_histogram(&histogram_nochange);
        intersection_histogram.clamp01();
        Ok(intersection_histogram)
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
            7, 9, 9, 9, 9,
            9, 9, 9, 9, 7,
            9, 9, 9, 9, 2,
        ];
        let output: Image = Image::try_create(5, 3, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let actual: Histogram = instance.single_line_diagonal_a().expect("ok");

        // Assert
        let mut expected: Histogram = Histogram::new();
        expected.increment(4);
        expected.increment(6);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30001_single_line_diagonal_a() {
        // Arrange
        let input_pixels: Vec<u8> = vec![
            3, 7, 7, 7, 7, 7,
            7, 7, 7, 3, 7, 7,
            7, 7, 3, 7, 7, 7,
            7, 7, 7, 7, 7, 7,
            9, 9, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(6, 5, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            5, 7, 5, 7, 7, 7,
            7, 5, 7, 3, 7, 7,
            7, 7, 5, 7, 5, 7,
            7, 7, 7, 5, 7, 5,
            9, 9, 7, 7, 5, 7,
        ];
        let output: Image = Image::try_create(6, 5, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let actual: Histogram = instance.single_line_diagonal_a().expect("ok");

        // Assert
        let mut expected: Histogram = Histogram::new();
        expected.increment(3);
        assert_eq!(actual, expected);
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
            9, 9, 9, 9, 9,
            9, 9, 9, 9, 9,
            4, 6, 9, 9, 2,
        ];
        let output: Image = Image::try_create(5, 3, output_pixels).expect("image");

        let instance: CompareInputOutput = CompareInputOutput::create(&input, &output).expect("instance");

        // Act
        let actual: Histogram = instance.single_line_diagonal_b().expect("ok");

        // Assert
        let mut expected: Histogram = Histogram::new();
        expected.increment(4);
        expected.increment(6);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40001_single_line_diagonal_b() {
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
        let actual: Histogram = instance.single_line_diagonal_a().expect("ok");

        // Assert
        let mut expected: Histogram = Histogram::new();
        expected.increment(3);
        assert_eq!(actual, expected);
    }
}
