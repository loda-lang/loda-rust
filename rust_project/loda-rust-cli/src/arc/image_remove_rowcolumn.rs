use super::Image;
use bit_set::BitSet;

pub trait ImageRemoveRowColumn {
    fn remove_rowcolumn(&self, rows: &BitSet, columns: &BitSet) -> anyhow::Result<Image>;
}

impl ImageRemoveRowColumn for Image {
    fn remove_rowcolumn(&self, rows: &BitSet, columns: &BitSet) -> anyhow::Result<Image> {
        if self.is_empty() {
            return Ok(Image::empty());
        }

        // Determine height
        let mut output_height = 0u8;
        let mut row_remove_count: usize = 0;
        for y in 0..self.height() {
            if rows.contains(y as usize) {
                row_remove_count += 1;
                continue;
            }
            output_height += 1;
        }
        if row_remove_count != rows.len() {
            return Err(anyhow::anyhow!("remove_rowcolumn: More rows are scheduled for removal than the height of the image."));
        }

        // Determine width
        let mut output_width = 0u8;
        let mut column_remove_count: usize = 0;
        for x in 0..self.width() {
            if columns.contains(x as usize) {
                column_remove_count += 1;
                continue;
            }
            output_width += 1;
        }
        if column_remove_count != columns.len() {
            return Err(anyhow::anyhow!("remove_rowcolumn: More columns are scheduled for removal than the width of the image."));
        }

        // Copy pixels of the rows to keep
        let mut output_image = Image::zero(output_width, output_height);
        let mut current_y: i32 = -1;
        for y in 0..self.height() {
            if rows.contains(y as usize) {
                continue;
            }
            current_y += 1;
            let mut current_x: i32 = -1;
            for x in 0..self.width() {
                if columns.contains(x as usize) {
                    continue;
                }
                current_x += 1;
                let pixel_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                let set_x: i32 = current_x;
                let set_y: i32 = current_y;
                match output_image.set(set_x, set_y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("remove_rowcolumn. Unable to set pixel ({}, {}) inside the result bitmap", set_x, set_y));
                    }
                }
            }
        }
        return Ok(output_image);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_remove_rowcolumn() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            0, 0, 0,
            1, 1, 1,
            0, 0, 1,
            0, 0, 1,
            0, 0, 0,
            0, 0, 0,
        ];
        let input: Image = Image::try_create(3, 7, pixels).expect("image");
        let columns = BitSet::new();
        let mut rows = BitSet::new();
        rows.insert(0);
        rows.insert(3);
        rows.insert(5);

        // Act
        let actual: Image = input.remove_rowcolumn(&rows, &columns).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0,
            1, 1, 1,
            0, 0, 1,
            0, 0, 0,
        ];
        let expected: Image = Image::try_create(3, 4, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_remove_rowcolumn() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            0, 1, 0, 0, 0,
            4, 1, 5, 0, 6,
            7, 1, 8, 0, 9,
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
        ];
        let input: Image = Image::try_create(5, 6, pixels).expect("image");

        let mut columns = BitSet::new();
        columns.insert(1);
        columns.insert(3);
        let mut rows = BitSet::new();
        rows.insert(0);
        rows.insert(1);
        rows.insert(4);
        rows.insert(5);

        // Act
        let actual: Image = input.remove_rowcolumn(&rows, &columns).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            4, 5, 6,
            7, 8, 9,
        ];
        let expected: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10002_remove_rowcolumn() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");
        let columns = BitSet::new();
        let rows = BitSet::new();

        // Act
        let actual: Image = input.remove_rowcolumn(&rows, &columns).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 2,
            3, 4,
        ];
        let expected: Image = Image::try_create(2, 2, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

}
