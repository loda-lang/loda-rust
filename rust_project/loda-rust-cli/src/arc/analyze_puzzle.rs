use super::{Image, convolution2x2, ImagePadding};
use bloomfilter::*;
use std::cell::Cell;
use std::cell::RefCell;

#[allow(unused_imports)]
use crate::arc::{HtmlLog, ImageToHTML};

struct State {
    bloom: Bloom::<String>,
}

impl State {
    fn new() -> Self {
        let bloom_items_count = 1000;
        let false_positive_rate = 0.01;
        let bloom = Bloom::<String>::new_for_fp_rate(bloom_items_count, false_positive_rate);

        Self {
            bloom
        }
    }
}


struct AnalyzePuzzle {
    bloom: Bloom::<String>,
}

impl AnalyzePuzzle {
    /// Populate bloomfilter
    fn analyze(image: &Image) -> anyhow::Result<Self> {
        HtmlLog::html(image.to_html());

        let count = Cell::<u64>::new(0);
        let state = RefCell::<State>::new(State::new());
        let _buffer_image: Image = convolution2x2(&image, |bm| {
            let mut c: u64 = count.get();
            c += 1;
            count.set(c);

            let tl: u8 = bm.get(0, 0).unwrap_or(255);
            let tr: u8 = bm.get(1, 0).unwrap_or(255);
            let bl: u8 = bm.get(0, 1).unwrap_or(255);
            let br: u8 = bm.get(1, 1).unwrap_or(255);
            let s = format!("{},{}\n{},{}", tl, tr, bl, br);

            // insert into bloomfilter
            state.borrow_mut().bloom.set(&s);

            // TODO: generate hashes for rotated/flipped variants
            // TODO: insert hashes into another bloomfilter

            Ok(0)
        })?;
        println!("count: {}", count.get());

        // let bloom_key: String = "9,9,9\n9,9,9\n9,9,9".to_string();
        // let is_contained: bool = state.borrow().bloom.check(&bloom_key);
        // println!("is_contained: {}", is_contained);

        let instance = Self {
            bloom: state.borrow().bloom.clone(),
        };
        Ok(instance)
    }

    fn check(&self, bloom_key: &String) -> bool {
        self.bloom.check(bloom_key)
    }

    fn compare(&self, image: &Image) -> anyhow::Result<()> {
        HtmlLog::html(image.to_html());

        let count = Cell::<u64>::new(0);
        let buffer_image: Image = convolution2x2(&image, |bm| {

            let mut c: u64 = count.get();
            c += 1;
            count.set(c);

            let tl: u8 = bm.get(0, 0).unwrap_or(255);
            let tr: u8 = bm.get(1, 0).unwrap_or(255);
            let bl: u8 = bm.get(0, 1).unwrap_or(255);
            let br: u8 = bm.get(1, 1).unwrap_or(255);
            let s = format!("{},{}\n{},{}", tl, tr, bl, br);

            let mut value: u8 = 0;
            if self.check(&s) {
                value += 1;
            }

            Ok(value)
        })?;
        println!("count: {}", count.get());
        HtmlLog::html(buffer_image.to_html());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_analyze() {
        let input_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 0, 0, 9, 9, 9,
            9, 0, 0, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
        ];
        let input: Image = Image::try_create(6, 7, input_pixels).expect("image");
        
        let output_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 0, 0,
            3, 9, 9, 9, 9, 9, 0, 0,
        ];
        let output: Image = Image::try_create(8, 7, output_pixels).expect("image");

        // populate bloomfilter for input
        let ap0: AnalyzePuzzle = AnalyzePuzzle::analyze(&input).expect("ok");
        // identify what parts of the output is contained in the input bloomfilter
        ap0.compare(&output).expect("ok");

        // populate bloomfilter for output
        let ap1: AnalyzePuzzle = AnalyzePuzzle::analyze(&output).expect("ok");
        // identify what parts of the input is contained in the input bloomfilter
        ap1.compare(&input).expect("ok");

        // TODO: populate bloomfilter for input
        // TODO: populate bloomfilter for output

        // TODO: stats_buffer_input, for each 3x3 slice of the input, if identical to output, if so set bit0=1
        // TODO: stats_buffer_input, for each 3x3 slice of the input, check if it exist in the bloomfilter for input, if so set bit1=1
        // TODO: stats_buffer_input, for each 3x3 slice of the input, check if it exist in the bloomfilter for output, if so set bit2=1
        // TODO: stats_buffer_input, for each 3x3 slice of the input, check if it exist in the bloomfilter for all board inputs, if so set bit3=1
        // TODO: stats_buffer_input, for each 3x3 slice of the input, check if it exist in the bloomfilter for all board outputs, if so set bit4=1
        // TODO: dump the stats_buffer_input to console

        // TODO: stats_buffer_output, for each 3x3 slice of the output, if identical to input, if so set bit0=1
        // TODO: stats_buffer_output, for each 3x3 slice of the output, check if it exist in the bloomfilter for input, if so set bit1=1
        // TODO: stats_buffer_output, for each 3x3 slice of the output, check if it exist in the bloomfilter for output, if so set bit2=1
        // TODO: stats_buffer_output, for each 3x3 slice of the input, check if it exist in the bloomfilter for all board inputs, if so set bit3=1
        // TODO: stats_buffer_output, for each 3x3 slice of the input, check if it exist in the bloomfilter for all board outputs, if so set bit4=1
        // TODO: dump the stats_buffer_output to console

        // TODO: in the console take a look at stats_buffer_input,stats_buffer_output and look for patterns.

    }
}
