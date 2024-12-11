use super::Image;

pub trait ImagePosition {
    fn positions_where_color_is(&self, color: u8) -> Vec<(u8, u8)>;
}

impl ImagePosition for Image {
    fn positions_where_color_is(&self, color: u8) -> Vec<(u8, u8)> {
        let mut positions = Vec::<(u8, u8)>::new();
        for y in 0..self.height() {
            for x in 0..self.width() {
                let pixel_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                if pixel_value == color {
                    positions.push((x, y));
                }
            }
        }
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_positions_where_color_is() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 0, 0, 0, 0,
            0, 0, 0, 1, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 0, 1,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Vec<(u8, u8)> = input.positions_where_color_is(1);

        // Assert
        let expected: Vec<(u8, u8)> = vec![
            (0, 0),
            (3, 1),
            (1, 2),
            (4, 3),
        ];
        assert_eq!(actual, expected);
    }

}
