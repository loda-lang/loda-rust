use super::{Image, convolution3x3, ImagePadding};
use bloomfilter::*;
use std::cell::Cell;

#[allow(unused_imports)]
use crate::arc::{HtmlLog, ImageToHTML};

struct State {
    bloom: Bloom::<String>
}

impl State {
    fn new() -> Self {
        let bloom_items_count = 1000000;
        let false_positive_rate = 0.01;
        let bloom = Bloom::<String>::new_for_fp_rate(bloom_items_count, false_positive_rate);

        Self {
            bloom
        }
    }
}


struct AnalyzePuzzle;

impl AnalyzePuzzle {
    fn populate_bloomfilter(image: &Image) -> anyhow::Result<()> {
        HtmlLog::html(image.to_html());

        let count = Cell::<u64>::new(0);
        let buffer_image: Image = convolution3x3(&image, |bm| {
            let mut c: u64 = count.get();
            c += 1;
            count.set(c);

            // TODO: how do I mutate the bloomfilter from within this callback?
            let tl: u8 = bm.get(0, 0).unwrap_or(255);
            let tc: u8 = bm.get(1, 0).unwrap_or(255);
            let tr: u8 = bm.get(2, 0).unwrap_or(255);
            let cl: u8 = bm.get(0, 1).unwrap_or(255);
            let cc: u8 = bm.get(1, 1).unwrap_or(255);
            let cr: u8 = bm.get(2, 1).unwrap_or(255);
            let bl: u8 = bm.get(0, 2).unwrap_or(255);
            let bc: u8 = bm.get(1, 2).unwrap_or(255);
            let br: u8 = bm.get(2, 2).unwrap_or(255);

            // TODO: generate hash
            // TODO: insert into bloomfilter

            // TODO: generate hashes for rotated/flipped variants
            // TODO: insert hashes into another bloomfilter

            Ok(0)
        })?;

        println!("count: {}", count.get());
        // TODO: return bloomfilter

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
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
        ];
        let input: Image = Image::try_create(6, 5, input_pixels).expect("image");

        let output_pixels: Vec<u8> = vec![
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9,
        ];
        let output: Image = Image::try_create(6, 5, output_pixels).expect("image");

        // populate bloomfilter for input
        AnalyzePuzzle::populate_bloomfilter(&input).expect("ok");
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
