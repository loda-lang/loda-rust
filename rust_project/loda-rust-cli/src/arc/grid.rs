use crate::arc::ImageDrawRect;

use super::{Histogram, Image, ImageHistogram, ImageRotate, Rectangle};
use std::collections::HashSet;

#[derive(Clone, Debug)]
struct GridPattern {
    color: u8,

    #[allow(dead_code)]
    mask: Image,

    #[allow(dead_code)]
    intersection: u32,

    #[allow(dead_code)]
    union: u32,

    #[allow(dead_code)]
    jaccard_index: f32,
}

#[derive(Clone, Debug)]
struct Candidate {
    color: u8,
    combo: Combo,

    #[allow(dead_code)]
    combo_status: ComboStatus,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Grid {
    horizontal_candidates: Vec<Candidate>,
    vertical_candidates: Vec<Candidate>,
    patterns: Vec<GridPattern>,
    grid_found: bool,
}

impl Grid {
    #[allow(dead_code)]
    pub fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut instance = Self::new();
        instance.perform_analyze(image)?;
        Ok(instance)
    }

    pub fn grid_found(&self) -> bool {
        self.grid_found
    }

    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            horizontal_candidates: vec!(),
            vertical_candidates: vec!(),
            patterns: vec!(),
            grid_found: false,
        }
    }

    fn perform_analyze(&mut self, image: &Image) -> anyhow::Result<()> {
        if image.width() < 2 || image.height() < 2 {
            // Image is too small. Must be 2x2 or bigger
            return Ok(());
        }
        let histogram: Histogram = image.histogram_all();
        let unique_colors: u32 = histogram.number_of_counters_greater_than_zero();
        if unique_colors < 2 {
            // Too few colors to draw a grid
            return Ok(());
        }
        self.perform_analyze_with_multiple_colors(image, true)?;
        let rotated_image: Image = image.rotate_cw()?;
        self.perform_analyze_with_multiple_colors(&rotated_image, false)?;

        // println!("horizontal_candidates: {:?}", self.horizontal_candidates);
        // println!("vertical_candidates: {:?}", self.vertical_candidates);

        let mut candidate_colors = Histogram::new();
        for candidate in &self.horizontal_candidates {
            candidate_colors.increment(candidate.color);
        }
        for candidate in &self.vertical_candidates {
            candidate_colors.increment(candidate.color);
        }
        let mut both_horz_vert = false;
        for (_count, color) in candidate_colors.pairs_descending() {
            let candidate0: Option<&Candidate> = self.horizontal_candidates.iter().find(|candidate| candidate.color == color);
            let candidate1: Option<&Candidate> = self.vertical_candidates.iter().find(|candidate| candidate.color == color);
            let mut mask = Image::zero(image.width(), image.height());
            let mut horizontal_lines = false;
            let mut vertical_lines = false;
            if let Some(candidate) = candidate1 {
                Self::draw_horizontal_lines(&mut mask, candidate)?;
                if candidate.combo_status.line_incorrect == 0 && candidate.combo_status.cell_incorrect == 0 {
                    horizontal_lines = true;
                }
            }
            if let Some(candidate) = candidate0 {
                Self::draw_vertical_lines(&mut mask, candidate)?;
                if candidate.combo_status.line_incorrect == 0 && candidate.combo_status.cell_incorrect == 0 {
                    vertical_lines = true;
                }
            }
            let overlap_histogram: Histogram = image.histogram_with_mask(&mask)?;
            let intersection: u32 = overlap_histogram.get(color);
            let union: u32 = overlap_histogram.sum();
            if intersection == 0 || union == 0 {
                continue;
            }

            let jaccard_index: f32 = (intersection as f32) / (union as f32);

            // println!("color: {} mask: {:?}", color, mask);
            let pattern = GridPattern {
                color,
                mask,
                intersection,
                union,
                jaccard_index,
            };
            self.patterns.push(pattern);

            if horizontal_lines && vertical_lines {
                both_horz_vert = true;
            }
        }
        self.patterns.sort_unstable_by_key(|k| k.color);
        self.grid_found = both_horz_vert;

        Ok(())
    }

    fn draw_horizontal_lines(result_image: &mut Image, candidate: &Candidate) -> anyhow::Result<()> {
        let mut x: i16 = candidate.combo.initial_position;
        let width: i16 = result_image.width() as i16;
        let mut mask: Image = result_image.clone();
        'outer: for _ in 0..30 {
            for _ in 0..candidate.combo.line_size {
                if x >= 0 && x < width {
                    let xx = (x & 255) as u8;
                    let r = Rectangle::new(xx, 0, 1, result_image.height());
                    mask = mask.fill_inside_rect(r, 1)?;
                }
                x += 1;
                if x >= width {
                    break 'outer;
                }
            }
            for _ in 0..candidate.combo.cell_size {
                x += 1;
                if x >= width {
                    break 'outer;
                }
            }
        }
        result_image.set_image(mask);
        Ok(())
    }

    fn draw_vertical_lines(result_image: &mut Image, candidate: &Candidate) -> anyhow::Result<()> {
        let mut y: i16 = candidate.combo.initial_position;
        let height: i16 = result_image.height() as i16;
        let mut mask: Image = result_image.clone();
        'outer: for _ in 0..30 {
            for _ in 0..candidate.combo.line_size {
                if y >= 0 && y < height {
                    let yy = (y & 255) as u8;
                    let r = Rectangle::new(0, yy, result_image.width(), 1);
                    mask = mask.fill_inside_rect(r, 1)?;
                }
                y += 1;
                if y >= height {
                    break 'outer;
                }
            }
            for _ in 0..candidate.combo.cell_size {
                y += 1;
                if y >= height {
                    break 'outer;
                }
            }
        }
        result_image.set_image(mask);
        Ok(())
    }

    fn perform_analyze_with_multiple_colors(&mut self, image: &Image, is_horizontal: bool) -> anyhow::Result<()> {
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

        // println!("row_colors: {:?}", row_colors);
        // println!("rows_histogram: {:?}", rows_histogram);

        // measure spacing between the lines, thickness of lines
        let mut candidates = Vec::<Candidate>::new();
        for (_count, color) in rows_histogram.pairs_descending() {
            let (combo, combo_status) = match Self::measure(color, &row_colors) {
                Ok(value) => value,
                _ => continue
            };
            let candidate = Candidate {
                color,
                combo,
                combo_status,
            };
            candidates.push(candidate);
        }

        if is_horizontal {
            self.horizontal_candidates = candidates;
        } else {
            self.vertical_candidates = candidates;
        }

        // draw grid

        // enumerate cells

        Ok(())
    }

    fn measure(color: u8, items: &Vec<Option<u8>>) -> anyhow::Result<(Combo, ComboStatus)> {
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
            return Err(anyhow::anyhow!("positions. Found none"));
        }
        if found_max_possible_line_size == 0 {
            return Err(anyhow::anyhow!("found_max_possible_line_size"));
        }
        if found_max_possible_cell_size == 0 {
            return Err(anyhow::anyhow!("found_max_possible_cell_size"));
        }

        let max_line_size: u8 = found_max_possible_line_size;
        let max_cell_size: u8 = found_max_possible_cell_size;
        // println!("color: {} positions: {:?}", color, positions);
        // println!("max_line_size: {}", max_line_size);
        // println!("max_cell_size: {}", max_cell_size);

        let mut position0: u8 = u8::MAX;
        for (index, position) in positions.iter().enumerate() {
            if index == 0 {
                position0 = *position;
                break;
            }
        }
        // println!("position0: {}", position0);

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

        // pick combo with optimal score
        let combo: Combo = match found_combo {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("unable to find a combo that fits the data"));
            }
        };
        // println!("found combo: {:?} status: {:?} error: {}", combo, best, current_error);
        Ok((combo, best))
    }

}

#[derive(Clone, Debug)]
struct Combo {
    initial_position: i16, 
    line_size: u8, 
    cell_size: u8
}

#[derive(Clone, Debug)]
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
        // println!("score: {} {} {} -> {} {} {} {}", self.initial_position, self.line_size, self.cell_size, line_correct, line_incorrect, cell_correct, cell_incorrect);
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
            9, 9, 9, 9, 9,
            9, 7, 9, 7, 9,
            9, 9, 9, 9, 9,
            9, 7, 9, 7, 9,
            9, 9, 9, 9, 9,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        let pattern: &GridPattern = instance.patterns.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(pattern.mask, expected);
    }

    #[test]
    fn test_10001_gridsize1_cellsize3() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 9, 9, 9, 5, 9, 9, 9, 5, 9, 9, 9, 5,
            5, 9, 9, 9, 5, 9, 9, 9, 5, 9, 9, 9, 5,
            5, 9, 9, 9, 5, 9, 9, 9, 5, 9, 9, 9, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 9, 9, 9, 5, 9, 9, 9, 5, 9, 9, 9, 5,
            5, 9, 9, 9, 5, 9, 9, 9, 5, 9, 9, 9, 5,
            5, 9, 9, 9, 5, 9, 9, 9, 5, 9, 9, 9, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        ];
        let input: Image = Image::try_create(13, 9, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        let pattern: &GridPattern = instance.patterns.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(13, 9, expected_pixels).expect("image");
        assert_eq!(pattern.mask, expected);
    }

    #[test]
    fn test_10002_gridsize2_cellsize1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 9, 7, 7, 9, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 9, 7, 7, 9, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7,
        ];
        let input: Image = Image::try_create(8, 8, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        let pattern: &GridPattern = instance.patterns.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(8, 8, expected_pixels).expect("image");
        assert_eq!(pattern.mask, expected);
    }

    #[test]
    fn test_10003_gridsize3_offset2_cellsize1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(7, 7, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        let pattern: &GridPattern = instance.patterns.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 0, 1, 1, 1, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 0, 1, 1, 1, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(7, 7, expected_pixels).expect("image");
        assert_eq!(pattern.mask, expected);
    }

    #[test]
    fn test_10004_two_grids_with_different_size() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 1, 1,
            2, 2, 2, 2,
            0, 0, 0, 0,
            0, 0, 0, 0,
            1, 1, 1, 1,
            0, 0, 0, 0,
            2, 2, 2, 2,
            0, 0, 0, 0,
            1, 1, 1, 1,
            0, 0, 0, 0,
            0, 0, 0, 0,
            2, 2, 2, 2,
            1, 1, 1, 1,
            0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(4, 14, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        // assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
    }
}
