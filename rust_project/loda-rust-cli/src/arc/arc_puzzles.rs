#[cfg(test)]
mod tests {
    use crate::arc::{ImageOverlay, ImageNoiseColor, ImageRemoveRowColumn, ImageRemoveGrid};
    use crate::arc::{Model, GridToImage, ImagePair, ImageFind, ImageToNumber, NumberToImage, ImageOutline, ImageRotate};
    use crate::arc::{Image, convolution2x2, convolution3x3};
    use crate::arc::{ImageResize, ImageTrim, ImageRemoveDuplicates, ImagePadding, ImageStack};
    use crate::arc::{ImageReplaceColor, ImageSymmetry, ImageOffset, ImageColorProfile};
    use crate::arc::{ImageNgram, RecordTrigram, ImageHistogram, Histogram, ImageDenoise, ImageDetectHole};
    use crate::arc::register_arc_functions;
    use crate::config::Config;
    use crate::common::find_json_files_recursively;
    use loda_rust_core::execute::ProgramId;
    use loda_rust_core::execute::{NodeLoopLimit, ProgramCache, ProgramRunner, RunMode};
    use loda_rust_core::execute::NodeRegisterLimit;
    use loda_rust_core::unofficial_function::UnofficialFunctionRegistry;
    use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
    use bit_set::BitSet;
    use num_bigint::{BigInt, BigUint, ToBigInt};
    use num_traits::Signed;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_10000_puzzle_4258a5f9() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("4258a5f9")?;
        assert_eq!(model.train().len(), 2);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train[1].input.to_image().expect("image");
        // let output: Image = model.train[1].output.to_image().expect("image");
        // let input: Image = model.test[0].input.to_image().expect("image");
        // let output: Image = model.test[0].output.to_image().expect("image");

        let input_padded: Image = input.zero_padding(1).expect("image");

        // TODO: port to LODA
        let result_bm: Image = convolution3x3(&input_padded, |bm| {
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
        }).expect("image");

        assert_eq!(result_bm, output);

        Ok(())
    }

    #[test]
    fn test_20000_puzzle_5614dbcf() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("5614dbcf")?;
        assert_eq!(model.train().len(), 2);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let input_padded: Image = input.zero_padding(1).expect("image");

        // TODO: port to LODA
        let result_bm: Image = convolution3x3(&input_padded, |bm| {
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
        }).expect("image");

        let result_bm2 = result_bm.resize(3, 3).expect("image");
        assert_eq!(result_bm2, output);
        Ok(())
    }

    #[test]
    fn test_30000_puzzle_2013d3e2() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("2013d3e2")?;
        assert_eq!(model.train().len(), 2);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let input_trimmed: Image = input.trim().expect("image");

        // TODO: port to LODA
        let mut result_bitmap = Image::zero(3, 3);
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
    fn test_40000_puzzle_90c28cc7_manual() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("90c28cc7")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let input_trimmed: Image = input.trim().expect("image");
        let result_bitmap: Image = input_trimmed.remove_duplicates().expect("image");
        assert_eq!(result_bitmap, output);
        Ok(())
    }

    const PROGRAM_90C28CC7: &'static str = "
    f11 $0,100003 ; trim
    f11 $0,100004 ; remove duplicates
    ";

    #[test]
    fn test_40001_puzzle_90c28cc7_loda() {
        let model: Model = Model::load_testdata("90c28cc7").expect("model");
        let program = PROGRAM_90C28CC7;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    fn create_dependency_manager() -> DependencyManager {
        let registry = UnofficialFunctionRegistry::new();
        register_arc_functions(&registry);
        let dm = DependencyManager::new(
            DependencyManagerFileSystemMode::Virtual,
            PathBuf::from("non-existing-dir"),
            registry,
        );
        dm
    }

    fn run_image<S: AsRef<str>>(program: S, input: &Image) -> anyhow::Result<Image> {
        let program_str: &str = program.as_ref();
        let mut dm = create_dependency_manager();
        let program_runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, program_str).expect("ProgramRunner");
        let mut cache = ProgramCache::new();
        run_image_inner(&program_runner, input, &mut cache)
    }
    
    fn run_image_inner(program_runner: &ProgramRunner, input: &Image, mut cache: &mut ProgramCache) -> anyhow::Result<Image> {
        let output_count: u8 = 1;
        let step_count_limit: u64 = 1000000000;
        let mut step_count: u64 = 0;
        
        // Input
        let input_number_uint: BigUint = input.to_number().expect("input image to number");
        let input_number_int: BigInt = input_number_uint.to_bigint().expect("input BigUint to BigInt");
        let input_bigint: Vec<BigInt> = vec![input_number_int];
        
        // Run
        let output_vec: Vec<BigInt> = program_runner.run_vec(
            input_bigint, 
            RunMode::Silent, 
            &mut step_count, 
            step_count_limit,
            NodeRegisterLimit::Unlimited,
            NodeLoopLimit::Unlimited,
            &mut cache,
            output_count,
        )?;

        // Output
        if output_vec.len() != 1 {
            return Err(anyhow::anyhow!("output_vec. Expected 1 output value, but got {:?}", output_vec.len()));
        }
        let output0_int: &BigInt = &output_vec[0];
        if output0_int.is_negative() {
            return Err(anyhow::anyhow!("output0. Expected non-negative number, but got {:?}", output0_int));
        }
        let output0_uint: BigUint = output0_int.to_biguint().expect("output biguint");
        let output0_image: Image = output0_uint.to_image().expect("output uint to image");
        Ok(output0_image)
    }

    #[test]
    fn test_50000_puzzle_7468f01a_manual() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("7468f01a")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let input_trimmed: Image = input.trim().expect("image");
        let result_bitmap: Image = input_trimmed.flip_x().expect("image");
        assert_eq!(result_bitmap, output);
        Ok(())
    }

    const PROGRAM_7468F01A: &'static str = "
    f11 $0,100003 ; trim
    f11 $0,100010 ; flip x
    ";

    #[test]
    fn test_50001_puzzle_7468f01a_loda() {
        let model: Model = Model::load_testdata("7468f01a").expect("model");
        let program = PROGRAM_7468F01A;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_60000_puzzle_63613498() -> anyhow::Result<()> {
        // TODO: port to LODA
        let model: Model = Model::load_testdata("63613498")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        // Extract needle
        let mut needle: Image = Image::zero(3, 3);
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
        let mut search_area: Image = input.clone();
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
            let needle_with_color: Image = needle.replace_color(1, color)?;
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
        let mut result_bitmap: Image = input.clone();
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
        // TODO: port to LODA
        let model: Model = Model::load_testdata("cdecee7f")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let background_pixel_color: u8 = input.most_popular_color().expect("pixel");

        // Traverse columns
        let mut stack: Vec<u8> = vec!();
        for x in 0..input.width() {
            // Take foreground pixels that is different than the background color, and append the foreground pixel to the stack
            for y in 0..input.height() {
                let pixel_value: u8 = input.get(x as i32, y as i32).unwrap_or(255);
                if pixel_value != background_pixel_color {
                    stack.push(pixel_value);
                }
            }
        }
        // Padding to 9 items
        while stack.len() < 9 {
            stack.push(0);
        }

        // Transfer values from the 9 element stack to the 3x3 bitmap
        let mut result_bitmap: Image = Image::zero(3, 3);
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

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.train()[3].input().to_image().expect("image");
        // let output: Image = model.train()[3].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        // TODO: port to LODA
        let mut result_bitmap: Image = Image::zero(9, 9);
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
    fn test_90000_puzzle_b9b7f026_manual() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("b9b7f026")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let background_color: u8 = input.most_popular_color().expect("color");

        // Detect corners / holes
        let corner_image: Image = input.detect_hole_type1(background_color).expect("image");
        // println!("input: {:?}", input);
        // println!("corner_image: {:?}", corner_image);

        // Extract color of the corner
        let corner_color: u8 = corner_image.least_popular_color().expect("color");
        let result_bitmap: Image = Image::color(1, 1, corner_color);
        assert_eq!(result_bitmap, output);
        Ok(())
    }

    const PROGRAM_B9B7F026: &'static str = "
    mov $1,$0
    f11 $1,100061 ; most popular color
    ; $1 is background color
    f21 $0,100110 ; detect holes

    mov $2,$0
    f11 $2,100071 ; least popular color
    ; $2 is the corner color

    mov $0,1 ; width=1
    mov $1,1 ; height=1
    f31 $0,100006 ; create image with color
    ";

    #[test]
    fn test_90001_puzzle_b9b7f026_loda() {
        let model: Model = Model::load_testdata("b9b7f026").expect("model");
        let program = PROGRAM_B9B7F026;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_100000_puzzle_a79310a0_manual() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("a79310a0")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let bitmap0: Image = input.offset_wrap(0, 1).expect("image");
        let bitmap1: Image = bitmap0.replace_color(8, 2).expect("image");

        assert_eq!(bitmap1, output);
        Ok(())
    }

    const PROGRAM_A79310A0: &'static str = "
    mov $1,0
    mov $2,1
    f31 $0,100001 ; offset dx,dy
    mov $1,8
    mov $2,2
    f31 $0,100050 ; replace color with color
    ";

    #[test]
    fn test_100001_puzzle_a79310a0_loda() {
        let model: Model = Model::load_testdata("a79310a0").expect("model");
        let program = PROGRAM_A79310A0;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_100002_puzzle_a79310a0_manual_without_hardcoded_colors() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("a79310a0").expect("model");
        
        // These images contain 2 colors. Build a mapping from source color to target color
        let train_pairs: Vec<ImagePair> = model.images_train().expect("pairs");
        let mut color_replacements = HashMap::<u8, u8>::new();
        for (index, pair) in train_pairs.iter().enumerate() {
            let input_histogram = pair.input.histogram_all();
            if input_histogram.number_of_counters_greater_than_zero() != 2 {
                return Err(anyhow::anyhow!("input[{}] Expected exactly 2 colors", index));
            }
            let output_histogram = pair.output.histogram_all();
            if output_histogram.number_of_counters_greater_than_zero() != 2 {
                return Err(anyhow::anyhow!("output[{}] Expected exactly 2 colors", index));
            }
            let in_color0: u8 = input_histogram.most_popular().expect("u8");
            let out_color0: u8 = output_histogram.most_popular().expect("u8");
            color_replacements.insert(in_color0, out_color0);

            let in_color1: u8 = input_histogram.least_popular().expect("u8");
            let out_color1: u8 = output_histogram.least_popular().expect("u8");
            color_replacements.insert(in_color1, out_color1);
        }

        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {

            let mut image: Image = pair.input.offset_wrap(0, 1).expect("image");
            for (key, value) in &color_replacements {
                image = image.replace_color(*key, *value).expect("image");
            }

            assert_eq!(image, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
        Ok(())
    }

    /// Detect corners and edges
    fn mask_and_repair_areas(input: &Image) -> anyhow::Result<(Image, Image)> {
        // Assign 0 to background, assign 1 to foreground
        let mut mask: Image = input.clone();
        for pixel_value in 2..=255 {
            mask = mask.replace_color(pixel_value, 1).expect("image");
        }
        // println!("mask: {:?}", mask);

        // Detect corners and edges
        let repair_areas: Image = convolution2x2(&mask, |bm| {
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
        }).expect("image");
        Ok((mask, repair_areas))
    }

    fn repair_corner_top_left(bitmap: &mut Image, x: i32, y: i32, trigram_x: &Vec<RecordTrigram>, trigram_y: &Vec<RecordTrigram>) -> anyhow::Result<()> {
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
        // TODO: port to LODA
        let model: Model = Model::load_testdata("0dfd9992")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");

        // TODO: make the rest of the tests pass OK. Currently these fails.
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

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
        
        let mut result_bitmap: Image = input.clone();

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

    #[test]
    fn test_120000_puzzle_3bdb4ada() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("3bdb4ada")?;
        assert_eq!(model.train().len(), 2);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        // TODO: port to LODA
        let mut image = input.clone();
        for yy in 0..((image.height() as i32) - 2) {
            for xx in 0..((image.width() as i32) - 2) {
                let top_left_pixel_value: u8 = image.get(xx, yy).unwrap_or(255);
                let mut same_count = 1;
                for y in 0..3 {
                    for x in 0..3 {
                        if x == 1 && y == 1 {
                            continue;
                        }
                        if x == 0 && y == 0 {
                            continue;
                        }
                        let pixel_value: u8 = image.get(xx + x, yy + y).unwrap_or(255); 
                        if pixel_value == top_left_pixel_value {
                            same_count += 1;
                        }
                    }
                }
                if same_count == 8 {
                    match image.set(xx + 1, yy + 1, 0) {
                        Some(()) => {},
                        None => {
                            return Err(anyhow::anyhow!("Unable to set pixel inside the result bitmap"));
                        }
                    }
                }
            }
        }
        assert_eq!(image, output);
        Ok(())
    }

    #[test]
    fn test_130000_puzzle_7fe24cdd() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("7fe24cdd")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        // println!("input: {:?}", input);

        let row0: Image = Image::hstack(vec![input.clone(), input.rotate(1).expect("image")])?;
        let row1: Image = Image::hstack(vec![input.rotate(3).expect("image"), input.rotate(2).expect("image")])?;
        // println!("row0: {:?}", row0);
        // println!("row1: {:?}", row1);

        let image = Image::vstack(vec![row0.clone(), row1.clone()])?;
        // println!("image: {:?}", image);

        assert_eq!(image, output);
        Ok(())
    }

    const PROGRAM_7FE24CDD: &'static str = "
    mov $5,$0 ; original corner

    ; construct top half
    mov $1,$0
    mov $2,1
    f21 $1,100002 ; rotate cw
    f21 $0,100032 ; hstack
    ; $0 is top half

    ; construct bottom half
    mov $6,2
    f21 $5,100002 ; rotate cw cw
    mov $1,$5
    mov $2,1
    f21 $1,100002 ; rotate cw
    mov $2,$5
    f21 $1,100032 ; hstack
    ; $1 is bottom half

    ; join top half and bottom half
    f21 $0,100042 ; vstack
    ";

    #[test]
    fn test_130001_puzzle_7fe24cdd_loda() {
        let model: Model = Model::load_testdata("7fe24cdd").expect("model");
        let program = PROGRAM_7FE24CDD;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_140000_puzzle_9565186b() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("9565186b")?;
        assert_eq!(model.train().len(), 4);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.train()[3].input().to_image().expect("image");
        // let output: Image = model.train()[3].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let pixel_color: u8 = input.most_popular_color().expect("color");
        let image: Image = input.replace_colors_other_than(pixel_color, 5).expect("image");
        assert_eq!(image, output);
        Ok(())
    }

    const PROGRAM_9565186B: &'static str = "
    mov $1,$0
    f11 $1,100061 ; most popular color
    mov $2,5
    f31 $0,100051 ; replace colors other than color
    ";

    #[test]
    fn test_140001_puzzle_9565186b_loda() {
        let model: Model = Model::load_testdata("9565186b").expect("model");
        let program = PROGRAM_9565186B;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 5);
    }

    #[test]
    fn test_150000_puzzle_3af2c5a8() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("3af2c5a8")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        // let input: Image = model.train()[0].input().to_image().expect("image");
        // let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        let input: Image = model.test()[0].input().to_image().expect("image");
        let output: Image = model.test()[0].output().to_image().expect("image");

        let row0: Image = Image::hstack(vec![input.clone(), input.flip_x().expect("image")])?;
        let row1: Image = row0.flip_y().expect("image");
        let image = Image::vstack(vec![row0.clone(), row1.clone()])?;
        assert_eq!(image, output);
        Ok(())
    }

    const PROGRAM_3AF2C5A8: &'static str = "
    mov $1,$0
    f11 $1,100010 ; flip x
    f21 $0,100032 ; hstack
    mov $1,$0
    f11 $1,100011 ; flip y
    f21 $0,100042 ; vstack
    ";

    #[test]
    fn test_150001_puzzle_3af2c5a8_loda() {
        let model: Model = Model::load_testdata("3af2c5a8").expect("model");
        let program = PROGRAM_3AF2C5A8;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    const PROGRAM_44F52BB0: &'static str = "
    mov $1,$0
    f11 $1,100010 ; flip x
    cmp $0,$1
    mov $2,1 ; color when there is symmetry
    mul $2,$0
    cmp $0,0
    mul $0,7 ; color when there is no symmetry
    add $2,$0
    mov $0,1 ; output image width
    mov $1,1 ; output image height
    f31 $0,100006 ; create image
    ";

    #[test]
    fn test_160000_puzzle_44f52bb0_loda() {
        let model: Model = Model::load_testdata("44f52bb0").expect("model");
        let program = PROGRAM_44F52BB0;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 8);
    }

    #[test]
    fn test_170000_puzzle_496994bd() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("496994bd")?;
        assert_eq!(model.train().len(), 2);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let background_pixel_color: u8 = input.most_popular_color().expect("color");
        let flipped_image: Image = input.flip_y().expect("image");
        let result_image: Image = input.overlay_with_mask_color(
            &flipped_image, 
            background_pixel_color
        ).expect("image");

        assert_eq!(result_image, output);
        Ok(())
    }

    const PROGRAM_496994BD: &'static str = "
    mov $1,$0
    mov $2,$0
    f11 $2,100061 ; most popular color
    f11 $1,100011 ; flip y
    f31 $0,100005 ; overlay
    ";

    #[test]
    fn test_170001_puzzle_496994bd_loda() {
        let model: Model = Model::load_testdata("496994bd").expect("model");
        let program = PROGRAM_496994BD;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_180000_puzzle_31aa019c() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("31aa019c")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let pixel_color: u8 = input.least_popular_color().expect("color");
        let image: Image = input.replace_colors_other_than(pixel_color, 0).expect("image");
        let outline_color: u8 = 2;
        let background_color: u8 = 0;
        let result_image: Image = image.outline_type1(outline_color, background_color).expect("image");

        assert_eq!(result_image, output);
        Ok(())
    }

    const PROGRAM_31AA019C: &'static str = "
    mov $1,$0
    f11 $1,100071 ; most unpopular color
    mov $2,0 ; background color
    f31 $0,100051 ; replace colors other than
    mov $1,2 ; outline color
    mov $2,0 ; background color
    f31 $0,100080 ; draw outline
    ";

    #[test]
    fn test_180001_puzzle_31aa019c_loda() {
        let model: Model = Model::load_testdata("31aa019c").expect("model");
        let program = PROGRAM_31AA019C;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_190000_puzzle_5ad4f10b() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("5ad4f10b")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        let input: Image = model.train()[0].input().to_image().expect("image");
        let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        // let input: Image = model.test()[0].input().to_image().expect("image");
        // let output: Image = model.test()[0].output().to_image().expect("image");

        let background_color: u8 = input.most_popular_color().expect("color");

        let denoised_image: Image = input.denoise_type1(background_color).expect("image");
        // println!("denoised: {:?}", denoised_image);

        // Pick the most popular noise color
        let noise_color_vec: Vec<u8> = input.noise_color_vec(&denoised_image).expect("vec with colors");
        let noise_color: u8 = *noise_color_vec.first().expect("1 or more colors");
        // println!("noise color: {}", noise_color);

        // Remove background around the object
        let trimmed_image: Image = denoised_image.trim().expect("image");

        // Remove duplicate rows/columns
        let image_without_duplicates: Image = trimmed_image.remove_duplicates().expect("image");

        // Change color of the object
        let result_image: Image = image_without_duplicates.replace_colors_other_than(background_color, noise_color).expect("image");

        assert_eq!(result_image, output);
        Ok(())
    }

    const PROGRAM_5AD4F10B: &'static str = "
    mov $1,$0
    mov $2,$0
    mov $3,$0
    mov $9,$0

    f11 $3,100061 ; most popular color
    ; $3 is background_color

    mov $5,$0 ; noisy image
    mov $6,$3 ; background_color
    f21 $5,100090 ; denoise image
    ; $5 is denoised image

    ; $9 is noisy image
    mov $10,$5 ; denoised image
    f21 $9,100101 ; extract 1 noise color
    ; $9 is the most popular noise color

    mov $12,$5 ; denoised image
    f11 $12,100003 ; trim
    f11 $12,100004 ; remove duplicates

    mov $0,$12
    mov $1,$3 ; background color
    mov $2,$9 ; noise color
    f31 $0,100051 ; replace colors other than
    ";

    #[test]
    fn test_190001_puzzle_5ad4f10b_loda() {
        let model: Model = Model::load_testdata("5ad4f10b").expect("model");
        let program = PROGRAM_5AD4F10B;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_200000_puzzle_1190e5a7() -> anyhow::Result<()> {
        let model: Model = Model::load_testdata("1190e5a7")?;
        assert_eq!(model.train().len(), 3);
        assert_eq!(model.test().len(), 1);

        // let input: Image = model.train()[0].input().to_image().expect("image");
        // let output: Image = model.train()[0].output().to_image().expect("image");
        // let input: Image = model.train()[1].input().to_image().expect("image");
        // let output: Image = model.train()[1].output().to_image().expect("image");
        // let input: Image = model.train()[2].input().to_image().expect("image");
        // let output: Image = model.train()[2].output().to_image().expect("image");
        let input: Image = model.test()[0].input().to_image().expect("image");
        let output: Image = model.test()[0].output().to_image().expect("image");

        let without_duplicates: Image = input.remove_duplicates().expect("image");
        let result_image: Image = without_duplicates.remove_grid().expect("image");
        assert_eq!(result_image, output);
        Ok(())
    }

    const PROGRAM_1190E5A7: &'static str = "
    f11 $0,100004 ; remove duplicates
    f11 $0,100120 ; remove grid
    ";

    #[test]
    fn test_200001_puzzle_1190e5a7_loda() {
        let model: Model = Model::load_testdata("1190e5a7").expect("model");
        let program = PROGRAM_1190E5A7;
        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {
            let output: Image = run_image(program, &pair.input).expect("image");
            assert_eq!(output, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    // #[test]
    fn test_210000_traverse_testdata() {
        let config = Config::load();
        let path: PathBuf = config.arc_repository_data_training();
        let paths: Vec<PathBuf> = find_json_files_recursively(&path);
        println!("number of json files: {}", paths.len());
        
        let mut items = Vec::<Item>::new();
        for path in &paths {
            let model = Model::load_with_json_file(path).expect("model");
            let item = Item {
                id: ItemId::Path { path: path.clone() },
                model,
            };
            items.push(item);
        }
        println!("number of items: {}", items.len());

        const _PROGRAM1: &'static str = "
        mov $1,1
        f21 $0,100002 ; rotate
        ";

        const PROGRAM2: &'static str = "
        mov $1,2
        f21 $0,100002 ; rotate
        ";

        const PROGRAM3: &'static str = "
        mov $1,-1
        f21 $0,100002 ; rotate
        ";

        const PROGRAM4: &'static str = "
        f11 $0,100003 ; trim
        ";

        const PROGRAM5: &'static str = "
        f11 $0,100010 ; flip x
        ";

        const PROGRAM6: &'static str = "
        f11 $0,100011 ; flip y
        ";

        const PROGRAM7: &'static str = "
        f11 $0,100012 ; flip xy
        ";

        const PROGRAM8: &'static str = "
        f11 $0,100004 ; remove duplicates
        ";

        const _PROGRAM10: &'static str = "
        mov $1,0
        f21 $0,100013 ; pad by 1 pixel evenly
        ";

        const _PROGRAM11: &'static str = "
        mov $1,0
        f21 $0,100014 ; pad by 1 pixel top/bottom
        ";

        const _PROGRAM12: &'static str = "
        mov $1,0
        f21 $0,100015 ; pad by 1 pixel left/right
        ";

        const _PROGRAM13: &'static str = "
        f11 $0,100003 ; trim
        mov $1,-1
        f21 $0,100002 ; rotate
        ";

        const _PROGRAM14: &'static str = "
        f11 $0,100003 ; trim
        mov $1,1
        f21 $0,100002 ; rotate
        ";

        const _PROGRAM15: &'static str = "
        f11 $0,100003 ; trim
        mov $1,2
        f21 $0,100002 ; rotate
        ";
        
        const PROGRAM16: &'static str = "
        f11 $0,100020 ; resize x*2 y*2
        ";
        
        const PROGRAM17: &'static str = "
        f11 $0,100021 ; resize x*3 y*3
        ";
        
        const _PROGRAM18: &'static str = "
        f11 $0,100022 ; resize x/2 y/2
        ";
        
        const PROGRAMS: &'static [&'static str] = &[
            PROGRAM_1190E5A7,
            PROGRAM_31AA019C,
            PROGRAM_3AF2C5A8,
            PROGRAM_44F52BB0,
            PROGRAM_496994BD,
            PROGRAM_5AD4F10B,
            PROGRAM_7468F01A,
            PROGRAM_7FE24CDD,
            PROGRAM_90C28CC7,
            PROGRAM_9565186B,
            PROGRAM_A79310A0,
            PROGRAM_B9B7F026,
            // PROGRAM1, 
            PROGRAM2, 
            PROGRAM3,
            PROGRAM4,
            PROGRAM5,
            PROGRAM6,
            PROGRAM7,
            PROGRAM8,
            // PROGRAM10,
            // PROGRAM11,
            // PROGRAM12,
            // PROGRAM13,
            // PROGRAM14,
            // PROGRAM15,
            PROGRAM16,
            PROGRAM17,
            // PROGRAM18,
        ];

        let mut dm = create_dependency_manager();
        let mut program_runners = Vec::<ProgramRunner>::new();
        for program_string in PROGRAMS {
            let program_runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, program_string).expect("ProgramRunner");
            program_runners.push(program_runner);
        }

        let mut cache = ProgramCache::new();
        let mut count_match: usize = 0;
        let mut count_mismatch: usize = 0;
        let mut found_program_indexes: Vec<usize> = vec!();
        for item in &items {
            let pairs: Vec<ImagePair> = item.model.images_all().expect("pairs");
    
            let mut found_one_or_more_solutions = false;
            for (program_index, program_runner) in program_runners.iter().enumerate() {

                let mut count = 0;
                for pair in &pairs {
                    let output: Image = match run_image_inner(&program_runner, &pair.input, &mut cache) {
                        Ok(value) => value,
                        Err(_error) => {
                            break;
                        }
                    };

                    if output == pair.output {
                        count += 1;
                    }
                }
    
                if count == pairs.len() {
                    found_one_or_more_solutions = true;
                    found_program_indexes.push(program_index);
                    println!("program {} is a solution for {:?}", program_index, item.id);
                }
            }

            if found_one_or_more_solutions {
                count_match += 1;
            } else {
                count_mismatch += 1;
            }
        }
        found_program_indexes.sort();

        println!("number of matches: {} mismatches: {}", count_match, count_mismatch);
        println!("found_program_indexes: {:?}", found_program_indexes);

    }

    #[derive(Clone, Debug)]
    enum ItemId {
        None,
        Path { path: PathBuf },
    }
    
    #[derive(Clone, Debug)]
    struct Item {
        id: ItemId,
        model: Model,
    }
    
}
