#[cfg(test)]
mod tests {
    use crate::arc::{Model, GridToBitmap, BitmapFind};
    use crate::arc::{Bitmap, convolution3x3};
    use crate::arc::{BitmapResize, BitmapTrim, BitmapRemoveDuplicates, Padding};
    use crate::arc::{BitmapReplaceColor, BitmapSymmetry};

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
}
