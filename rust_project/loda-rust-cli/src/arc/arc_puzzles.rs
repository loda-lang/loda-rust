#[cfg(test)]
mod tests {
    use crate::arc::arc_json_model::{Task, ImagePair};
    use crate::arc::arc_work_model::{self, PairType};
    use crate::arc::{ActionLabel, convolution3x3, ImageCollect, ImageSize, ImageLayout, ImageLayoutMode};
    use crate::arc::{RunWithProgram, RunWithProgramResult, SolutionSimple, SolutionSimpleData, AnalyzeAndSolve, ImageRepeat, ImagePeriodicity};
    use crate::arc::{ImageOverlay, ImageNoiseColor, ImageGrid, ImageExtractRowColumn, ConnectedComponent, PixelConnectivity, ConnectedComponentItem, ImageMask, Histogram};
    use crate::arc::{ImageFind, ImageOutline, ImageRotate, ImageBorder, ImageCompare, ImageCrop, ImageResize};
    use crate::arc::{Image, PopularObjects, ImageNeighbour, ImageNeighbourDirection, ImageRepairPattern, ImageFill};
    use crate::arc::{ObjectsMeasureMass, ObjectsUniqueColorCount, ObjectWithSmallestValue, ObjectWithDifferentColor};
    use crate::arc::{ObjectsToGrid, ObjectsToGridMode, SubstitutionRule, ReverseColorPopularity, ObjectsAndMass};
    use crate::arc::{ImageTrim, ImageRemoveDuplicates, ImageStack, ImageMaskCount, ImageSetPixelWhere, GridPattern};
    use crate::arc::{ImageReplaceColor, ImageSymmetry, ImageOffset, ImageColorProfile, ImageCreatePalette, ImageDrawLineWhere};
    use crate::arc::{ImageHistogram, ImageDenoise, ImageDetectHole, ImageTile, ImagePadding, Rectangle, ImageObjectEnumerate};
    use crate::arc::{ImageReplaceRegex, ImageReplaceRegexToColor, ImagePosition, ImageMaskBoolean, ImageCountUniqueColors};
    use crate::arc::{ImageDrawRect, SingleColorObjects, SingleColorObjectClusterContainer, ObjectsAndGravity, ObjectsAndGravityDirection};
    use crate::arc::{MixMode, ImageMix, GravityDirection, ImageGravity, ImageSort, ImageSortMode, Color};
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
                optional_position = search_area.find_first(&needle_with_color).expect("some position");
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
            let background_color: u8 = input.most_popular_color().expect("pixel");

            // Collect pixels by traversing columns
            let rotated: Image = input.rotate_cw()?;
            let mask: Image = rotated.to_mask_where_color_is_different(background_color);
            let pixels: Image = rotated.collect_pixels_as_image(&mask)?;

            // Layout the collected pixels in a 3x3 image
            let size = ImageSize { width: 3, height: 3 };
            let result_image: Image = pixels.layout(size, 0, ImageLayoutMode::ReverseOddRows).expect("ok");
            Ok(result_image)
        };
        let result: String = solution.run("cdecee7f").expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_CDECEE7F: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,103 ; address of vector[0].PredictedOutputWidth
    mov $84,104 ; address of vector[0].PredictedOutputHeight
    mov $85,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$85 ; most popular color across inputs

        ; rotate input by 90 degrees clockwise
        mov $3,$0
        mov $4,1
        f21 $3,101170 ; rotate cw
        ; $3 is the rotated input image
    
        ; extract mask
        mov $5,$3
        mov $6,$1
        f21 $5,101251 ; where color is different than background color
        ; $5 is the mask

        ; collect pixels
        mov $7,$3 ; the rotated input image
        mov $8,$5 ; mask
        f21 $7,102230 ; collect pixels
        ; $7 is a single row with the collected pixels

        ; change layout of the pixels
        mov $9,$7 ; pixels to be re-layouted
        mov $10,$$83 ; width = predicted width
        mov $11,$$84 ; height = predicted height
        mov $12,$1 ; background = most popular color
        f41 $9,102241 ; layout pixels with ReverseOddRows

        mov $0,$9

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
        add $84,100
        add $85,100
    lpe
    ";

    #[test]
    fn test_70001_puzzle_cdecee7f_loda() {
        let result: String = run_advanced("cdecee7f", PROGRAM_CDECEE7F).expect("String");
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
            let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &input, &background_ignore_mask)
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

    mod solve_ea32f347 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let enumerated_objects: &Image = match &pair.input.enumerated_objects {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("no enumerated_objects"));
                    }
                };
                Ok(enumerated_objects.clone())
            }
        }
    }

    #[test]
    fn test_240001_puzzle_ea32f347() {
        let mut instance = solve_ea32f347::MySolution {};
        let result: String = run_analyze_and_solve("ea32f347", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_ENUMERATED_OBJECTS: &'static str = "
    mov $80,$99
    mov $81,110 ; address of vector[0].EnumeratedObjects
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $$82,$$81
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_240002_puzzle_ea32f347_loda() {
        let result: String = run_advanced("ea32f347", PROGRAM_ENUMERATED_OBJECTS).expect("String");
        assert_eq!(result, "4 1");
    }

    #[test]
    fn test_250000_puzzle_7bb29440() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.histogram_border().most_popular_color().expect("color");
            let object_mask: Image = input.to_mask_where_color_is(background_color);
    
            // Objects that is not the background
            let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &object_mask, &object_mask)
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
            let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &object_mask, &object_mask)
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
            let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &object_mask, &object_mask)
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

    mod solve_aabf363d {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let background_color: u8 = pair.input.most_popular_intersection_color.expect("color");
                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");
                let mut result_image: Image = input.replace_color(noise_color, background_color)?;
                result_image = result_image.replace_colors_other_than(background_color, noise_color)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_280001_puzzle_aabf363d() {
        let mut instance = solve_aabf363d::MySolution {};
        let result: String = run_analyze_and_solve("aabf363d", &mut instance).expect("String");
        assert_eq!(result, "2 1");
    }

    const PROGRAM_AABF363D: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    mov $84,115 ; address of vector[0].InputSinglePixelNoiseColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$84 ; noise color
        mov $2,$$83 ; background color
        f31 $0,101050 ; Replace noise pixels with background color

        mov $3,$1
        mov $1,$2 ; background color
        mov $2,$3 ; noise color
        f31 $0,101051 ; Replace non-background-pixels with noise color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
        add $84,100
    lpe
    ";

    #[test]
    fn test_280002_puzzle_aabf363d_loda() {
        let result: String = run_advanced("aabf363d", PROGRAM_AABF363D).expect("String");
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
            let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects(PixelConnectivity::Connectivity4, &input).expect("find_objects");
    
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

    const PROGRAM_00D62C1B: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; from_color = most popular color
        mov $2,255 ; to_color = auto assign color
        f31 $0,102180 ; Flood fill at every pixel along the border, connectivity-4.
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_290001_puzzle_00d62c1b_loda() {
        let result: String = run_advanced("00d62c1b", PROGRAM_00D62C1B).expect("String");
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

    const PROGRAM_F8FF0B80: &'static str = "
    f11 $0,101230 ; Histogram of image. The most popular to the left, least popular to the right. The top row is the counters. The bottom row is the colors.

    mov $1,1
    f21 $0,101224 ; remove top-most row

    ; $1 is 1
    f21 $0,101226 ; remove left-most column
    ";

    #[test]
    fn test_340001_puzzle_f8ff0b80_loda() {
        let result: String = run_simple("f8ff0b80", PROGRAM_F8FF0B80).expect("String");
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
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $20,$$83 ; most popular color across inputs
        mov $21,1 ; pixel spacing = 1

        mov $10,$$81 ; input image
        mov $11,$21 ; spacing
        f22 $10,102261 ; split into 2 rows
        ; $10..$11 are the 2 rows

        mov $15,$10
        mov $16,$21 ; spacing
        f22 $15,102260 ; split into 2 columns
        ; $15..$16 are the 2 columns

        mov $17,$11
        mov $18,$21 ; spacing
        f22 $17,102260 ; split into 2 columns
        ; $17..$18 are the 2 columns

        ; $15 = cell top left
        ; $16 = cell top right
        ; $17 = cell bottom left
        ; $18 = cell bottom right

        mov $0,$20 ; transparent color
        mov $1,$18 ; layer 0 lowest layer
        mov $2,$17 ; layer 1
        mov $3,$16 ; layer 2
        mov $4,$15 ; layer 3 top
        f51 $0,101152 ; Z-stack images: Overlay multiple images using a transparency color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_350001_puzzle_a68b268e_loda() {
        let result: String = run_advanced("a68b268e", PROGRAM_A68B268E).expect("String");
        assert_eq!(result, "6 1");
    }

    #[test]
    fn test_360000_puzzle_6b9890af() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let histogram: Histogram = input.histogram_all();
            let background_color: u8 = histogram.most_popular_color().expect("color");

            let ignore_mask: Image = input.to_mask_where_color_is(background_color);
            let mut objects: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &input, &ignore_mask).expect("images");

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
            let objects: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &input, &ignore_mask).expect("images");

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
    fn test_440001_puzzle_73ccf9c2() {
        let mut instance = solve_crop_first_object::MySolution {};
        let result: String = run_analyze_and_solve("73ccf9c2", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_440002_puzzle_73ccf9c2_loda() {
        let result: String = run_advanced("73ccf9c2", PROGRAM_CROP_FIRST_OBJECT).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_450000_puzzle_72ca375d() {
        let solution: SolutionSimple = |data| {
            let input = data.image;
            let background_color: u8 = input.most_popular_color().expect("pixel");
            let ignore_mask: Image = input.to_mask_where_color_is(background_color);
            let objects: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &input, &ignore_mask).expect("images");

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
    fn test_450001_puzzle_72ca375d() {
        let mut instance = solve_crop_first_object::MySolution {};
        let result: String = run_analyze_and_solve("72ca375d", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_450002_puzzle_72ca375d_loda() {
        let result: String = run_advanced("72ca375d", PROGRAM_CROP_FIRST_OBJECT).expect("String");
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

    mod solve_dbc1a6ce {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");
                let mut result_image: Image = input.clone();
                _ = result_image.draw_line_connecting_two_colors(noise_color, noise_color, 255)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_460001_puzzle_dbc1a6ce() {
        let mut instance = solve_dbc1a6ce::MySolution {};
        let result: String = run_analyze_and_solve("dbc1a6ce", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_DBC1A6CE: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,115 ; address of vector[0].InputSinglePixelNoiseColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; noise color

        ; $1 is color0 = noise color
        mov $2,$1 ; color1 = noise color
        mov $3,255 ; line color = 255 auto fill in
        f41 $0,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_460002_puzzle_dbc1a6ce_loda() {
        let result: String = run_advanced("dbc1a6ce", PROGRAM_DBC1A6CE).expect("String");
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
            let objects: Vec<Image> = ConnectedComponent::find_objects(PixelConnectivity::Connectivity4, &mask)?;
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
                        let position = result_image.find_first(key)?;
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
    fn test_540000_puzzle_a699fb00() {
        let mut instance = solve_a699fb00::MySolution::new();
        let result: String = run_analyze_and_solve("a699fb00", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_540001_puzzle_a699fb00() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;
            let background_color: u8 = match input.most_popular_color() {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("unclear what is the most popular color"));
                }
            };
            let image_padded: Image = input.padding_with_color(1, background_color)?;
            let result_image: Image = convolution3x3(&image_padded, |bm| {
                let left: u8 = bm.get(0, 1).unwrap_or(255);
                let center: u8 = bm.get(1, 1).unwrap_or(255);
                let right: u8 = bm.get(2, 1).unwrap_or(255);
                let result_color: u8;
                // flag the pixel when it have 2 neighbors with same color, but different center pixel.
                if left != background_color && left != center && left == right {
                    result_color = 255;
                } else {
                    result_color = center;
                }
                Ok(result_color)
            })?;
    
            Ok(result_image)
        };
        let result: String = solution.run("a699fb00").expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_a699fb00_version2 {
        use super::*;
    
        pub struct MySolution {
            substitution_rule: Option<SubstitutionRule>,
        }
    
        impl MySolution {
            pub fn new() -> Self {
                Self {
                    substitution_rule: None,
                }
            }
        }
        
        impl AnalyzeAndSolve for MySolution {
            fn analyze(&mut self, task: &arc_work_model::Task) -> anyhow::Result<()> {
                let mut image_pairs = Vec::<(Image, Image)>::new();
                for pair in &task.pairs {
                    if pair.pair_type != PairType::Train {
                        continue;
                    }
                    image_pairs.push((pair.input.image.clone(), pair.output.image.clone()));
                }

                let rule: SubstitutionRule = SubstitutionRule::find_rule(image_pairs)?;
                // println!("substitution_rule.source: {:?}", rule.source);
                // println!("substitution_rule.destination: {:?}", rule.destination);

                self.substitution_rule = Some(rule);
                Ok(())   
            }
    
            fn solve(&self, data: &SolutionSimpleData, _task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let rule: &SubstitutionRule = match &self.substitution_rule {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("expected some substitution_rule"));
                    }
                };
                rule.apply(&data.image)
            }
        }
    }

    const PROGRAM_SUBSTITUTION_RULE_APPLIED: &'static str = "
    mov $80,$99
    mov $81,111 ; address of vector[0].SubstitutionRuleApplied
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $$82,$$81
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_540002_puzzle_a699fb00() {
        let mut instance = solve_a699fb00_version2::MySolution::new();
        let result: String = run_analyze_and_solve("a699fb00", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_540003_puzzle_a699fb00_loda() {
        let result: String = run_advanced("a699fb00", PROGRAM_SUBSTITUTION_RULE_APPLIED).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_541000_puzzle_b60334d2() {
        let mut instance = solve_a699fb00_version2::MySolution::new();
        let result: String = run_analyze_and_solve("b60334d2", &mut instance).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_541001_puzzle_b60334d2_loda() {
        let result: String = run_advanced("b60334d2", PROGRAM_SUBSTITUTION_RULE_APPLIED).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_542000_puzzle_d364b489() {
        let mut instance = solve_a699fb00_version2::MySolution::new();
        let result: String = run_analyze_and_solve("d364b489", &mut instance).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_542001_puzzle_d364b489_loda() {
        let result: String = run_advanced("d364b489", PROGRAM_SUBSTITUTION_RULE_APPLIED).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_543000_puzzle_6c434453() {
        let mut instance = solve_a699fb00_version2::MySolution::new();
        let result: String = run_analyze_and_solve("6c434453", &mut instance).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_543001_puzzle_6c434453_loda() {
        let result: String = run_advanced("6c434453", PROGRAM_SUBSTITUTION_RULE_APPLIED).expect("String");
        assert_eq!(result, "2 1");
    }

    #[test]
    fn test_544000_puzzle_95990924() {
        let mut instance = solve_a699fb00_version2::MySolution::new();
        let result: String = run_analyze_and_solve("95990924", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_544001_puzzle_95990924_loda() {
        let result: String = run_advanced("95990924", PROGRAM_SUBSTITUTION_RULE_APPLIED).expect("String");
        assert_eq!(result, "3 1");
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
    mov $1,$0
    f11 $1,101001 ; get height
    mod $1,2 ; spacing between the columns
    
    f22 $0,102261 ; split into 2 rows
    ; $0..$1 are the 2 rows
    
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

    const PROGRAM_25D8A9C8: &'static str = "
    mov $1,$0
    f11 $1,101000 ; get width
    ; $1 is the width of the input image

    f11 $0,101241 ; count unique colors per row

    ; $1 is the width of the input image
    mov $2,1
    f31 $0,102120 ; repeat image
    ";

    #[test]
    fn test_580001_puzzle_25d8a9c8_loda() {
        let result: String = run_simple("25d8a9c8", PROGRAM_25D8A9C8).expect("String");
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

    const PROGRAM_C1D99E64: &'static str = "
    mov $2,$0
    f11 $0,102270 ; Mask, where the cells are the value is 1 and where the grid lines are the value is 0. Don't care about the color of the grid lines.
    mov $1,42
    f31 $0,102130 ; Pick pixels from color and image. When the mask is 0 then pick the `default_color`. When the mask is [1..255] then pick from the image.
    ";

    #[test]
    fn test_600001_puzzle_c1d99e64_loda() {
        let result: String = run_simple("c1d99e64", PROGRAM_C1D99E64).expect("String");
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
    mov $1,$0
    f11 $1,101000 ; get width
    mod $1,2 ; spacing between the columns

    f22 $0,102260 ; split into 2 columns
    ; $0..$1 are the 2 columns

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
            let objects: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity4, &cell_mask, &ignore_mask)?;

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

    const PROGRAM_CF98881B: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $20,$$83 ; most popular color across inputs

        mov $10,$$81 ; input image
        mov $11,1 ; 1 pixel spacing
        f23 $10,102260 ; split into 3 columns
        ; $10..$12 are the 3 columns

        mov $0,$20 ; transparent color
        mov $1,$12 ; layer 0 lowest layer
        mov $2,$11 ; layer 1
        mov $3,$10 ; layer 2 top
        f41 $0,101152 ; Z-stack images: Overlay multiple images using a transparency color
      
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_670001_puzzle_cf98881b_loda() {
        let result: String = run_advanced("cf98881b", PROGRAM_CF98881B).expect("String");
        assert_eq!(result, "5 1");
    }

    #[test]
    fn test_680000_puzzle_8731374e() {
        let solution: SolutionSimple = |data| {
            let input: Image = data.image;

            let color_count: Image = input.count_duplicate_pixels_in_3x3()?;
            let ignore_mask: Image = color_count.to_mask_where_color_is_equal_or_less_than(3);
            let mut objects: Vec<ConnectedComponentItem> = ConnectedComponent::find_objects_with_ignore_mask_inner(PixelConnectivity::Connectivity4, &input, &ignore_mask)?;
            objects.sort_unstable_by_key(|item| (item.mass, item.x, item.y));
            objects.reverse();
            let biggest_object: ConnectedComponentItem = match objects.first() {
                Some(value) => value.clone(),
                None => {
                    return Err(anyhow::anyhow!("biggest object"));
                }
            };
            let color_to_be_trimmed: u8 = 0;
            
            // Idea, with the actionlabel, check the size of the masked area correspond to the output size
            let rect: Rectangle = biggest_object.mask.inner_bounding_box_after_trim_with_color(color_to_be_trimmed)?;
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

            let mask: Image = cropped_input.to_mask_where_color_is(least_popular_color);
            _ = result_image.draw_line_where_mask_is_nonzero(&mask, line_color)?;

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
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let grid_pattern: &GridPattern = match &pair.input.grid_pattern {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Missing grid_pattern for input"));
                    }
                };
                let grid_color: u8 = grid_pattern.color;

                let enumerated_objects: &Image = match &pair.input.enumerated_objects {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Missing enumerated_objects for input"));
                    }
                };

                let mut ignore_colors = Histogram::new();
                ignore_colors.increment(grid_color);

                // Find the cell with the unusual color
                let mask: Image = ObjectWithDifferentColor::run(&input, &enumerated_objects, Some(&ignore_colors))?;

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

    const PROGRAM_0B148D64: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,109 ; address of vector[0].GridColor
    mov $84,110 ; address of vector[0].EnumeratedObjects
    lps $80
        mov $0,$$81
        mov $1,$$84 ; enumerated objects
        mov $2,$$83 ; grid color

        f31 $0,104111 ; Find the single object that has different colors than the other objects. With 1 ignore color.

        mov $1,255 ; color for the area to be trimmed
        mov $2,$$81
        f31 $0,102130 ; Pick pixels from color and image

        ; $1 is the color to be trimmed
        f21 $0,101161 ; trim with color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
        add $84,100
    lpe
    ";

    #[test]
    fn test_730001_puzzle_0b148d64_loda() {
        let result: String = run_advanced("0b148d64", PROGRAM_0B148D64).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_c3202e5a {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let enumerated_objects: &Image = match &pair.input.enumerated_objects {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Missing enumerated_objects for input"));
                    }
                };

                // Number of unique colors inside each object
                let unique_colors: Image = ObjectsUniqueColorCount::run(input, &enumerated_objects, None)?;

                // Pick the object with the lowest number of unique colors
                let mask: Image = ObjectWithSmallestValue::run(&unique_colors, &enumerated_objects)?;

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

    const PROGRAM_C3202E5A: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,110 ; address of vector[0].EnumeratedObjects
    lps $80
        mov $0,$$81

        mov $1,$$83 ; enumerated objects
        f21 $0,104000 ; Count unique colors in each object

        ; $1 is the enumerated objects
        f21 $0,104100 ; Pick object with the smallest value

        mov $1,255 ; color for the area to be trimmed
        mov $2,$$81
        f31 $0,102130 ; Pick pixels from color and image

        ; $1 is the color to be trimmed
        f21 $0,101161 ; trim with color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_740001_puzzle_c3202e5a_loda() {
        let result: String = run_advanced("c3202e5a", PROGRAM_C3202E5A).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_1c0d0a4b {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
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

    const PROGRAM_1C0D0A4B: &'static str = "
    mov $80,$99
    mov $81,100
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,108 ; address of vector[0].GridMask
    mov $84,109 ; address of vector[0].GridColor
    lps $80
        mov $0,$$81

        mov $1,$$84 ; grid color
        f21 $0,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.

        mov $1,$$83 ; grid mask
        f21 $0,101254 ; xor

        mov $$82,$0

        add $81,100
        add $82,100
        add $83,100
        add $84,100
    lpe
    ";

    #[test]
    fn test_750001_puzzle_1c0d0a4b_loda() {
        let result: String = run_advanced("1c0d0a4b", PROGRAM_1C0D0A4B).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_6773b310 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
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
                let cells: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity4, &blank, grid_mask)?;
                if cells.is_empty() {
                    return Err(anyhow::anyhow!("Expected 1 or more cells"));
                }
                let enumerated_cells: Image = Image::object_enumerate(&cells).expect("image");

                let mut ignore_colors = Histogram::new();
                ignore_colors.increment(background_color);
                ignore_colors.increment(grid_color);
                let mass_of_objects: Image = ObjectsMeasureMass::run(input, &enumerated_cells, Some(&ignore_colors))?;

                // Layout the objects in a grid
                let grid_width: u8 = grid_pattern.horizontal_cell_count;
                let grid_height: u8 = grid_pattern.vertical_cell_count;
                if grid_width < 1 || grid_height < 1 {
                    return Err(anyhow::anyhow!("Too small grid. Must be 1x1 or bigger"));
                }
                let result_image: Image = ObjectsToGrid::run(
                    &mass_of_objects,
                    &enumerated_cells,
                    grid_width,
                    grid_height,
                    ObjectsToGridMode::MostPopularColor,
                )?;

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

    mod solve_crop_first_object {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let enumerated_objects: &Image = match &pair.input.enumerated_objects {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Missing enumerated_objects for input"));
                    }
                };

                let mask: Image = enumerated_objects.to_mask_where_color_is(1);
                let crop_rect: Rectangle = match mask.bounding_box() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Cannot determine crop_rect for mask"));
                    }
                };

                let result_image = input.crop(crop_rect)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_770000_puzzle_be94b721() {
        let mut instance = solve_crop_first_object::MySolution {};
        let result: String = run_analyze_and_solve("be94b721", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_CROP_FIRST_OBJECT: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,110 ; address of vector[0].EnumeratedObjects
    lps $80
        ; extract the background image
        mov $1,$$83 ; enumerated objects
        mov $2,0 ; the 0th object is the background object
        f21 $1,101250 ; where color is the mask of the 0th object

        ; histogram of the background image
        mov $0,$$81 ; input image
        f21 $0,101231 ; histogram with mask

        ; get pixel at x=0, y=1, this is the most popular color
        mov $1,0
        mov $2,1
        f31 $0,101002  ; get pixel of the most popular color
        mov $10,$0

        ; extract object 1, the biggest object
        mov $0,$$83 ; enumerated objects
        mov $1,1 ; the 1st object is the biggest object
        f21 $0,101250 ; where color is the mask of the 1st object

        ; surround the object with the background-color
        mov $1,$10 ; color for the area to be trimmed
        mov $2,$$81
        f31 $0,102130 ; Pick pixels from color and image

        ; $1 is the color to be trimmed
        f21 $0,101161 ; trim with color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_770001_puzzle_be94b721_loda() {
        let result: String = run_advanced("be94b721", PROGRAM_CROP_FIRST_OBJECT).expect("String");
        assert_eq!(result, "4 1");
    }

    mod solve_cd3c21df {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;

                let background_color: u8;
                {
                    let histogram: Histogram = pair.input.histogram.clone();
                    background_color = match histogram.most_popular_color_disallow_ambiguous() {
                        Some(value) => value,
                        None => {
                            return Err(anyhow::anyhow!("ambiguous what the background color is"));
                        }
                    };
                }

                // Segment the image into objects
                let non_background_mask: Image = input.to_mask_where_color_is(background_color);

                let blank: Image = Image::zero(input.width(), input.height());
                let objects: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity4, &blank, &non_background_mask)?;

                if objects.is_empty() {
                    return Err(anyhow::anyhow!("Expected 1 or more cells"));
                }

                let mut object_histogram = HashMap::<Image, Item>::new();
                for (object_index, object) in objects.iter().enumerate() {
                    let key_with_padding: Image = object.select_from_color_and_image(255, input)?;
                    let key: Image = key_with_padding.trim_color(255)?;

                    if let Some(item) = object_histogram.get_mut(&key) {
                        item.count += 1;
                        item.object_indexes.push(object_index);
                    } else {
                        let item = Item {
                            count: 1,
                            object_indexes: vec![object_index]
                        };
                        object_histogram.insert(key, item);
                    }
                }
                // println!("object_histogram: {:?}", object_histogram);

                let mut found_item: Option<&Item> = None;
                let mut ambiguity_count: usize = 0;
                for (_key, value) in &object_histogram {
                    if value.count == 1 {
                        found_item = Some(value);
                        ambiguity_count += 1;
                    }
                }
                if ambiguity_count > 1 {
                    return Err(anyhow::anyhow!("Found multiple objects that occur once. Ambiguous which one to pick"));
                }
                let item: &Item = match found_item {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Didn't find any object that occur exactly once."));
                    }
                };
                // println!("found: {:?}", item);

                let object_index: usize = match item.object_indexes.first() {
                    Some(value) => *value,
                    None => {
                        return Err(anyhow::anyhow!("There is supposed to be exactly 1 object_index, but got none."));
                    }
                };
                let object_mask: Image = match objects.get(object_index) {
                    Some(value) => value.clone(),
                    None => {
                        return Err(anyhow::anyhow!("Unable to lookup the object_index between the objects."));
                    }
                };

                let crop_rect: Rectangle = match object_mask.bounding_box() {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("Cannot determine crop_rect for mask"));
                    }
                };
                let result_image = input.crop(crop_rect)?;
                Ok(result_image)
            }
        }

        #[derive(Debug)]
        struct Item {
            count: usize,
            object_indexes: Vec<usize>,
        }
    }

    #[test]
    fn test_780000_puzzle_cd3c21df() {
        let mut instance = solve_cd3c21df::MySolution {};
        let result: String = run_analyze_and_solve("cd3c21df", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_45737921 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;

                let enumerated_objects: &Image = match &pair.input.enumerated_objects {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("missing enumerated_object"));
                    }
                };

                let result_image: Image = ReverseColorPopularity::apply_to_objects(&input, enumerated_objects)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_790000_puzzle_45737921() {
        let mut instance = solve_45737921::MySolution {};
        let result: String = run_analyze_and_solve("45737921", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_45737921: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,110 ; address of vector[0].EnumeratedObjects
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; enumerated objects
        f21 $0,102171 ; Takes 2 parameters: Image, EnumeratedObjects. Reorder the color palette, so that the `most popular color` changes place with the `least popular color`
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_790001_puzzle_45737921_loda() {
        let result: String = run_advanced("45737921", PROGRAM_45737921).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_6e82a1ae {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let enumerated_objects: &Image = match &pair.input.enumerated_objects {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("missing enumerated_object"));
                    }
                };
                let oam: ObjectsAndMass = ObjectsAndMass::new(enumerated_objects)?;
                let result_image: Image = oam.group3_small_medium_big(false)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_800000_puzzle_6e82a1ae() {
        let mut instance = solve_6e82a1ae::MySolution {};
        let result: String = run_analyze_and_solve("6e82a1ae", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_6E82A1AE: &'static str = "
    mov $80,$99
    mov $81,110 ; address of vector[0].EnumeratedObjects
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $0,$$81 ; enumerated objects
        mov $1,0 ; reverse = false
        f21 $0,104200 ; Group the objects into 3 bins based on mass: small=1, medium=2, big=3.
        mov $$82,$0
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_800001_puzzle_6e82a1ae_loda() {
        let result: String = run_advanced("6e82a1ae", PROGRAM_6E82A1AE).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_d2abd087 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let enumerated_objects: &Image = match &pair.input.enumerated_objects {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("missing enumerated_object"));
                    }
                };
                let oam: ObjectsAndMass = ObjectsAndMass::new(enumerated_objects)?;
                let result_image: Image = oam.group2_mass_different(6, false)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_810000_puzzle_d2abd087() {
        let mut instance = solve_d2abd087::MySolution {};
        let result: String = run_analyze_and_solve("d2abd087", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_D2ABD087: &'static str = "
    mov $80,$99
    mov $81,110 ; address of vector[0].EnumeratedObjects
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $0,$$81 ; enumerated objects
        mov $1,6 ; objects with 'mass = 6'
        mov $2,0 ; reverse = false
        f31 $0,104201 ; Group the objects into 2 bins based on mass: objects that has the matching mass=1, objects that have a different mass=2.
        mov $$82,$0
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_810000_puzzle_d2abd087_loda() {
        let result: String = run_advanced("d2abd087", PROGRAM_D2ABD087).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_d631b094 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let image: &Image = match &pair.input.predicted_single_color_image {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("missing enumerated_object"));
                    }
                };
                let result_image: Image = image.clone();
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_820000_puzzle_d631b094() {
        let mut instance = solve_d631b094::MySolution {};
        let result: String = run_analyze_and_solve("d631b094", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    mod solve_810b9b61 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let background_color: u8 = input.most_popular_color().expect("color");
                let background_ignore_mask: Image = input.to_mask_where_color_is(background_color);

                // Objects that is not the background
                let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(PixelConnectivity::Connectivity8, &input, &background_ignore_mask)?;
                let mut result_image: Image = Image::zero(input.width(), input.height());
                for object in &object_mask_vec {
                    let rect: Rectangle = object.bounding_box().expect("some");
                    
                    // flood fill at every border pixel around the object
                    let mut object_image: Image = object.crop(rect)?;
                    object_image.border_flood_fill(0, 1, PixelConnectivity::Connectivity4);

                    // if there are unfilled areas, then it's because there is one or more holes
                    let count: u16 = object_image.mask_count_zero();
                    if count > 0 {
                        // object with one or more holes
                        result_image = object.select_from_image_and_color(&result_image, 1)?;
                    } else {
                        // object without any holes
                        result_image = object.select_from_image_and_color(&result_image, 2)?;
                    }
                }

                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_830000_puzzle_810b9b61() {
        let mut instance = solve_810b9b61::MySolution {};
        let result: String = run_analyze_and_solve("810b9b61", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_56ff96f3 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let background_color: u8 = pair.input.most_popular_intersection_color.expect("color");
                let result_image: Image = input.draw_rect_filled_foreach_color(background_color)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_840000_puzzle_56ff96f3() {
        let mut instance = solve_56ff96f3::MySolution {};
        let result: String = run_analyze_and_solve("56ff96f3", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_56FF96F3: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; background color
        f21 $0,102250 ; Draw non-overlapping filled rectangles over the bounding boxes of each color
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_840001_puzzle_56ff96f3_loda() {
        let result: String = run_advanced("56ff96f3", PROGRAM_56FF96F3).expect("String");
        assert_eq!(result, "4 1");
    }

    mod solve_7e0986d6 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let background_color: u8 = pair.input.most_popular_intersection_color.expect("color");
                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");
                let result_image = input.denoise_type4(noise_color, background_color)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_850000_puzzle_7e0986d6() {
        let mut instance = solve_7e0986d6::MySolution {};
        let result: String = run_analyze_and_solve("7e0986d6", &mut instance).expect("String");
        assert_eq!(result, "2 1");
    }

    const PROGRAM_7E0986D6: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    mov $84,115 ; address of vector[0].InputSinglePixelNoiseColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$84 ; noise color
        mov $2,$$83 ; background color
        f31 $0,101093 ; Denoise type4. denoise noisy pixels.
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
        add $84,100
    lpe
    ";

    #[test]
    fn test_850001_puzzle_7e0986d6_loda() {
        let result: String = run_advanced("7e0986d6", PROGRAM_7E0986D6).expect("String");
        assert_eq!(result, "2 1");
    }

    mod solve_ded97339 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");
                let mut result_image: Image = input.clone();
                _ = result_image.draw_line_connecting_two_colors(noise_color, noise_color, noise_color)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_860000_puzzle_ded97339() {
        let mut instance = solve_ded97339::MySolution {};
        let result: String = run_analyze_and_solve("ded97339", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_DED97339: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,115 ; address of vector[0].InputSinglePixelNoiseColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; color0 = noise color
        mov $2,$1 ; color1 = noise color
        mov $3,$1 ; line_color = noise color
        f41 $0,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_860001_puzzle_ded97339_loda() {
        let result: String = run_advanced("ded97339", PROGRAM_DED97339).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_f5b8619d {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");
                let mask: Image = input.to_mask_where_color_is(noise_color);
                let mut result_image: Image = input.clone();
                _ = result_image.draw_line_column_where_mask_is_nonzero(&mask, 42)?;
                result_image = mask.select_from_images(&result_image, input)?;
                result_image = result_image.repeat_by_count(2, 2)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_870000_puzzle_f5b8619d() {
        let mut instance = solve_f5b8619d::MySolution {};
        let result: String = run_analyze_and_solve("f5b8619d", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_F5B8619D: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,115 ; address of vector[0].InputSinglePixelNoiseColor
    lps $80
        mov $0,$$81 ; input image

        mov $8,$0
        mov $9,$$83 ; noise color
        f21 $8,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
        ; $8 is now the mask

        mov $10,$0 ; image
        mov $11,$8 ; mask
        mov $12,42 ; line color
        f31 $10,102222 ; Draw a vertical line if the `mask` contains one or more non-zero pixels.
        ; $10 is now the columns image

        mov $13,$8 ; mask
        mov $14,$10 ; the columns image
        mov $15,$0 ; input image
        f31 $13,102132 ; Pick pixels from two images. When the mask is 0 then pick `image_a`. When the mask is [1..255] then pick from `image_b`.

        mov $0,$13
        mov $1,2
        mov $2,2
        f31 $0,102120 ; repeat image

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_870001_puzzle_f5b8619d_loda() {
        let result: String = run_advanced("f5b8619d", PROGRAM_F5B8619D).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_af902bf9 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");
                let mut a: Image = input.to_mask_where_color_is(noise_color);
                _ = a.draw_line_connecting_two_colors(1, 1, 2)?;
                _ = a.draw_line_connecting_two_colors(2, 2, 3)?;
                let b: Image = a.to_mask_where_color_is(3);
                let c: Image = b.select_from_image_and_color(&input, 255)?;
                Ok(c)
            }
        }
    }

    #[test]
    fn test_880000_puzzle_af902bf9() {
        let mut instance = solve_af902bf9::MySolution {};
        let result: String = run_analyze_and_solve("af902bf9", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_AF902BF9: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,115 ; address of vector[0].InputSinglePixelNoiseColor
    lps $80
        mov $0,$$81 ; input image

        mov $1,$0
        mov $2,$$83 ; noise color
        f21 $1,101250 ; mask where color is noise color

        ; $1 = image
        mov $2,1 ; color0 = 1
        mov $3,1 ; color1 = 1
        mov $4,2 ; line_color = 2
        f41 $1,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.

        ; $1 = image
        mov $2,2 ; color0 = 2
        mov $3,2 ; color1 = 2
        mov $4,3 ; line_color = 3
        f41 $1,102210 ; Draw lines between the `color0` pixels and `color1` pixels when both occur in the same column/row.

        mov $2,3
        f21 $1,101250 ; mask where color is 3

        mov $2,$0
        mov $3,255
        f31 $1,102131 ; Pick pixels from image and color. When the mask is 0 then pick from the image. When the mask is [1..255] then use the `default_color`.

        mov $0,$1

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_880001_puzzle_af902bf9_loda() {
        let result: String = run_advanced("af902bf9", PROGRAM_AF902BF9).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_2c608aff {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let mut found = false;
                for action_label in &task.action_label_set_intersection {
                    match action_label {
                        ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage => {
                            found = true;
                        },
                        _ => {}
                    }
                }
                if !found {
                    return Err(anyhow::anyhow!("OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage not found"));
                }
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;

                let background_color: u8 = input.most_popular_color().expect("color");
                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");

                let mut histogram: Histogram = pair.input.histogram.clone();
                histogram.set_counter_to_zero(background_color);
                histogram.set_counter_to_zero(noise_color);
                if histogram.number_of_counters_greater_than_zero() != 1 {
                    return Err(anyhow::anyhow!("Expected exactly 1 color that is not background_color or noise_color"));
                }
                let sticky_color: u8 = histogram.most_popular_color_disallow_ambiguous().expect("color");

                let mut result_image: Image = input.clone();
                result_image.draw_line_connecting_two_colors(sticky_color, noise_color, noise_color)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_890000_puzzle_2c608aff() {
        let mut instance = solve_2c608aff::MySolution {};
        let result: String = run_analyze_and_solve("2c608aff", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    mod solve_21f83797 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let mut found = false;
                for action_label in &task.action_label_set_intersection {
                    match action_label {
                        ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage => {
                            found = true;
                        },
                        _ => {}
                    }
                }
                if !found {
                    return Err(anyhow::anyhow!("OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage not found"));
                }
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;

                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");                
                let mask: Image = input.to_mask_where_color_is(noise_color);
                
                let single_color_objects: &SingleColorObjects = pair.input.image_meta.single_color_objects.as_ref().expect("some");
                let mut result_image: Image = input.clone();
                for object in &single_color_objects.sparse_vec {
                    if object.color != noise_color {
                        continue;
                    }
                    result_image = result_image.draw_rect_filled(object.bounding_box, 42)?;
                }

                result_image.draw_line_where_mask_is_nonzero(&mask, noise_color)?;

                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_900000_puzzle_21f83797() {
        let mut instance = solve_21f83797::MySolution {};
        let result: String = run_analyze_and_solve("21f83797", &mut instance).expect("String");
        assert_eq!(result, "2 1");
    }

    mod solve_1e0a9b12 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let background_color: u8 = task.input_histogram_intersection.most_popular_color_disallow_ambiguous().expect("color");
                let result_image: Image = input.gravity(background_color, GravityDirection::Down)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_910000_puzzle_1e0a9b12() {
        let mut instance = solve_1e0a9b12::MySolution {};
        let result: String = run_analyze_and_solve("1e0a9b12", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_1E0A9B12: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; most popular color across inputs
        f21 $0,102191 ; Gravity in the down direction
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_910001_puzzle_1e0a9b12_loda() {
        let result: String = run_advanced("1e0a9b12", PROGRAM_1E0A9B12).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_beb8660c {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let background_color: u8 = pair.input.most_popular_intersection_color.expect("color");
                let image_with_gravity: Image = input.gravity(background_color, GravityDirection::Right)?;
                let result_image: Image = image_with_gravity.sort_by_mass(background_color, ImageSortMode::RowsAscending)?;
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_920000_puzzle_beb8660c() {
        let mut instance = solve_beb8660c::MySolution {};
        let result: String = run_analyze_and_solve("beb8660c", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_BEB8660C: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; most popular color across inputs
        f21 $0,102193 ; Gravity in the right direction

        ; $1 holds the most popular color across inputs
        f21 $0,102200 ; Sort rows-ascending by color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_920001_puzzle_beb8660c_loda() {
        let result: String = run_advanced("beb8660c", PROGRAM_BEB8660C).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_d5d6de2d {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let background_color: u8 = task.input_histogram_intersection.most_popular_color_disallow_ambiguous().expect("color");

                let single_color_objects: &SingleColorObjects = pair.input.image_meta.single_color_objects.as_ref().expect("some");

                let mut result_image: Image = Image::zero(input.width(), input.height());
                for object in &single_color_objects.sparse_vec {
                    if object.color == background_color {
                        continue;
                    }
                    let container: &SingleColorObjectClusterContainer = match &object.container4 {
                        Some(value) => value,
                        None => {
                            continue;
                        }
                    };
                    result_image = container.holes_mask_uncropped.clone();
                }

                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_930000_puzzle_d5d6de2d() {
        let mut instance = solve_d5d6de2d::MySolution {};
        let result: String = run_analyze_and_solve("d5d6de2d", &mut instance).expect("String");
        assert_eq!(result, "3 2");
    }

    mod solve_84db8fc4 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let color: u8 = pair.input.removal_color.expect("color");
                let mut result_image: Image = input.clone();
                result_image.border_flood_fill(color, 255, PixelConnectivity::Connectivity4);
                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_940000_puzzle_84db8fc4() {
        let mut instance = solve_84db8fc4::MySolution {};
        let result: String = run_analyze_and_solve("84db8fc4", &mut instance).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_84DB8FC4: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,113 ; address of vector[0].RemovalColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; set source color = removal color
        mov $2,42 ; set destination color to 42
        f31 $0,102180 ; Flood fill at every pixel along the border, connectivity-4.
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_940001_puzzle_84db8fc4_loda() {
        let result: String = run_advanced("84db8fc4", PROGRAM_84DB8FC4).expect("String");
        assert_eq!(result, "4 1");
    }

    mod solve_e7639916 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let mut found = false;
                for action_label in &task.action_label_set_intersection {
                    match action_label {
                        ActionLabel::OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage => {
                            found = true;
                        },
                        _ => {}
                    }
                }
                if !found {
                    return Err(anyhow::anyhow!("OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage not found"));
                }
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let noise_color: u8 = pair.input.single_pixel_noise_color.expect("some");

                let single_color_objects: &SingleColorObjects = pair.input.image_meta.single_color_objects.as_ref().expect("some");
                let mut result_image: Image = input.clone();
                for object in &single_color_objects.sparse_vec {
                    if object.color != noise_color {
                        continue;
                    }
                    let rect: Rectangle = object.bounding_box;
                    result_image = result_image.draw_rect_border(rect.min_x(), rect.min_y(), rect.max_x(), rect.max_y(), 42)?;
                    result_image = object.mask.select_from_image_and_color(&result_image, object.color)?;
                }

                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_960000_puzzle_e7639916() {
        let mut instance = solve_e7639916::MySolution {};
        let result: String = run_analyze_and_solve("e7639916", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_e0fb7511 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let mut found = false;
                for action_label in &task.action_label_set_intersection {
                    match action_label {
                        ActionLabel::OutputImageHasSameStructureAsInputImage => {
                            found = true;
                        },
                        _ => {}
                    }
                }
                if !found {
                    return Err(anyhow::anyhow!("OutputImageHasSameStructureAsInputImage not found"));
                }
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;

                let single_color_objects: &SingleColorObjects = pair.input.image_meta.single_color_objects.as_ref().expect("some");

                // Histogram of the isolated pixels
                let mut isolated_pixels: Image = input.count_duplicate_pixels_in_neighbours()?;
                isolated_pixels = isolated_pixels.to_mask_where_color_is(1);
                let histogram: Histogram = input.histogram_with_mask(&isolated_pixels)?;
                // println!("histogram: {:?}", histogram.pairs_descending());
                let noise_color: u8 = histogram.most_popular_color_disallow_ambiguous().expect("color");

                let mut result_image: Image = input.clone();
                for object in &single_color_objects.sparse_vec {
                    if object.color != noise_color {
                        continue;
                    }
                    let container: &SingleColorObjectClusterContainer = match &object.container4 {
                        Some(value) => value,
                        None => {
                            continue;
                        }
                    };
                    let mut enumerated_objects: Image = container.enumerated_clusters_uncropped.clone();
                    for cluster in &container.cluster_vec {
                        if cluster.mass_cluster <= 1 {
                            continue;
                        }
                        if cluster.cluster_id > (u8::MAX as usize) {
                            continue;
                        }
                        let source_color: u8 = cluster.cluster_id as u8;
                        enumerated_objects = enumerated_objects.replace_color(source_color, 0)?;
                    }
                    let mask: Image = enumerated_objects.to_mask_where_color_is_different(0);
                    result_image = mask.select_from_image_and_color(&result_image, 42)?;
                }

                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_970000_puzzle_e0fb7511() {
        let mut instance = solve_e0fb7511::MySolution {};
        let result: String = run_analyze_and_solve("e0fb7511", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    mod solve_62ab2642 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let mut found = false;
                for action_label in &task.action_label_set_intersection {
                    match action_label {
                        ActionLabel::OutputImageHasSameStructureAsInputImage => {
                            found = true;
                        },
                        _ => {}
                    }
                }
                if !found {
                    return Err(anyhow::anyhow!("OutputImageHasSameStructureAsInputImage not found"));
                }
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let background_color: u8 = task.input_histogram_intersection.most_popular_color_disallow_ambiguous().expect("color");
                let single_color_objects: &SingleColorObjects = pair.input.image_meta.single_color_objects.as_ref().expect("some");

                let mut result_image: Image = input.clone();
                for object in &single_color_objects.sparse_vec {
                    if object.color != background_color {
                        continue;
                    }
                    let container: &SingleColorObjectClusterContainer = match &object.container4 {
                        Some(value) => value,
                        None => {
                            continue;
                        }
                    };
                    let oam: ObjectsAndMass = ObjectsAndMass::new(&container.enumerated_clusters_uncropped)?;
                    let enumerated_by_mass: Image = oam.group3_small_medium_big(false)?;
                    result_image = result_image.mix(&enumerated_by_mass, MixMode::Plus)?;
                }

                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_980000_puzzle_62ab2642() {
        let mut instance = solve_62ab2642::MySolution {};
        let result: String = run_analyze_and_solve("62ab2642", &mut instance).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_3906DE3D: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $1,$$83 ; most popular color across inputs
        f21 $0,102190 ; Gravity in the up direction
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_990000_puzzle_3906de3d_loda() {
        let result: String = run_advanced("3906de3d", PROGRAM_3906DE3D).expect("String");
        assert_eq!(result, "3 1");
    }

    #[test]
    fn test_1000000_puzzle_75b8110e_loda() {
        let result: String = run_advanced("75b8110e", PROGRAM_EA9794B1).expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_CCE03E0D: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        ; tile_width
        mov $2,$0
        f11 $2,101000 ; Get width of image
    
        ; tile_height
        mov $3,$0
        f11 $3,101001 ; Get height of image
    
        ; tile
        mov $7,$20 ; color
        mov $6,$3 ; height
        mov $5,$2 ; width
        f31 $5,101010 ; Create new image with size (x, y) and filled with color z
    
        ; mask
        mov $10,$0 ; image
        mov $11,2 ; color
        f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
    
        mov $11,$0 ; tile0
        mov $12,$5 ; tile1
        f31 $10,102110 ; Create a big composition of tiles.
    
        mov $0,$10
    
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1010000_puzzle_cce03e0d_loda() {
        let result: String = run_advanced("cce03e0d", PROGRAM_CCE03E0D).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_8D5021E8: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $0,$$81 ; input image
        mov $1,$0

        f11 $0,101190 ; flip x
        f21 $0,101030 ; hstack

        mov $1,$0
        f11 $0,101191 ; flip y
        mov $2,$0
        f31 $0,101040 ; vstack
    
        mov $$82,$0
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_1020000_puzzle_8d5021e8_loda() {
        let result: String = run_advanced("8d5021e8", PROGRAM_8D5021E8).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_6A11F6DA: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $20,$$83 ; most popular color across inputs

        mov $10,$$81 ; input image
        mov $11,0 ; no spacing
        f23 $10,102261 ; split into 3 rows
        ; $10..$12 are the 3 columns

        mov $0,$20 ; transparent color
        mov $1,$11 ; layer 0 lowest layer
        mov $2,$10 ; layer 1
        mov $3,$12 ; layer 2 top
        f41 $0,101152 ; Z-stack images: Overlay multiple images using a transparency color
              
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1030000_puzzle_6a11f6da_loda() {
        let result: String = run_advanced("6a11f6da", PROGRAM_6A11F6DA).expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_6F473927: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        ; determine if the image needs to be flipped
        mov $8,$0
        mov $9,1
        f21 $8,101222 ; get N left columns
        mov $9,$20 ; most popular color
        f21 $8,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
        f11 $8,101243 ; number of zeroes in image
        mod $8,2
        ; $8 is 1 when the input image has its content on the right side, and needs flipping. Otherwise it's 0.

        mov $9,$8

        ; flip the input image so it's content is on the right side
        lps $8
            f11 $0,101190 ; flip x
        lpe

        mov $1,$0
        f11 $1,101190 ; flip x
        f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
        f21 $0,101030 ; hstack

        ; restore the x axis
        lps $9
            f11 $0,101190 ; flip x
        lpe
        
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1040000_puzzle_6f473927_loda() {
        let result: String = run_advanced("6f473927", PROGRAM_6F473927).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_C48954C1: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $0,$$81 ; input image

        mov $1,1
        mov $2,1
        mov $3,1
        mov $4,1
        f51 $0,102122 ; Make a big image by repeating the current image and doing flip x, flip y, flip xy.
        
        mov $$82,$0
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_1050000_puzzle_c48954c1_loda() {
        let result: String = run_advanced("c48954c1", PROGRAM_C48954C1).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_281123B4: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        mov $1,1 ; spacing is 1 pixel
        f24 $0,102260 ; split into 4 columns
        ; $0..$3 are the 4 columns

        mov $10,$20 ; transparent color
        mov $11,$1 ; layer 0 lowest layer
        mov $12,$0 ; layer 1
        mov $13,$3 ; layer 2
        mov $14,$2 ; layer 3 top
        f51 $10,101152 ; Z-stack images: Overlay multiple images using a transparency color
      
        mov $0,$10
        
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1060000_puzzle_281123b4_loda() {
        let result: String = run_advanced("281123b4", PROGRAM_281123B4).expect("String");
        assert_eq!(result, "6 1");
    }

    const PROGRAM_3D31C5B3: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $20,$$83 ; most popular color across inputs

        mov $10,$$81 ; input image
        mov $11,0 ; no spacing
        f24 $10,102261 ; split into 4 rows
        ; $10..$13 are the 4 rows

        mov $0,$20 ; transparent color
        mov $1,$12 ; layer 0 lowest layer
        mov $2,$13 ; layer 1
        mov $3,$11 ; layer 2
        mov $4,$10 ; layer 3 top
        f51 $0,101152 ; Z-stack images: Overlay multiple images using a transparency color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1070000_puzzle_3d31c5b3_loda() {
        let result: String = run_advanced("3d31c5b3", PROGRAM_3D31C5B3).expect("String");
        assert_eq!(result, "6 1");
    }

    const PROGRAM_E99362F0: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $20,$$83 ; most popular color across inputs
        mov $21,1 ; pixel spacing = 1

        mov $10,$$81 ; input image
        mov $11,$21 ; spacing
        f22 $10,102261 ; split into 2 rows
        ; $10..$11 are the 2 rows

        mov $15,$10
        mov $16,$21 ; spacing
        f22 $15,102260 ; split into 2 columns
        ; $15..$16 are the 2 columns

        mov $17,$11
        mov $18,$21 ; spacing
        f22 $17,102260 ; split into 2 columns
        ; $17..$18 are the 2 columns

        ; $15 = cell top left
        ; $16 = cell top right
        ; $17 = cell bottom left
        ; $18 = cell bottom right

        mov $0,$20 ; transparent color
        mov $1,$17 ; layer 0 lowest layer
        mov $2,$16 ; layer 1
        mov $3,$15 ; layer 2
        mov $4,$18 ; layer 3 top
        f51 $0,101152 ; Z-stack images: Overlay multiple images using a transparency color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1080000_puzzle_e99362f0_loda() {
        let result: String = run_advanced("e99362f0", PROGRAM_E99362F0).expect("String");
        assert_eq!(result, "6 1");
    }

    const PROGRAM_BC4146BD: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $0,$$81 ; input image
        mov $1,0
        mov $2,0
        mov $3,0
        mov $4,4 ; grow right 4 times
        f51 $0,102122 ; repeat symmetry
        mov $$82,$0
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_1090000_puzzle_bc4146bd_loda() {
        let result: String = run_advanced("bc4146bd", PROGRAM_BC4146BD).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_8E2EDD66: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        ; tile_width
        mov $2,$0
        f11 $2,101000 ; Get width of image
    
        ; tile_height
        mov $3,$0
        f11 $3,101001 ; Get height of image
    
        ; tile0
        mov $7,$20 ; color
        mov $6,$3 ; height
        mov $5,$2 ; width
        f31 $5,101010 ; Create new image with size (x, y) and filled with color z
    
        ; mask
        mov $10,$0 ; image
        mov $11,$20 ; color
        f21 $10,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
    
        ; tile1
        f11 $0,102170 ; Reorder the color palette, so that the `most popular color` changes place with the `least popular color`

        mov $11,$5 ; tile0
        mov $12,$0 ; tile1
        f31 $10,102110 ; Create a big composition of tiles.
    
        mov $0,$10
    
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1100000_puzzle_8e2edd66_loda() {
        let result: String = run_advanced("8e2edd66", PROGRAM_8E2EDD66).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_A59B95C0: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $0,$$81 ; input image

        mov $1,$0
        f11 $1,101240 ; Number of unique colors in image.

        mov $2,$1
        f31 $0,102120 ; Repeat image by the number of unique colors in image.
    
        mov $$82,$0
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_1110000_puzzle_a59b95c0_loda() {
        let result: String = run_advanced("a59b95c0", PROGRAM_A59B95C0).expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_0692E18C: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        ; tile_width
        mov $2,$0
        f11 $2,101000 ; Get width of image
    
        ; tile_height
        mov $3,$0
        f11 $3,101001 ; Get height of image
    
        ; tile0
        mov $7,$20 ; color
        mov $6,$3 ; height
        mov $5,$2 ; width
        f31 $5,101010 ; Create new image with size (x, y) and filled with color z
    
        ; mask
        mov $10,$0 ; image
        mov $11,$20 ; color
        f21 $10,101251 ; Convert to a mask image by converting `color` to 0 and converting anything else to to 1.
    
        ; tile1
        f11 $0,102170 ; Reorder the color palette, so that the `most popular color` changes place with the `least popular color`

        mov $11,$5 ; tile0
        mov $12,$0 ; tile1
        f31 $10,102110 ; Create a big composition of tiles.
    
        mov $0,$10
    
        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1120000_puzzle_0692e18c_loda() {
        let result: String = run_advanced("0692e18c", PROGRAM_0692E18C).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_48131B3C: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $0,$$81 ; input image
        f11 $0,102170 ; Reorder the color palette, so that the `most popular color` changes place with the `least popular color`

        mov $1,2
        mov $2,2
        f31 $0,102120 ; Repeat image
    
        mov $$82,$0
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_1130000_puzzle_48131b3c_loda() {
        let result: String = run_advanced("48131b3c", PROGRAM_48131B3C).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_D037B0A7: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        ; Construct ignore_mask based on the most popular color
        mov $1,$0
        mov $2,$20
        f21 $1,101250 ; Convert to a mask image by converting `color` to 1 and converting anything else to to 0.
        ; $1 is the pixels to be ignored

        ; determine the nearest color in the direction 'up'
        mov $3,$0
        mov $4,$1 ; ignore mask
        mov $5,$20 ; color_when_there_is_no_neighbour
        f31 $3,102060 ; color of nearest neighbour pixel 'up'
        ; $3 is an image of the nearest color in the direction 'up'
    
        ; combine images based on the ignore mask
        mov $6,$1
        mov $7,$0
        mov $8,$3
        f31 $6,102132 ; Pick pixels from two images. When the mask is 0 then pick `image_a`. When the mask is [1..255] then pick from `image_b`.
        ; $6 is a combination of the original image and the nearest color in the direction 'up'
        mov $0,$6

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1140000_puzzle_d037b0a7_loda() {
        let result: String = run_advanced("d037b0a7", PROGRAM_D037B0A7).expect("String");
        assert_eq!(result, "3 1");
    }

    const PROGRAM_EA9794B1: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs
        mov $21,0 ; pixel spacing = 0

        mov $10,$$81 ; input image
        mov $11,$21 ; spacing
        f22 $10,102261 ; split into 2 rows
        ; $10..$11 are the 2 rows

        mov $15,$10
        mov $16,$21 ; spacing
        f22 $15,102260 ; split into 2 columns
        ; $15..$16 are the 2 columns

        mov $17,$11
        mov $18,$21 ; spacing
        f22 $17,102260 ; split into 2 columns
        ; $17..$18 are the 2 columns

        ; $15 = cell top left
        ; $16 = cell top right
        ; $17 = cell bottom left
        ; $18 = cell bottom right

        mov $0,$20 ; transparent color
        mov $1,$15 ; layer 0 lowest layer
        mov $2,$18 ; layer 1
        mov $3,$17 ; layer 2
        mov $4,$16 ; layer 3 top
        f51 $0,101152 ; Z-stack images: Overlay multiple images using a transparency color

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1150000_puzzle_ea9794b1_loda() {
        let result: String = run_advanced("ea9794b1", PROGRAM_EA9794B1).expect("String");
        assert_eq!(result, "6 1");
    }

    const PROGRAM_23581191: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $0,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        mov $1,$0
        mov $2,$20
        f21 $1,101251 ; where color is different than the most popular color

        mov $1,$1 ; mask
        mov $2,255 ; overlap color
        f31 $0,102223 ; Shoot out lines in all directions where mask is non-zero. Preserving the color.

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1160000_puzzle_23581191_loda() {
        let result: String = run_advanced("23581191", PROGRAM_23581191).expect("String");
        assert_eq!(result, "2 1");
    }

    const PROGRAM_BA26E723: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $10,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        mov $15,$10
        mov $16,$20
        f21 $15,101250 ; where color is the most popular color
        ; $15 = mask where the pixels have the same value as the most popular color

        mov $0,$10
        f11 $0,101000 ; width of image
        
        mov $1,$10
        f11 $1,101001 ; height of image

        ; $0 = width
        ; $1 = height
        mov $2,1 ; color0
        mov $3,1 ; count0
        mov $4,0 ; color1
        mov $5,2 ; count1
        f61 $0,101260 ; Alternating columns with two colors

        mov $1,$15
        f21 $0,101255 ; boolean AND
        ; $0 = intersection of the most popular color pixels with the alternating columns

        mov $1,$10
        mov $2,255
        f31 $0,102131 ; Pick pixels from image and color. When the mask is 0 then pick from the image. When the mask is [1..255] then use the `default_color`.

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1170000_puzzle_ba26e723_loda() {
        let result: String = run_advanced("ba26e723", PROGRAM_BA26E723).expect("String");
        assert_eq!(result, "5 1");
    }

    const PROGRAM_D406998B: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    mov $83,114 ; address of vector[0].InputMostPopularColor
    lps $80
        mov $10,$$81 ; input image
        mov $20,$$83 ; most popular color across inputs

        mov $15,$10
        mov $16,$20
        f21 $15,101251 ; where color is different than the most popular color
        ; $15 = mask

        mov $0,$10
        f11 $0,101000 ; width of image
        
        mov $1,$10
        f11 $1,101001 ; height of image

        ; $0 = width
        ; $1 = height
        mov $2,1 ; color0
        mov $3,1 ; count0
        mov $4,0 ; color1
        mov $5,1 ; count1
        f61 $0,101260 ; Alternating columns with two colors
        ; $0 = stripes that starts from the left edge
        
        f11 $0,101190 ; flip x
        ; $0 = stripes that starts from the right edge

        mov $1,$15
        f21 $0,101255 ; boolean AND
        ; $0 = intersection of the mask with the alternating columns

        mov $1,$10
        mov $2,255
        f31 $0,102131 ; Pick pixels from image and color. When the mask is 0 then pick from the image. When the mask is [1..255] then use the `default_color`.

        mov $$82,$0
        add $81,100
        add $82,100
        add $83,100
    lpe
    ";

    #[test]
    fn test_1180000_puzzle_d406998b_loda() {
        let result: String = run_advanced("d406998b", PROGRAM_D406998B).expect("String");
        assert_eq!(result, "4 1");
    }

    const PROGRAM_E9AFCF9A: &'static str = "
    mov $80,$99
    mov $81,100 ; address of vector[0].InputImage
    mov $82,102 ; address of vector[0].ComputedOutputImage
    lps $80
        mov $10,$$81 ; input image
        f11 $10,102206 ; sort columns by pixel value

        mov $11,$10
        f11 $11,101191 ; flip y

        mov $0,$10
        f11 $0,101000 ; width of image
        
        mov $1,$10
        f11 $1,101001 ; height of image

        ; $0 = width
        ; $1 = height
        mov $2,1 ; color0
        mov $3,1 ; count0
        mov $4,0 ; color1
        mov $5,1 ; count1
        f61 $0,101260 ; Alternating columns with two colors
        ; $0 = stripes that starts from the left edge

        mov $1,$10
        mov $2,$11
        f31 $0,102132 ; Pick pixels from two images. When the mask is 0 then pick `image_a`. When the mask is [1..255] then pick from `image_b`.

        mov $$82,$0
        add $81,100
        add $82,100
    lpe
    ";

    #[test]
    fn test_1190000_puzzle_e9afcf9a_loda() {
        let result: String = run_advanced("e9afcf9a", PROGRAM_E9AFCF9A).expect("String");
        assert_eq!(result, "2 1");
    }

    mod solve_6a1e5592 {
        use super::*;

        pub struct MySolution;
    
        impl AnalyzeAndSolve for MySolution {
            fn solve(&self, data: &SolutionSimpleData, task: &arc_work_model::Task) -> anyhow::Result<Image> {
                let pair: &arc_work_model::Pair = &task.pairs[data.index];
                let input: &Image = &pair.input.image;
                let single_color_objects: &SingleColorObjects = pair.input.image_meta.single_color_objects.as_ref().expect("some");

                let mut found_enumerated_objects: Option<Image> = None;
                for object in &single_color_objects.sparse_vec {
                    if object.color != Color::Grey as u8 {
                        continue;
                    }
                    let container: &SingleColorObjectClusterContainer = match &object.container4 {
                        Some(value) => value,
                        None => {
                            continue;
                        }
                    };
                    found_enumerated_objects = Some(container.enumerated_clusters_uncropped.clone());
                    break;
                }

                let enumerated_objects: Image = match found_enumerated_objects {
                    Some(value) => value,
                    None => {
                        return Err(anyhow::anyhow!("No enumerated objects found"));
                    }
                };

                let solid_mask: Image = input.to_mask_where_color_is(Color::Red as u8);

                let enumerated_objects_at_new_position: Image = ObjectsAndGravity::gravity(&enumerated_objects, &solid_mask, ObjectsAndGravityDirection::GravityUp)?;
                let mut result_image = enumerated_objects_at_new_position.to_mask_where_color_is_different(0);
                result_image = solid_mask.select_from_images(&result_image, &input)?;

                Ok(result_image)
            }
        }
    }

    #[test]
    fn test_1200000_puzzle_6a1e5592() {
        let mut instance = solve_6a1e5592::MySolution {};
        let result: String = run_analyze_and_solve("6a1e5592", &mut instance).expect("String");
        assert_eq!(result, "2 1");
    }
}
