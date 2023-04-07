use super::Image;

pub trait ImageMaskCount {
    fn mask_count(&self) -> (u16, u16, u16);
    fn mask_count_zero(&self) -> u16;
    fn mask_count_one(&self) -> u16;
    fn mask_count_other(&self) -> u32;
}

impl ImageMaskCount for Image {
    fn mask_count(&self) -> (u16, u16, u16) {
        let mut count0: u16 = 0;
        let mut count1: u16 = 0;
        let mut count_other: u16 = 0;
        for y in 0..self.height() {
            for x in 0..self.width() {
                let color_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                match color_value {
                    0 => {
                        count0 += 1;
                    },
                    1 => {
                        count1 += 1;
                    },
                    _ => {
                        count_other += 1;
                    }
                }
            }
        }
        (count0, count1, count_other)
    }

    fn mask_count_zero(&self) -> u16 {
        let (count0, _count1, _count_other) = self.mask_count();
        count0
    }

    fn mask_count_one(&self) -> u16 {
        let (_count0, count1, _count_other) = self.mask_count();
        count1
    }

    fn mask_count_other(&self) -> u32 {
        let (_count0, _count1, count_other) = self.mask_count();
        count_other as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_histogram_all() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0,
            1, 1, 1,
            1, 1, 2,
        ];
        let input: Image = Image::try_create(3, 3, pixels).expect("image");

        // Act
        let (count0, count1, count_other) = input.mask_count();

        // Assert
        assert_eq!(count0, 3);
        assert_eq!(count1, 5);
        assert_eq!(count_other, 1);
    }
}
