use super::{Histogram, Image, ImageCompare, ImageCrop, ImageHistogram, ImageMaskCount, ImageRotate, ImageSymmetry, Rectangle, ImageMask};
use std::collections::HashSet;

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
        let mut found_max_possible_line_size: u8 = 0;
        let mut current_possible_line_size: u8 = 0;
        let mut found_max_possible_cell_size: u8 = 0;
        let mut current_possible_cell_size: u8 = 0;
        let mut positions = Vec::<u8>::new();
        let mut position_set = HashSet::<i16>::new();
        for (index, item_color) in items.iter().enumerate() {
            if *item_color != Some(color) {
                current_possible_line_size = 0;

                if current_possible_cell_size < u8::MAX {
                    current_possible_cell_size += 1;
                }
                if current_possible_cell_size > found_max_possible_cell_size {
                    found_max_possible_cell_size = current_possible_cell_size;
                }
    
                continue;
            }
            current_possible_cell_size = 0;

            let position: u8 = (index & 255) as u8;
            positions.push(position);
            position_set.insert(position as i16);
            if current_possible_line_size < u8::MAX {
                current_possible_line_size += 1;
            }
            if current_possible_line_size > found_max_possible_line_size {
                found_max_possible_line_size = current_possible_line_size;
            }
        }
        if positions.is_empty() {
            return;
        }
        if found_max_possible_line_size == 0 {
            return;
        }
        if found_max_possible_cell_size == 0 {
            return;
        }

        let max_line_size: u8 = found_max_possible_line_size;
        let max_cell_size: u8 = found_max_possible_cell_size;
        println!("color: {} positions: {:?}", color, positions);
        println!("max_line_size: {}", max_line_size);
        println!("max_cell_size: {}", max_cell_size);

        let mut position0: u8 = u8::MAX;
        for (index, position) in positions.iter().enumerate() {
            if index == 0 {
                position0 = *position;
                break;
            }
        }
        println!("position0: {}", position0);

        let mut best = ComboStatus {
            line_correct: 0,
            line_incorrect: u8::MAX,
            cell_correct: 0,
            cell_incorrect: u8::MAX
        };
        let mut current_error: i32 = i32::MIN;
        let mut found_combo: Option<Combo> = None;
        let max_position: i16 = ((items.len() & 255) as i16) - 1;
        for cell_size in 1..=max_cell_size {
            for line_size in 1..=max_line_size {
                for offset in 0..line_size {
                    let combo = Combo {
                        initial_position: -(offset as i16), 
                        line_size,
                        cell_size
                    };
                    let status: ComboStatus = combo.score(max_position, &position_set);
                    let error: i32 = status.error();
                    if error > current_error {
                        current_error = error;
                        best = status;
                        found_combo = Some(combo);
                    }
                }
            }
        }

        // TODO: pick combo with optimal score
        if let Some(combo) = &found_combo {
            println!("found combo: {:?} status: {:?} error: {}", combo, best, current_error);
        }

    }

}

#[derive(Debug)]
struct Combo {
    initial_position: i16, 
    line_size: u8, 
    cell_size: u8
}

#[derive(Debug)]
struct ComboStatus {
    line_correct: u8,
    line_incorrect: u8,
    cell_correct: u8,
    cell_incorrect: u8,
}

impl ComboStatus {
    fn error(&self) -> i32 {
        let line_correct2: u16 = (self.line_correct as u16) * (self.line_correct as u16);
        let cell_correct2: u16 = (self.cell_correct as u16) * (self.cell_correct as u16);
        let line_incorrect2: u16 = (self.line_incorrect as u16) * (self.line_incorrect as u16);
        let cell_incorrect2: u16 = (self.cell_incorrect as u16) * (self.cell_incorrect as u16);
        let sum: i32 = (line_correct2 as i32) + (cell_correct2 as i32) - (line_incorrect2 as i32) - (cell_incorrect2 as i32);
        sum
    }
}

impl Combo {
    fn score(&self, max_position: i16, position_set: &HashSet<i16>) -> ComboStatus {
        let mut line_correct: u8 = 0;
        let mut line_incorrect: u8 = 0;
        let mut cell_correct: u8 = 0;
        let mut cell_incorrect: u8 = 0;
        let mut current_position: i16 = self.initial_position;
        let biggest_arc_grid_size: u8 = 30;
        for _ in 0..biggest_arc_grid_size {
            for _ in 0..self.line_size {
                if current_position >= 0 && current_position <= max_position {
                    if position_set.contains(&current_position) {
                        line_correct += 1;
                    } else {
                        line_incorrect += 1;
                    }
                }
                current_position += 1;
            }
            for _ in 0..self.cell_size {
                if current_position >= 0 && current_position <= max_position {
                    if !position_set.contains(&current_position) {
                        cell_correct += 1;
                    } else {
                        cell_incorrect += 1;
                    }
                }
                current_position += 1;
            }
            if current_position > max_position {
                break;
            }
        }
        let status = ComboStatus {
            line_correct,
            line_incorrect,
            cell_correct,
            cell_incorrect,
        };
        println!("score: {} {} {} -> {} {} {} {}", self.initial_position, self.line_size, self.cell_size, line_correct, line_incorrect, cell_correct, cell_incorrect);
        status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_gridsize1_cellsize1() {
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
    fn test_10001_gridsize1_cellsize3() {
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
    fn test_10002_gridsize2_cellsize1() {
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

    #[test]
    fn test_10003_gridsize3_offset2_cellsize1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 0, 1, 1, 1, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 0, 1, 1, 1, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let input: Image = Image::try_create(7, 7, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        // assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
    }
}
