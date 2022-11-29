#[cfg(test)]
mod tests {
    use bit_set::BitSet;

    use crate::arc::{Model, GridToBitmap, BitmapFind};
    use crate::arc::{Bitmap, convolution2x2, convolution3x3};
    use crate::arc::{BitmapResize, BitmapTrim, BitmapRemoveDuplicates, Padding};
    use crate::arc::{BitmapReplaceColor, BitmapSymmetry, BitmapOffset, BitmapBigram, RecordBigram};

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

    // #[test]
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

        let bigram_x_unfiltered: Vec<RecordBigram> = input.bigram_x().expect("bitmap");
        let bigram_y_unfiltered: Vec<RecordBigram> = input.bigram_y().expect("bitmap");
        // println!("bigram_x_unfiltered: {:?}", bigram_x_unfiltered);
        // println!("bigram_y_unfiltered: {:?}", bigram_y_unfiltered);

        // Remove bigrams where the background pixel (0) is contained in
        let bigram_x_refs: Vec<&RecordBigram> = bigram_x_unfiltered.iter().filter(|&record| {
            record.word0 != 0 && record.word1 != 0
        }).collect();
        let bigram_y_refs: Vec<&RecordBigram> = bigram_y_unfiltered.iter().filter(|&record| {
            record.word0 != 0 && record.word1 != 0
        }).collect();
        let bigram_x: Vec<RecordBigram> = bigram_x_refs.iter().map(|&i| i.clone()).collect();
        let bigram_y: Vec<RecordBigram> = bigram_y_refs.iter().map(|&i| i.clone()).collect();
        // println!("bigram_x: {:?}", bigram_x);
        // println!("bigram_y: {:?}", bigram_y);
        
        let mut mask: Bitmap = input.clone();
        for pixel_value in 2..=255 {
            mask = mask.replace_color(pixel_value, 1).expect("bitmap");
        }
        println!("mask: {:?}", mask);

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

        println!("repair areas: {:?}", repair_areas);

        let mut result_bitmap: Bitmap = input.clone();
        for y in 0..repair_areas.height() {
            for x in 0..repair_areas.width() {
                let pixel_value: u8 = repair_areas.get(x as i32, y as i32).unwrap_or(255);
                if pixel_value == 0 {
                    continue;
                }

                // repair position
                let repair_x = (x as i32) + 1;
                let repair_y = (y as i32) + 1;

                // if pixel_value >= 1 && pixel_value <= 4 {
                //     println!("repair corner: {}, {}", x, y);
                // }
                // if pixel_value >= 5 {
                //     println!("repair edge: {}, {}", x, y);
                // }

                if pixel_value == 1 {
                    if repair_x < 1 || repair_y < 1 {
                        println!("repair top left corner: {}, {} - insufficient room to make bigram", repair_x, repair_y);
                        continue;
                    }
                    println!("repair top left corner: {}, {}", repair_x, repair_y);

                    let pixel_top: u8 = input.get(repair_x, repair_y - 1).unwrap_or(255);
                    let pixel_left: u8 = input.get(repair_x - 1, repair_y).unwrap_or(255);


                    let mut bitset_x = BitSet::with_capacity(256);
                    let mut bitset_y = BitSet::with_capacity(256);
                    for candidate in 0..255u8 {
                        for record in bigram_x.iter() {
                            if record.word0 == pixel_left && record.word1 == candidate {
                                bitset_x.insert(candidate as usize);
                            }
                        }
                        for record in bigram_y.iter() {
                            if record.word0 == pixel_top && record.word1 == candidate {
                                bitset_y.insert(candidate as usize);
                            }
                        }
                    }
                    let mut bitset = BitSet::with_capacity(256);
                    bitset.clone_from(&bitset_x);
                    bitset.intersect_with(&bitset_y);
                    println!("candidates: {:?}", bitset);

                    match result_bitmap.set(repair_x, repair_y, 15) {
                        Some(()) => {},
                        None => {
                            return Err(anyhow::anyhow!("Unable to set pixel inside the result bitmap"));
                        }
                    }
                }
            }
        }


        assert_eq!(result_bitmap, output);
        Ok(())
    }
}
