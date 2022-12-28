use super::Image;

pub trait ImageMaskCount {
    fn mask_count(&self) -> (u32, u32, u32);
    fn mask_count_zero(&self) -> u32;
    fn mask_count_one(&self) -> u32;
    fn mask_count_other(&self) -> u32;
}

impl ImageMaskCount for Image {
    fn mask_count(&self) -> (u32, u32, u32) {
        let mut count0: u32 = 0;
        let mut count1: u32 = 0;
        let mut count_other: u32 = 0;
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let color_value: u8 = self.get(x, y).unwrap_or(255);
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

    fn mask_count_zero(&self) -> u32 {
        let (count0, _count1, _count_other) = self.mask_count();
        count0
    }

    fn mask_count_one(&self) -> u32 {
        let (_count0, count1, _count_other) = self.mask_count();
        count1
    }

    fn mask_count_other(&self) -> u32 {
        let (_count0, _count1, count_other) = self.mask_count();
        count_other
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
