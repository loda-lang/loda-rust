//! Conway's Game of Life
//! 
//! https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
//! 
//! - Any live cell with fewer than two live neighbours dies, as if by underpopulation.
//! - Any live cell with two or three live neighbours lives on to the next generation.
//! - Any live cell with more than three live neighbours dies, as if by overpopulation.
//! - Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
//! 
//! These rules, which compare the behaviour of the automaton to real life, can be condensed into the following:
//! 
//! - Any live cell with two or three live neighbours survives.
//! - Any dead cell with three live neighbours becomes a live cell.
//! - All other live cells die in the next generation. Similarly, all other dead cells stay dead.
use super::{Image, ImageSize, HtmlLog};

#[derive(Debug, Clone, PartialEq, Eq)]
struct GameOfLife {
    current: Image,
    next: Image,
}

impl GameOfLife {
    #[allow(dead_code)]
    pub fn new(size: ImageSize) -> Self {
        Self {
            current: Image::zero(size.width, size.height),
            next: Image::zero(size.width, size.height),
        }
    }

    #[allow(dead_code)]
    pub fn with_image(image: &Image) -> Self {
        Self {
            current: image.clone(),
            next: image.clone_zero(),
        }
    }

    #[allow(dead_code)]
    pub fn step(&mut self, count: u8) {
        for _ in 0..count {
            self.step_once();
        }
    }

    #[allow(dead_code)]
    pub fn step_and_log(&mut self, count: u8) {
        HtmlLog::image(&self.current);
        for _ in 0..count {
            self.step_once();
            HtmlLog::image(&self.current);
        }
    }

    #[allow(dead_code)]
    pub fn step_once(&mut self) {
        for y in 0..self.current.height() {
            for x in 0..self.current.width() {

                // Obtain the 8 neighbor values
                let mut neighbors: [u8; 8] = [0; 8];
                let mut index: usize = 0;
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i == 0 && j == 0 {
                            continue;
                        }
                        let value: u8 = self.current.get_wrap(x as i32 + i, y as i32 + j).unwrap_or(0);
                        neighbors[index] = value;
                        index += 1;
                    }
                }

                // Get center value
                let center: u8 = self.current.get(x as i32, y as i32).unwrap_or(0);

                // Apply the rules
                let set_value: u8 = Self::callback(center, &neighbors);

                _ = self.next.set(x as i32, y as i32, set_value);
            }
        }
        std::mem::swap(&mut self.current, &mut self.next);
    }

    fn callback(center: u8, neighbors: &[u8; 8]) -> u8 {
        let mut alive_count: u8 = 0;
        for value in neighbors {
            if *value > 0 {
                alive_count += 1;
            }
        }
        match (center, alive_count) {
            (0, 3) => 1,
            (1, 2) => 1,
            (1, 3) => 1,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_glider() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let mut game_of_life: GameOfLife = GameOfLife::with_image(&input);

        // Act
        game_of_life.step_once();
        let actual: Image = game_of_life.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0,
            1, 0, 1, 0, 0,
            0, 1, 1, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_glider_wraparound() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            1, 1, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");

        let mut game_of_life: GameOfLife = GameOfLife::with_image(&input);

        // Act
        game_of_life.step_once();
        let actual: Image = game_of_life.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 1, 0, 0, 1,
            1, 1, 0, 0, 0,
            1, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
