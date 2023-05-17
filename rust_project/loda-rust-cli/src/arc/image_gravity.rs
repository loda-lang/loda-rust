use super::{Image, ImageRotate, ImageSymmetry};

#[allow(dead_code)]
pub enum GravityDirection {
    Up,
    Down,
    Left,
    Right,
}

#[allow(dead_code)]
pub trait ImageGravity {
    fn gravity(&self, background_color: u8, direction: GravityDirection) -> anyhow::Result<Image>;
}

impl ImageGravity for Image {
    fn gravity(&self, background_color: u8, direction: GravityDirection) -> anyhow::Result<Image> {
        match direction {
            GravityDirection::Up => Gravity::gravity_up(&self, background_color),
            GravityDirection::Down => Gravity::gravity_down(&self, background_color),
            GravityDirection::Left => Gravity::gravity_left(&self, background_color),
            GravityDirection::Right => Gravity::gravity_right(&self, background_color)
        }
    }
}

struct Gravity;

impl Gravity {
    fn gravity_up(input: &Image, background_color: u8) -> anyhow::Result<Image> {
        if input.height() <= 1 {
            return Ok(input.clone());
        }
        let mut result_image: Image = Image::color(input.width(), input.height(), background_color);
        for x in 0..input.width() {
            let mut current: i32 = 0;
            for y in 0..input.height() {
                let color: u8 = input.get(x as i32, y as i32).unwrap_or(255);
                if color == background_color {
                    continue;
                }
                _ = result_image.set(x as i32, current, color);
                current += 1;
            }
        }
        Ok(result_image)
    }

    fn gravity_down(input: &Image, background_color: u8) -> anyhow::Result<Image> {
        let image1: Image = input.flip_y()?;
        let image2: Image = Self::gravity_up(&image1, background_color)?;
        image2.flip_y()
    }

    fn gravity_left(input: &Image, background_color: u8) -> anyhow::Result<Image> {
        let image1: Image = input.rotate_cw()?;
        let image2: Image = Self::gravity_up(&image1, background_color)?;
        image2.rotate_ccw()
    }

    fn gravity_right(input: &Image, background_color: u8) -> anyhow::Result<Image> {
        let image1: Image = input.rotate_ccw()?;
        let image2: Image = Self::gravity_up(&image1, background_color)?;
        image2.rotate_cw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_gravity_up() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 5, 2, 5, 1,
            5, 4, 4, 5, 3,
            5, 2, 7, 5, 3,
            5, 5, 5, 3, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.gravity(5, GravityDirection::Up).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 4, 2, 3, 1,
            5, 2, 4, 5, 3,
            5, 5, 7, 5, 3,
            5, 5, 5, 5, 5,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_gravity_down() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 5, 2, 5, 1,
            5, 4, 4, 5, 3,
            5, 2, 7, 5, 3,
            5, 5, 5, 3, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.gravity(5, GravityDirection::Down).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 5, 5, 5,
            5, 5, 2, 5, 1,
            5, 4, 4, 5, 3,
            7, 2, 7, 3, 3,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_gravity_left() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 5, 2, 5, 1,
            5, 4, 4, 5, 3,
            5, 2, 7, 5, 3,
            5, 5, 5, 3, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.gravity(5, GravityDirection::Left).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            7, 2, 1, 5, 5,
            4, 4, 3, 5, 5,
            2, 7, 3, 5, 5,
            3, 5, 5, 5, 5,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40000_gravity_right() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 5, 2, 5, 1,
            5, 4, 4, 5, 3,
            5, 2, 7, 5, 3,
            5, 5, 5, 3, 5,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: Image = input.gravity(5, GravityDirection::Right).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 5, 7, 2, 1,
            5, 5, 4, 4, 3,
            5, 5, 2, 7, 3,
            5, 5, 5, 5, 3,
        ];
        let expected = Image::create_raw(5, 4, expected_pixels);
        assert_eq!(actual, expected);
    }
}
