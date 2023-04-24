#[cfg(test)]
mod tests {
    use crate::arc::arc_json_model::{Task, ImagePair};
    use crate::arc::arc_work_model::{self, PairType};
    use crate::arc::{ActionLabel};
    use crate::arc::{RunWithProgram, RunWithProgramResult, SolutionSimple, SolutionSimpleData, AnalyzeAndSolve, ImageRepeat, ImagePeriodicity};
    use crate::arc::{ImageOverlay, ImageNoiseColor, ImageGrid, ImageExtractRowColumn, ImageSegment, ImageSegmentAlgorithm, ImageSegmentItem, ImageMask, Histogram};
    use crate::arc::{ImageFind, ImageOutline, ImageRotate, ImageBorder, ImageCompare, ImageCrop, ImageResize};
    use crate::arc::{Image, PopularObjects, ImageNeighbour, ImageNeighbourDirection, ImageRepairPattern};
    use crate::arc::{ImageTrim, ImageRemoveDuplicates, ImageStack, ImageMaskCount, ImageSetPixelWhere, GridPattern};
    use crate::arc::{ImageReplaceColor, ImageSymmetry, ImageOffset, ImageColorProfile, ImageCreatePalette, ImageDrawLineWhere};
    use crate::arc::{ImageHistogram, ImageDenoise, ImageDetectHole, ImageTile, ImagePadding, Rectangle, ImageObjectEnumerate};
    use crate::arc::{ImageReplaceRegex, ImageReplaceRegexToColor, ImagePosition, ImageMaskBoolean, ImageCountUniqueColors};
    use std::collections::HashMap;
    use regex::Regex;

    #[allow(unused_imports)]
    use crate::arc::{HtmlLog, ImageToHTML};

    trait RunWithTestdata {
        fn run(self, task_name: &str) -> anyhow::Result<String>;
    }

    impl RunWithTestdata for SolutionSimple {
        fn run(self, task_name: &str) -> anyhow::Result<String> {
            let json_task: Task = Task::load_testdata(task_name)?;
            let task = arc_work_model::Task::try_from(&json_task)?;
            let instance: RunWithProgram = RunWithProgram::new(task, true);
            let result: RunWithProgramResult = instance.run_solution(self)?;
            let mut string: String = format!("{} {}", result.count_train_correct(), result.count_test_correct());
            let messages: String = result.messages();
            if !messages.is_empty() {
                string = format!("{} - {}", string, messages);
            }
            Ok(string)
        }
    }

    #[allow(dead_code)]
    pub fn run_analyze_and_solve(
        task_name: &str,
        analyze_and_solve: &mut dyn AnalyzeAndSolve,
    ) -> anyhow::Result<String> {
        let json_task: Task = Task::load_testdata(task_name)?;
        let task = arc_work_model::Task::try_from(&json_task)?;
        let instance: RunWithProgram = RunWithProgram::new(task, true);
        let result: RunWithProgramResult = instance.run_analyze_and_solve(analyze_and_solve)?;
        let mut string: String = format!("{} {}", result.count_train_correct(), result.count_test_correct());
        let messages: String = result.messages();
        if !messages.is_empty() {
            string = format!("{} - {}", string, messages);
        }
        Ok(string)
    }

    fn run_simple(task_name: &str, program: &str) -> anyhow::Result<String> {
        let json_task: Task = Task::load_testdata(task_name)?;
        let task = arc_work_model::Task::try_from(&json_task)?;
        let instance: RunWithProgram = RunWithProgram::new(task, true);
        let result: RunWithProgramResult = instance.run_simple(program)?;
        let mut string: String = format!("{} {}", result.count_train_correct(), result.count_test_correct());
        let messages: String = result.messages();
        if !messages.is_empty() {
            string = format!("{} - {}", string, messages);
        }
        Ok(string)
    }

    fn run_advanced(task_name: &str, program: &str) -> anyhow::Result<String> {
        let json_task: Task = Task::load_testdata(task_name)?;
        let task = arc_work_model::Task::try_from(&json_task)?;
        let instance: RunWithProgram = RunWithProgram::new(task, true);
        let result: RunWithProgramResult = instance.run_advanced(program)?;
        let mut string: String = format!("{} {}", result.count_train_correct(), result.count_test_correct());
        let messages: String = result.messages();
        if !messages.is_empty() {
            string = format!("{} - {}", string, messages);
        }
        Ok(string)
    }

    #[test]
    fn test_10000_puzzle_4258a5f9() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_pixel_color: u8 = input.most_popular_color().expect("pixel");
            let result_image: Image = input.outline_type1(1, background_pixel_color).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("4258a5f9").expect("String");
        assert_eq!(result, "2 1");
    }

    const ADVANCED_PROGRAM_4258A5F9: &'static str = r#"
    mov $40,0 ; outline color

    ; process "train" vector
    mov $80,$97 ; set iteration counter = length of "train" vector
    mov $81,100 ; address of first training data train[0].input
    mov $82,101 ; address of first training data train[0].output
    lps $80
        mov $0,$$81 ; load train[x].input image
        mov $1,$$82 ; load train[x].output image

        ; analyze the output images
        f12 $1,101070 ; least popular colors
        mov $40,$2 ; get the outline color

        ; next iteration
        add $81,100 ; jump to address of next training input image
        add $82,100 ; jump to address of next training output image
    lpe
    
    ; process "train"+"test" vectors
    mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
    mov $81,100 ; address of vector[0].input
    mov $82,102 ; address of vector[0].computed_output
    lps $80
        mov $0,$$81 ; load vector[x].input image

        mov $5,$0
        f11 $5,101060 ; most popular color

        mov $1,$40 ; outline color
        mov $2,$5 ; background color
        f31 $0,101080 ; draw outline

        mov $$82,$0 ; save vector[x].computed_output image

        ; next iteration
        add $81,100 ; jump to address of next input image
        add $82,100 ; jump to address of next computed_output image
    lpe
    "#;

    #[test]
    fn test_10001_puzzle_4258a5f9_loda() {
        let result: String = run_advanced("4258a5f9", ADVANCED_PROGRAM_4258A5F9).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_20000_puzzle_5614dbcf() {
        let solution: SolutionSimple = |data| {
            let image_denoised: Image = data.image.denoise_type3(3).expect("image");
            let result_image: Image = image_denoised.remove_duplicates().expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("5614dbcf").expect("String");
        assert_eq!(result, "2 1");
    }

    const PROGRAM_5614DBCF: &'static str = "
    mov $1,3 ; number of noise colors to remove
    f21 $0,101092 ; denoise type 3
    f11 $0,101140 ; remove duplicates
    ";

    #[test]
    fn test_20001_puzzle_5614dbcf_loda() {
        let result: String = run_simple("5614dbcf", PROGRAM_5614DBCF).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_30000_puzzle_2013d3e2() {
        let solution: SolutionSimple = |data| {
            let input_trimmed: Image = data.image.trim().expect("image");

            // Extract top/left corner
            let top_rows: Image = input_trimmed.top_rows(input_trimmed.height() / 2).expect("image");
            let result_image: Image = top_rows.left_columns(input_trimmed.height() / 2).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("2013d3e2").expect("String");
        assert_eq!(result, "2 1");
    }

    const PROGRAM_2013D3E2: &'static str = "
    f11 $0,101160 ; trim

    mov $4,$0
    mov $5,$0

    f11 $4,101000 ; get width
    f11 $5,101001 ; get height

    div $4,2
    div $5,2

    mov $1,$4
    f21 $0,101220 ; get top rows
    
    mov $1,$5
    f21 $0,101222 ; get left columns
    ";

    #[test]
    fn test_30001_puzzle_2013d3e2_loda() {
        let result: String = run_simple("2013d3e2", PROGRAM_2013D3E2).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_40000_puzzle_90c28cc7_manual() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let input_trimmed: Image = input.trim().expect("image");
            let result_image: Image = input_trimmed.remove_duplicates().expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("90c28cc7").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_90C28CC7: &'static str = "
    f11 $0,101160 ; trim
    f11 $0,101140 ; remove duplicates
    ";

    #[test]
    fn test_40001_puzzle_90c28cc7_loda() {
        let result: String = run_simple("90c28cc7", PROGRAM_90C28CC7).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_50000_puzzle_7468f01a_manual() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let input_trimmed: Image = input.trim().expect("image");
            let result_image: Image = input_trimmed.flip_x().expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("7468f01a").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_7468F01A: &'static str = "
    f11 $0,101160 ; trim
    f11 $0,101190 ; flip x
    ";

    #[test]
    fn test_50001_puzzle_7468f01a_loda() {
        let result: String = run_simple("7468f01a", PROGRAM_7468F01A).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_60000_puzzle_63613498() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
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
            Ok(result_bitmap)
        };
        let result: String = solution.run("63613498").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_70000_puzzle_cdecee7f() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
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

            Ok(result_bitmap)
        };
        let result: String = solution.run("cdecee7f").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_80000_puzzle_007bbfb7() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let mut image: Image = Image::zero(9, 9);
            for y in 0..input.height() {
                for x in 0..input.width() {
                    let mask_value: u8 = input.get(x as i32, y as i32).unwrap_or(255);
                    if mask_value == 0 {
                        continue;
                    }
                    image = image.overlay_with_position(&input, (x * 3) as i32, (y * 3) as i32)?;
                }
            }
            Ok(image)
        };
        let result: String = solution.run("007bbfb7").expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_007BBFB7: &'static str = "
    ; tile_width
    mov $2,$0
    f11 $2,101000 ; Get width of image

    ; tile_height
    mov $3,$0
    f11 $3,101001 ; Get height of image

    ; tile
    mov $7,0 ; color
    mov $6,$3 ; height
    mov $5,$2 ; width
    f31 $5,101010 ; Create new image with size (x, y) and filled with color z

    ; mask
    mov $10,$0 ; image
    mov $11,$1 ; color
    f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

    mov $11,$5 ; tile0
    mov $12,$0 ; tile1
    f31 $10,102110 ; Create a big composition of tiles.

    mov $0,$10
    ";

    #[test]
    fn test_80001_puzzle_007bbfb7_loda() {
        let result: String = run_simple("007bbfb7", PROGRAM_007BBFB7).expect("String");
        assert_eq!(result, "5 1");
    }

    #[test]
    fn test_90000_puzzle_b9b7f026_manual() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.most_popular_color().expect("color");

            // Detect corners / holes
            let corner_image: Image = input.detect_hole_type1(background_color).expect("image");
            // println!("input: {:?}", input);
            // println!("corner_image: {:?}", corner_image);
    
            // Extract color of the corner
            let corner_color: u8 = corner_image.least_popular_color().expect("color");
            let result_image: Image = Image::color(1, 1, corner_color);
            Ok(result_image)
        };
        let result: String = solution.run("b9b7f026").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_B9B7F026: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color
    ; $1 is background color
    f21 $0,101110 ; detect holes

    mov $2,$0
    f11 $2,101070 ; least popular color
    ; $2 is the corner color

    mov $0,1 ; width=1
    mov $1,1 ; height=1
    f31 $0,101010 ; create image with color
    ";

    #[test]
    fn test_90001_puzzle_b9b7f026_loda() {
        let result: String = run_simple("b9b7f026", PROGRAM_B9B7F026).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_100000_puzzle_a79310a0_manual() {
        let solution: SolutionSimple = |data| {
            let image_with_offset: Image = data.image.offset_wrap(0, 1).expect("image");
            let result_image: Image = image_with_offset.replace_color(8, 2).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("a79310a0").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_A79310A0: &'static str = "
    mov $1,0
    mov $2,1
    f31 $0,101180 ; offset dx,dy
    mov $1,8
    mov $2,2
    f31 $0,101050 ; replace color with color
    ";

    #[test]
    fn test_100001_puzzle_a79310a0_loda() {
        let result: String = run_simple("a79310a0", PROGRAM_A79310A0).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_100002_puzzle_a79310a0_manual_without_hardcoded_colors() -> anyhow::Result<()> {
        // Pseudo code for a LODA program:
        // Loop through training input/output images.
        // Extract color palette from image. Nx2 where N is the number of histogram entries. Top row is the color, bottom row the count.
        // Merge color palette images. hstack images.
        // Remove duplicates from palette images.
        // Use color palette images for replacement.

        let model: Task = Task::load_testdata("a79310a0").expect("model");

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
            let in_color0: u8 = input_histogram.most_popular_color().expect("u8");
            let out_color0: u8 = output_histogram.most_popular_color().expect("u8");
            color_replacements.insert(in_color0, out_color0);

            let in_color1: u8 = input_histogram.least_popular_color().expect("u8");
            let out_color1: u8 = output_histogram.least_popular_color().expect("u8");
            color_replacements.insert(in_color1, out_color1);
        }

        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {

            let mut image: Image = pair.input.offset_wrap(0, 1).expect("image");
            image = image.replace_colors_with_hashmap(&color_replacements).expect("image");

            assert_eq!(image, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
        Ok(())
    }

    #[test]
    fn test_100003_puzzle_a79310a0_manual_without_hashmap() {
        let model: Task = Task::load_testdata("a79310a0").expect("model");

        // These images contain 2 colors. Build a mapping from source color to target color
        let train_pairs: Vec<ImagePair> = model.images_train().expect("pairs");
        let mut palette_image = Image::empty();
        for pair in &train_pairs {
            let image: Image = pair.input.palette_using_histogram(&pair.output, false).expect("image");
            palette_image = palette_image.hjoin(image).expect("image");
        }

        let pairs: Vec<ImagePair> = model.images_all().expect("pairs");
        let mut count = 0;
        for (index, pair) in pairs.iter().enumerate() {

            let mut image: Image = pair.input.offset_wrap(0, 1).expect("image");
            image = image.replace_colors_with_palette_image(&palette_image).expect("image");

            assert_eq!(image, pair.output, "pair: {}", index);
            count += 1;
        }
        assert_eq!(count, 4);
    }

    const ADVANCED_PROGRAM_A79310A0: &'static str = r#"
    mov $40,0 ; palette image accumulated

    ; process "train" vector
    mov $80,$97 ; set iteration counter = length of "train" vector
    mov $81,100 ; address of first training data train[0].input
    mov $82,101 ; address of first training data train[0].output
    lps $80
        mov $0,$$81 ; load train[x].input image
        mov $1,$$82 ; load train[x].output image

        ; analyze the images
        f21 $0,101130 ; build palette image with color mapping from input to output
        mov $41,$0
        f21 $40,101030 ; hstack of the palette images

        ; next iteration
        add $81,100 ; jump to address of next training input image
        add $82,100 ; jump to address of next training output image
    lpe
    
    ; process "train"+"test" vectors
    mov $80,$99 ; set iteration counter = length of "train"+"test" vectors
    mov $81,100 ; address of vector[0].input
    mov $82,102 ; address of vector[0].computed_output
    lps $80
        mov $0,$$81 ; load vector[x].input image

        ; change offset of the image
        mov $1,0 ; offset x=0
        mov $2,1 ; offset y=+1
        f31 $0,101180 ; offset x, y

        ; replace colors of the image using the palette image
        mov $1,$40 ; palette image
        f21 $0,101052 ; replace colors using palette image

        mov $$82,$0 ; save vector[x].computed_output image

        ; next iteration
        add $81,100 ; jump to address of next input image
        add $82,100 ; jump to address of next computed_output image
    lpe
    "#;

    #[test]
    fn test_100004_puzzle_a79310a0_loop_over_images_in_loda() {
        let result: String = run_advanced("a79310a0", ADVANCED_PROGRAM_A79310A0).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_110000_puzzle_0dfd9992() {
        let solution: SolutionSimple = |data| {
            let input = data.image;

            let repair_color: u8 = 0;

            let result_image: Image = input.repair_pattern_with_color(repair_color).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("0dfd9992").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_0DFD9992: &'static str = "
    ; $0 is the image that is to be repaired
    mov $1,0 ; repair color
    f21 $0,102151 ; Repair damaged pixels and recreate big repeating patterns such as mosaics.
    ";

    #[test]
    fn test_110001_puzzle_0dfd9992_loda() {
        let result: String = run_simple("0dfd9992", PROGRAM_0DFD9992).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_120000_puzzle_3bdb4ada() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
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
            Ok(image)
        };
        let result: String = solution.run("3bdb4ada").expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_130000_puzzle_7fe24cdd() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let row0: Image = Image::hstack(vec![input.clone(), input.rotate(1).expect("image")]).expect("image");
            let row1: Image = Image::hstack(vec![input.rotate(3).expect("image"), input.rotate(2).expect("image")]).expect("image");
            let result_image = Image::vstack(vec![row0.clone(), row1.clone()]).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("7fe24cdd").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_7FE24CDD: &'static str = "
    mov $5,$0 ; original corner

    ; construct top half
    mov $1,$0
    mov $2,1
    f21 $1,101170 ; rotate cw
    f21 $0,101030 ; hstack
    ; $0 is top half

    ; construct bottom half
    mov $6,2
    f21 $5,101170 ; rotate cw cw
    mov $1,$5
    mov $2,1
    f21 $1,101170 ; rotate cw
    mov $2,$5
    f21 $1,101030 ; hstack
    ; $1 is bottom half

    ; join top half and bottom half
    f21 $0,101040 ; vstack
    ";

    #[test]
    fn test_130001_puzzle_7fe24cdd_loda() {
        let result: String = run_simple("7fe24cdd", PROGRAM_7FE24CDD).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_140000_puzzle_9565186b() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let pixel_color: u8 = input.most_popular_color().expect("color");
            let result_image: Image = input.replace_colors_other_than(pixel_color, 5).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("9565186b").expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_9565186B: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color
    mov $2,5
    f31 $0,101051 ; replace colors other than color
    ";

    #[test]
    fn test_140001_puzzle_9565186b_loda() {
        let result: String = run_simple("9565186b", PROGRAM_9565186B).expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_150000_puzzle_3af2c5a8() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let row0: Image = Image::hstack(vec![input.clone(), input.flip_x().expect("image")]).expect("image");
            let row1: Image = row0.flip_y().expect("image");
            let result_image = Image::vstack(vec![row0.clone(), row1.clone()]).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("3af2c5a8").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_3AF2C5A8: &'static str = "
    mov $1,$0
    f11 $1,101190 ; flip x
    f21 $0,101030 ; hstack
    mov $1,$0
    f11 $1,101191 ; flip y
    f21 $0,101040 ; vstack
    ";

    #[test]
    fn test_150001_puzzle_3af2c5a8_loda() {
        let result: String = run_simple("3af2c5a8", PROGRAM_3AF2C5A8).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_44F52BB0: &'static str = "
    mov $1,$0
    f11 $1,101190 ; flip x
    cmp $0,$1
    mov $2,1 ; color when there is symmetry
    mul $2,$0
    cmp $0,0
    mul $0,7 ; color when there is no symmetry
    add $2,$0
    mov $0,1 ; output image width
    mov $1,1 ; output image height
    f31 $0,101010 ; create image
    ";

    #[test]
    fn test_160000_puzzle_44f52bb0_loda() {
        let result: String = run_simple("44f52bb0", PROGRAM_44F52BB0).expect("String");
        assert_eq!(result, "6 2");
    }

    #[test]
    fn test_170000_puzzle_496994bd() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_pixel_color: u8 = input.most_popular_color().expect("color");
            let flipped_image: Image = input.flip_y().expect("image");
            let result_image: Image = input.overlay_with_mask_color(
                &flipped_image, 
                background_pixel_color
            ).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("496994bd").expect("String");
        assert_eq!(result, "2 1");
    }

    const PROGRAM_496994BD: &'static str = "
    mov $1,$0
    mov $2,$0
    f11 $2,101060 ; most popular color
    f11 $1,101191 ; flip y
    f31 $0,101150 ; overlay
    ";

    #[test]
    fn test_170001_puzzle_496994bd_loda() {
        let result: String = run_simple("496994bd", PROGRAM_496994BD).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_180000_puzzle_31aa019c() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let pixel_color: u8 = input.least_popular_color().expect("color");
            let image: Image = input.replace_colors_other_than(pixel_color, 0).expect("image");
            let outline_color: u8 = 2;
            let background_color: u8 = 0;
            let result_image: Image = image.outline_type1(outline_color, background_color).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("31aa019c").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_31AA019C: &'static str = "
    mov $1,$0
    f11 $1,101070 ; most unpopular color
    mov $2,0 ; background color
    f31 $0,101051 ; replace colors other than
    mov $1,2 ; outline color
    mov $2,0 ; background color
    f31 $0,101080 ; draw outline
    ";

    #[test]
    fn test_180001_puzzle_31aa019c_loda() {
        let result: String = run_simple("31aa019c", PROGRAM_31AA019C).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_190000_puzzle_5ad4f10b() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
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
            Ok(result_image)
        };
        let result: String = solution.run("5ad4f10b").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_5AD4F10B: &'static str = "
    mov $1,$0
    mov $2,$0
    mov $3,$0
    mov $9,$0

    f11 $3,101060 ; most popular color
    ; $3 is background_color

    mov $5,$0 ; noisy image
    mov $6,$3 ; background_color
    f21 $5,101090 ; denoise image
    ; $5 is denoised image

    ; $9 is noisy image
    mov $10,$5 ; denoised image
    f21 $9,101100 ; extract 1 noise color
    ; $9 is the most popular noise color

    mov $12,$5 ; denoised image
    f11 $12,101160 ; trim
    f11 $12,101140 ; remove duplicates

    mov $0,$12
    mov $1,$3 ; background color
    mov $2,$9 ; noise color
    f31 $0,101051 ; replace colors other than
    ";

    #[test]
    fn test_190001_puzzle_5ad4f10b_loda() {
        let result: String = run_simple("5ad4f10b", PROGRAM_5AD4F10B).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_200000_puzzle_1190e5a7() {
        let solution: SolutionSimple = |data| {
            let without_duplicates: Image = data.image.remove_duplicates().expect("image");
            let result_image: Image = without_duplicates.remove_grid().expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("1190e5a7").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_1190E5A7: &'static str = "
    f11 $0,101140 ; remove duplicates
    f11 $0,101120 ; remove grid
    ";

    #[test]
    fn test_200001_puzzle_1190e5a7_loda() {
        let result: String = run_simple("1190e5a7", PROGRAM_1190E5A7).expect("String");
        assert_eq!(result, "3 1");
    }
    
    #[test]
    fn test_210000_puzzle_39a8645d() {
        let solution: SolutionSimple = |data| {
            let result_image: Image = PopularObjects::most_popular_object(&data.image).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("39a8645d").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_39A8645D: &'static str = "
    f11 $0,102000 ; most popular object
    ";

    #[test]
    fn test_210001_puzzle_39a8645d_loda() {
        let result: String = run_simple("39a8645d", PROGRAM_39A8645D).expect("String");
        assert_eq!(result, "3 1");
    }
    
    #[test]
    fn test_220000_puzzle_88a62173() {
        let solution: SolutionSimple = |data| {
            let result_image: Image = PopularObjects::least_popular_object(&data.image).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("88a62173").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_88A62173: &'static str = "
    f11 $0,102001 ; least popular object
    ";

    #[test]
    fn test_220001_puzzle_88a62173_loda() {
        let result: String = run_simple("88a62173", PROGRAM_88A62173).expect("String");
        assert_eq!(result, "3 1");
    }
    
    #[test]
    fn test_230000_puzzle_bbc9ae5d() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let repeat_count: u8 = input.width() / 2;
            let mut result_image: Image = Image::empty();
            for i in 0..repeat_count {
                let m = input.clone();
                let j = m.offset_clamp(i as i32, 0).expect("image");
                result_image = result_image.vjoin(j).expect("image");
            }
            Ok(result_image)
        };
        let result: String = solution.run("bbc9ae5d").expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_BBC9AE5D: &'static str = "
    mov $10,$0
    f11 $10,101000 ; get image width
    div $10,2
    ; $10 is the height of the final image
    
    mov $2,0
    mov $7,0
    lps $10

        ; clone the input image, and offset it
        mov $4,$7
        mov $5,0
        mov $3,$0
        f31 $3,101181 ; offset clamp

        ; glue onto the bottom of the result image
        f21 $2,101040 ; vstack

        add $7,1
    lpe
    mov $0,$2
    ";

    #[test]
    fn test_230001_puzzle_bbc9ae5d_loda() {
        let result: String = run_simple("bbc9ae5d", PROGRAM_BBC9AE5D).expect("String");
        assert_eq!(result, "5 1");
    }

    #[test]
    fn test_240000_puzzle_ea32f347() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.most_popular_color().expect("color");
            let background_ignore_mask: Image = input.to_mask_where_color_is(background_color);
            // println!("background_ignore_mask: {:?}", background_ignore_mask);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = input.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &background_ignore_mask)
                .expect("find_objects_with_ignore_mask");
    
            // Count the number of pixels in each object
            let f = |image: &Image| -> (Image, u32) {
                let count: u32 = image.mask_count_one() as u32;
                (image.clone(), count)
            };
            let mut object_count_vec: Vec<(Image, u32)> = object_mask_vec.iter().map(f).collect();
    
            // Sort objects by their number of pixels
            object_count_vec.sort_unstable_by_key(|item| (item.1));
            object_count_vec.reverse();
    
            // Object size to color value
            let mut color_mapping = HashMap::<usize, u8>::new();
            color_mapping.insert(0, 1); // biggest object
            color_mapping.insert(1, 4); // medium object
            color_mapping.insert(2, 2); // smallest object
    
            // Build the result image
            let mut result_image: Image = Image::color(input.width(), input.height(), background_color);
            for (index, item) in object_count_vec.iter().enumerate() {
                let mask_image: Image = item.0.clone();
                // Obtain color for the object size
                let mut assign_color: u8 = 255;
                if let Some(color) = color_mapping.get(&index) {
                    assign_color = *color;
                }
                // Change color of the object
                let colored_object_image: Image = mask_image.replace_color(1, assign_color).expect("Image");
    
                // Overlay each object onto the result image
                result_image = mask_image.select_from_images(&result_image, &colored_object_image).expect("image");
            }
            Ok(result_image)
        };
        let result: String = solution.run("ea32f347").expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_250000_puzzle_7bb29440() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let object_mask: Image = input.to_mask_where_color_is(background_color);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = object_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &object_mask)
                .expect("find_objects_with_ignore_mask");
    
            // Traverse each object, and count holes in each object
            let mut object_count_vec = Vec::<(Image, u32)>::new();
            for mask_image in &object_mask_vec {
                let histogram: Histogram = input.histogram_with_mask(&mask_image).expect("histogram");
                let mut pairs: Vec<(u32,u8)> = histogram.pairs_ascending();
    
                // Remove the background color of the rectangle
                pairs.pop();
    
                // Number of holes inside the rectangle
                let mut pixel_count: u32 = 0;
                for pair in &pairs {
                    pixel_count += pair.0;
                }
    
                object_count_vec.push((mask_image.clone(), pixel_count));
            }
    
            // Sort objects by their number of pixels
            object_count_vec.sort_unstable_by_key(|item| (item.1));
    
            // Pick the first the object with lowest pixel count
            let (mask_image, _pixel_count) = object_count_vec.first().expect("first object");
    
            // Extract pixels from input image, just for the object
            let image: Image = mask_image.select_from_color_and_image(background_color, &input).expect("image");
    
            let result_image = image.trim().expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("7bb29440").expect("String");
        assert_eq!(result, "5 1");
    }

    #[test]
    fn test_260000_puzzle_5521c0d9() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let object_mask: Image = input.to_mask_where_color_is(background_color);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = object_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &object_mask)
                .expect("find_objects_with_ignore_mask");
    
            // Adjust offsets for all objects
            let mut result_image: Image = Image::color(input.width(), input.height(), background_color);
            for mask_image in &object_mask_vec {
    
                // Bounding box of object
                let rect: Rectangle = match mask_image.bounding_box() {
                    Some(value) => value,
                    None => {
                        continue;
                    }
                };
    
                // Determine how much to adjust offset
                let distance_from_bottom: i32 = (input.height() as i32) - (rect.y() as i32);
                let offset_y: i32 = -distance_from_bottom;
    
                // Adjust offset
                let mask_with_offset: Image = mask_image.offset_wrap(0, offset_y).expect("image");
                let image_with_offset: Image = input.offset_wrap(0, offset_y).expect("image");
    
                result_image = mask_with_offset.select_from_images(&result_image, &image_with_offset).expect("image");
            }
            Ok(result_image)
        };
        let result: String = solution.run("5521c0d9").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_270000_puzzle_7f4411dc() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let result_image: Image = input.denoise_type1(background_color).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("7f4411dc").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_7F4411DC: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color

    ; $0 is noisy image
    ; $1 is background_color
    f21 $0,101090 ; denoise type 1
    ; $0 is denoised image
    ";

    #[test]
    fn test_270001_puzzle_7f4411dc_loda() {
        let result: String = run_simple("7f4411dc", PROGRAM_7F4411DC).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_280000_puzzle_aabf363d() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let object_mask: Image = input.to_mask_where_color_is(background_color);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = object_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &object_mask)
                .expect("find_objects_with_ignore_mask");
    
            // Traverse each object, and measure object size
            let mut object_count_vec = Vec::<(Image, u32)>::new();
            for mask_image in &object_mask_vec {
                let histogram: Histogram = input.histogram_with_mask(&mask_image).expect("histogram");
                let pairs: Vec<(u32,u8)> = histogram.pairs_ascending();
    
                // Measure size of the object
                let mut pixel_count: u32 = 0;
                for pair in &pairs {
                    if pair.1 == background_color {
                        continue;
                    }
                    pixel_count += pair.0;
                }
    
                object_count_vec.push((mask_image.clone(), pixel_count));
            }
    
            // Sort objects by their number of pixels
            object_count_vec.sort_unstable_by_key(|item| (item.1));
    
            // Pick the first the object with lowest pixel count
            let (mask_image_biggest, _pixel_count) = object_count_vec.last().expect("first object");
            let (mask_image_smallest, _pixel_count) = object_count_vec.first().expect("first object");
    
            let histogram_smallest: Histogram = input.histogram_with_mask(&mask_image_smallest).expect("histogram");
            let fill_color: u8 = histogram_smallest.most_popular_color().expect("color");
    
            let mut result_image: Image = mask_image_biggest.clone();
            result_image = result_image.replace_color(0, background_color).expect("image");
            result_image = result_image.replace_color(1, fill_color).expect("image");
            Ok(result_image)
        };
        let result: String = solution.run("aabf363d").expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_290000_puzzle_00d62c1b() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let replacement_color: u8 = 4;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let border_mask_image: Image = Image::border_inside(input.width(), input.height(), 0, 1, 1).expect("image");
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = input.find_objects(ImageSegmentAlgorithm::Neighbors).expect("find_objects");
    
            // Traverse the interior objects. Replace color for the interior object.
            let mut result_image: Image = input.clone();
            for mask_image in &object_mask_vec {
                let mask_image_border_overlap: Image = border_mask_image.select_from_color_and_image(0, mask_image).expect("image");
                let border_histogram: Histogram = input.histogram_with_mask(&mask_image_border_overlap).expect("histogram");
                if let Some(border_color) = border_histogram.most_popular_color() {
                    if border_color == background_color {
                        // println!("skip background object: {:?}", mask_image);
                        continue;
                    }
                }
                
                let mask_neighbour: Image = mask_image.outline_mask_neighbour().expect("image");
    
                // println!("mask_image: {:?}", mask_image);
                // println!("mask_neighbour: {:?}", mask_neighbour);
                let histogram: Histogram = input.histogram_with_mask(&mask_neighbour).expect("histogram");
                let pairs: Vec<(u32,u8)> = histogram.pairs_ascending();
                if pairs.len() != 1 {
                    println!("expected 1 color in the histogram, but got: {:?}", pairs); 
                    continue;
                }
                let outline_color: u8 = histogram.most_popular_color().expect("expected 1 color");
                if outline_color == background_color {
                    // Ignore non-interior objects
                    continue;
                }
                // println!("outline_color: {:?}", outline_color);
                // println!("mask_image: {:?}", mask_image);
                // println!("mask_neighbour: {:?}", mask_neighbour);
    
                // Replace color only for the interior objects
                let mask_inverted: Image = mask_image.invert_mask();
                result_image = mask_inverted.select_from_color_and_image(replacement_color, &result_image).expect("image");
            }
            Ok(result_image)
        };
        let result: String = solution.run("00d62c1b").expect("String");
        assert_eq!(result, "5 1");
    }

    #[test]
    fn test_300000_puzzle_ae3edfdc() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_all().most_popular_color().expect("color");
            let ignore_mask = input.to_mask_where_color_is(background_color);
            let color_when_there_is_no_neighbour: u8 = 255;
    
            let neighbour_up: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Up, color_when_there_is_no_neighbour).expect("image");
            let neighbour_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Left, color_when_there_is_no_neighbour).expect("image");
            let neighbour_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Right, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Down, color_when_there_is_no_neighbour).expect("image");
    
            let mut result_image: Image = Image::color(input.width(), input.height(), background_color);
            for y in 0..(input.height() as i32) {
                for x in 0..(input.width() as i32) {
                    let mask_value: u8 = ignore_mask.get(x, y).unwrap_or(255);
                    if mask_value == 1 {
                        continue;
                    }
    
                    let color_up: u8 = neighbour_up.get(x, y).unwrap_or(255);
                    let color_down: u8 = neighbour_down.get(x, y).unwrap_or(255);
                    let color_left: u8 = neighbour_left.get(x, y).unwrap_or(255);
                    let color_right: u8 = neighbour_right.get(x, y).unwrap_or(255);
    
                    let mut histogram = Histogram::new();
                    if color_up != color_when_there_is_no_neighbour {
                        histogram.increment(color_up);
                    }
                    if color_down != color_when_there_is_no_neighbour {
                        histogram.increment(color_down);
                    }
                    if color_left != color_when_there_is_no_neighbour {
                        histogram.increment(color_left);
                    }
                    if color_right != color_when_there_is_no_neighbour {
                        histogram.increment(color_right);
                    }
    
                    if let Some(count) = histogram.most_popular_count() {
                        if count < 2 {
                            continue;
                        }
                    } else {
                        continue;
                    }
    
                    let color_value: u8 = input.get(x, y).unwrap_or(255);
                    let _ = result_image.set(x, y, color_value);
                    if color_up != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x, y - 1, color_up);
                    }
                    if color_down != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x, y + 1, color_down);
                    }
                    if color_left != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x - 1, y, color_left);
                    }
                    if color_right != color_when_there_is_no_neighbour {
                        let _ = result_image.set(x + 1, y, color_right);
                    }
                }
            }
            Ok(result_image)
        };
        let result: String = solution.run("ae3edfdc").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_310000_puzzle_1f876c06() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_all().most_popular_color().expect("color");
            let ignore_mask = input.to_mask_where_color_is(background_color);
            let color_when_there_is_no_neighbour: u8 = 255;
    
            let neighbour_up_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpLeft, color_when_there_is_no_neighbour).expect("image");
            let neighbour_up_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpRight, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownLeft, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownRight, color_when_there_is_no_neighbour).expect("image");
    
            let mut output: Image = input.clone();
            output.set_pixel_where_two_images_agree(&neighbour_down_left, &neighbour_up_right, color_when_there_is_no_neighbour).expect("ok");
            output.set_pixel_where_two_images_agree(&neighbour_up_left, &neighbour_down_right, color_when_there_is_no_neighbour).expect("ok");
            Ok(output)
        };
        let result: String = solution.run("1f876c06").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_1F876C06: &'static str = "
    mov $20,255 ; color when there is no neighbour

    ; ignore mask
    mov $1,$0
    mov $2,$0
    f11 $2,101060 ; most popular color
    f21 $1,101250 ; mask where color is
    ; $2 is most popular color
    ; $1 is the ignore mask

    ; neighbour_up_left
    mov $10,$0
    mov $11,$1
    mov $12,$20
    f31 $10,102064 ; neighbour 'UpLeft'
    mov $3,$10

    ; neighbour_up_right
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102065 ; neighbour 'UpRight'
    mov $4,$10

    ; neighbour_down_left
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102066 ; neighbour 'DownLeft'
    mov $5,$10

    ; neighbour_down_right
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102067 ; neighbour 'DownRight'
    mov $6,$10

    ; prepare the output image
    mov $14,$0 ; clone input image

    ; set pixel where the two images agree
    mov $17,$20 ; color to ignore
    mov $16,$5 ; neighbour_down_left
    mov $15,$4 ; neighbour_up_right
    f41 $14,102100 ; set pixel where two images agree

    ; set pixel where the two images agree
    mov $17,$20 ; color to ignore
    mov $16,$6 ; neighbour_down_right
    mov $15,$3 ; neighbour_up_left
    f41 $14,102100 ; set pixel where two images agree

    mov $0,$14
    ";

    #[test]
    fn test_310001_puzzle_1f876c06_loda() {
        let result: String = run_simple("1f876c06", PROGRAM_1F876C06).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_320000_puzzle_623ea044() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let background_color: u8 = input.histogram_all().most_popular_color().expect("color");
            let ignore_mask = input.to_mask_where_color_is(background_color);
            let color_when_there_is_no_neighbour: u8 = 255;

            let neighbour_up_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpLeft, color_when_there_is_no_neighbour).expect("image");
            let neighbour_up_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::UpRight, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownLeft, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::DownRight, color_when_there_is_no_neighbour).expect("image");

            let mut result_image: Image = input.clone();
            result_image.set_pixel_where_image_has_different_color(&neighbour_up_left, color_when_there_is_no_neighbour)?;
            result_image.set_pixel_where_image_has_different_color(&neighbour_up_right, color_when_there_is_no_neighbour)?;
            result_image.set_pixel_where_image_has_different_color(&neighbour_down_left, color_when_there_is_no_neighbour)?;
            result_image.set_pixel_where_image_has_different_color(&neighbour_down_right, color_when_there_is_no_neighbour)?;
            Ok(result_image)
        };
        let result: String = solution.run("623ea044").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_623EA044: &'static str = "
    mov $20,255 ; color when there is no neighbour

    ; ignore mask
    mov $1,$0
    mov $2,$0
    f11 $2,101060 ; most popular color
    f21 $1,101250 ; mask where color is
    ; $2 is most popular color
    ; $1 is the ignore mask

    ; neighbour_up_left
    mov $10,$0
    mov $11,$1
    mov $12,$20
    f31 $10,102064 ; neighbour 'UpLeft'
    mov $3,$10

    ; neighbour_up_right
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102065 ; neighbour 'UpRight'
    mov $4,$10

    ; neighbour_down_left
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102066 ; neighbour 'DownLeft'
    mov $5,$10

    ; neighbour_down_right
    mov $10,$0
    mov $11,$1
    mov $13,$20
    f31 $10,102067 ; neighbour 'DownRight'
    mov $6,$10

    ; prepare the output image
    mov $14,$0 ; clone input image

    ; draw diagonal line - down right
    mov $15,$3 ; neighbour_up_left
    mov $16,$20
    f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter. 

    ; draw diagonal line - down left
    mov $15,$4 ; neighbour_up_right
    mov $16,$20
    f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter. 

    ; draw diagonal line - up right
    mov $15,$5 ; neighbour_down_left
    mov $16,$20
    f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter. 

    ; draw diagonal line - up left
    mov $15,$6 ; neighbour_down_right
    mov $16,$20
    f31 $14,102101 ; Set pixel where the image has a pixel value different than the color parameter. 

    mov $0,$14
    ";

    #[test]
    fn test_320001_puzzle_623ea044_loda() {
        let result: String = run_simple("623ea044", PROGRAM_623EA044).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_330000_puzzle_f8b3ba0a() {
        let solution: SolutionSimple = |data| {
            let histogram: Histogram = data.image.histogram_all();
            let histogram_image: Image = histogram.to_image().expect("image");
    
            // Take the row with the colors, discard the row with the counters
            let colors = histogram_image.bottom_rows(1).expect("image");
    
            // Discard the 2 most popular colors
            let trimmed = colors.remove_left_columns(2).expect("image");
    
            let output = trimmed.rotate(1).expect("image");
            Ok(output)
        };
        let result: String = solution.run("f8b3ba0a").expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_340000_puzzle_f8ff0b80() {
        let solution: SolutionSimple = |data| {
            let histogram: Histogram = data.image.histogram_all();
            let histogram_image: Image = histogram.to_image().expect("image");
    
            // Take the row with the colors, discard the row with the counters
            let colors = histogram_image.bottom_rows(1).expect("image");
    
            // Discard the 1 most popular color
            let trimmed = colors.remove_left_columns(1).expect("image");
    
            let output = trimmed.rotate(1).expect("image");
            Ok(output)
        };
        let result: String = solution.run("f8ff0b80").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_350000_puzzle_a68b268e() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let histogram: Histogram = input.histogram_all();
            let background_color: u8 = histogram.most_popular_color().expect("color");

            let top: Image = input.top_rows(4).expect("image");
            let top_left: Image = top.left_columns(4).expect("image");
            let top_right: Image = top.right_columns(4).expect("image");
            let bottom: Image = input.bottom_rows(4).expect("image");
            let bottom_left: Image = bottom.left_columns(4).expect("image");
            let bottom_right: Image = bottom.right_columns(4).expect("image");

            let mut output: Image = bottom_right;
            output = output.overlay_with_mask_color(&bottom_left, background_color).expect("image");
            output = output.overlay_with_mask_color(&top_right, background_color).expect("image");
            output = output.overlay_with_mask_color(&top_left, background_color).expect("image");
            Ok(output)
        };
        let result: String = solution.run("a68b268e").expect("String");
        assert_eq!(result, "6 1");
    }

    const PROGRAM_A68B268E: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color

    ; W = compute (width-1) / 2
    mov $2,$0
    f11 $2,101000 ; Get width of image
    sub $2,1
    div $2,2

    ; H = compute (height-1) / 2
    mov $3,$0
    f11 $3,101001 ; Get height of image
    sub $3,1
    div $3,2

    ; top left corner of size WxH
    mov $10,$0
    mov $11,$3
    f21 $10,101220 ; get N top rows
    mov $11,$2
    f21 $10,101222 ; get N left columns
  
    ; top right corner of size WxH
    mov $15,$0
    mov $16,$3
    f21 $15,101220 ; get N top rows
    mov $16,$2
    f21 $15,101223 ; get N right columns
  
    ; bottom left corner of size WxH
    mov $20,$0
    mov $21,$3
    f21 $20,101221 ; get N bottom rows
    mov $21,$2
    f21 $20,101222 ; get N left columns

    ; bottom right corner of size WxH
    mov $25,$0
    mov $26,$3
    f21 $25,101221 ; get N bottom rows
    mov $26,$2
    f21 $25,101223 ; get N right columns

    ; zstack where the images are placed on top of each other
    ; zindex 0 - the bottom
    mov $30,$25 ; bottom right

    ; zindex 1
    mov $31,$20 ; bottom left
    mov $32,$1 ; most popular color
    f31 $30,101150 ; overlay image

    ; zindex 2
    mov $31,$15 ; top right
    mov $32,$1 ; most popular color
    f31 $30,101150 ; overlay image

    ; zindex 3 - the top
    mov $31,$10 ; top left
    mov $32,$1 ; most popular color
    f31 $30,101150 ; overlay image

    mov $0,$30
    ";

    #[test]
    fn test_350001_puzzle_a68b268e_loda() {
        let result: String = run_simple("a68b268e", PROGRAM_A68B268E).expect("String");
        assert_eq!(result, "6 1");
    }

    #[test]
    fn test_360000_puzzle_6b9890af() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let histogram: Histogram = input.histogram_all();
            let background_color: u8 = histogram.most_popular_color().expect("color");

            let ignore_mask: Image = input.to_mask_where_color_is(background_color);
            let mut objects: Vec<Image> = input.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &ignore_mask).expect("images");

            if objects.len() != 2 {
                return Err(anyhow::anyhow!("Expected exactly 2 objects, but got a different count"));
            }

            objects.sort_unstable_by(|lhs, rhs| { 
                let a = lhs.mask_count_one();
                let b = rhs.mask_count_one();
                a.cmp(&b)
            });

            let smallest_object: Image = match objects.first() {
                Some(image) => image.clone(),
                None => {
                    return Err(anyhow::anyhow!("Expected an object, but got none"));
                }
            };

            let biggest_object: Image = match objects.last() {
                Some(image) => image.clone(),
                None => {
                    return Err(anyhow::anyhow!("Expected an object, but got none"));
                }
            };

            // Extract the biggest object
            let biggest_image_full: Image = biggest_object.select_from_color_and_image(background_color, &input).expect("image");
            let biggest_image: Image = biggest_image_full.trim().expect("image");

            // Extract the smallest object
            let smallest_image_full: Image = smallest_object.select_from_color_and_image(background_color, &input).expect("image");
            let smallest_image: Image = smallest_image_full.trim().expect("image");

            let width: u8 = biggest_image.width();
            let x_ratio = width / smallest_image.width();
            let x_ratio_remain = width % smallest_image.width();
            // println!("x_ratio: {} {}", x_ratio, x_ratio_remain);

            let height: u8 = biggest_image.height();
            let y_ratio = height / smallest_image.height();
            let y_ratio_remain = height % smallest_image.height();
            // println!("y_ratio: {} {}", y_ratio, y_ratio_remain);

            if x_ratio != y_ratio {
                return Err(anyhow::anyhow!("Expected same ratio, but different x y ratio: {} {}", x_ratio, y_ratio));
            }

            // Scale up the smallest object so it fits inside the biggest object
            let new_width: u8 = smallest_image.width() * x_ratio;
            let new_height: u8 = smallest_image.height() * y_ratio;
            let fit_image: Image = smallest_image.resize(new_width, new_height).expect("image");
            
            // Overlay the smallest object on top of the biggest object
            let mut output: Image = biggest_image;
            let x = (x_ratio_remain / 2) as i32;
            let y = (y_ratio_remain / 2) as i32;
            output = output.overlay_with_position(&fit_image, x, y).expect("image");

            Ok(output)
        };
        let result: String = solution.run("6b9890af").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_370000_puzzle_2281f1f4() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let background_color: u8 = 0;
            let set_value: u8 = 2;
            let row: Image = input.top_rows(1).expect("image");
            let column: Image = input.right_columns(1).expect("image");
            let mut output: Image = input.clone();
            for y in 0..output.height() {
                for x in 0..output.width() {
                    if y == 0 {
                        continue;
                    }
                    if x + 1 == output.width() {
                        continue;
                    }
                    let value0: u8 = row.get(x as i32, 0).unwrap_or(255); 
                    let value1: u8 = column.get(0, y as i32).unwrap_or(255);
                    if value0 > background_color && value1 > background_color {
                        _ = output.set(x as i32, y as i32, set_value);
                    }
                }
            }
            Ok(output)
        };
        let result: String = solution.run("2281f1f4").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_380000_puzzle_d687bc17_manual() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let mut area: Image = input.remove_left_columns(1).expect("image");
            area = area.remove_right_columns(1).expect("image");
            area = area.remove_top_rows(1).expect("image");
            area = area.remove_bottom_rows(1).expect("image");
            let histogram_rows: Vec<Histogram> = area.histogram_rows();
            let histogram_columns: Vec<Histogram> = area.histogram_columns();

            // Empty overlay image with the most popular color
            let most_popular_color: u8 = area.most_popular_color().expect("color");
            let mut overlay: Image = Image::color(area.width(), area.height(), most_popular_color);

            // Draw pixels for histogram_rows
            for (y, histogram_row) in histogram_rows.iter().enumerate() {
                let y1: i32 = (y + 1) as i32;
                let counters: &[u32; 256] = histogram_row.counters();
                {
                    let color: u8 = input.get(0, y1).unwrap_or(255);
                    let count: u32 = counters[color as usize];
                    for i in 0..count {
                        _ = overlay.set(i as i32, y as i32, color);
                    }
                }
                {
                    let color: u8 = input.get((input.width() as i32) - 1, y1).unwrap_or(255);
                    let count: u32 = counters[color as usize];
                    for i in 0..count {
                        _ = overlay.set((overlay.width() as i32) - (i + 1) as i32, y as i32, color);
                    }
                }
            }

            // Draw pixels for histogram_columns
            for (x, histogram_column) in histogram_columns.iter().enumerate() {
                let x1: i32 = (x + 1) as i32;
                let counters: &[u32; 256] = histogram_column.counters();
                {
                    let color: u8 = input.get(x1, 0).unwrap_or(255);
                    let count: u32 = counters[color as usize];
                    for i in 0..count {
                        _ = overlay.set(x as i32, i as i32, color);
                    }
                }
                {
                    let color: u8 = input.get(x1, (input.height() as i32) - 1).unwrap_or(255);
                    let count: u32 = counters[color as usize];
                    for i in 0..count {
                        _ = overlay.set(x as i32, (overlay.height() as i32) - (i + 1) as i32, color);
                    }
                }
            }

            let output: Image = input.overlay_with_position(&overlay, 1, 1).expect("image");
            Ok(output)
        };
        let result: String = solution.run("d687bc17").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_390000_puzzle_5b6cbef5() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = 0;
            let mask: Image = input.to_mask_where_color_is_different(background_color);
            let tile1: Image = input.clone();
            let tile0: Image = Image::color(tile1.width(), tile1.height(), background_color);
            let output: Image = mask.select_two_tiles(&tile0, &tile1)?;
            Ok(output)
        };
        let result: String = solution.run("5b6cbef5").expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_5B6CBEF5: &'static str = "
    ; tile_width
    mov $2,$0
    f11 $2,101000 ; Get width of image

    ; tile_height
    mov $3,$0
    f11 $3,101001 ; Get height of image

    ; tile
    mov $7,0 ; color
    mov $6,$3 ; height
    mov $5,$2 ; width
    f31 $5,101010 ; Create new image with size (x, y) and filled with color z

    ; mask
    mov $10,$0 ; image
    mov $11,$1 ; color
    f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

    mov $11,$5 ; tile0
    mov $12,$0 ; tile1
    f31 $10,102110 ; Create a big composition of tiles.

    mov $0,$10
    ";

    #[test]
    fn test_390001_puzzle_5b6cbef5_loda() {
        let result: String = run_simple("5b6cbef5", PROGRAM_5B6CBEF5).expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_CCD554AC: &'static str = "
    mov $1,$0
    f11 $1,101000 ; Get width of image

    mov $2,$0
    f11 $2,101001 ; Get height of image

    ; $1 is count x = width of the image
    ; $2 is count y = height of the image
    f31 $0,102120 ; Make a big image by repeating the current image.
    ";

    #[test]
    fn test_400001_puzzle_ccd554ac_loda() {
        let result: String = run_simple("ccd554ac", PROGRAM_CCD554AC).expect("String");
        assert_eq!(result, "6 1");
    }

    const PROGRAM_27F8CE4F: &'static str = "
    mov $1,$0
    f11 $1,101060 ; most popular color

    ; tile_width
    mov $2,$0
    f11 $2,101000 ; Get width of image

    ; tile_height
    mov $3,$0
    f11 $3,101001 ; Get height of image

    ; tile
    mov $7,0 ; color
    mov $6,$3 ; height
    mov $5,$2 ; width
    f31 $5,101010 ; Create new image with size (x, y) and filled with color z

    ; mask
    mov $10,$0 ; image
    mov $11,$1 ; color
    f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

    mov $11,$0 ; tile0
    mov $12,$5 ; tile1
    f31 $10,102110 ; Create a big composition of tiles.

    mov $0,$10
    ";

    #[test]
    fn test_410000_puzzle_27f8ce4f_loda() {
        let result: String = run_simple("27f8ce4f", PROGRAM_27F8CE4F).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_1F85A75F: &'static str = "
    mov $3,$0
    f11 $3,101060 ; most popular color
    ; $3 is background_color

    ; remove noisy pixels
    mov $4,$0
    mov $5,3 ; number of noise colors to remove
    f21 $4,101092 ; denoise type 3
    ;mov $0,$4

    ; mask
    mov $6,$4 ; image
    mov $7,$3 ; color
    f21 $6,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
    ;mov $0,$6

    ; multiply input image by mask
    mov $8,$6
    mov $9,$3
    mov $10,$0
    f31 $8,102130 ; Pick pixels from one image.
    mov $0,$8

    ; remove space around the object
    f11 $0,101160 ; trim
    ";

    #[test]
    fn test_420000_puzzle_1f85a75f_loda() {
        let result: String = run_simple("1f85a75f", PROGRAM_1F85A75F).expect("String");
        assert_eq!(result, "2 1");
    }

    const PROGRAM_80AF3007: &'static str = "
    mov $3,$0
    f11 $3,101060 ; most popular color
    ; $3 is background_color

    ; remove space around the object
    mov $5,$0
    f11 $5,101160 ; trim

    ; scaled down to 3x3
    mov $8,$5
    mov $9,3
    mov $10,3
    f31 $8,101200 ; resize

    ; mask
    mov $10,$8 ; image
    mov $11,$3 ; color
    f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

    ; an empty tile
    mov $14,$3 ; color
    mov $13,3 ; height
    mov $12,3 ; width
    f31 $12,101010 ; Create new image with size (x, y) and filled with color z

    ; Layout tiles
    mov $15,$10
    mov $16,$12 ; tile0
    mov $17,$8 ; tile1
    f31 $15,102110 ; Create a big composition of tiles.
    mov $0,$15
    ";

    #[test]
    fn test_430000_puzzle_80af3007_loda() {
        let result: String = run_simple("80af3007", PROGRAM_80AF3007).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_440000_puzzle_73ccf9c2() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.most_popular_color().expect("pixel");
            let ignore_mask: Image = input.to_mask_where_color_is(background_color);
            let objects: Vec<Image> = input.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &ignore_mask).expect("images");

            // Identify the asymmetric objects
            let mut asymmetric_objects: Vec<Image> = vec!();
            for object in &objects {
                let trimmed: Image = object.trim_color(background_color)?;
                if !trimmed.is_symmetric_x()? {
                    asymmetric_objects.push(object.clone());
                }
            }

            let mut output: Image = Image::empty();
            for object in &asymmetric_objects {
                // Obtain original colors from the input image
                let extracted_object: Image = object.select_from_color_and_image(background_color, &input)?;

                // Trim borders
                let image: Image = extracted_object.trim_color(background_color)?;
                output = image;
                break;
            }
            Ok(output)
        };
        let result: String = solution.run("73ccf9c2").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_450000_puzzle_72ca375d() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.most_popular_color().expect("pixel");
            let ignore_mask: Image = input.to_mask_where_color_is(background_color);
            let objects: Vec<Image> = input.find_objects_with_ignore_mask(ImageSegmentAlgorithm::All, &ignore_mask).expect("images");

            // Identify the symmetric objects
            let mut symmetric_objects: Vec<Image> = vec!();
            for object in &objects {
                let trimmed: Image = object.trim_color(background_color)?;
                if trimmed.is_symmetric_x()? {
                    symmetric_objects.push(object.clone());
                }
            }

            let mut output: Image = Image::empty();
            for object in &symmetric_objects {
                // Obtain original colors from the input image
                let extracted_object: Image = object.select_from_color_and_image(background_color, &input)?;

                // Trim borders
                let image: Image = extracted_object.trim_color(background_color)?;
                output = image;
                break;
            }
            Ok(output)
        };
        let result: String = solution.run("72ca375d").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_460000_puzzle_dbc1a6ce() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let background_color: u8 = input.histogram_all().most_popular_color().expect("color");
            let ignore_mask = input.to_mask_where_color_is(background_color);
            let color_when_there_is_no_neighbour: u8 = 255;

            let neighbour_up: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Up, color_when_there_is_no_neighbour).expect("image");
            let neighbour_down: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Down, color_when_there_is_no_neighbour).expect("image");
            let neighbour_left: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Left, color_when_there_is_no_neighbour).expect("image");
            let neighbour_right: Image = input.neighbour_color(&ignore_mask, ImageNeighbourDirection::Right, color_when_there_is_no_neighbour).expect("image");

            let mut lines: Image = input.clone();
            lines.set_pixel_where_two_images_agree(&neighbour_up, &neighbour_down, color_when_there_is_no_neighbour).expect("ok");
            lines.set_pixel_where_two_images_agree(&neighbour_left, &neighbour_right, color_when_there_is_no_neighbour).expect("ok");
            lines = lines.replace_color(1, 8)?;

            let result_image: Image = ignore_mask.select_from_images(&input, &lines)?;
            Ok(result_image)
        };
        let result: String = solution.run("dbc1a6ce").expect("String");
        assert_eq!(result, "4 1");
    }

    mod solve_ea959feb {
        use super::*;

        pub struct MySolution {
            repair_color: u8,
        }
    
        impl MySolution {
            pub fn new() -> Self {
                Self {
                    repair_color: 255
                }
            }
        }
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, task: &arc_work_model::Task) -> anyhow::Result<()> {
                // Obtain the color is used for damaged pixels
                let mut found_color: Option<u8> = None;
                for label in &task.action_label_set_intersection {
                    match label {
                        ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color } => {
                            found_color = Some(*color);
                            break;
                        },
                        _ => {}
                    };
                }

                let color: u8 = match found_color {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Expected a repair color"));
                    }
                };
    
                self.repair_color = color;
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, _task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let input: &Image = &data.image;
                let result_image: Image = input.repair_pattern_with_color(self.repair_color)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_470000_puzzle_1d0a4b61() {
        let mut instance = solve_ea959feb::MySolution::new();
        let result: String = run_analyze_and_solve("1d0a4b61", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_470001_puzzle_29ec7d0e() {
        let mut instance = solve_ea959feb::MySolution::new();
        let result: String = run_analyze_and_solve("29ec7d0e", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_470002_puzzle_ca8f78db() {
        let mut instance = solve_ea959feb::MySolution::new();
        let result: String = run_analyze_and_solve("ca8f78db", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }    

    #[test]
    fn test_470003_puzzle_e95e3d8e() {
        let mut instance = solve_ea959feb::MySolution::new();
        let result: String = run_analyze_and_solve("e95e3d8e", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }            

    #[test]
    fn test_470004_puzzle_ea959feb() {
        let mut instance = solve_ea959feb::MySolution::new();
        let result: String = run_analyze_and_solve("ea959feb", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const ADVANCED_PROGRAM_EA959FEB: &'static str = r#"
    mov $80,$99
    mov $81,100
    mov $82,102
    mov $83,105 ; address of vector[0].OutputImageIsInputImageWithChangesLimitedToPixelsWithColor
    lps $80
      mov $0,$$81
      mov $1,$$83
      f21 $0,102151 ; Repair damaged pixels and recreate big repeating patterns such as mosaics.
      mov $$82,$0
      add $81,100
      add $82,100
      add $83,100
    lpe
    "#;

    #[test]
    fn test_470005_puzzle_ea959feb_loda() {
        let result: String = run_advanced("ea959feb", ADVANCED_PROGRAM_EA959FEB).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_520000_puzzle_49d1d64f() {
        let solution: SolutionSimple = |data| {
            data.image.border_grow(1, 0)
        };
        let result: String = solution.run("49d1d64f").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_49D1D64F: &'static str = "
    mov $1,1
    mov $2,0
    f31 $0,102160 ; Expand by repeating the outer-most pixel border
    ";

    #[test]
    fn test_520001_puzzle_49d1d64f_loda() {
        let result: String = run_simple("49d1d64f", PROGRAM_49D1D64F).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_530000_puzzle_780d0b14() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let mask: Image = input.mask_for_gridcells(Some(0))?;
            let objects: Vec<Image> = mask.find_objects(ImageSegmentAlgorithm::Neighbors)?;
            let mut result_image = Image::zero(input.width(), input.height());
            for object in &objects {
                let histogram: Histogram = input.histogram_with_mask(object)?;
                let color: u8 = histogram.most_popular_color().expect("color");
                result_image = object.select_from_image_and_color(&result_image, color)?;
            }
            result_image = result_image.remove_grid()?;
            result_image = result_image.remove_duplicates()?;
            Ok(result_image)
        };
        let result: String = solution.run("780d0b14").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_540000_puzzle_a699fb00() {
        let mut instance = solve_a699fb00::MySolution::new();
        let result: String = run_analyze_and_solve("a699fb00", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_a699fb00 {
        use super::*;

        type Dict = HashMap<Image, Image>;
    
        pub struct MySolution {
            dict_outer: Dict,
        }
    
        impl MySolution {
            pub fn new() -> Self {
                Self {
                    dict_outer: Dict::new(),
                }
            }
        }
        
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, task: &arc_work_model::Task) -> anyhow::Result<()> {
                let mut dict_inner = Dict::new();
                for pair in &task.pairs {
                    if pair.pair_type != PairType::Train {
                        continue;
                    }
                    let diff_mask: Image = pair.input.image.diff(&pair.output.image)?;
                    let width: u8 = pair.input.image.width();
                    let height: u8 = pair.input.image.height();
                    for y in 0..height {
                        for x in 0..width {
                            let get_x: i32 = x as i32;
                            let get_y: i32 = y as i32;
                            let is_different0: u8 = diff_mask.get(get_x, get_y).unwrap_or(255);
                            let is_different1: u8 = diff_mask.get(get_x+1, get_y).unwrap_or(255);
                            let is_different2: u8 = diff_mask.get(get_x+2, get_y).unwrap_or(255);
                            let should_replace_horizontal = is_different0 == 0 && is_different1 == 1 && is_different2 == 0;
                            if !should_replace_horizontal {
                                continue;
                            }
                            let rect = Rectangle::new(x, y, 3, 1);
                            let replace_source: Image = pair.input.image.crop(rect)?;
                            let replace_target: Image = pair.output.image.crop(rect)?;
    
                            if let Some(value) = dict_inner.get(&replace_source) {
                                if *value != replace_target {
                                    return Err(anyhow::anyhow!("No consensus on what replacements are to be done"));
                                }
                            }
                            dict_inner.insert(replace_source, replace_target);
    
                            // let pixel_value0: u8 = pair.input.image.get(get_x, get_y).unwrap_or(255);
                            // let pixel_value1: u8 = pair.output.image.get(get_x, get_y).unwrap_or(255);
                            // println!("replace from {} to {}", pixel_value0, pixel_value1);
                        }
                    }
                }
                // println!("number of items in dict: {}", dict_inner.len());
                self.dict_outer = dict_inner;
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, _task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let input: &Image = &data.image;
                let mut result_image: Image = input.clone();
                // Do substitutions from the dictionary
                for _ in 0..100 {
                    let mut stop = true;
                    for (key, value) in &self.dict_outer {
                        let position = result_image.find_exact(key)?;
                        if let Some((x, y)) = position {
                            result_image = result_image.overlay_with_position(value, x as i32, y as i32)?;
                            stop = false;
                        }
                    }
                    if stop {
                        break;
                    }
                }
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_550000_puzzle_94f9d214() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let height: u8 = input.height() / 2;
            let input_top: Image = input.top_rows(height)?;
            let input_bottom: Image = input.bottom_rows(height)?;
            let mask_top: Image = input_top.to_mask_where_color_is_different(0);
            let mask_bottom: Image = input_bottom.to_mask_where_color_is_different(0);
            let mut result_image: Image = mask_top.clone();
            result_image = result_image.overlay_with_mask_color(&mask_bottom, 0)?;
            result_image = result_image.replace_color(0, 2)?;
            result_image = result_image.replace_color(1, 0)?;
            Ok(result_image)
        };
        let result: String = solution.run("94f9d214").expect("String");
        assert_eq!(result, "4 1");
    }

    mod solve_5c0a986e_regex_manual {
        use super::*;

        pub struct MySolution;
    
        impl MySolution {
            pub fn new() -> Self {
                Self {}
            }
        }
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, _task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let input: &Image = &data.image;

                let pattern1a: &str = 
                "^\\d+,\\d+,\\d+,\\d+,\\d+,\
                \\d+,\\d+,\\d+,\\d+,\\d+,\
                \\d+,\\d+,0,\\d+,\\d+,\
                \\d+,\\d+,\\d+,1,1,\
                \\d+,\\d+,\\d+,1,1$";

                let pattern1b: &str = 
                "^\\d+,\\d+,\\d+,\\d+,\\d+,\
                \\d+,\\d+,\\d+,\\d+,\\d+,\
                \\d+,\\d+,0,\\d+,\\d+,\
                \\d+,\\d+,\\d+,1,0,\
                \\d+,\\d+,\\d+,0,1$";

                let pattern2a: &str = 
                "^2,2,\\d+,\\d+,\\d+,\
                2,2,\\d+,\\d+,\\d+,\
                \\d+,\\d+,0,\\d+,\\d+,\
                \\d+,\\d+,\\d+,\\d+,\\d+,\
                \\d+,\\d+,\\d+,\\d+,\\d+$";
        
                let pattern2b: &str = 
                "^2,0,\\d+,\\d+,\\d+,\
                0,2,\\d+,\\d+,\\d+,\
                \\d+,\\d+,0,\\d+,\\d+,\
                \\d+,\\d+,\\d+,\\d+,\\d+,\
                \\d+,\\d+,\\d+,\\d+,\\d+$";
        
                let replacements: Vec<ImageReplaceRegexToColor> = vec![
                    ImageReplaceRegexToColor {
                        regex: Regex::new(pattern1a).expect("regex"),
                        color: 1,
                    },
                    ImageReplaceRegexToColor {
                        regex: Regex::new(pattern1b).expect("regex"),
                        color: 1,
                    },
                    ImageReplaceRegexToColor {
                        regex: Regex::new(pattern2a).expect("regex"),
                        color: 2,
                    },
                    ImageReplaceRegexToColor {
                        regex: Regex::new(pattern2b).expect("regex"),
                        color: 2,
                    }
                ];

                let mut result_image: Image = input.padding_with_color(2, 0)?;
                let _count: usize = result_image.replace_5x5_regex(&replacements, 14, 14)?;
                // println!("replace_5x5_regex. count: {}", count);
                let rect = Rectangle::new(2, 2, input.width(), input.height());
                let result_image_cropped: Image = result_image.crop(rect)?;
                Ok(result_image_cropped)
            }
        }
    }

    #[test]
    fn test_560003_puzzle_5c0a986e_using_regex_manual() {
        let mut instance = solve_5c0a986e_regex_manual::MySolution::new();
        let result: String = run_analyze_and_solve("5c0a986e", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_5c0a986e_regex_advanced {
        use super::*;

        pub struct MySolution {
            replacements: Vec<ImageReplaceRegexToColor>,
        }
    
        impl MySolution {
            pub fn new() -> Self {
                Self {
                    replacements: vec!()
                }
            }

            fn similarity_of_two5x5(source_image: &Image, target_image: &Image, ignore_color: u8) -> anyhow::Result<(u8, u8)> {
                if source_image.width() != 5 || source_image.height() != 5 || target_image.width() != 5 || target_image.height() != 5 {
                    return Err(anyhow::anyhow!("both images must have the exact size 5x5"));
                }
                let target_pixel_value: u8 = target_image.get(2, 2).unwrap_or(255);

                let mut count_same_as_target: u8 = 0;
                let mut count_same: u8 = 0;
                let mut count_different: u8 = 0;
                for y in 0..5 {
                    for x in 0..5 {
                        if y == 2 && x == 2 {
                            // Ignore the center pixel of the 5x5
                            continue;
                        }
                        let pixel_value0: u8 = source_image.get(x as i32, y as i32).unwrap_or(255);
                        let pixel_value1: u8 = target_image.get(x as i32, y as i32).unwrap_or(255);
                        if pixel_value0 == ignore_color && pixel_value1 == ignore_color {
                            continue;
                        }
                        if pixel_value0 != pixel_value1 {
                            count_different += 1;
                            continue;
                        }
                        // Idea, use distance from the center pixel to assign more weight to pixels near center
                        if pixel_value0 == target_pixel_value {
                            count_same_as_target += 1;
                        } else {
                            count_same += 1;
                        }
                    }
                }
                count_same += count_same_as_target * 10;
                Ok((count_same, count_different))
            }

            fn pattern_of_two5x5(source_image: &Image, target_image: &Image, ignore_color: u8) -> anyhow::Result<String> {
                if source_image.width() != 5 || source_image.height() != 5 || target_image.width() != 5 || target_image.height() != 5 {
                    return Err(anyhow::anyhow!("both images must have the exact size 5x5"));
                }
                let target_center_pixel_value: u8 = target_image.get(2, 2).unwrap_or(255);
                let mut pattern_parts = Vec::<String>::new();
                for y in 0..5 {
                    for x in 0..5 {
                        let pixel_value0: u8 = source_image.get(x as i32, y as i32).unwrap_or(255);
                        if y == 2 && x == 2 {
                            // Special treatment for the center pixel of the 5x5
                            pattern_parts.push(format!("{}", pixel_value0));
                            continue;
                        }
                        let pixel_value1: u8 = target_image.get(x as i32, y as i32).unwrap_or(255);
                        if pixel_value0 == ignore_color && pixel_value1 == ignore_color {
                            pattern_parts.push("\\d+".into());
                            continue;
                        }
                        if pixel_value0 == target_center_pixel_value {
                            pattern_parts.push(format!("{}", pixel_value0));
                            continue;
                        }
                        pattern_parts.push("\\d+".into());
                    }
                }

                let pattern: String = format!("^{}$", pattern_parts.join(","));
                Ok(pattern)
            }

            fn analyze_train_pair(pair: &arc_work_model::Pair) -> anyhow::Result<Vec<ImageReplaceRegexToColor>> {
                let background_color: u8 = 0;
                let mut input_image: Image = pair.input.image.padding_with_color(2, background_color)?;
                let output_image: Image = pair.output.image.padding_with_color(2, background_color)?;

                // HtmlLog::text("analyze train pair");
                let mut replacements = Vec::<ImageReplaceRegexToColor>::new();

                for _iteration in 0..20 {
                    // HtmlLog::text(format!("iteration: {}", iteration));

                    let rect = Rectangle::new(2, 2, pair.input.image.width(), pair.input.image.height());
                    let current_input: Image = input_image.crop(rect)?;
                    let diff_mask: Image = current_input.diff(&pair.output.image)?;
                    // HtmlLog::image(&diff_mask);
    
                    let positions: Vec<(u8, u8)> = diff_mask.positions_where_color_is(1);
                    // println!("positions: {:?}", positions);
                    if positions.is_empty() {
                        break;
                    }
   
                    let mut found_x: u8 = 0;
                    let mut found_y: u8 = 0;
                    let mut found_score: u8 = 0;
                    for (x, y) in &positions {

                        let rect = Rectangle::new(*x, *y, 5, 5);
                        let input_crop: Image = input_image.crop(rect)?;
                        let output_crop: Image = output_image.crop(rect)?;

                        // Compare input_crop with output_crop, ignoring the center pixel
                        // If they are nearly identical, then we know that it's only the center pixel that has changed.
                        // And that we can establish a pattern at this position.
                        let (count_same, _count_diff) = Self::similarity_of_two5x5(&input_crop, &output_crop, background_color)?;
                        // println!("position: {},{}  same: {}  diff: {}", x, y, count_same, count_diff);
                        if count_same > found_score {
                            found_x = *x;
                            found_y = *y;
                            found_score = count_same;
                        }
                    }
                    if found_score == 0 {
                        break;
                    }
                    // println!("found position: {},{}", found_x, found_y);

                    let x: u8 = found_x;
                    let y: u8 = found_y;
                    let rect = Rectangle::new(x, y, 5, 5);
                    let input_crop: Image = input_image.crop(rect)?;
                    let output_crop: Image = output_image.crop(rect)?;

                    let pattern: String = Self::pattern_of_two5x5(&input_crop, &output_crop, background_color)?;
                    // println!("pattern: {}", pattern);
                    
                    let target_color: u8 = output_crop.get(2, 2).unwrap_or(255);
                    // println!("target_color: {}", target_color);

                    let item = ImageReplaceRegexToColor {
                        regex: Regex::new(&pattern)?,
                        color: target_color,
                    };
                    replacements.push(item);

                    let _replace_count: usize = input_image.replace_5x5_regex(&replacements, 14, 14)?;
                    // println!("replace_count: {}", replace_count);
                    // HtmlLog::image(&input_image);
                }
                // HtmlLog::text("separator");
                Ok(replacements)
            }

            fn intersection(items0: &Vec<ImageReplaceRegexToColor>, items1: &Vec<ImageReplaceRegexToColor>) -> Vec<ImageReplaceRegexToColor> {
                let mut result_items = Vec::<ImageReplaceRegexToColor>::new();
                for item0 in items0 {
                    for item1 in items1 {
                        if *item0.regex.as_str() != *item1.regex.as_str() {
                            continue;
                        }
                        if item0.color != item1.color {
                            continue;
                        }
                        result_items.push(item0.clone());
                    }
                }
                result_items
            }
        }

        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, task: &arc_work_model::Task) -> anyhow::Result<()> {
                let mut is_first = true;
                let mut replacements_intersection = Vec::<ImageReplaceRegexToColor>::new();
                for pair in &task.pairs {
                    if pair.pair_type != PairType::Train {
                        continue;
                    }
                    let replacements: Vec<ImageReplaceRegexToColor> = Self::analyze_train_pair(pair)?;
                    if is_first {
                        is_first = false;
                        replacements_intersection = replacements;
                    } else {
                        replacements_intersection = Self::intersection(&replacements_intersection, &replacements);
                    }
                    if replacements_intersection.is_empty() {
                        break;
                    }
                }
                // println!("rules.len: {}", replacements_intersection.len());
                // println!("rules: {:?}", replacements_intersection);
                self.replacements = replacements_intersection;
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, _task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let input: &Image = &data.image;
                let mut result_image: Image = input.padding_with_color(2, 0)?;
                let _count: usize = result_image.replace_5x5_regex(&self.replacements, 14, 14)?;
                // println!("replace_5x5_regex. count: {}", count);
                let rect = Rectangle::new(2, 2, input.width(), input.height());
                let result_image_cropped: Image = result_image.crop(rect)?;
                Ok(result_image_cropped)
            }
        }
    }

    #[test]
    fn test_560004_puzzle_5c0a986e_using_regex_advanced() {
        let mut instance = solve_5c0a986e_regex_advanced::MySolution::new();
        let result: String = run_analyze_and_solve("5c0a986e", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_570000_puzzle_3428a4f5() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let half_height: u8 = input.height() / 2;
            let a: Image = input.top_rows(half_height)?;
            let b: Image = input.bottom_rows(half_height)?;
            let result_image: Image = a.mask_xor(&b)?;
            Ok(result_image)
        };
        let result: String = solution.run("3428a4f5").expect("String");
        assert_eq!(result, "4 2");
    }

    const PROGRAM_3428A4F5: &'static str = "
    mov $5,$0
    f11 $5,101001 ; get height
    div $5,2

    mov $4,$0
    f21 $4,101221 ; get N bottom rows
    
    mov $1,$5
    f21 $0,101220 ; get N top rows

    mov $1,$4
    f21 $0,101254 ; xor
    ";

    #[test]
    fn test_570001_puzzle_3428a4f5_loda() {
        let result: String = run_simple("3428a4f5", PROGRAM_3428A4F5).expect("String");
        assert_eq!(result, "4 2");
    }

    #[test]
    fn test_580000_puzzle_25d8a9c8() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let mut result_image: Image = input.count_unique_colors_per_row()?;
            result_image = result_image.to_mask_where_color_is_different(1);
            result_image = result_image.repeat_by_count(input.width(), 1)?;
            Ok(result_image)
        };
        let result: String = solution.run("25d8a9c8").expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_590000_puzzle_50cb2852() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let color_count: Image = input.count_duplicate_pixels_in_3x3()?;
            let count_mask: Image = color_count.to_mask_where_color_is_equal_or_greater_than(8);
            let object_mask: Image = input.to_mask_where_color_is_different(0);
            let mask: Image = count_mask.mask_and(&object_mask)?;
            let result_image: Image = mask.select_from_image_and_color(&input, 42)?;
            Ok(result_image)
        };
        let result: String = solution.run("50cb2852").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_50CB2852: &'static str = "
    mov $1,$0
    f11 $1,102140 ; Traverse all pixels in the 3x3 convolution and count how many have the same color as the center.
    mov $2,8
    f21 $1,101253 ; Convert to a mask image by converting `pixel_color >= threshold_color` to 1 and converting anything else to to 0.

    mov $2,$0
    mov $3,0
    f21 $2,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

    f21 $1,101255 ; AND between two masks

    mov $3,42
    mov $2,$0
    f31 $1,102131 ; Pick pixels from image and color. When the mask is 0 then pick from the image. When the mask is [1..255] then use the `default_color`.

    mov $0,$1
    ";

    #[test]
    fn test_590001_puzzle_50cb2852_loda() {
        let result: String = run_simple("50cb2852", PROGRAM_50CB2852).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_600000_puzzle_c1d99e64() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let mask: Image = input.mask_for_gridcells(None)?;
            let result_image: Image = mask.select_from_color_and_image(42, &input)?;
            Ok(result_image)
        };
        let result: String = solution.run("c1d99e64").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_610000_puzzle_f2829549() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let half_width: u8 = input.width() / 2;
            let a: Image = input.left_columns(half_width)?;
            let b: Image = input.right_columns(half_width)?;
            let result_image: Image = a.mask_or(&b)?;
            Ok(result_image)
        };
        let result: String = solution.run("f2829549").expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_F2829549: &'static str = "
    mov $5,$0
    f11 $5,101000 ; get width
    div $5,2

    mov $4,$0
    f21 $4,101222 ; get N left columns
    
    mov $1,$5
    f21 $0,101223 ; get N right columns

    mov $1,$4
    f21 $0,101256 ; or
    ";

    #[test]
    fn test_610001_puzzle_f2829549_loda() {
        let result: String = run_simple("f2829549", PROGRAM_F2829549).expect("String");
        assert_eq!(result, "5 1");
    }

    #[test]
    fn test_620000_puzzle_662c240a() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let height: u8 = input.height() / 3;
            let mut images = Vec::<Image>::new();
            for i in 0..3 {
                let y: i32 = (height as i32) * i;
                if y > (u8::MAX as i32) {
                    return Err(anyhow::anyhow!("cannot split image"));
                }
                let rect = Rectangle::new(0, y as u8, input.width(), height);
                let image: Image = input.crop(rect)?;
                images.push(image);
            }
            for image in &images {
                if image.is_symmetric_any_diagonal()? {
                    continue;
                }
                return Ok(image.clone()); 
            }
            Err(anyhow::anyhow!("no non-symmetric image found"))
        };
        let result: String = solution.run("662c240a").expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_630000_puzzle_b6afb2da() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let color_count: Image = input.count_duplicate_pixels_in_3x3()?;
            let object_mask: Image = input.to_mask_where_color_is_different(0);
            let result_image: Image = object_mask.select_from_color_and_image(42, &color_count)?;
            Ok(result_image)
        };
        let result: String = solution.run("b6afb2da").expect("String");
        assert_eq!(result, "2 1");
    }

    const PROGRAM_B6AFB2DA: &'static str = "
    mov $80,$99
    mov $81,100
    mov $82,102
    mov $83,105 ; address of vector[0].OutputImageIsInputImageWithChangesLimitedToPixelsWithColor
    lps $80
        mov $0,$$81

        mov $4,$0
        f11 $4,102140 ; Traverse all pixels in the 3x3 convolution and count how many have the same color as the center.

        mov $5,$0
        mov $6,$$83
        f21 $5,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

        mov $8,42
        mov $7,$4
        mov $6,$5
        f31 $6,102131 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.

        mov $$82,$6
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_630001_puzzle_b6afb2da_loda() {
        let result: String = run_advanced("b6afb2da", PROGRAM_B6AFB2DA).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_640000_puzzle_c444b776() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;

            // Objects from the grid
            let cell_mask: Image = input.mask_for_gridcells(Some(4))?;
            let ignore_mask: Image = cell_mask.invert_mask();
            let objects: Vec<Image> = cell_mask.find_objects_with_ignore_mask(ImageSegmentAlgorithm::Neighbors, &ignore_mask)?;

            // Identify the single template image
            let mut image_to_insert: Option<Image> = None;
            let mut number_of_images_found: usize = 0;
            for object in &objects {
                let rect: Rectangle = match object.bounding_box() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("expected all objects to have a bounding box"));
                    }
                };
                let image: Image = input.crop(rect)?;
                let count: u16 = image.histogram_all().number_of_counters_greater_than_zero();
                if count == 1 {
                    continue;
                }
                image_to_insert = Some(image);
                number_of_images_found += 1;
            }
            if number_of_images_found >= 2 {
                return Err(anyhow::anyhow!("Found 2 or more patterns to insert"));
            }
            let template_image: Image = match image_to_insert {
                Some(image) => image,
                None => {
                    return Err(anyhow::anyhow!("Didn't find any pattern for insertion"));
                }
            };

            // Insert the template image into all the empty cells
            let mut result_image: Image = input.clone();
            for object in &objects {
                let rect: Rectangle = match object.bounding_box() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("expected all objects to have a bounding box"));
                    }
                };
                let image: Image = input.crop(rect)?;
                let count: u16 = image.histogram_all().number_of_counters_greater_than_zero();
                if count != 1 {
                    continue;
                }
                result_image = result_image.overlay_with_position(&template_image, rect.x() as i32, rect.y() as i32)?;
            }
            Ok(result_image)
        };
        let result: String = solution.run("c444b776").expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_650000_puzzle_017c7c7b() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let ignore_mask = Image::zero(input.width(), input.height());
            let periodicity_optional: Option<u8> = input.vertical_periodicity(&ignore_mask)?;
            let periodicity: u8 = periodicity_optional.expect("u8");
            if periodicity < 1 {
                return Err(anyhow::anyhow!("expected periodicity to be 1 or more"));
            }
            let repeat_count: u8 = (9 / periodicity) + 1;
            let pattern: Image = input.top_rows(periodicity)?;
            let mut result_image: Image = pattern.repeat_by_count(1, repeat_count)?;
            let rect = Rectangle::new(0, 0, input.width(), 9);
            result_image = result_image.crop(rect)?;
            Ok(result_image)
        };
        let result: String = solution.run("017c7c7b").expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_660000_puzzle_6f8cd79b() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let rect = Rectangle::new(1, 1, input.width() - 2, input.height() - 2);
            let cropped: Image = input.crop(rect)?;
            let result_image: Image = cropped.padding_with_color(1, 42)?;
            Ok(result_image)
        };
        let result: String = solution.run("6f8cd79b").expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_670000_puzzle_cf98881b() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let width: u8 = (input.width() - 2) / 3;
            let mut images = Vec::<Image>::new();
            for i in 0..3 {
                let x: i32 = (width as i32 + 1) * i;
                if x > (u8::MAX as i32) {
                    return Err(anyhow::anyhow!("cannot split image"));
                }
                let rect = Rectangle::new(x as u8, 0, width, input.height());
                let image: Image = input.crop(rect)?;
                images.push(image);
            }
            images.reverse();
            let mut result_image = Image::empty();
            for (index, image) in images.iter().enumerate() {
                if index == 0 {
                    result_image = image.clone();
                    continue;
                }
                let mask: Image = image.to_mask_where_color_is_different(0);
                result_image = mask.select_from_images(&result_image, &image)?;
            }
            Ok(result_image)
        };
        let result: String = solution.run("cf98881b").expect("String");
        assert_eq!(result, "5 1");
    }

    #[test]
    fn test_680000_puzzle_8731374e() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;

            let color_count: Image = input.count_duplicate_pixels_in_3x3()?;
            let ignore_mask: Image = color_count.to_mask_where_color_is_equal_or_less_than(3);
            let mut objects: Vec<ImageSegmentItem> = input.find_objects_with_ignore_mask_inner(ImageSegmentAlgorithm::Neighbors, &ignore_mask)?;
            objects.sort_unstable_by_key(|item| (item.mass(), item.x(), item.y()));
            objects.reverse();
            let biggest_object: ImageSegmentItem = match objects.first() {
                Some(value) => value.clone(),
                None => {
                    return Err(anyhow::anyhow!("biggest object"));
                }
            };
            let color_to_be_trimmed: u8 = 0;
            
            // Idea, with the actionlabel, check the size of the masked area correspond to the output size
            let rect: Rectangle = biggest_object.mask().inner_bounding_box_after_trim_with_color(color_to_be_trimmed)?;
            if rect.is_empty() {
                return Err(anyhow::anyhow!("bounding box is empty"));
            }
            
            // Idea, save the bounding box as a mask and provide it to the .asm program
            // let mut the_mask = Image::zero(input.width(), input.height());
            // the_mask = the_mask.fill_inside_rect(rect, 1)?;

            // Crop out the strongly-connected-object from input image
            let cropped_input: Image = input.crop(rect)?;

            let histogram_all: Histogram = cropped_input.histogram_all();
            if histogram_all.number_of_counters_greater_than_zero() != 2 {
                return Err(anyhow::anyhow!("expected 2 colors in the cropped area"));
            }

            let least_popular_color: u8 = match histogram_all.least_popular_color_disallow_ambiguous() {
                Some(color) => color,
                None => {
                    return Err(anyhow::anyhow!("expected a least popular colors in the cropped area"));
                }
            };
            
            let line_color: u8 = least_popular_color;
            let mut result_image: Image = cropped_input.clone();

            // Shoot out lines in all directions
            _ = result_image.draw_line_where_row_or_column_contains_color(&cropped_input, least_popular_color, line_color)?;

            Ok(result_image)
        };
        let result: String = solution.run("8731374e").expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_f9012d9b {
        use super::*;

        pub struct MySolution;
    
        impl MySolution {
            pub fn new() -> Self {
                Self {}
            }
        }
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let input: &Image = &data.image;

                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let repair_mask: Image = match &pair.input.repair_mask {
                    Some(value) => value.clone(),
                    None => {
                        return Err(anyhow::anyhow!("Expected a repair mask"));
                    }
                };
                let rect: Rectangle = match repair_mask.bounding_box() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Unable to determine bounding box of repair mask"));
                    }
                };

                let mut result_image: Image = input.repair_pattern_with_mask(&repair_mask)?;
                result_image = result_image.crop(rect)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_690000_puzzle_f9012d9b() {
        let mut instance = solve_f9012d9b::MySolution::new();
        let result: String = run_analyze_and_solve("f9012d9b", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod repair_symmetry_crop {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let repair_mask: Image = match &pair.input.repair_mask {
                    Some(value) => value.clone(),
                    None => {
                        return Err(anyhow::anyhow!("Expected a repair mask"));
                    }
                };
                let repair_mask_bounding_box: Rectangle = match repair_mask.bounding_box() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Unable to determine bounding box of repair mask"));
                    }
                };
                let repaired_image: Image = match &pair.input.repaired_image {
                    Some(value) => value.clone(),
                    None => {
                        return Err(anyhow::anyhow!("Expected repaired_image"));
                    }
                };
                let result_image: Image = repaired_image.crop(repair_mask_bounding_box)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_700000_puzzle_de493100() {
        let mut instance = repair_symmetry_crop::MySolution {};
        let result: String = run_analyze_and_solve("de493100", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_700001_puzzle_dc0a314f() {
        let mut instance = repair_symmetry_crop::MySolution {};
        let result: String = run_analyze_and_solve("dc0a314f", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_700002_puzzle_ff805c23() {
        let mut instance = repair_symmetry_crop::MySolution {};
        let result: String = run_analyze_and_solve("ff805c23", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_FF805C23: &'static str = "
    mov $80,$99
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,106 ; address of vector[0].RepairMask
    mov $84,107 ; address of vector[0].RepairedImage
    lps $80
        ; replace what is outside the repair mask with the color 255
        mov $0,$$83 ; repair mask
        mov $1,255 ; color for what to be removed
        mov $2,$$84 ; repaired image
        f31 $0,102130 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.

        ; crop out the repair mask
        mov $1,255
        f21 $0,101161 ; trim with color

        mov $$82,$0
        add $82,100
        add $83,100
        add $84,100
    lpe
    ";

    #[test]
    fn test_700002_puzzle_ff805c23_loda() {
        let result: String = run_advanced("ff805c23", PROGRAM_FF805C23).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_700003_puzzle_67b4a34d() {
        let mut instance = repair_symmetry_crop::MySolution {};
        let result: String = run_analyze_and_solve("67b4a34d", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_700004_puzzle_9ecd008a() {
        // Predicts the correct size of the output, but for the wrong reasons.
        // It doesn't detect that it's a strongly connected area of 3x3 pixels, and that it's colored black.
        let mut instance = repair_symmetry_crop::MySolution {};
        let result: String = run_analyze_and_solve("9ecd008a", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod repair_symmetry {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let repaired_image: Image = match &pair.input.repaired_image {
                    Some(value) => value.clone(),
                    None => {
                        return Err(anyhow::anyhow!("Expected repaired_image"));
                    }
                };
                Ok(repaired_image)
            }
        }
    }

    #[test]
    fn test_710000_puzzle_af22c60d() {
        let mut instance = repair_symmetry::MySolution {};
        let result: String = run_analyze_and_solve("af22c60d", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_710001_puzzle_b8825c91() {
        let mut instance = repair_symmetry::MySolution {};
        let result: String = run_analyze_and_solve("b8825c91", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_710002_puzzle_3631a71a() {
        let mut instance = repair_symmetry::MySolution {};
        let result: String = run_analyze_and_solve("3631a71a", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_710003_puzzle_929ab4e9() {
        let mut instance = repair_symmetry::MySolution {};
        let result: String = run_analyze_and_solve("929ab4e9", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_710004_puzzle_47996f11() {
        let mut instance = repair_symmetry::MySolution {};
        let result: String = run_analyze_and_solve("47996f11", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_47996F11: &'static str = "
    mov $80,$99
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,107 ; address of vector[0].RepairedImage
    lps $80
        mov $$82,$$83
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_710004_puzzle_47996f11_loda() {
        let result: String = run_advanced("47996f11", PROGRAM_47996F11).expect("String");
        assert_eq!(result, "4 1");
    }

    #[allow(dead_code)]
    mod inspect_grid {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                println!("grid: {:?}", pair.input.grid);
                let result_image: Image = pair.input.image.clone();
                Ok(result_image)
            }
        }
    }

    #[allow(dead_code)]
    // #[test]
    fn test_720000_puzzle_95a58926() {
        let mut instance = inspect_grid::MySolution {};
        let result: String = run_analyze_and_solve("95a58926", &mut instance).expect("String");
        assert_eq!(result, "5 1");
    }

    mod solve_0b148d64 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                // println!("grid: {:?}", pair.input.grid);
                let input: &Image = &pair.input.image;
                let grid_pattern: &GridPattern = match &pair.input.grid_pattern {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Missing grid_pattern for input"));
                    }
                };
                let grid_mask: &Image = &grid_pattern.line_mask;
                let grid_color: u8 = grid_pattern.color;

                // Segment the image into cells
                let blank: Image = Image::zero(input.width(), input.height());
                let cells: Vec<Image> = blank.find_objects_with_ignore_mask(ImageSegmentAlgorithm::Neighbors, grid_mask)?;
                if cells.is_empty() {
                    return Err(anyhow::anyhow!("Expected 1 or more cells"));
                }
                let enumerated_cells: Image = Image::object_enumerate(&cells).expect("image");

                // Determine the most popular color of each cell
                let object_count: usize = cells.len();
                let mut cell_colors = Vec::<u8>::new();
                for object_index in 0..object_count {
                    let object_color: u8 = ((object_index + 1) & 255) as u8;

                    let mask: Image = enumerated_cells.to_mask_where_color_is(object_color);
                    let mut histogram: Histogram = input.histogram_with_mask(&mask)?;

                    // Ignore pixels that has the grid_color
                    histogram.set_counter_to_zero(grid_color);

                    let cell_color: u8 = match histogram.most_popular_color_disallow_ambiguous() {
                        Some(value) => value,
                        None => {
                            return Err(anyhow::anyhow!("Ambiguous what is the most popular color"));
                        }
                    };
                    cell_colors.push(cell_color);
                }
                // println!("cell_colors: {:?}", cell_colors);

                // histogram of all cell colors
                let mut cell_histogram = Histogram::new();
                for cell_color in &cell_colors {
                    cell_histogram.increment(*cell_color);
                }

                // pick the cell with the least popular color
                let least_popular_cell_color: u8 = match cell_histogram.least_popular_color_disallow_ambiguous() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Ambiguous what is the least popular color"));
                    }
                };
                // println!("least_popular_cell_color: {}", least_popular_cell_color);
                let mut found_cell_index: Option<usize> = None;
                for (index, cell_color) in cell_colors.iter().enumerate() {
                    if *cell_color == least_popular_cell_color {
                        found_cell_index = Some(index);
                        break;
                    }
                }
                let cell_index: usize = match found_cell_index {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Unable to identify the cell of interest"));
                    }
                };

                // mask for the cell
                let object_color: u8 = ((cell_index + 1) & 255) as u8;
                let mask: Image = enumerated_cells.to_mask_where_color_is(object_color);

                // crop out the cell 
                let crop_rect: Rectangle = match mask.bounding_box() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("cannot determine bounding box"));
                    }
                };
                let result_image: Image = input.crop(crop_rect)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_730000_puzzle_0b148d64() {
        let mut instance = solve_0b148d64::MySolution {};
        let result: String = run_analyze_and_solve("0b148d64", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_c3202e5a {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let grid_pattern: &GridPattern = match &pair.input.grid_pattern {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Missing grid_pattern for input"));
                    }
                };
                let grid_mask: &Image = &grid_pattern.line_mask;

                // Segment the image into cells
                let blank: Image = Image::zero(input.width(), input.height());
                let cells: Vec<Image> = blank.find_objects_with_ignore_mask(ImageSegmentAlgorithm::Neighbors, grid_mask)?;
                if cells.is_empty() {
                    return Err(anyhow::anyhow!("Expected 1 or more cells"));
                }
                let enumerated_cells: Image = Image::object_enumerate(&cells).expect("image");

                // Determine the most popular color of each cell
                let object_count: usize = cells.len();
                let mut cell_index_and_unique_color_counts = Vec::<(usize, u16)>::new();
                for object_index in 0..object_count {
                    let object_color: u8 = ((object_index + 1) & 255) as u8;

                    let mask: Image = enumerated_cells.to_mask_where_color_is(object_color);
                    let histogram: Histogram = input.histogram_with_mask(&mask)?;

                    let unique_color_count: u16 = histogram.number_of_counters_greater_than_zero();
                    cell_index_and_unique_color_counts.push((object_index, unique_color_count));
                }
                // println!("cell_unique_color_counts: {:?}", cell_unique_color_counts);
                // println!("cells_with_single_color: {:?}", cell_indexes_with_single_color);

                // determine the cell indexes with fewest unique colors.
                cell_index_and_unique_color_counts.sort_unstable_by_key(|t| t.1 );
                // println!("cell_index_and_unique_color_counts: {:?}", cell_index_and_unique_color_counts);

                // pick the first cell index with fewest unique colors. Disallow ambiguous results.
                let mut histogram = Histogram::new();
                for (_cell_index, unique_color_count) in &cell_index_and_unique_color_counts {
                    if *unique_color_count <= (u8::MAX as u16) {
                        histogram.increment(*unique_color_count as u8);
                    }
                }
                let mut found: Option<u8> = None;
                for i in 0..=255u8 {
                    let count: u32 = histogram.get(i);
                    if count == 0 {
                        continue;
                    }
                    if count == 1 {
                        found = Some(i);
                        break;
                    }
                    return Err(anyhow::anyhow!("Multiple cells with the same number of unique colors. Ambiguous which color to pick."));
                }
                let color_count: u8 = match found {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Did not found any color. Cannot decide on which color to pick."));
                    }
                };

                let mut found_cell_index: Option<usize> = None;
                for (cell_index, unique_color_count) in &cell_index_and_unique_color_counts {
                    if *unique_color_count == (color_count as u16) {
                        found_cell_index = Some(*cell_index);
                        break;
                    }
                }

                let cell_index: usize = match found_cell_index {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Unable to identify the cell of interest"));
                    }
                };

                // mask for the cell
                let object_color: u8 = ((cell_index + 1) & 255) as u8;
                let mask: Image = enumerated_cells.to_mask_where_color_is(object_color);

                // crop out the cell 
                let crop_rect: Rectangle = match mask.bounding_box() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("cannot determine bounding box"));
                    }
                };
                let result_image: Image = input.crop(crop_rect)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_740000_puzzle_c3202e5a() {
        let mut instance = solve_c3202e5a::MySolution {};
        let result: String = run_analyze_and_solve("c3202e5a", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_1c0d0a4b {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let grid_pattern: &GridPattern = match &pair.input.grid_pattern {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Missing grid_pattern for input"));
                    }
                };
                let grid_mask: &Image = &grid_pattern.line_mask;
                let grid_color: u8 = grid_pattern.color;
                let cell_content: Image = input.to_mask_where_color_is_different(grid_color);
                let result_image: Image = cell_content.mask_xor(&grid_mask)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_750000_puzzle_1c0d0a4b() {
        let mut instance = solve_1c0d0a4b::MySolution {};
        let result: String = run_analyze_and_solve("1c0d0a4b", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_6773b310 {
        use crate::arc::GridPattern;

        use super::*;

        fn measure_mass_of_objects(image: &Image, enumerated_objects: &Image, ignore_colors: Option<&Histogram>) -> anyhow::Result<Image> {
            let mut result_image = Image::zero(image.width(), image.height());
            for color in 0..=255u8 {
                let mask: Image = enumerated_objects.to_mask_where_color_is(color);
                let mut histogram: Histogram = image.histogram_with_mask(&mask)?;
                if let Some(other) = ignore_colors {
                    histogram.subtract_histogram(other);
                }
                let mass_of_object: u32 = histogram.sum();
                let set_color: u8 = mass_of_object.min(255) as u8;
                for y in 0..image.height() {
                    for x in 0..image.width() {
                        let mask_value: u8 = mask.get(x as i32, y as i32).unwrap_or(0);
                        if mask_value == 0 {
                            continue;
                        }
                        _ = result_image.set(x as i32, y as i32, set_color);
                    }
                }
            }
            Ok(result_image)
        }

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, _task: &arc_work_model::Task) -> anyhow::Result<()> {
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let grid_pattern: &GridPattern = match &pair.input.grid_pattern {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Missing grid_pattern for input"));
                    }
                };
                let grid_mask: &Image = &grid_pattern.line_mask;
                let grid_color: u8 = grid_pattern.color;
                let background_color: u8;
                {
                    let mut histogram: Histogram = pair.input.histogram.clone();
                    histogram.set_counter_to_zero(grid_color);
                    background_color = match histogram.most_popular_color_disallow_ambiguous() {
                        Some(value) => value,
                        None => {
                            return Err(anyhow::anyhow!("ambiguous what the background color is"));
                        }
                    };
                }

                // Segment the image into cells
                let blank: Image = Image::zero(input.width(), input.height());
                let cells: Vec<Image> = blank.find_objects_with_ignore_mask(ImageSegmentAlgorithm::Neighbors, grid_mask)?;
                if cells.is_empty() {
                    return Err(anyhow::anyhow!("Expected 1 or more cells"));
                }
                let enumerated_cells: Image = Image::object_enumerate(&cells).expect("image");

                let mut ignore_colors = Histogram::new();
                ignore_colors.increment(background_color);
                ignore_colors.increment(grid_color);
                let mass_of_objects: Image = measure_mass_of_objects(input, &enumerated_cells, Some(&ignore_colors))?;

                // Create output image with the size of the grid
                let grid_width: u8 = grid_pattern.horizontal_cell_count;
                let grid_height: u8 = grid_pattern.vertical_cell_count;
                if grid_width < 1 || grid_height < 1 {
                    return Err(anyhow::anyhow!("Too small grid. Must be 1x1 or bigger"));
                }
                let mut result_image: Image = Image::zero(grid_width, grid_height);

                // Determine the number of unique colors for each cell
                let object_count: usize = cells.len();
                for object_index in 0..object_count {
                    let object_color: u8 = ((object_index + 1) & 255) as u8;

                    let mask: Image = enumerated_cells.to_mask_where_color_is(object_color);
                    let histogram: Histogram = mass_of_objects.histogram_with_mask(&mask)?;
                    let set_color: u8 = match histogram.most_popular_color_disallow_ambiguous() {
                        Some(value) => value,
                        None => {
                            return Err(anyhow::anyhow!("Cannot decide what color is the most popular. ambiguous"));
                        }
                    };

                    let y_usize: usize = object_index / (grid_width as usize);
                    let x_usize: usize = object_index % (grid_width as usize);
                    let x: u8 = (x_usize & 255) as u8;
                    let y: u8 = (y_usize & 255) as u8;
                    _ = result_image.set(x as i32, y as i32, set_color);
                }

                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_760000_puzzle_6773b310() {
        let mut instance = solve_6773b310::MySolution {};
        let result: String = run_analyze_and_solve("6773b310", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }
}
