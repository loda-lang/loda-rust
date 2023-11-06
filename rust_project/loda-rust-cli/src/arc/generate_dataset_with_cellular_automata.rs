use super::{CellularAutomaton, cellular_automaton::rule};
use super::{Image, ImageSize, RandomImage};
use rand::rngs::StdRng;
use rand::SeedableRng;
use crate::arc::HtmlLog;

struct GenerateDataset;

impl GenerateDataset {
    fn curriculum_easy() {
        let sizes = [
            3, 4, 5, 6
        ];
        let temperatures = [
            10, 20, 30, 40, 50, 60, 70, 80, 90
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_10000_do_something() {
        for i in 0..10u64 {
            let size = ImageSize::new(6, 5);
            let step0: Image = RandomImage::two_colors(&mut StdRng::seed_from_u64(i), size, 0, 1, 25).expect("ok");
            let mut images = Vec::<Image>::new();
            images.push(step0.clone());
            let mut ca: CellularAutomaton<_> = CellularAutomaton::<rule::GameOfLife>::with_image(&step0);
            for _ in 0..2 {
                ca.step_once();
                images.push(ca.image().clone());
            }
            HtmlLog::compare_images(images);
        }
    }
}
