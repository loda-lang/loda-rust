#[cfg(test)]
mod tests {
    use bit_set::BitSet;

    use crate::arc::{Model, GridToBitmap, BitmapFind};
    use crate::arc::{Bitmap, convolution2x2, convolution3x3};
    use crate::arc::{BitmapResize, BitmapTrim, BitmapRemoveDuplicates, Padding};
    use crate::arc::{BitmapReplaceColor, BitmapSymmetry, BitmapOffset};
    use crate::arc::{BitmapNgram, RecordBigram, RecordTrigram};

    #[test]
    fn test_10000_puzzle_4258a5f9() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("4258a5f9")?;
        assert_eq!(model.train().len(), 2);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train[1].input.to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train[1].output.to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test[0].input.to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test[0].output.to_bitmap().expect("bitmap");

        let input_padded: Bitmap = input.zero_padding(1).expect("bitmap");

        let result_bm: Bitmap = convolution3x3(&input_padded, |bm| {
            let mut found = false;
            for y in 0..3i32 {
                for x in 0..3i32 {
                    if x == 1 && y == 1 {
                        continue;
                    }
                    let pixel_value: u8 = bm.get(x, y).unwrap_or(255);
                    if pixel_value == 5 {
                        found = true;
                    }
                }
            }
            let mut value: u8 = bm.get(1, 1).unwrap_or(255);
            if found {
                value = 1;
            }
            Ok(value)
        }).expect("bitmap");

        assert_eq!(result_bm, output);

        Ok(())
    }

    #[test]
    fn test_20000_puzzle_5614dbcf() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("5614dbcf")?;
        assert_eq!(model.train().len(), 2);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        let input_padded: Bitmap = input.zero_padding(1).expect("bitmap");

        let result_bm: Bitmap = convolution3x3(&input_padded, |bm| {
            let value: u8 = bm.get(1, 1).unwrap_or(255);
            if value != 5 {
                // not a noisy pixel
                return Ok(value);
            }
            // this is a noise pixel. Look at the surrounding pixels, and take the most popular
            let mut histogram: Vec<u8> = vec![0; 256];
            for y in 0..3i32 {
                for x in 0..3i32 {
                    let pixel_value: u8 = bm.get(x, y).unwrap_or(255);
                    let original_count: u8 = match histogram.get(pixel_value as usize) {
                        Some(value) => *value,
                        None => {
                            return Err(anyhow::anyhow!("Integrity error. Counter in histogram out of bounds"));
                        }
                    };
                    let count: u8 = (original_count + 1) & 255;
                    histogram[pixel_value as usize] = count;
                }
            }
            let mut found_count: u8 = 0;
            let mut found_value: usize = 0;
            for (pixel_value, number_of_occurences) in histogram.iter().enumerate() {
                if *number_of_occurences > found_count {
                    found_count = *number_of_occurences;
                    found_value = pixel_value;
                }
            }
            let value: u8 = (found_value & 255) as u8;
            Ok(value)
        }).expect("bitmap");

        let result_bm2 = result_bm.resize(3, 3).expect("bitmap");
        assert_eq!(result_bm2, output);
        Ok(())
    }

    #[test]
    fn test_30000_puzzle_2013d3e2() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("2013d3e2")?;
        assert_eq!(model.train().len(), 2);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        let input_trimmed: Bitmap = input.trim().expect("bitmap");

        let mut result_bitmap = Bitmap::zeroes(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                let pixel_value: u8 = input_trimmed.get(x, y).unwrap_or(255);
                match result_bitmap.set(x, y, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Unable to set pixel inside the result bitmap"));
                    }
                }
            }
        }

        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_40000_puzzle_90c28cc7() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("90c28cc7")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[2].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[2].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        let input_trimmed: Bitmap = input.trim().expect("bitmap");
        let result_bitmap: Bitmap = input_trimmed.remove_duplicates().expect("bitmap");
        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_50000_puzzle_7468f01a() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("7468f01a")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[2].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[2].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        let input_trimmed: Bitmap = input.trim().expect("bitmap");
        let result_bitmap: Bitmap = input_trimmed.flip_x().expect("bitmap");
        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_60000_puzzle_63613498() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("63613498")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[2].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[2].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        // Extract needle
        let mut needle: Bitmap = Bitmap::zeroes(3, 3);
        let center_pixel_color: u8 = input.get(1, 1).unwrap_or(255);
        for y in 0..3i32 {
            for x in 0..3i32 {
                let pixel_value: u8 = input.get(x, y).unwrap_or(255);
                let mut mask_value: u8 = 0;
                if pixel_value == center_pixel_color {
                    mask_value = 1;
                }
                match needle.set(x, y, mask_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) inside the needle bitmap", x, y));
                    }
                }
            }
        }

        // Clear the needle area from the search area
        let mut search_area: Bitmap = input.clone();
        for y in 0..4i32 {
            for x in 0..4i32 {
                match search_area.set(x, y, 0) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) inside the search area", x, y));
                    }
                }
            }
        }
        // println!("needle: {:?}", needle);
        // println!("search area: {:?}", search_area);

        // Find the pattern
        let mut optional_position: Option<(u8, u8)> = None;
        for color in 1..=255u8 {
            let needle_with_color: Bitmap = needle.replace_color(1, color)?;
            optional_position = search_area.find_exact(&needle_with_color).expect("some position");
            if optional_position == None {
                continue;
            }
            break;
        }
        let position: (u8, u8) = match optional_position {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Didn't find needle inside the search area"));
            }
        };
        // println!("position: {:?}", position);

        // Clear color of the found pattern
        let mut result_bitmap: Bitmap = input.clone();
        for y in 0..3i32 {
            for x in 0..3i32 {
                let xx = x + (position.0 as i32);
                let yy = y + (position.1 as i32);
                let pixel_value: u8 = needle.get(x, y).unwrap_or(255);
                if pixel_value == 0 {
                    continue;
                }
                match result_bitmap.set(xx, yy, 5) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) in the result_bitmap", x, y));
                    }
                }
            }
        }

        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_70000_puzzle_cdecee7f() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("cdecee7f")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[2].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[2].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        // Traverse columns
        let mut stack: Vec<u8> = vec!();
        for x in 0..input.width() {
            // Take the pixel greater than 0 and append the pixel to the stack
            for y in 0..input.height() {
                let pixel_value: u8 = input.get(x as i32, y as i32).unwrap_or(255);
                if pixel_value > 0 {
                    stack.push(pixel_value);
                }
            }
        }
        // Padding to 9 items
        while stack.len() < 9 {
            stack.push(0);
        }

        // Transfer values from the 9 element stack to the 3x3 bitmap
        let mut result_bitmap: Bitmap = Bitmap::zeroes(3, 3);
        for (index, pixel_value) in stack.iter().enumerate() {
            let y: usize = index / 3;
            let mut x: usize = index % 3;
            if y == 1 {
                // The middle row is reversed
                x = 2 - x;
            }
            let set_x: i32 = x as i32;
            let set_y: i32 = y as i32;
            match result_bitmap.set(set_x, set_y, *pixel_value) {
                Some(()) => {},
                None => {
                    return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) in the result_bitmap", x, y));
                }
            }
        }

        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_80000_puzzle_007bbfb7() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("007bbfb7")?;
        assert_eq!(model.train().len(), 5);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[2].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[2].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[3].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[3].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        let mut result_bitmap: Bitmap = Bitmap::zeroes(9, 9);
        for y in 0..input.height() {
            for x in 0..input.width() {
                let mask_value: u8 = input.get(x as i32, y as i32).unwrap_or(255);
                if mask_value == 0 {
                    continue;
                }
                // Copy the entire input image
                for yy in 0..input.height() {
                    for xx in 0..input.width() {
                        let pixel_value: u8 = input.get(xx as i32, yy as i32).unwrap_or(255);
                        let set_x: i32 = (xx as i32) + (x as i32) * 3;
                        let set_y: i32 = (yy as i32) + (y as i32) * 3;
                        match result_bitmap.set(set_x, set_y, pixel_value) {
                            Some(()) => {},
                            None => {
                                return Err(anyhow::anyhow!("Unable to set pixel ({}, {}) in the result_bitmap", set_x, set_y));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_90000_puzzle_b9b7f026() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("b9b7f026")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[2].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[2].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        // Detect corners
        let corner_bitmap: Bitmap = convolution2x2(&input, |bm| {
            let pixel00: u8 = bm.get(0, 0).unwrap_or(255);
            let pixel10: u8 = bm.get(1, 0).unwrap_or(255);
            let pixel01: u8 = bm.get(0, 1).unwrap_or(255);
            let pixel11: u8 = bm.get(1, 1).unwrap_or(255);
            let mut mask: u8 = 0;
            if pixel00 == pixel10 { mask |= 1; }
            if pixel01 == pixel11 { mask |= 2; }
            if pixel00 == pixel01 { mask |= 4; }
            if pixel10 == pixel11 { mask |= 8; }
            let value: u8 = match mask {
                5 => pixel00,
                6 => pixel01,
                9 => pixel10,
                10 => pixel11,
                _ => 0,
            };
            Ok(value)
        }).expect("bitmap");

        // println!("input: {:?}", input);
        // println!("bitmap0: {:?}", bitmap0);

        // Extract color of the corner
        let mut result_bitmap: Bitmap = Bitmap::zeroes(1, 1);
        for y in 0..corner_bitmap.height() {
            for x in 0..corner_bitmap.width() {
                let pixel_value: u8 = corner_bitmap.get(x as i32, y as i32).unwrap_or(255);
                if pixel_value == 0 {
                    continue;
                }
                match result_bitmap.set(0, 0, pixel_value) {
                    Some(()) => {},
                    None => {
                        return Err(anyhow::anyhow!("Unable to set pixel in the result_bitmap"));
                    }
                }
            }
        }
        assert_eq!(result_bitmap, output);
        Ok(())
    }

    #[test]
    fn test_100000_puzzle_a79310a0() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("a79310a0")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[2].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[2].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        let bitmap0: Bitmap = input.offset_wrap(0, 1).expect("bitmap");
        let bitmap1: Bitmap = bitmap0.replace_color(8, 2).expect("bitmap");

        assert_eq!(bitmap1, output);
        Ok(())
    }

    /// Detect corners and edges
    fn mask_and_repair_areas(input: &Bitmap) -> anyhow::Result<(Bitmap, Bitmap)> {
        // Assign 0 to background, assign 1 to foreground
        let mut mask: Bitmap = input.clone();
        for pixel_value in 2..=255 {
            mask = mask.replace_color(pixel_value, 1).expect("bitmap");
        }
        // println!("mask: {:?}", mask);

        // Detect corners and edges
        let repair_areas: Bitmap = convolution2x2(&mask, |bm| {
            let pixel00: u8 = bm.get(0, 0).unwrap_or(255);
            let pixel10: u8 = bm.get(1, 0).unwrap_or(255);
            let pixel01: u8 = bm.get(0, 1).unwrap_or(255);
            let pixel11: u8 = bm.get(1, 1).unwrap_or(255);
            let number_of_zeros: u8 = 
                u8::min(pixel00, 1) + 
                u8::min(pixel10, 1) + 
                u8::min(pixel01, 1) + 
                u8::min(pixel11, 1);
            if number_of_zeros <= 1 {
                // 1 mask pixel turned on, and 3 pixels is the background, don't consider this as a corner.
                // 0 mask pixels turned on, all 4 pixels are the background then ignore.
                return Ok(0);
            }

            let mut mask: u8 = 0;
            if pixel00 == pixel10 { mask |= 1; }
            if pixel01 == pixel11 { mask |= 2; }
            if pixel00 == pixel01 { mask |= 4; }
            if pixel10 == pixel11 { mask |= 8; }
            let value: u8 = match mask {
                3 => 5, // edge, rows differ
                5 => 1, // corner top left
                6 => 2, // corner bottom left
                9 => 3, // corner top right
                10 => 4, // corner bottom right
                12 => 6, // edge, columns differ
                _ => 0,
            };
            Ok(value)
        }).expect("bitmap");
        Ok((mask, repair_areas))
    }

    fn repair_corner_top_left(bitmap: &mut Bitmap, x: i32, y: i32, trigram_x: &Vec<RecordTrigram>, trigram_y: &Vec<RecordTrigram>) -> anyhow::Result<()> {
        if x < 1 || y < 1 {
            println!("repair corner top left: {}, {} - insufficient room to make bigram", x, y);
            return Ok(());
        }
        println!("repair corner top left: {}, {}", x, y);

        let pixel_top_top: u8 = bitmap.get(x, y - 2).unwrap_or(255);
        let pixel_top: u8 = bitmap.get(x, y - 1).unwrap_or(255);
        let pixel_left_left: u8 = bitmap.get(x - 2, y).unwrap_or(255);
        let pixel_left: u8 = bitmap.get(x - 1, y).unwrap_or(255);

        let mut bitset_trigram_x = BitSet::with_capacity(256);
        let mut bitset_trigram_y = BitSet::with_capacity(256);
        for candidate in 0..255u8 {
            for record in trigram_x.iter() {
                if record.word0 == pixel_left_left && record.word1 == pixel_left && record.word2 == candidate {
                    bitset_trigram_x.insert(candidate as usize);
                }
            }
            for record in trigram_y.iter() {
                if record.word0 == pixel_top_top && record.word1 == pixel_top && record.word2 == candidate {
                    bitset_trigram_y.insert(candidate as usize);
                }
            }
        }
        let mut bitset_trigram = BitSet::with_capacity(256);
        bitset_trigram.clone_from(&bitset_trigram_x);
        bitset_trigram.intersect_with(&bitset_trigram_y);
        if bitset_trigram.len() >= 2 {
            println!("more than 1 candidate. trigram: {:?}", bitset_trigram);
        }

        let mut found_color = 255;
        for index in bitset_trigram.iter() {
            if index > 255 {
                return Err(anyhow::anyhow!("Integrity error. Encountered bitset index outside of u8 range [0..255]"));
            }
            found_color = index as u8;
            break;
        }
        println!("repair ({}, {}) = {:?}", x, y, found_color);

        match bitmap.set(x, y, found_color) {
            Some(()) => {},
            None => {
                return Err(anyhow::anyhow!("Unable to set pixel inside the result bitmap"));
            }
        }
        Ok(())
    }

    #[test]
    fn test_110000_puzzle_0dfd9992() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("0dfd9992")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Bitmap = model.train()[0].input().to_bitmap().expect("bitmap");
        let output: Bitmap = model.train()[0].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[1].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[1].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.train()[2].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.train()[2].output().to_bitmap().expect("bitmap");
        // let input: Bitmap = model.test()[0].input().to_bitmap().expect("bitmap");
        // let output: Bitmap = model.test()[0].output().to_bitmap().expect("bitmap");

        // Bigrams
        // let bigram_x_unfiltered: Vec<RecordBigram> = input.bigram_x().expect("bigram");
        // let bigram_y_unfiltered: Vec<RecordBigram> = input.bigram_y().expect("bigram");
        // println!("bigram_x_unfiltered: {:?}", bigram_x_unfiltered);
        // println!("bigram_y_unfiltered: {:?}", bigram_y_unfiltered);
        // Remove bigrams where the background pixel (0) is contained in
        // let bigram_x_refs: Vec<&RecordBigram> = bigram_x_unfiltered.iter().filter(|&record| {
        //     record.word0 != 0 && record.word1 != 0
        // }).collect();
        // let bigram_y_refs: Vec<&RecordBigram> = bigram_y_unfiltered.iter().filter(|&record| {
        //     record.word0 != 0 && record.word1 != 0
        // }).collect();
        // let bigram_x: Vec<RecordBigram> = bigram_x_refs.iter().map(|&i| i.clone()).collect();
        // let bigram_y: Vec<RecordBigram> = bigram_y_refs.iter().map(|&i| i.clone()).collect();
        // println!("bigram_x: {:?}", bigram_x);
        // println!("bigram_y: {:?}", bigram_y);

        // Trigrams
        let trigram_x_unfiltered: Vec<RecordTrigram> = input.trigram_x().expect("trigram");
        let trigram_y_unfiltered: Vec<RecordTrigram> = input.trigram_y().expect("trigram");
        // println!("trigram_x_unfiltered: {:?}", trigram_x_unfiltered);
        // println!("trigram_y_unfiltered: {:?}", trigram_y_unfiltered);
        // Remove trigrams where the background pixel (0) is contained in
        let trigram_x_refs: Vec<&RecordTrigram> = trigram_x_unfiltered.iter().filter(|&record| {
            record.word0 != 0 && record.word1 != 0 && record.word2 != 0
        }).collect();
        let trigram_y_refs: Vec<&RecordTrigram> = trigram_y_unfiltered.iter().filter(|&record| {
            record.word0 != 0 && record.word1 != 0 && record.word2 != 0
        }).collect();
        let trigram_x: Vec<RecordTrigram> = trigram_x_refs.iter().map(|&i| i.clone()).collect();
        let trigram_y: Vec<RecordTrigram> = trigram_y_refs.iter().map(|&i| i.clone()).collect();
        // println!("trigram_x: {:?}", trigram_x);
        // println!("trigram_y: {:?}", trigram_y);
        
        let mut result_bitmap: Bitmap = input.clone();

        let mut last_repair_count: usize = 0;
        for iteration in 0..13 {
            let (mask, repair_areas) = mask_and_repair_areas(&result_bitmap)?;
            println!("iteration#{} mask: {:?}", iteration, mask);
            println!("iteration#{} repair areas: {:?}", iteration, repair_areas);
            let mut repair_count: usize = 0;
            for y in 0..repair_areas.height() {
                for x in 0..repair_areas.width() {
                    let pixel_value: u8 = repair_areas.get(x as i32, y as i32).unwrap_or(255);
                    if pixel_value == 0 {
                        continue;
                    }
                    repair_count += 1;
    
                    // repair position
                    let repair_x = (x as i32) + 1;
                    let repair_y = (y as i32) + 1;
    
                    // if pixel_value >= 1 && pixel_value <= 4 {
                    //     println!("repair corner: {}, {}", x, y);
                    //     TODO: deal with all the cases
                    // }
                    // if pixel_value >= 5 {
                    //     println!("repair edge: {}, {}", x, y);
                    // }
    
                    if pixel_value == 1 {
                        repair_corner_top_left(&mut result_bitmap, repair_x, repair_y, &trigram_x, &trigram_y)?;
                    }
                }
            }
            println!("iteration#{} repair_count: {}", iteration, repair_count);
            if iteration > 0 {
                if last_repair_count == repair_count {
                    println!("making no progress with repairs done. aborting");
                    break;
                }
            }
            if repair_count == 0 {
                println!("repair done. no more pixels to be repaired");
                break;
            }
            last_repair_count = repair_count;
        }

        assert_eq!(result_bitmap, output);
        Ok(())
    }
}
