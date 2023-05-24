use super::{Image, ImageCrop, ImageOverlay, ImageSymmetry, Rectangle, ImageRotate};

#[derive(Clone, Copy, Debug)]
pub enum ImageSortMode {
    RowsAscending,
    RowsDescending,
    ColumnsAscending,
    ColumnsDescending,
}

#[allow(dead_code)]
pub trait ImageSort {
    fn sort_by_color(&self, background_color: u8, mode: ImageSortMode) -> anyhow::Result<Image>;
}

impl ImageSort for Image {
    fn sort_by_color(&self, background_color: u8, mode: ImageSortMode) -> anyhow::Result<Image> {
        match mode {
            ImageSortMode::RowsAscending => Sorter::sort_rows(&self, background_color),
            ImageSortMode::RowsDescending => Sorter::sort_rows_reverse(&self, background_color),
            ImageSortMode::ColumnsAscending => Sorter::sort_columns(&self, background_color),
            ImageSortMode::ColumnsDescending => Sorter::sort_columns_reverse(&self, background_color),
        }
    }
}

struct Sorter;

impl Sorter {
    fn sort_rows(image: &Image, background_color: u8) -> anyhow::Result<Image> {
        if image.height() <= 1 {
            return Ok(image.clone());
        }
        let mut count_y_vec = Vec::<(u8, u8)>::new();
        for y in 0..image.height() {
            let mut count: u8 = 0;
            for x in 0..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                if color == background_color {
                    continue;
                }
                count += 1;
            }
            count_y_vec.push((count, y));
        }
        count_y_vec.sort();

        let mut result_image: Image = Image::color(image.width(), image.height(), background_color);
        for y in 0..image.height() {
            let (_count, source_y) = match count_y_vec.get(y as usize) {
                Some(value) => *value,
                None => {
                    continue;
                }
            };
            let rect = Rectangle::new(0, source_y, image.width(), 1);
            let image: Image = image.crop(rect)?;
            result_image = result_image.overlay_with_position(&image, 0, y as i32)?;
        }
        Ok(result_image)
    }

    fn sort_rows_reverse(image: &Image, background_color: u8) -> anyhow::Result<Image> {
        let image2: Image = Self::sort_rows(image, background_color)?;
        image2.flip_y()
    }

    fn sort_columns(image: &Image, background_color: u8) -> anyhow::Result<Image> {
        let image1: Image = image.rotate_cw()?;
        let image2: Image = Self::sort_rows(&image1, background_color)?;
        image2.rotate_ccw()
    }

    fn sort_columns_reverse(image: &Image, background_color: u8) -> anyhow::Result<Image> {
        let image1: Image = image.rotate_cw()?;
        let image2: Image = Self::sort_rows_reverse(&image1, background_color)?;
        image2.rotate_ccw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_sort_rows() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 7, 7,
            5, 5, 5, 5, 5,
            5, 5, 8, 8, 8,
            5, 5, 5, 5, 5,
            5, 5, 5, 5, 9,
            3, 3, 3, 3, 3,
            5, 5, 5, 2, 2,
        ];
        let input: Image = Image::try_create(5, 7, pixels).expect("image");

        // Act
        let actual: Image = input.sort_by_color(5, ImageSortMode::RowsAscending).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 5, 5, 5, 5,
            5, 5, 5, 5, 9,
            5, 5, 5, 7, 7,
            5, 5, 5, 2, 2,
            5, 5, 8, 8, 8,
            3, 3, 3, 3, 3,
        ];
        let expected = Image::create_raw(5, 7, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_sort_rows_reversed() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 7, 7,
            5, 5, 5, 5, 5,
            5, 5, 8, 8, 8,
            5, 5, 5, 5, 5,
            5, 5, 5, 5, 9,
            3, 3, 3, 3, 3,
            5, 5, 5, 2, 2,
        ];
        let input: Image = Image::try_create(5, 7, pixels).expect("image");

        // Act
        let actual: Image = input.sort_by_color(5, ImageSortMode::RowsDescending).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3,
            5, 5, 8, 8, 8,
            5, 5, 5, 2, 2,
            5, 5, 5, 7, 7,
            5, 5, 5, 5, 9,
            5, 5, 5, 5, 5,
            5, 5, 5, 5, 5,
        ];
        let expected = Image::create_raw(5, 7, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_sort_columns() {
        // Arrange
        let pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3, 3,
            3, 3, 5, 7, 3, 9,
            3, 0, 5, 7, 3, 3,
            3, 0, 5, 7, 3, 3,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let actual: Image = input.sort_by_color(3, ImageSortMode::ColumnsAscending).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3, 3,
            3, 3, 9, 3, 5, 7,
            3, 3, 3, 0, 5, 7,
            3, 3, 3, 0, 5, 7,
        ];
        let expected = Image::create_raw(6, 4, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_sort_columns_reverse() {
        // Arrange
        let pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3, 3,
            3, 3, 5, 7, 3, 9,
            3, 0, 5, 7, 3, 3,
            3, 0, 5, 7, 3, 3,
        ];
        let input: Image = Image::try_create(6, 4, pixels).expect("image");

        // Act
        let actual: Image = input.sort_by_color(3, ImageSortMode::ColumnsDescending).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3, 3, 3,
            7, 5, 3, 9, 3, 3,
            7, 5, 0, 3, 3, 3,
            7, 5, 0, 3, 3, 3,
        ];
        let expected = Image::create_raw(6, 4, expected_pixels);
        assert_eq!(actual, expected);
    }
}
