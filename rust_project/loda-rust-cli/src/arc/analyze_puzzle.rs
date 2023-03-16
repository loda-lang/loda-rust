use super::{Image, convolution2x2};
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
    bloom_normal: Bloom::<String>,
    bloom_flipped: Bloom::<String>,
    bloom_rotated: Bloom::<String>,
}

impl AnalyzePuzzle {
    /// Populate bloomfilter
    fn analyze(image: &Image) -> anyhow::Result<Self> {
        HtmlLog::html(image.to_html());

        let count = Cell::<u64>::new(0);
        let state_normal = RefCell::<State>::new(State::new());
        let state_flipped = RefCell::<State>::new(State::new());
        let state_rotated = RefCell::<State>::new(State::new());
        let _buffer_image: Image = convolution2x2(&image, |bm| {
            let mut c: u64 = count.get();
            c += 1;
            count.set(c);

            let tl: u8 = bm.get(0, 0).unwrap_or(255);
            let tr: u8 = bm.get(1, 0).unwrap_or(255);
            let bl: u8 = bm.get(0, 1).unwrap_or(255);
            let br: u8 = bm.get(1, 1).unwrap_or(255);
            {
                // normal
                let s = format!("{},{}\n{},{}", tl, tr, bl, br);
                state_normal.borrow_mut().bloom.set(&s);
            }
            {
                // flip x
                let s = format!("{},{}\n{},{}", tr, tl, br, bl);
                state_flipped.borrow_mut().bloom.set(&s);
            }
            {
                // flip y
                let s = format!("{},{}\n{},{}", bl, br, tl, tr);
                state_flipped.borrow_mut().bloom.set(&s);
            }
            {
                // rotate cw 90
                let s = format!("{},{}\n{},{}", bl, tl, br, tr);
                state_rotated.borrow_mut().bloom.set(&s);
            }
            {
                // rotate cw 180
                let s = format!("{},{}\n{},{}", br, bl, tr, tl);
                state_rotated.borrow_mut().bloom.set(&s);
            }
            {
                // rotate cw 270
                let s = format!("{},{}\n{},{}", tr, br, tl, bl);
                state_rotated.borrow_mut().bloom.set(&s);
            }
            Ok(0)
        })?;
        println!("count: {}", count.get());

        // let bloom_key: String = "9,9,9\n9,9,9\n9,9,9".to_string();
        // let is_contained: bool = state.borrow().bloom.check(&bloom_key);
        // println!("is_contained: {}", is_contained);

        let instance = Self {
            bloom_normal: state_normal.borrow().bloom.clone(),
            bloom_flipped: state_flipped.borrow().bloom.clone(),
            bloom_rotated: state_rotated.borrow().bloom.clone(),
        };
        Ok(instance)
    }

    fn compute_score(&self, bloom_key: &String) -> u8 {
        let mut score: u8 = 0;
        if self.bloom_normal.check(bloom_key) {
            score += 1;
        }
        if self.bloom_flipped.check(bloom_key) {
            score += 2;
        }
        if self.bloom_rotated.check(bloom_key) {
            score += 4;
        }
        score
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

            let score = self.compute_score(&s);
            Ok(score)
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
    use crate::arc::arc_json_model;
    use crate::arc::arc_json_model::GridToImage;

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
        let _xinput: Image = Image::try_create(6, 7, input_pixels).expect("image");
        
        let output_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 0, 0,
            3, 9, 9, 9, 9, 9, 0, 0,
        ];
        let _xoutput: Image = Image::try_create(8, 7, output_pixels).expect("image");

        // let model: Model = Model::load_testdata("ea959feb").expect("model");
        // let model: Model = Model::load_testdata("dbc1a6ce").expect("model");
        // let model: Model = Model::load_testdata("72ca375d").expect("model");
        // let model: Model = Model::load_testdata("80af3007").expect("model");
        // let model: Model = Model::load_testdata("1f85a75f").expect("model");
        // let model: Model = Model::load_testdata("d687bc17").expect("model");
        let model: arc_json_model::Model = arc_json_model::Model::load_testdata("6b9890af").expect("model");
        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");


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
