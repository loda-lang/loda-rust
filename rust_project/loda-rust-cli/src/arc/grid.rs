//! Detect grid patterns in images.
use super::{Histogram, Image, ImageDrawRect, ImageHistogram, ImageRotate, Rectangle};
use std::collections::HashSet;

// ARC tasks where the grid cannot be detected
// 83302e8f - missing grid_mask for train3.input.
// 97239e3d - weird grid_mask for test1.input. It's good for the training pairs.

#[derive(Clone, Debug)]
pub struct GridPattern {
    pub color: u8,

    #[allow(dead_code)]
    pub line_mask: Image,

    #[allow(dead_code)]
    pub intersection: u32,

    #[allow(dead_code)]
    pub union: u32,

    #[allow(dead_code)]
    pub jaccard_index: f32,

    #[allow(dead_code)]
    pub horizontal_line_count: u8,

    #[allow(dead_code)]
    pub horizontal_cell_count: u8,

    #[allow(dead_code)]
    pub vertical_line_count: u8,

    #[allow(dead_code)]
    pub vertical_cell_count: u8,

    // Ideas for more:
    // horizontal/vertical periodicity
    // enumerated cell objects
    // corner mask
}

#[derive(Clone, Debug, PartialEq)]
struct Candidate {
    color: u8,
    combo: Combo,

    #[allow(dead_code)]
    combo_status: ComboStatus,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Grid {
    horizontal_candidates_full: Vec<Candidate>,
    vertical_candidates_full: Vec<Candidate>,
    horizontal_candidates_partial: Vec<Candidate>,
    vertical_candidates_partial: Vec<Candidate>,

    /// Disallow patterns with the same color.
    /// 
    /// The patterns are supposed to have unique colors, otherwise sorting the patterns would be non-deterministic.
    patterns_full: Vec<GridPattern>,
    patterns_partial: Vec<GridPattern>,

    grid_found: bool,
    grid_color: u8,
    grid_with_mismatches_found: bool,
}

impl Grid {
    pub fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut instance = Self::new();
        instance.perform_analyze(image)?;

        // Future experiment:
        // detect horizontal stacks. N cells wide, 1 cell tall.
        // detect vertical stacks. 1 cell wide, N cells tall.
        // enumerate cells

        Ok(instance)
    }

    /// Is there an uninterrupted grid.
    /// 
    /// When a single color is used for both horizontal lines and vertical lines.
    /// And the lines spans from edge to edge.
    /// 
    /// Then there is a grid. And this function returns `true`.
    /// 
    /// If it's interrupted, then it's more uncertain if there is a grid.
    /// Here the function returns `false`.
    pub fn grid_found(&self) -> bool {
        self.grid_found
    }

    /// The color used for the uninterrupted grid.
    /// Then there can only be a single color for the grid.
    /// 
    /// It's not possible to have an uninterrupted grid using multiple colors.
    pub fn grid_color(&self) -> u8 {
        self.grid_color
    }

    pub fn patterns_full(&self) -> &Vec<GridPattern> {
        &self.patterns_full
    }

    #[allow(dead_code)]
    pub fn patterns_partial(&self) -> &Vec<GridPattern> {
        &self.patterns_partial
    }

    pub fn find_full_pattern_with_color(&self, color: u8) -> Option<&GridPattern> {
        for pattern in &self.patterns_full {
            if pattern.color == color {
                return Some(pattern);
            }
        }
        None
    }

    pub fn find_partial_pattern_with_color(&self, color: u8) -> Option<&GridPattern> {
        for pattern in &self.patterns_partial {
            if pattern.color == color {
                return Some(pattern);
            }
        }
        None
    }

    /// Is there a grid structure with a few mismatches.
    /// 
    /// This makes no sense for tiny images, smaller than 5 pixels.
    /// 
    /// The majority of pixels must agree on a single grid color.
    /// 
    /// The number of allowed mismatches depends on the size of the images.
    /// - For bigger images, several mismatches are allowed.
    /// - For medium sized images, fewer mismatches are allowed.
    pub fn grid_with_mismatches_found(&self) -> bool {
        self.grid_with_mismatches_found
    }

    fn new() -> Self {
        Self {
            horizontal_candidates_full: vec!(),
            vertical_candidates_full: vec!(),
            horizontal_candidates_partial: vec!(),
            vertical_candidates_partial: vec!(),
            patterns_full: vec!(),
            patterns_partial: vec!(),
            grid_found: false,
            grid_color: u8::MAX,
            grid_with_mismatches_found: false,
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

        self.update_patterns(
            image, 
            self.horizontal_candidates_full.clone(), 
            self.vertical_candidates_full.clone(),
            true,
        )?;

        self.update_patterns(
            image, 
            self.horizontal_candidates_partial.clone(), 
            self.vertical_candidates_partial.clone(),
            false,
        )?;

        // The patterns are supposed to have unique colors.
        // Thus sorting by color should be deterministic.
        self.patterns_full.sort_unstable_by_key(|k| k.color);
        self.patterns_partial.sort_unstable_by_key(|k| k.color);

        Ok(())
    }

    fn update_patterns(&mut self, image: &Image, horizontal_candidates: Vec<Candidate>, vertical_candidates: Vec<Candidate>, is_full: bool) -> anyhow::Result<()> {
        let mut candidate_colors = Histogram::new();
        for candidate in &horizontal_candidates {
            candidate_colors.increment(candidate.color);
        }
        for candidate in &vertical_candidates {
            candidate_colors.increment(candidate.color);
        }
        let mut grid_found = false;
        let mut grid_color = u8::MAX;
        let mut grid_with_mismatches_found = false;
        for (_count, color) in candidate_colors.pairs_descending() {
            let candidate0: Option<&Candidate> = horizontal_candidates.iter().find(|candidate| candidate.color == color);
            let candidate1: Option<&Candidate> = vertical_candidates.iter().find(|candidate| candidate.color == color);
            let mut mask = Image::zero(image.width(), image.height());
            let mut horizontal_lines = false;
            let mut vertical_lines = false;
            let mut horizontal_line_count: u8 = 0;
            let mut horizontal_cell_count: u8 = 1;
            let mut vertical_line_count: u8 = 0;
            let mut vertical_cell_count: u8 = 1;
            if let Some(candidate) = candidate1 {
                (horizontal_line_count, horizontal_cell_count) = Self::draw_columns(&mut mask, candidate)?;
                if candidate.combo_status.line_incorrect == 0 && candidate.combo_status.cell_incorrect == 0 {
                    horizontal_lines = true;
                }
            }
            if let Some(candidate) = candidate0 {
                (vertical_line_count, vertical_cell_count) = Self::draw_rows(&mut mask, candidate)?;
                if candidate.combo_status.line_incorrect == 0 && candidate.combo_status.cell_incorrect == 0 {
                    vertical_lines = true;
                }
            }
            let cell_count: u16 = (horizontal_cell_count as u16) * (vertical_cell_count as u16);
            if cell_count < 2 {
                continue;
            }
            let overlap_histogram: Histogram = image.histogram_with_mask(&mask)?;
            let intersection: u32 = overlap_histogram.get(color);
            let union: u32 = overlap_histogram.sum();
            // println!("is_full: {} intersection: {} union: {}", is_full, intersection, union);
            if intersection == 0 || union == 0 {
                continue;
            }

            // println!("horizontal_lines: {}", horizontal_lines);
            // println!("vertical_lines: {}", vertical_lines);

            let jaccard_index: f32 = (intersection as f32) / (union as f32);
            // println!("jaccard_index: {}", jaccard_index);
            if is_full == false && jaccard_index < 0.5 {
                // The grid pattern has low similarity with the image, ignoring.
                // println!("ignoring grid that has low similarity with the image");
                continue;
            }

            // println!("color: {} mask: {:?}", color, mask);
            let pattern = GridPattern {
                color,
                line_mask: mask,
                intersection,
                union,
                jaccard_index,
                horizontal_line_count,
                horizontal_cell_count,
                vertical_line_count,
                vertical_cell_count,
            };
            if is_full {
                self.patterns_full.push(pattern);
            } else {
                self.patterns_partial.push(pattern);
            }

            if horizontal_lines && vertical_lines {
                if intersection == union {
                    grid_found = true;
                    grid_color = color;
                } else {
                    grid_with_mismatches_found = true;
                }
            }
        }

        // Only update grid_found when processing "full grid". Don't update grid_found when analyzing partial patterns
        if is_full {
            self.grid_found = grid_found;
            self.grid_color = grid_color;
        }

        // Only update when processing "partial grid patterns". Don't update when processing "full grid". 
        if !is_full {
            self.grid_with_mismatches_found = grid_with_mismatches_found;
        }

        Ok(())
    }

    fn draw_columns(result_image: &mut Image, candidate: &Candidate) -> anyhow::Result<(u8, u8)> {
        let mut x: i16 = candidate.combo.initial_position;
        let width: i16 = result_image.width() as i16;
        let mut mask: Image = result_image.clone();
        let mut line_count: u8 = 0;
        let mut cell_count: u8 = 0;
        'outer: for _ in 0..30 {
            let mut line_count_increment: u8 = 1;
            for _ in 0..candidate.combo.line_size {
                if x >= 0 && x < width {
                    line_count += line_count_increment;
                    line_count_increment = 0;

                    let xx = (x & 255) as u8;
                    let r = Rectangle::new(xx, 0, 1, result_image.height());
                    mask = mask.fill_inside_rect(r, 1)?;
                }
                x += 1;
                if x >= width {
                    break 'outer;
                }
            }
            let mut cell_count_increment: u8 = 1;
            for _ in 0..candidate.combo.cell_size {
                if x >= 0 && x < width {
                    cell_count += cell_count_increment;
                    cell_count_increment = 0;
                }
                x += 1;
                if x >= width {
                    break 'outer;
                }
            }
        }
        result_image.set_image(mask);
        Ok((line_count, cell_count))
    }

    fn draw_rows(result_image: &mut Image, candidate: &Candidate) -> anyhow::Result<(u8, u8)> {
        let mut y: i16 = candidate.combo.initial_position;
        let height: i16 = result_image.height() as i16;
        let mut mask: Image = result_image.clone();
        let mut line_count: u8 = 0;
        let mut cell_count: u8 = 0;
        'outer: for _ in 0..30 {
            let mut line_count_increment: u8 = 1;
            for _ in 0..candidate.combo.line_size {
                if y >= 0 && y < height {
                    line_count += line_count_increment;
                    line_count_increment = 0;

                    let yy = (y & 255) as u8;
                    let r = Rectangle::new(0, yy, result_image.width(), 1);
                    mask = mask.fill_inside_rect(r, 1)?;
                }
                y += 1;
                if y >= height {
                    break 'outer;
                }
            }
            let mut cell_count_increment: u8 = 1;
            for _ in 0..candidate.combo.cell_size {
                if y >= 0 && y < height {
                    cell_count += cell_count_increment;
                    cell_count_increment = 0;
                }
                y += 1;
                if y >= height {
                    break 'outer;
                }
            }
        }
        result_image.set_image(mask);
        Ok((line_count, cell_count))
    }

    fn perform_analyze_with_multiple_colors(&mut self, image: &Image, is_horizontal: bool) -> anyhow::Result<()> {
        let rows: Vec<Histogram> = image.histogram_rows();
        let mut full_row_colors = Vec::<Option<u8>>::new();
        let mut partial_row_colors = Vec::<Option<u8>>::new();
        let mut rows_histogram = Histogram::new();
        for (_y, histogram) in rows.iter().enumerate() {
            let unique_colors: u32 = histogram.number_of_counters_greater_than_zero();
            if unique_colors != 1 && image.width() < 5 {
                full_row_colors.push(None);
                partial_row_colors.push(None);
                continue;
            }
            let (color, count) = match histogram.most_popular_pair_disallow_ambiguous() {
                Some(value) => value,
                None => {
                    full_row_colors.push(None);
                    partial_row_colors.push(None);
                    continue;
                }
            };

            // Detect grid and allow for some mismatches. 
            // It's a full line when it spans from edge to edge uninterrupted with just one color.
            // It's a partial line when it's interrupted and contains a few pixels with a different color.
            if count < (image.width() as u32) * 7 / 10 {
                full_row_colors.push(None);
                partial_row_colors.push(None);
                continue;
            }

            // println!("row y: {} color: {}", y, color);
            if count == image.width() as u32 {
                full_row_colors.push(Some(color));
                partial_row_colors.push(Some(color));
                rows_histogram.increment(color);
            } else {
                full_row_colors.push(None);
                partial_row_colors.push(Some(color));
                rows_histogram.increment(color);
            }
        }

        // Process full lines
        // measure spacing between the lines, thickness of lines
        let mut full_candidates = Vec::<Candidate>::new();
        for (_count, color) in rows_histogram.pairs_descending() {
            let (combo, combo_status) = match Self::measure(color, &full_row_colors) {
                Ok(value) => value,
                _ => continue
            };
            let candidate = Candidate {
                color,
                combo,
                combo_status,
            };
            full_candidates.push(candidate);
        }

        // Process partial+full lines
        // When a partial candidate is identical to an already found full_candidate, then discard the partial candidate
        let mut partial_candidates = Vec::<Candidate>::new();
        for (_count, color) in rows_histogram.pairs_descending() {
            let (combo, combo_status) = match Self::measure(color, &partial_row_colors) {
                Ok(value) => value,
                _ => continue
            };
            let candidate = Candidate {
                color,
                combo,
                combo_status,
            };
            if full_candidates.contains(&candidate) {
                // println!("partial candidate is identical to already found full candidate. Ignoring the partial candidate");
                continue;
            }
            partial_candidates.push(candidate);
        }

        if is_horizontal {
            self.horizontal_candidates_full = full_candidates;
            self.horizontal_candidates_partial = partial_candidates;
        } else {
            self.vertical_candidates_full = full_candidates;
            self.vertical_candidates_partial = partial_candidates;
        }

        Ok(())
    }

    fn measure(measure_color: u8, row_colors: &Vec<Option<u8>>) -> anyhow::Result<(Combo, ComboStatus)> {
        let mut found_max_possible_line_size: u8 = 0;
        let mut current_possible_line_size: u8 = 0;
        let mut found_max_possible_cell_size: u8 = 0;
        let mut current_possible_cell_size: u8 = 0;
        let mut positions = Vec::<u8>::new();
        let mut position_set = HashSet::<i16>::new();
        for (index, row_color) in row_colors.iter().enumerate() {
            if *row_color != Some(measure_color) {
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

        let mut best = ComboStatus {
            line_correct: 0,
            line_incorrect: u8::MAX,
            cell_correct: 0,
            cell_incorrect: u8::MAX
        };
        let mut current_error: i32 = i32::MIN;
        let mut found_combo: Option<Combo> = None;
        let max_position: i16 = ((row_colors.len() & 255) as i16) - 1;
        for cell_size in 1..=max_cell_size {
            for line_size in 1..=max_line_size {
                let periodicity_u16: u16 = (cell_size as u16) + (line_size as u16);
                let periodicity: u8 = (periodicity_u16 & 255) as u8;

                for offset in 0..periodicity {
                    let initial_position: i16 = -(offset as i16);
                    let combo = Combo {
                        initial_position,
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

#[derive(Clone, Debug, PartialEq)]
struct Combo {
    initial_position: i16, 
    line_size: u8, 
    cell_size: u8
}

#[derive(Clone, Debug, PartialEq)]
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
        let biggest_arc_grid_size: u8 = 30 * 2;
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
        let pattern: &GridPattern = instance.patterns_full.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
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
        let pattern: &GridPattern = instance.patterns_full.first().expect("GridPattern");
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
        assert_eq!(pattern.line_mask, expected);
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
        let pattern: &GridPattern = instance.patterns_full.first().expect("GridPattern");
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
        assert_eq!(pattern.line_mask, expected);
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
        let pattern: &GridPattern = instance.patterns_full.first().expect("GridPattern");
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
        assert_eq!(pattern.line_mask, expected);
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
        assert_eq!(instance.patterns_full.len(), 3);
        {
            let pattern: &GridPattern = &instance.patterns_full[1];
            let expected_pixels: Vec<u8> = vec![
                1, 1, 1, 1,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                1, 1, 1, 1,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                1, 1, 1, 1,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                1, 1, 1, 1,
                0, 0, 0, 0,
            ];
            let expected: Image = Image::try_create(4, 14, expected_pixels).expect("image");
            assert_eq!(pattern.line_mask, expected);
        }
        {
            let pattern: &GridPattern = &instance.patterns_full[2];
            let expected_pixels: Vec<u8> = vec![
                0, 0, 0, 0,
                1, 1, 1, 1,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                1, 1, 1, 1,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                1, 1, 1, 1,
                0, 0, 0, 0,
                0, 0, 0, 0,
            ];
            let expected: Image = Image::try_create(4, 14, expected_pixels).expect("image");
            assert_eq!(pattern.line_mask, expected);
        }
    }

    #[test]
    fn test_10005_split_middle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 9, 9, 7, 7, 9, 9, 9,            
            9, 9, 9, 7, 7, 9, 9, 9,            
            7, 7, 7, 7, 7, 7, 7, 7,
            9, 9, 9, 7, 7, 9, 9, 9,             
            9, 9, 9, 7, 7, 9, 9, 9,             
        ];
        let input: Image = Image::try_create(8, 5, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        let pattern: &GridPattern = instance.patterns_full.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0, 0,
            1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(8, 5, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
    }

    #[test]
    fn test_10006_detect_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 5, 0, 0, 0,
            0, 0, 0, 5, 0, 0, 0,
            0, 5, 0, 5, 0, 0, 0,
            0, 5, 0, 5, 0, 0, 0,
            0, 5, 0, 5, 0, 5, 0,
            0, 5, 0, 5, 0, 5, 0,
        ];
        let input: Image = Image::try_create(7, 7, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        let pattern: &GridPattern = instance.patterns_full.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 1, 0, 1,
        ];
        let expected: Image = Image::try_create(7, 7, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
    }

    #[test]
    fn test_10007_detect_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 8, 0, 0, 0, 0, 0,
            0, 8, 8, 0, 0, 0, 0,
            0, 0, 0, 0, 8, 8, 0,
            0, 0, 0, 0, 0, 8, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(7, 7, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        let pattern: &GridPattern = instance.patterns_full.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 1, 0, 0, 1,
            1, 0, 0, 1, 0, 0, 1,
            1, 0, 0, 1, 0, 0, 1,
            1, 0, 0, 1, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let expected: Image = Image::try_create(7, 7, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
    }

    #[test]
    fn test_10008_detect_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 2, 2,
            1, 1, 0, 2, 2,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.grid_found, true);
        assert_eq!(instance.grid_color, 0);
        assert_eq!(instance.patterns_full.len(), 1);
        let pattern: &GridPattern = instance.patterns_full.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
    }

    #[test]
    fn test_20000_partial_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 5, 2, 2,
            1, 1, 5, 2, 2,
            5, 5, 7, 5, 5,
            3, 3, 5, 4, 4,
            3, 3, 5, 4, 4,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.grid_found(), false);
        assert_eq!(instance.patterns_partial.len(), 1);
        let pattern: &GridPattern = instance.patterns_partial.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
        assert_eq!(pattern.color, 5);
    }

    #[test]
    fn test_20001_partial_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 2, 5, 2, 2,
            2, 2, 5, 2, 2,
            5, 5, 2, 5, 5,
            2, 2, 5, 2, 2,
            2, 2, 5, 2, 2,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.grid_found(), false);
        assert_eq!(instance.patterns_partial.len(), 1);
        let pattern: &GridPattern = instance.patterns_partial.first().expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
        assert_eq!(pattern.color, 5);
    }

    #[test]
    fn test_20002_partial_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            2, 2, 5, 2, 2, 2, 5,
            2, 2, 5, 2, 2, 2, 5,
            5, 5, 2, 5, 5, 5, 2,
            2, 2, 5, 2, 2, 2, 5,
            2, 2, 5, 2, 2, 2, 5,
            2, 2, 5, 2, 2, 2, 5,
            5, 5, 2, 5, 5, 5, 2,
            2, 2, 5, 2, 2, 2, 5,
        ];
        let input: Image = Image::try_create(7, 8, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.grid_found(), false);
        assert_eq!(instance.patterns_partial.len(), 2);
        let pattern: &GridPattern = instance.find_partial_pattern_with_color(5).expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0, 0, 1,
            0, 0, 1, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            0, 0, 1, 0, 0, 0, 1,
            0, 0, 1, 0, 0, 0, 1,
            0, 0, 1, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            0, 0, 1, 0, 0, 0, 1,
        ];
        let expected: Image = Image::try_create(7, 8, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
        assert_eq!(pattern.color, 5);
    }

    #[test]
    fn test_20003_partial_grid() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 2, 5, 0, 0, 2, 0, 5, 0, 0, 0,
            2, 0, 0, 0, 5, 0, 2, 0, 0, 5, 0, 0, 0,
            0, 0, 0, 0, 2, 0, 2, 0, 0, 5, 0, 0, 0,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0,
            2, 5, 5, 5, 5, 5, 5, 5, 2, 5, 5, 5, 5,
            0, 2, 0, 0, 5, 0, 2, 0, 0, 5, 0, 0, 0,
            0, 0, 2, 0, 5, 0, 0, 2, 0, 5, 0, 2, 0,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 2, 5, 5,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0,
            0, 2, 0, 0, 5, 0, 2, 0, 0, 5, 0, 0, 0,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 2,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0,
            0, 0, 0, 0, 5, 0, 0, 0, 0, 5, 2, 0, 0,
        ];
        let input: Image = Image::try_create(13, 19, pixels).expect("image");

        // Act
        let instance = Grid::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.grid_found(), true);
        assert_eq!(instance.patterns_partial.len(), 2);
        let pattern: &GridPattern = instance.find_partial_pattern_with_color(5).expect("GridPattern");
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(13, 19, expected_pixels).expect("image");
        assert_eq!(pattern.line_mask, expected);
        assert_eq!(pattern.color, 5);
    }
}
