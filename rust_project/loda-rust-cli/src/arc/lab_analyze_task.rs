use super::{Image, convolution2x2};
use bloomfilter::*;
use std::cell::Cell;
use std::cell::RefCell;

#[allow(unused_imports)]
use crate::arc::{HtmlLog, ImageToHTML};

const DUMP_TO_CONSOLE: bool = false;

/// This is unused code, and experimental.
/// 
/// This code identifies what is happening with the pixels in an ARC task.
/// - Do they stay at the same place.
/// - Do they move around.
/// - Do they get rotated.
/// - Do they get flipped.
/// 
/// If there is consensus between all the training tasks, that the pixels gets rotated,
/// Then it's likely that the grid gets rotated.
/// 
/// If there is consensus between all the training tasks, that the pixels preserve the orientation and doesn't get flipped.
/// Then the pixels may be moved around.
/// 
/// If it's mixed, some pixels being flipped, others pixels being rotated, 
/// Then it may be harder to solve the task.
/// 
/// It's unclear to me how to use this knowledge to guide the mutations.
/// Maybe store it in a HashMap, and make it available to `genome.rs` so it can pick wiser mutations. 

#[allow(dead_code)]
struct State {
    bloom: Bloom::<String>,
}

impl State {
    #[allow(dead_code)]
    fn new() -> Self {
        let bloom_items_count = 1000;
        let false_positive_rate = 0.01;
        let bloom = Bloom::<String>::new_for_fp_rate(bloom_items_count, false_positive_rate);

        Self {
            bloom
        }
    }
}


#[allow(dead_code)]
struct AnalyzeTask {
    bloom_normal: Bloom::<String>,
    bloom_flipped: Bloom::<String>,
    bloom_rotated: Bloom::<String>,
}

impl AnalyzeTask {
    /// Populate bloomfilter
    #[allow(dead_code)]
    fn analyze(image: &Image) -> anyhow::Result<Self> {
        if DUMP_TO_CONSOLE {
            HtmlLog::html(image.to_html());
        }

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
        if DUMP_TO_CONSOLE {
            println!("count: {}", count.get());
        }

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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn compare(&self, image: &Image) -> anyhow::Result<()> {
        if DUMP_TO_CONSOLE {
            HtmlLog::html(image.to_html());
        }

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
        if DUMP_TO_CONSOLE {
            println!("count: {}", count.get());
            HtmlLog::html(buffer_image.to_html());
        }

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

        // let task: arc_json_model::Task = arc_json_model::Task::load_testdata("ea959feb").expect("task");
        // let task: arc_json_model::Task = arc_json_model::Task::load_testdata("dbc1a6ce").expect("task");
        // let task: arc_json_model::Task = arc_json_model::Task::load_testdata("72ca375d").expect("task");
        // let task: arc_json_model::Task = arc_json_model::Task::load_testdata("80af3007").expect("task");
        // let task: arc_json_model::Task = arc_json_model::Task::load_testdata("1f85a75f").expect("task");
        // let task: arc_json_model::Task = arc_json_model::Task::load_testdata("d687bc17").expect("task");
        let task: arc_json_model::Task = arc_json_model::Task::load_testdata("6b9890af").expect("task");
        let input: Image = task.train()[0].input().to_image().expect("image");
        let output: Image = task.train()[0].output().to_image().expect("image");


        // populate bloomfilter for input
        let ap0: AnalyzeTask = AnalyzeTask::analyze(&input).expect("ok");
        // identify what parts of the output is contained in the input bloomfilter
        ap0.compare(&output).expect("ok");

        // populate bloomfilter for output
        let ap1: AnalyzeTask = AnalyzeTask::analyze(&output).expect("ok");
        // identify what parts of the input is contained in the input bloomfilter
        ap1.compare(&input).expect("ok");

        // Plan for what to do next with this experiment:
        // populate bloomfilter for input
        // populate bloomfilter for output
        //
        // stats_buffer_input, for each 3x3 slice of the input, if identical to output, if so set bit0=1
        // stats_buffer_input, for each 3x3 slice of the input, check if it exist in the bloomfilter for input, if so set bit1=1
        // stats_buffer_input, for each 3x3 slice of the input, check if it exist in the bloomfilter for output, if so set bit2=1
        // stats_buffer_input, for each 3x3 slice of the input, check if it exist in the bloomfilter for all board inputs, if so set bit3=1
        // stats_buffer_input, for each 3x3 slice of the input, check if it exist in the bloomfilter for all board outputs, if so set bit4=1
        // dump the stats_buffer_input to console
        //
        // stats_buffer_output, for each 3x3 slice of the output, if identical to input, if so set bit0=1
        // stats_buffer_output, for each 3x3 slice of the output, check if it exist in the bloomfilter for input, if so set bit1=1
        // stats_buffer_output, for each 3x3 slice of the output, check if it exist in the bloomfilter for output, if so set bit2=1
        // stats_buffer_output, for each 3x3 slice of the input, check if it exist in the bloomfilter for all board inputs, if so set bit3=1
        // stats_buffer_output, for each 3x3 slice of the input, check if it exist in the bloomfilter for all board outputs, if so set bit4=1
        // dump the stats_buffer_output to console
        //
        // in the console take a look at stats_buffer_input,stats_buffer_output and look for patterns.

    }
}
