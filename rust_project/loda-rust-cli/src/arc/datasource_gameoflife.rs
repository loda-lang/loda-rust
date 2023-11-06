//! Conway's Game of Life
//! 
//! https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
use super::{Image, ImageSize, HtmlLog};
use std::marker::PhantomData;

/// `CARule` is a trait that defines the behavior of a single cell within a cellular automaton
/// based on its current state and the states of its eight neighboring cells.
///
/// This trait can be implemented for different rule sets, allowing for the simulation
/// of various cellular automata beyond Conway's Game of Life, such as Highlife or Wireworld.
pub trait CARule {
    fn apply(center: u8, neighbors: &[u8; 8]) -> u8;
}

#[derive(Debug)]
pub struct CellularAutomaton<R: CARule> {
    current: Image,
    next: Image,
    _rule: PhantomData<R>,
}

impl<R: CARule> CellularAutomaton<R> {
    #[allow(dead_code)]
    pub fn new(size: ImageSize) -> Self {
        Self {
            current: Image::zero(size.width, size.height),
            next: Image::zero(size.width, size.height),
            _rule: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn with_image(image: &Image) -> Self {
        Self {
            current: image.clone(),
            next: image.clone_zero(),
            _rule: PhantomData,
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
                let set_value: u8 = R::apply(center, &neighbors);

                _ = self.next.set(x as i32, y as i32, set_value);
            }
        }
        std::mem::swap(&mut self.current, &mut self.next);
    }
}

/// https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
/// 
/// - Any live cell with fewer than two live neighbours dies, as if by underpopulation.
/// - Any live cell with two or three live neighbours lives on to the next generation.
/// - Any live cell with more than three live neighbours dies, as if by overpopulation.
/// - Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
/// 
/// These rules, which compare the behavior of the automaton to real life, can be condensed into the following:
/// 
/// - Any live cell with two or three live neighbours survives.
/// - Any dead cell with three live neighbours becomes a live cell.
/// - All other live cells die in the next generation. Similarly, all other dead cells stay dead.
struct GameOfLife;

impl CARule for GameOfLife {
    fn apply(center: u8, neighbors: &[u8; 8]) -> u8 {
        let alive_count: usize = neighbors.iter().filter(|&&value| value > 0).count();
        match (center, alive_count) {
            (0, 3) => 1,
            (1, 2) => 1,
            (1, 3) => 1,
            _ => 0,
        }
    }
}

/// https://conwaylife.com/wiki/OCA:HighLife
/// 
/// Cells survive from one generation to the next if they have 2 or 3 neighbours, and are born if they have 3 or 6 neighbours.
struct HighLife;

impl CARule for HighLife {
    fn apply(center: u8, neighbors: &[u8; 8]) -> u8 {
        let alive_count: usize = neighbors.iter().filter(|&&value| value > 0).count();
        match (center, alive_count) {
            // A dead cell with exactly three or six neighbors becomes a live cell
            (0, 3) | (0, 6) => 1,
            // A live cell with two or three neighbors stays alive
            (1, 2) | (1, 3) => 1,
            // In all other cases, the cell dies or remains dead
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_gameoflife_glider() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<GameOfLife>::with_image(&input);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
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
    fn test_10001_gameoflife_glider_wraparound() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            1, 1, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<GameOfLife>::with_image(&input);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
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

    #[test]
    fn test_20000_highlife_predecessor_replicator() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 1, 0,
            0, 1, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<HighLife>::with_image(&input);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 1, 0, 0,
            0, 0, 1, 1, 0, 0,
            0, 1, 0, 1, 0, 0,
            1, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
