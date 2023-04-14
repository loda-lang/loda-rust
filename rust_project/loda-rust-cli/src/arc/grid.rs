use super::{Histogram, Image, ImageCompare, ImageCrop, ImageHistogram, ImageMaskCount, ImageRotate, ImageSymmetry, Rectangle, ImageMask};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Grid {
}

impl Grid {
    #[allow(dead_code)]
    pub fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut instance = Self::new();
        instance.perform_analyze(image)?;
        Ok(instance)
    }

    #[allow(dead_code)]
    fn new() -> Self {
        Self {
        }
    }

    fn perform_analyze(&mut self, image: &Image) -> anyhow::Result<()> {
        if image.is_empty() {
            return Ok(());
        }
        let histogram: Histogram = image.histogram_all();
        let unique_colors: u32 = histogram.number_of_counters_greater_than_zero();
        match unique_colors {
            0 => {},
            1 => {},
            _ => {
                self.perform_analyze_with_multiple_colors(image)?;
            }
        }
        Ok(())
    }

    fn perform_analyze_with_multiple_colors(&mut self, image: &Image) -> anyhow::Result<()> {
        let rows: Vec<Histogram> = image.histogram_rows();
        let mut row_colors = Vec::<Option<u8>>::new();
        let mut rows_histogram = Histogram::new();
        for (_index, histogram) in rows.iter().enumerate() {
            let unique_colors: u32 = histogram.number_of_counters_greater_than_zero();
            if unique_colors != 1 {
                row_colors.push(None);
                continue;
            }
            let color: u8 = match histogram.most_popular_color_disallow_ambiguous() {
                Some(value) => value,
                None => {
                    row_colors.push(None);
                    continue;
                }
            };
            // println!("row: {} color: {}", index, color);
            row_colors.push(Some(color));
            rows_histogram.increment(color);
        }

        println!("row_colors: {:?}", row_colors);
        // println!("rows_histogram: {:?}", rows_histogram);

        // measure spacing between the lines, thickness of lines
        for (_count, color) in rows_histogram.pairs_descending() {
            Self::measure(color, &row_colors);
        }

        // draw grid

        // enumerate cells

        Ok(())
    }

    fn measure(color: u8, items: &Vec<Option<u8>>) {
        let mut found_max_possible_line_width: u8 = 0;
        let mut current_possible_line_width: u8 = 0;
        let mut positions = Vec::<u8>::new();
        for (index, item_color) in items.iter().enumerate() {
            if *item_color != Some(color) {
                current_possible_line_width = 0;
                continue;
            }
            positions.push((index & 255) as u8);
            if current_possible_line_width < u8::MAX {
                current_possible_line_width += 1;
            }
            if current_possible_line_width > found_max_possible_line_width {
                found_max_possible_line_width = current_possible_line_width;
            }
        }
        if positions.is_empty() {
            return;
        }
        if found_max_possible_line_width == 0 {
            return;
        }

        let max_possible_line_width: u8 = found_max_possible_line_width;
        println!("color: {} positions: {:?}", color, positions);
        println!("max_possible_line_width: {}", max_possible_line_width);

        let mut position0: u8 = u8::MAX;
        for (index, position) in positions.iter().enumerate() {
            if index == 0 {
                position0 = *position;
                break;
            }
        }
        println!("position0: {}", position0);

        let mut max_possible_line_width: i32 = 1;
        for line_width in (max_possible_line_width as i32)..0 {

        }

        let mut line_size: i32 = 1;
        let mut cell_size: i32 = -1;

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_grid_tiny() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        // assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
    }

    #[test]
    fn test_10001_grid_medium() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(9, 9, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        // assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
    }

    #[test]
    fn test_10002_grid_thickness2px_medium() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(8, 8, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        // assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
    }
}
