use super::{Histogram, Image, ImageCompare, ImageCrop, ImageHistogram, ImageMaskCount, ImageRotate, ImageSymmetry, Rectangle, ImageMask};
use std::fmt;

const MAX_INSET_VALUE: u8 = 5;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Symmetry {
    pub left: u8,
    pub right: u8,
    pub horizontal_mismatches: u16,
    pub found_horizontal_symmetry: bool,
    pub horizontal_rect: Option<Rectangle>,

    pub top: u8,
    pub bottom: u8,
    pub vertical_mismatches: u16,
    pub found_vertical_symmetry: bool,
    pub vertical_rect: Option<Rectangle>,

    pub diagonal_a_found: bool,
    pub diagonal_a_x: u8,
    pub diagonal_a_y: u8,
    pub diagonal_a_size: u8,
    pub diagonal_a_mismatches_new: u16,

    pub diagonal_a_mismatches: u16,
    pub diagonal_a_is_symmetric: bool,

    pub diagonal_b_found: bool,
    pub diagonal_b_x: u8,
    pub diagonal_b_y: u8,
    pub diagonal_b_size: u8,
    pub diagonal_b_mismatches_new: u16,

    pub diagonal_b_mismatches: u16,
    pub diagonal_b_is_symmetric: bool,

    // TODO: "diagonal_a_rect" and "diagonal_b_rect"
    // pub diagonal_a_rect: Option<Rectangle>,
    // pub diagonal_b_rect: Option<Rectangle>,
    
    pub repair_color: Option<u8>,
    // Idea for more
    // repair plan for the damaged pixels
}

impl Symmetry {
    #[allow(dead_code)]
    pub fn analyze(image: &Image) -> anyhow::Result<Self> {
        let mut instance = Self::new();
        instance.perform_analyze(image)?;
        Ok(instance)
    }

    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            left: u8::MAX,
            right: u8::MAX,
            found_horizontal_symmetry: false,
            horizontal_mismatches: u16::MAX,
            horizontal_rect: None,
            top: u8::MAX,
            bottom: u8::MAX,
            found_vertical_symmetry: false,
            vertical_mismatches: u16::MAX,
            vertical_rect: None,
            diagonal_a_found: false,
            diagonal_a_x: u8::MAX,
            diagonal_a_y: u8::MAX,
            diagonal_a_size: u8::MAX,
            diagonal_a_mismatches_new: u16::MAX,
            diagonal_a_mismatches: u16::MAX,
            diagonal_a_is_symmetric: false,
            diagonal_b_found: false,
            diagonal_b_x: u8::MAX,
            diagonal_b_y: u8::MAX,
            diagonal_b_size: u8::MAX,
            diagonal_b_mismatches_new: u16::MAX,
            diagonal_b_mismatches: u16::MAX,
            diagonal_b_is_symmetric: false,
            repair_color: None,
        }
    }

    #[allow(dead_code)]
    fn horizontal_to_string(&self) -> String {
        if !self.found_horizontal_symmetry {
            return "no horizontal symmetry".to_string();
        }
        if self.horizontal_mismatches == 0 {
            return format!("horizontal symmetry, left: {} right: {}", self.left, self.right);
        }
        format!("partial horizontal symmetry, left: {} right: {} mismatches: {}", self.left, self.right, self.horizontal_mismatches)
    }

    #[allow(dead_code)]
    fn vertical_to_string(&self) -> String {
        if !self.found_vertical_symmetry {
            return "no vertical symmetry".to_string();
        }
        if self.vertical_mismatches == 0 {
            return format!("vertical symmetry, top: {} bottom: {}", self.top, self.bottom);
        }
        format!("partial vertical symmetry, top: {} bottom: {} mismatches: {}", self.top, self.bottom, self.vertical_mismatches)
    }

    #[allow(dead_code)]
    fn diagonal_a_to_string(&self) -> String {
        if !self.diagonal_a_is_symmetric {
            return "no diagonal-a symmetry".to_string();
        }
        if self.diagonal_a_mismatches == 0 {
            return "diagonal-a symmetry".to_string();
        }
        format!("partial diagonal-a symmetry, mismatches: {}", self.diagonal_a_mismatches)
    }

    #[allow(dead_code)]
    fn diagonal_b_to_string(&self) -> String {
        if !self.diagonal_b_is_symmetric {
            return "no diagonal-b symmetry".to_string();
        }
        if self.diagonal_b_mismatches == 0 {
            return "diagonal-b symmetry".to_string();
        }
        format!("partial diagonal-b symmetry, mismatches: {}", self.diagonal_b_mismatches)
    }

    #[allow(dead_code)]
    fn diagonal_a_new_to_string(&self) -> String {
        if !self.diagonal_a_found {
            return "no diagonal-a symmetry".to_string();
        }
        if self.diagonal_a_mismatches_new == 0 {
            return format!("diagonal-a symmetry, x: {} y: {} size: {}", self.diagonal_a_x, self.diagonal_a_y, self.diagonal_a_size);
        }
        format!("partial diagonal-a symmetry, x: {} y: {} size: {} mismatches: {}", self.diagonal_a_x, self.diagonal_a_y, self.diagonal_a_size, self.diagonal_a_mismatches_new)
    }

    #[allow(dead_code)]
    fn diagonal_b_new_to_string(&self) -> String {
        if !self.diagonal_b_found {
            return "no diagonal-b symmetry".to_string();
        }
        if self.diagonal_b_mismatches_new == 0 {
            return format!("diagonal-b symmetry, x: {} y: {} size: {}", self.diagonal_b_x, self.diagonal_b_y, self.diagonal_b_size);
        }
        format!("partial diagonal-b symmetry, x: {} y: {} size: {} mismatches: {}", self.diagonal_b_x, self.diagonal_b_y, self.diagonal_b_size, self.diagonal_b_mismatches_new)
    }

    fn perform_analyze(&mut self, image: &Image) -> anyhow::Result<()> {
        self.analyze_horizontal_symmetry(image)?;
        self.analyze_vertical_symmetry(image)?;
        self.suppress_false_positive(image)?;
        self.analyze_diagonal_symmetry(image)?;
        self.analyze_symmetry_diagonal_new(image)?;
        self.update_horizontal_rect(image)?;
        self.update_vertical_rect(image)?;
        self.find_repair_color(image)?;
        Ok(())
    }

    fn find_repair_color(&mut self, image: &Image) -> anyhow::Result<()> {
        let horizontal_rect: Rectangle;
        let vertical_rect: Rectangle;
        match (self.horizontal_rect, self.vertical_rect) {
            (Some(value0), Some(value1)) => {
                horizontal_rect = value0;
                vertical_rect = value1;
            },
            _ => {
                return Ok(());
            }
        };

        let overlap_rect: Rectangle = horizontal_rect.intersection(vertical_rect);
        if overlap_rect.is_empty() {
            return Ok(());
        }

        let half_width: u8 = overlap_rect.width();
        let half_height: u8 = overlap_rect.height();

        let mut histogram = Histogram::new();
        let mut repair_mask = Image::zero(image.width(), image.height());
        for y in 0..half_height {
            for x in 0..half_width {
                let x0: i32 = overlap_rect.min_x() + (x as i32);
                let x1: i32 = overlap_rect.max_x() - (x as i32);
                let y0: i32 = overlap_rect.min_y() + (y as i32);
                let y1: i32 = overlap_rect.max_y() - (y as i32);

                if x0 == x1 || y0 == y1 {
                    // Cannot agree on a single color in the center row/column.
                    continue;
                }

                let color00: u8 = image.get(x0, y0).unwrap_or(255);
                let color01: u8 = image.get(x0, y1).unwrap_or(255);
                let color10: u8 = image.get(x1, y0).unwrap_or(255);
                let color11: u8 = image.get(x1, y1).unwrap_or(255);

                histogram.reset();
                histogram.increment(color00);
                histogram.increment(color01);
                histogram.increment(color10);
                histogram.increment(color11);

                let unique_color_count: u32 = histogram.number_of_counters_greater_than_zero();
                if unique_color_count != 2 {
                    // Either all the 4 pixels agree on a single color, in which case there is nothing to be repaired.
                    //
                    // Or there are too much disagreement about what the color should be, in which case it's unclear 
                    // which pixels should be repaired.
                    continue;
                }
                let most_popular_color: u8 = match histogram.most_popular_color_disallow_ambiguous() {
                    Some(value) => value,
                    None => {
                        // Unclear what color there is agreement on. Cannot repair.
                        continue;
                    }
                };

                if color00 != most_popular_color {
                    _ = repair_mask.set(x0, y0, 1);
                }
                if color01 != most_popular_color {
                    _ = repair_mask.set(x0, y1, 1);
                }
                if color10 != most_popular_color {
                    _ = repair_mask.set(x1, y0, 1);
                }
                if color11 != most_popular_color {
                    _ = repair_mask.set(x1, y1, 1);
                }
            }
        }

        // println!("repair_mask: {:?}", repair_mask);

        let histogram2: Histogram = image.histogram_with_mask(&repair_mask)?;
        let repair_color: u8 = match histogram2.most_popular_color_disallow_ambiguous() {
            Some(value) => value,
            None => {
                // println!("histogram2: {:?}", histogram2);
                return Ok(());
            }
        };
        // println!("repair_color: {:?}", repair_color);
        self.repair_color = Some(repair_color);

        Ok(())
    }

    fn update_horizontal_rect(&mut self, image: &Image) -> anyhow::Result<()> {
        if !self.found_horizontal_symmetry {
            return Ok(());
        }
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let mut x0: i32 = r.min_x();
        let y0: i32 = r.min_y();
        let mut x1: i32 = r.max_x();
        let y1: i32 = r.max_y();
        if self.found_horizontal_symmetry {
            x0 += self.left as i32;
            x1 -= self.right as i32;
        }
        self.horizontal_rect = Rectangle::span(x0, y0, x1, y1);
        Ok(())
    }

    fn update_vertical_rect(&mut self, image: &Image) -> anyhow::Result<()> {
        if !self.found_vertical_symmetry {
            return Ok(());
        }
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let x0: i32 = r.min_x();
        let mut y0: i32 = r.min_y();
        let x1: i32 = r.max_x();
        let mut y1: i32 = r.max_y();
        if self.found_vertical_symmetry {
            y0 += self.top as i32;
            y1 -= self.bottom as i32;
        }
        self.vertical_rect = Rectangle::span(x0, y0, x1, y1);
        Ok(())
    }

    fn analyze_diagonal_symmetry(&mut self, image: &Image) -> anyhow::Result<()> {
        let two_way_symmetric: bool = self.found_horizontal_symmetry && self.found_vertical_symmetry;
        if !two_way_symmetric {
            return Ok(());
        }
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let x0: i32 = r.min_x() + self.left as i32;
        let y0: i32 = r.min_y() + self.top as i32;
        let x1: i32 = r.max_x() - self.right as i32;
        let y1: i32 = r.max_y() - self.bottom as i32;
        let crop_rect: Rectangle = match Rectangle::span(x0, y0, x1, y1) {
            Some(value) => value,
            None => {
                return Ok(());
            }
        };
        if crop_rect.width() != crop_rect.height() {
            return Ok(());
        }
        let image_cropped: Image = image.crop(crop_rect)?;
        let area: u16 = (image_cropped.width() as u16) * (image_cropped.height() as u16);

        {
            let flipped_image: Image = image_cropped.flip_diagonal_a()?;
            let diff: Image = flipped_image.diff(&image_cropped)?;
            let mismatch_count: u16 = diff.mask_count_one();
            if mismatch_count <= (area / 2) {
                self.diagonal_a_is_symmetric = true;
                self.diagonal_a_mismatches = mismatch_count;
            }
        }
        {
            let flipped_image: Image = image_cropped.flip_diagonal_b()?;
            let diff: Image = flipped_image.diff(&image_cropped)?;
            let mismatch_count: u16 = diff.mask_count_one();
            if mismatch_count <= (area / 2) {
                self.diagonal_b_is_symmetric = true;
                self.diagonal_b_mismatches = mismatch_count;
            }
        }
        Ok(())
    }

    fn analyze_symmetry_diagonal_new(&mut self, image: &Image) -> anyhow::Result<()> {
        self.analyze_symmetry_diagonal_inner(image, true)?;
        self.analyze_symmetry_diagonal_inner(image, false)?;
        Ok(())
    }

    fn analyze_symmetry_diagonal_inner(&mut self, image: &Image, is_diagonal_a: bool) -> anyhow::Result<()> {
        let min_size: u8 = image.width().min(image.height());
        let max_size: u8 = image.width().max(image.height());
        let max_minus_min: u8 = max_size - min_size;

        let mut x_iterations: u8 = 1;
        let mut y_iterations: u8 = 1;
        if image.width() > image.height() {
            x_iterations = max_minus_min + 1;
        }
        if image.width() < image.height() {
            y_iterations = max_minus_min + 1;
        }
        println!("x_iterations: {}", x_iterations);
        println!("y_iterations: {}", y_iterations);

        let area: u16 = (min_size as u16) * (min_size as u16);
        let limit: u16 = area / 2;
        println!("limit: {}", limit);

        let mut found: bool = false;
        let mut found_x: u8 = u8::MAX;
        let mut found_y: u8 = u8::MAX;
        let mut found_mismatches: u16 = u16::MAX;

        for x in 0..x_iterations {
            for y in 0..y_iterations {
                let rect: Rectangle = Rectangle::new(x, y, min_size, min_size);
                let image_cropped: Image = image.crop(rect)?;

                let flipped_image: Image = match is_diagonal_a {
                    true => {
                        image_cropped.flip_diagonal_a()?
                    },
                    false => {
                        image_cropped.flip_diagonal_b()?      
                    }
                };
                let diff: Image = flipped_image.diff(&image_cropped)?;
                let mismatch_count: u16 = diff.mask_count_one();
                // println!("x: {} y: {} mismatches: {}", x, y, mismatch_count);
                println!("x: {} y: {} diff: {:?}", x, y, diff);
                if mismatch_count > limit {
                    println!("x: {} y: {} mismatches: {}  ignoring", x, y, mismatch_count);
                    continue;
                }
                println!("x: {} y: {} mismatches: {}", x, y, mismatch_count);

                if !found {
                    found = true;
                    found_mismatches = mismatch_count;
                    found_x = x;
                    found_y = y;
                    continue;
                }
        
                if found_mismatches < mismatch_count {
                    continue;
                }
                found_mismatches = mismatch_count;
                found_x = x;
                found_y = y;
            }
        }

        if !found {
            println!("did not find diagonal");
            return Ok(());
        }
        if is_diagonal_a {
            println!("found diagonal_a. x: {} y: {} mismatches: {}", found_x, found_y, found_mismatches);
            self.diagonal_a_found = found;
            self.diagonal_a_x = found_x;
            self.diagonal_a_y = found_y;
            self.diagonal_a_size = min_size;
            self.diagonal_a_mismatches_new = found_mismatches;
        } else {
            println!("found diagonal_b. x: {} y: {} mismatches: {}", found_x, found_y, found_mismatches);
            self.diagonal_b_found = found;
            self.diagonal_b_x = found_x;
            self.diagonal_b_y = found_y;
            self.diagonal_b_size = min_size;
            self.diagonal_b_mismatches_new = found_mismatches;
        }

        Ok(())
    }

    fn suppress_false_positive(&mut self, image: &Image) -> anyhow::Result<()> {
        let two_way_symmetric: bool = self.found_horizontal_symmetry && self.found_vertical_symmetry;
        if !two_way_symmetric {
            return Ok(());
        }
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let x0: i32 = r.min_x() + self.left as i32;
        let y0: i32 = r.min_y() + self.top as i32;
        let x1: i32 = r.max_x() - self.right as i32;
        let y1: i32 = r.max_y() - self.bottom as i32;
        let crop_rect: Rectangle = match Rectangle::span(x0, y0, x1, y1) {
            Some(value) => value,
            None => {
                return Ok(());
            }
        };
        let cropped_image: Image = image.crop(crop_rect)?;
        let histogram: Histogram = cropped_image.histogram_all();
        if histogram.number_of_counters_greater_than_zero() >= 2 {
            return Ok(());
        }

        // The cropped area is a single color, all the content has been trimmed off.
        // This is symmetric in a terrible way, not useful. Let's ignore this kind of symmetry.
        self.found_horizontal_symmetry = false;
        self.found_vertical_symmetry = false;
        Ok(())
    }

    #[allow(dead_code)]
    fn analyze_horizontal_symmetry(&mut self, image: &Image) -> anyhow::Result<()> {
        self.analyze_horizontal_symmetry_inner(image, false)
    }

    #[allow(dead_code)]
    fn analyze_vertical_symmetry(&mut self, image: &Image) -> anyhow::Result<()> {
        let image_rotated: Image = image.rotate_ccw()?;
        self.analyze_horizontal_symmetry_inner(&image_rotated, true)
    }

    fn analyze_horizontal_symmetry_inner(&mut self, image: &Image, should_update_vertical_data: bool) -> anyhow::Result<()> {
        let r = Rectangle::new(0, 0, image.width(), image.height());
        let mut found: bool = false;
        let mut found_left: u8 = u8::MAX;
        let mut found_right: u8 = u8::MAX;
        let mut found_mismatches: u16 = u16::MAX;
        let part_of_width: u8 = (image.width() / 3) + 1;
        let max_inset: u8 = part_of_width.min(MAX_INSET_VALUE);
        for j in 0..max_inset {
            for i in 0..2 {
                // Only once try out the left=0 and right=0. Second time ignore this combo.
                if i == 1 && j == 0 {
                    continue;
                }

                // Alternate between inset left and inset right
                let left: u8;
                let right: u8;
                if i == 0 {
                    left = 0;
                    right = j;
                } else {
                    left = j;
                    right = 0;
                }

                let x0: i32 = r.min_x() + (left as i32);
                let x1: i32 = r.max_x() - (right as i32);
                if x0 > x1 {
                    continue;
                }
                if x0 < 0 || x0 > (u8::MAX as i32) {
                    continue;
                }
                if x1 < 0 || x1 > (u8::MAX as i32) {
                    continue;
                }
                let width: i32 = x1 - x0 + 1;
                if width <= 0 {
                    continue;
                }
                let x0_u8: u8 = x0 as u8;
                let width_u8: u8 = width as u8;
                let rect = Rectangle::new(x0_u8, 0, width_u8, image.height());
                let image_cropped: Image = match image.crop(rect) {
                    Ok(value) => value,
                    Err(_) => {
                        continue;
                    }
                };
                let area: u16 = (image_cropped.width() as u16) * (image_cropped.height() as u16);
                let image: Image = image_cropped.flip_x()?;
                let diff: Image = image.diff(&image_cropped)?;
                let histogram_mask: Image = diff.to_mask_where_color_is(0);
                let histogram: Histogram = image.histogram_with_mask(&histogram_mask)?;
                if histogram.number_of_counters_greater_than_zero() < 2 {
                    continue;
                }
                let mismatch_count: u16 = diff.mask_count_one();
                if mismatch_count > (area / 2) {
                    continue;
                }
                if !found {
                    found_left = left;
                    found_right = right;
                    found_mismatches = mismatch_count;
                    found = true;
                    continue;
                }
                let error0: u64 = Self::compute_error(found_left, found_right, found_mismatches);
                let error1: u64 = Self::compute_error(left, right, mismatch_count);
                if error1 >= error0 {
                    continue;
                }
                found_left = left;
                found_right = right;
                found_mismatches = mismatch_count;
            }
        }

        if should_update_vertical_data {
            if found {
                self.found_vertical_symmetry = true;
                self.top = found_left;
                self.bottom = found_right;
                self.vertical_mismatches = found_mismatches;
            } else {
                self.found_vertical_symmetry = false;
                self.top = u8::MAX;
                self.bottom = u8::MAX;
                self.vertical_mismatches = u16::MAX;
            }
        } else {
            if found {
                self.found_horizontal_symmetry = true;
                self.left = found_left;
                self.right = found_right;
                self.horizontal_mismatches = found_mismatches;
            } else {
                self.found_horizontal_symmetry = false;
                self.left = u8::MAX;
                self.right = u8::MAX;
                self.horizontal_mismatches = u16::MAX;
            }
        }

        Ok(())
    }

    fn compute_error(left: u8, right: u8, mismatches: u16) -> u64 {
        let left_squared: u16 = (left as u16) * (left as u16);
        let right_squared: u16 = (right as u16) * (right as u16);
        let mismatches_squared: u32 = (mismatches as u32) * (mismatches as u32);
        let sum: u64 = (left_squared as u64) + (right_squared as u64) + (mismatches_squared as u64);
        sum
    }

}

impl fmt::Debug for Symmetry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "{}, {}, {}, {}", 
            self.horizontal_to_string(), 
            self.vertical_to_string(),
            self.diagonal_a_to_string(),
            self.diagonal_b_to_string(),
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_horizontal_symmetry_perfect() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 3, 2, 1, 0,
        ];
        let input: Image = Image::try_create(7, 1, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
    }

    #[test]
    fn test_10001_horizontal_symmetry_left1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 3, 2, 1,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 1 right: 0");
    }

    #[test]
    fn test_10002_horizontal_symmetry_right1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 2, 1, 0,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
    }

    #[test]
    fn test_10003_horizontal_symmetry_left1junk_right1junk() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 1, 2, 3, 2, 1, 8,
        ];
        let input: Image = Image::try_create(7, 1, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 2");
    }

    #[test]
    fn test_10004_horizontal_symmetry_none() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6,
        ];
        let input: Image = Image::try_create(7, 1, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "no horizontal symmetry");
    }

    #[test]
    fn test_10005_horizontal_symmetry_alternating() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0, 1,
        ];
        let input: Image = Image::try_create(6, 1, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_horizontal_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
    }

    #[test]
    fn test_20000_vertical_symmetry_perfect() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0,
            1,
            2,
            3,
            2,
            1,
            0,
        ];
        let input: Image = Image::try_create(1, 7, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 0");
    }

    #[test]
    fn test_20001_vertical_symmetry_top1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0,
            1,
            2,
            3,
            2,
            1,
        ];
        let input: Image = Image::try_create(1, 6, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 1 bottom: 0");
    }

    #[test]
    fn test_20002_vertical_symmetry_bottom1() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1,
            2,
            3,
            2,
            1,
            0,
        ];
        let input: Image = Image::try_create(1, 6, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 1");
    }

    #[test]
    fn test_20003_vertical_symmetry_top1junk_bottom1junk() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9,
            1,
            2,
            3,
            2,
            1,
            8,
        ];
        let input: Image = Image::try_create(1, 7, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 0 mismatches: 2");
    }

    #[test]
    fn test_20000_vertical_symmetry_none() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0,
            1,
            2,
            3,
            4,
            5,
            6,
        ];
        let input: Image = Image::try_create(1, 7, pixels).expect("image");

        // Act
        let mut instance = Symmetry::new();
        instance.analyze_vertical_symmetry(&input).expect("ok");

        // Assert
        assert_eq!(instance.vertical_to_string(), "no vertical symmetry");
    }

    #[test]
    fn test_30000_analyze_checkerboard() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 1, 0,
            0, 1, 0, 1, 0, 1,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 0");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30001_analyze_checkerboard_with_one_junk_pixel() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 1,
            0, 0, 1, 9, 0, 0,
            0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 2");
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 0 mismatches: 2");
        assert_eq!(instance.diagonal_a_to_string(), "diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "partial diagonal-b symmetry, mismatches: 2");
    }

    #[test]
    fn test_30002_analyze_nosymmetry() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3, 4, 5, 6,
            1, 2, 3, 4, 5, 6,
            8, 8, 8, 8, 8, 8,
            8, 1, 8, 1, 8, 1,
            0, 0, 1, 1, 2, 2,
            0, 0, 1, 1, 2, 2,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "no horizontal symmetry");
        assert_eq!(instance.vertical_to_string(), "no vertical symmetry");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30003_analyze_one_junk_pixel() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 1,
            0, 0, 1, 9, 0, 0,
            1, 1, 0, 0, 1, 1,
            1, 1, 0, 0, 1, 1,
        ];
        let input: Image = Image::try_create(6, 5, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 2");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 0");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30004_analyze_lines() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            5, 5, 5, 5, 5, 5,
            0, 0, 1, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 7, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 2 bottom: 0");
        assert_eq!(instance.diagonal_a_to_string(), "partial diagonal-a symmetry, mismatches: 8");
        assert_eq!(instance.diagonal_b_to_string(), "partial diagonal-b symmetry, mismatches: 8");
    }

    #[test]
    fn test_30005_analyze_boxes() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 1, 0, 0, 1, 1, 0,
            1, 1, 0, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 0, 0,
            0, 0, 1, 1, 0, 0, 0,
            1, 1, 0, 0, 1, 1, 0,
            1, 1, 0, 0, 1, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(7, 8, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 1");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 2");
        assert_eq!(instance.diagonal_a_to_string(), "diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "diagonal-b symmetry");
    }

    #[test]
    fn test_30006_analyze_border_pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 1, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "no horizontal symmetry");
        assert_eq!(instance.vertical_to_string(), "no vertical symmetry");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30007_analyze_border_pixels() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0,
            0, 1, 1, 1, 0, 1, 0,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "no horizontal symmetry");
        assert_eq!(instance.vertical_to_string(), "no vertical symmetry");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
    }

    #[test]
    fn test_30008_analyze_diagonal_symmetry() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 1, 0,
            1, 8, 8, 8, 1,
            2, 8, 8, 8, 2,
            1, 8, 8, 8, 1,
            0, 1, 2, 1, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "horizontal symmetry, left: 0 right: 0");
        assert_eq!(instance.vertical_to_string(), "vertical symmetry, top: 0 bottom: 0");
        assert_eq!(instance.diagonal_a_to_string(), "diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "diagonal-b symmetry");
        assert_eq!(instance.repair_color, None);
    }

    #[test]
    fn test_30009_diagonal_a() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 5, 5, 5, 0, 0,
            1, 5, 0, 5, 0, 0,
            1, 5, 5, 5, 0, 0,
            1, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 5, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 1 mismatches: 10");
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 1 mismatches: 8");
        assert_eq!(instance.diagonal_a_new_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_new_to_string(), "diagonal-b symmetry, x: 1 y: 0 size: 5");
        assert_eq!(instance.repair_color, None);
    }

    #[test]
    fn test_30010_diagonal_b() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 5, 5, 5,
            1, 0, 0, 5, 0, 5,
            1, 0, 0, 5, 5, 5,
            1, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 5, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 2 right: 0 mismatches: 8");
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 1 mismatches: 8");
        assert_eq!(instance.diagonal_a_new_to_string(), "diagonal-a symmetry, x: 1 y: 0 size: 5");
        assert_eq!(instance.diagonal_b_new_to_string(), "no diagonal-b symmetry");
        assert_eq!(instance.repair_color, None);
    }

    #[test]
    fn test_40000_find_repair_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 5, 0, 5, 0,
            1, 0, 1, 0, 1,
            2, 5, 8, 5, 2,
            1, 0, 1, 0, 1,
            0, 5, 0, 5, 7,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 2");
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 0 mismatches: 2");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
        assert_eq!(instance.repair_color, Some(7));
    }

    #[test]
    fn test_40001_find_repair_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 5, 3, 3, 5, 0,
            1, 0, 7, 7, 0, 1,
            2, 2, 7, 7, 0, 1,
            2, 2, 3, 3, 5, 0,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 8");
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 0 mismatches: 8");
        assert_eq!(instance.diagonal_a_to_string(), "no diagonal-a symmetry");
        assert_eq!(instance.diagonal_b_to_string(), "no diagonal-b symmetry");
        assert_eq!(instance.repair_color, Some(2));
    }

    #[test]
    fn test_40002_find_repair_color() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 9, 5, 5, 3, 5, 6, 9, 9, 9, 5, 6, 6, 5, 9, 9, 9, 6, 5, 3, 5, 5, 9, 7,
            9, 7, 5, 1, 5, 3, 9, 3, 8, 6, 6, 2, 2, 2, 6, 8, 3, 9, 3, 5, 1, 5, 7, 9,
            5, 5, 1, 3, 9, 3, 9, 8, 9, 5, 3, 2, 2, 2, 5, 9, 8, 9, 3, 9, 3, 1, 5, 5,
            5, 1, 3, 7, 9, 9, 9, 6, 5, 9, 6, 2, 2, 2, 9, 5, 6, 9, 9, 9, 7, 3, 1, 5,
            2, 2, 2, 2, 2, 9, 5, 6, 3, 6, 3, 2, 2, 2, 6, 3, 6, 5, 9, 3, 9, 9, 5, 3,
            2, 2, 2, 2, 2, 3, 6, 6, 3, 9, 8, 2, 2, 2, 9, 3, 6, 6, 3, 9, 9, 3, 3, 5,
            2, 2, 2, 2, 2, 6, 9, 7, 9, 9, 4, 2, 2, 2, 9, 9, 7, 9, 6, 5, 9, 9, 9, 6,
            2, 2, 2, 2, 2, 6, 7, 9, 9, 1, 1, 4, 4, 1, 1, 9, 9, 7, 6, 6, 6, 8, 3, 9,
            2, 2, 2, 2, 2, 3, 9, 9, 1, 7, 4, 3, 3, 4, 7, 1, 9, 9, 3, 3, 5, 9, 8, 9,
            2, 2, 2, 2, 2, 9, 9, 1, 7, 1, 9, 7, 7, 9, 1, 7, 2, 2, 2, 2, 9, 5, 6, 9,
            5, 6, 3, 6, 3, 8, 4, 1, 4, 9, 3, 9, 9, 3, 9, 4, 2, 2, 2, 2, 6, 3, 6, 5,
            6, 6, 3, 9, 8, 9, 4, 4, 3, 7, 9, 7, 7, 9, 7, 3, 2, 2, 2, 2, 9, 3, 6, 6,
            6, 6, 3, 9, 8, 9, 4, 4, 3, 7, 9, 7, 7, 9, 7, 3, 2, 2, 2, 2, 9, 3, 6, 6,
            5, 6, 3, 6, 3, 8, 2, 2, 2, 2, 2, 2, 9, 3, 9, 4, 2, 2, 2, 2, 6, 3, 6, 5,
            9, 6, 5, 9, 6, 9, 2, 2, 2, 2, 2, 2, 7, 9, 1, 7, 2, 2, 2, 2, 9, 5, 6, 9,
            9, 8, 9, 5, 3, 3, 2, 2, 2, 2, 2, 2, 3, 4, 7, 1, 9, 9, 3, 3, 5, 9, 8, 9,
            9, 3, 8, 6, 6, 6, 2, 2, 2, 2, 2, 2, 4, 1, 1, 9, 9, 7, 6, 6, 6, 8, 3, 9,
            6, 9, 9, 9, 5, 6, 9, 7, 9, 9, 4, 4, 4, 4, 9, 9, 7, 9, 6, 5, 9, 9, 9, 6,
            5, 3, 3, 9, 9, 3, 6, 6, 3, 9, 8, 9, 9, 8, 9, 3, 6, 6, 3, 9, 9, 3, 3, 5,
            3, 5, 9, 9, 3, 9, 5, 6, 3, 6, 3, 8, 8, 3, 6, 3, 6, 5, 9, 3, 9, 9, 5, 3,
            5, 1, 3, 7, 9, 9, 9, 6, 5, 9, 6, 9, 9, 6, 9, 5, 6, 9, 9, 9, 7, 3, 1, 5,
            5, 5, 1, 3, 9, 3, 9, 8, 9, 5, 3, 3, 3, 3, 5, 9, 8, 9, 3, 9, 3, 1, 5, 5,
            9, 7, 5, 1, 5, 3, 9, 3, 8, 6, 6, 6, 6, 6, 6, 8, 3, 9, 3, 5, 1, 5, 7, 9,
            7, 9, 5, 5, 3, 5, 6, 9, 9, 9, 5, 6, 6, 5, 9, 9, 9, 6, 5, 3, 5, 5, 9, 7
        ];
        let input: Image = Image::try_create(24, 24, pixels).expect("image");

        // Act
        let instance = Symmetry::analyze(&input).expect("ok");

        // Assert
        assert_eq!(instance.horizontal_to_string(), "partial horizontal symmetry, left: 0 right: 0 mismatches: 148");
        assert_eq!(instance.vertical_to_string(), "partial vertical symmetry, top: 0 bottom: 0 mismatches: 144");
        assert_eq!(instance.diagonal_a_to_string(), "partial diagonal-a symmetry, mismatches: 124");
        assert_eq!(instance.diagonal_b_to_string(), "partial diagonal-b symmetry, mismatches: 174");
        assert_eq!(instance.repair_color, Some(2));
    }
}
