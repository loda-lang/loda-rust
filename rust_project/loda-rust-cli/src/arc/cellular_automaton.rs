//! Cellular Automaton (singular) and Cellular Automata (plural)
//! https://en.wikipedia.org/wiki/Cellular_automaton
//! 
//! Conway's Game of Life is one cellular automaton.
//! https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
//! 
//! Ideas for more cellular automata:
//! https://en.wikipedia.org/wiki/Langton%27s_ant
//! https://conwaylife.com/wiki/OCA:Maze
//! https://conwaylife.com/wiki/OCA:Life_without_death
//! https://conwaylife.com/wiki/OCA:Seeds
//! https://en.wikipedia.org/wiki/Brian's_Brain
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
    outside_color: Option<u8>,
    _rule: PhantomData<R>,
}

impl<R: CARule> CellularAutomaton<R> {
    #[allow(dead_code)]
    pub fn new(size: ImageSize, outside_color: Option<u8>) -> Self {
        Self {
            current: Image::zero(size.width, size.height),
            next: Image::zero(size.width, size.height),
            outside_color,
            _rule: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn with_image(image: &Image, outside_color: Option<u8>) -> Self {
        Self {
            current: image.clone(),
            next: image.clone_zero(),
            outside_color,
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

    #[allow(dead_code)]
    pub fn images_for_n_steps(&mut self, count: u8) -> Vec<Image> {
        let mut images = Vec::<Image>::new();
        images.push(self.current.clone());
        for _ in 0..count {
            self.step_once();
            images.push(self.current.clone());
        }
        images
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
                        let value: u8 = if let Some(outside_color) = self.outside_color {
                            self.current.get(x as i32 + i, y as i32 + j).unwrap_or(outside_color)
                        } else {
                            self.current.get_wrap(x as i32 + i, y as i32 + j).unwrap_or(0)
                        };
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

    #[allow(dead_code)]
    pub fn image(&self) -> &Image {
        &self.current
    }
}

pub mod rule {
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
    pub struct GameOfLife;

    impl super::CARule for GameOfLife {
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

    /// GameOfLife with extra states
    /// 
    /// 0 = dead
    /// 1 = alive
    /// 2 = just born
    /// 3 = just dead
    pub struct GameOfLifeExtra;

    impl super::CARule for GameOfLifeExtra {
        fn apply(center: u8, neighbors: &[u8; 8]) -> u8 {
            let alive_count = neighbors.iter().filter(|&&state| state == 1 || state == 2).count();

            if (center == 1 || center == 2) && (alive_count < 2 || alive_count > 3) {
                // Any live cell with fewer than two live neighbours dies, as if by underpopulation.
                // Any live cell with more than three live neighbours dies, as if by overpopulation.
                return 3; // just dead
            }

            if (center == 1 || center == 2) && (alive_count == 2 || alive_count == 3) {
                // Any live cell with two or three live neighbours lives on to the next generation.
                return 1; // alive
            }    

            if (center == 0 || center == 3) && (alive_count == 3) {
                // Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
                return 2; // just born
            }

            if center == 0 {
                // dead stays dead
                return 0; // dead
            }

            if center == 3 {
                // "just dead" becomes fully dead
                return 0; // dead
            }

            // anything else becomes "just dead"
            3 // just dead
        }
    }


    /// https://conwaylife.com/wiki/OCA:HighLife
    /// 
    /// Cells survive from one generation to the next if they have 2 or 3 neighbours, and are born if they have 3 or 6 neighbours.
    pub struct HighLife;

    impl super::CARule for HighLife {
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

    /// https://en.wikipedia.org/wiki/Wireworld
    /// 
    /// 0 = empty
    /// 1 = electron head
    /// 2 = electron tail
    /// 3 = conductor
    pub struct Wireworld;

    impl super::CARule for Wireworld {
        fn apply(center: u8, neighbors: &[u8; 8]) -> u8 {
            if center == 1 { // electron head
                return 2; // electron tail
            }

            if center == 2 { // electron tail
                return 3; // conductor
            }
            
            let electron_head_count: usize = neighbors.iter().filter(|&&value| value == 1).count();
            if center == 3 && (electron_head_count == 1 || electron_head_count == 2) { // conductor with 1 or 2 electron heads
                return 1; // electron head
            }

            center
        }
    }

    /// https://conwaylife.com/wiki/OCA:Serviettes
    /// also known as "Persian rugs"
    pub struct Serviettes;

    impl super::CARule for Serviettes {
        fn apply(center: u8, neighbors: &[u8; 8]) -> u8 {
            let alive_count: usize = neighbors.iter().filter(|&&value| value > 0).count();
            match (center, alive_count) {
                // A dead cell with 2, 3, 4 becomes alive
                (0, 2) | (0, 3) | (0, 4) => 1,
                // In all other cases, the cell dies or remains dead
                _ => 0,
            }
        }
    }

} // mod rule


#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_gameoflife_glider() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 0, 0,
            0, 1, 0, 1, 0, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&input, None);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 6, expected_pixels).expect("image");
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
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&input, None);

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
    fn test_10002_gameoflife_glider_use_outside_color() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 1, 0, 0, 0,
            1, 1, 0, 0, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(5, 5, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&input, Some(0));

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(5, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20000_gameoflife_extra_glider() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 0, 0,
            0, 1, 0, 1, 0, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLifeExtra>::with_image(&input, None);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 2, 3, 0, 0,
            0, 3, 0, 1, 2, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_20001_gameoflife_extra_glider() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 2, 3, 0, 0,
            0, 3, 0, 1, 2, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLifeExtra>::with_image(&input, None);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 3, 2, 0, 0,
            0, 0, 0, 3, 1, 0,
            0, 0, 1, 1, 2, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_30000_highlife_predecessor_replicator() {
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
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::HighLife>::with_image(&input, None);

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

    #[test]
    fn test_40000_wireworld_diode() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 3, 3, 0, 0, 0,
            3, 2, 1, 0, 3, 3, 3, 3,
            0, 0, 0, 3, 3, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(8, 5, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::Wireworld>::with_image(&input, None);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 3, 0, 0, 0,
            3, 3, 2, 0, 3, 3, 3, 3,
            0, 0, 0, 1, 3, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(8, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_40001_wireworld_diode() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 3, 3, 0, 0, 0,
            3, 2, 1, 3, 0, 3, 3, 3,
            0, 0, 0, 3, 3, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(8, 5, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::Wireworld>::with_image(&input, None);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 3, 0, 0, 0,
            3, 3, 2, 1, 0, 3, 3, 3,
            0, 0, 0, 1, 3, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(8, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_50000_serviettes() {
        // Act
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");
        let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::Serviettes>::with_image(&input, None);

        // Act
        ca.step_once();
        let actual: Image = ca.current;
        
        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 0,
            0, 1, 0, 0, 1, 0,
            0, 1, 0, 0, 1, 0,
            0, 0, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
