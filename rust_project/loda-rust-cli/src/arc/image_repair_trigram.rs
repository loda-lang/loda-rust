use super::{Image, ImageMask, ImageNgram, ImagePadding, RecordTrigram, convolution3x3};
use bit_set::BitSet;

#[allow(unused_imports)]
use crate::arc::{HtmlLog, ImageToHTML};

const IMAGE_REPAIR_VERBOSE: bool = false;


pub trait ImageRepairTrigram {
    /// Fix damaged pixels and recreate simple repeating patterns.
    fn repair_trigram_algorithm(&mut self, repair_color: u8) -> anyhow::Result<()>;
}

impl ImageRepairTrigram for Image {
    fn repair_trigram_algorithm(&mut self, repair_color: u8) -> anyhow::Result<()> {
        let repair_mask: Image = self.to_mask_where_color_is(repair_color);

        if IMAGE_REPAIR_VERBOSE {
            println!("repair color: {}", repair_color);
        }

        // Trigrams
        let trigram_x_unfiltered: Vec<RecordTrigram> = self.trigram_x()?;
        let trigram_y_unfiltered: Vec<RecordTrigram> = self.trigram_y()?;
        if IMAGE_REPAIR_VERBOSE {
            // println!("trigram_x_unfiltered: {:?}", trigram_x_unfiltered);
            // println!("trigram_y_unfiltered: {:?}", trigram_y_unfiltered);
            println!("trigram_x_unfiltered.len: {} trigram_y_unfiltered.len: {}", trigram_x_unfiltered.len(), trigram_y_unfiltered.len());
        }

        // Remove trigrams that contains the repair_color
        let trigram_x_refs: Vec<&RecordTrigram> = trigram_x_unfiltered.iter().filter(|&record| {
            record.word0 != repair_color && record.word1 != repair_color && record.word2 != repair_color
        }).collect();
        let trigram_y_refs: Vec<&RecordTrigram> = trigram_y_unfiltered.iter().filter(|&record| {
            record.word0 != repair_color && record.word1 != repair_color && record.word2 != repair_color
        }).collect();
        let trigram_x: Vec<RecordTrigram> = trigram_x_refs.iter().map(|&i| i.clone()).collect();
        let trigram_y: Vec<RecordTrigram> = trigram_y_refs.iter().map(|&i| i.clone()).collect();
        if IMAGE_REPAIR_VERBOSE {
            // println!("trigram_x: {:?}", trigram_x);
            // println!("trigram_y: {:?}", trigram_y);
            println!("trigram_x.len: {} trigram_y.len: {}", trigram_x.len(), trigram_y.len());
        }

        repair_image(self, &repair_mask, &trigram_x, &trigram_y)?;
        Ok(())
    }
}

/// Find damaged pixels that have some pixel data around the center pixel,
/// so that it can be repaired using trigram repair.
/// 
/// The `mask_with_1px_border` must have a 1 pixel border with `color=255`.
/// The pixels inside the mask must have `color=1` if it's damaged and needs repair.
/// The pixels inside the mask must have `color=0` if it's already good.
/// 
/// Set `color=1` when it's a candidates for repair.
/// - A damaged pixel that has data all around it, that's a good candidate for repair.
/// 
/// Set `color=0` when it's not a candidate for repair.
/// - It makes no sense to try to repair a pixel that is already in good condition.
/// - It makes no sense to try to repair a pixel that so damaged that it has insufficient data for repairing it.
/// - A damaged pixel that has 3 damaged pixels around it and 1 neighbour with data, that's also a terrible candidate.
/// - A damaged pixel that has all damaged pixels around it, that cannot be repaired.
fn identify_repairable_pixels(mask_with_1px_border: &Image) -> anyhow::Result<Image> {
    let repair_areas: Image = convolution3x3(&mask_with_1px_border, |bm| {
        let center_color: u8 = bm.get(1, 1).unwrap_or(255);
        if center_color == 0 {
            // The center pixel is not set
            return Ok(0);
        }
        let pairs_horizontal: [(u8,u8); 2] = [
            (0,1), // left
            (2,1), // right
        ];
        let pairs_vertical: [(u8,u8); 2] = [
            (1,0), // top
            (1,2)  // bottom
        ];
        let mut count_horizontal: u8 = 0;
        for (x, y) in pairs_horizontal {
            let color: u8 = bm.get(x as i32, y as i32).unwrap_or(255);
            if color > 0 {
                count_horizontal += 1;
            }
        }
        let mut count_vertical: u8 = 0;
        for (x, y) in pairs_vertical {
            let color: u8 = bm.get(x as i32, y as i32).unwrap_or(255);
            if color > 0 {
                count_vertical += 1;
            }
        }

        match (count_horizontal, count_vertical) {
            (0, 0) => {
                // The pixels up/left/right/down all have valid pixel data
                // The center pixel has yet to be computed.
                return Ok(1);
            },
            (1, 1) => {
                // This is a corner
                return Ok(1);
            },
            (0, 1) => {
                // This is a corner
                return Ok(1);
            },
            (1, 0) => {
                // This is a corner
                return Ok(1);
            },
            _ => {
                // This is not a corner
                return Ok(0);
            }
        }
    })?;
    Ok(repair_areas)
}

/// Returns `true` when the pixel was successfully repaired.
/// 
/// Returns `false` when the pixel couldn't be repaired.
/// 
/// Returns an error in case the coordinate is outside the canvas.
fn repair_pixel(image: &mut Image, x: i32, y: i32, trigram_x: &Vec<RecordTrigram>, trigram_y: &Vec<RecordTrigram>) -> anyhow::Result<bool> {
    if x < 0 || y < 0 || x >= (image.width() as i32) || y >= (image.height() as i32) {
        return Err(anyhow::anyhow!("Unable to repair pixel. The coordinate ({}, {}) is outside image size.", x, y));
    }
    if IMAGE_REPAIR_VERBOSE {
        println!("repair corner: {}, {}", x, y);
    }

    let pixel_up2: u8    = image.get(x, y - 2).unwrap_or(255);
    let pixel_up1: u8    = image.get(x, y - 1).unwrap_or(255);
    let pixel_left2: u8  = image.get(x - 2, y).unwrap_or(255);
    let pixel_left1: u8  = image.get(x - 1, y).unwrap_or(255);
    let pixel_right2: u8 = image.get(x + 2, y).unwrap_or(255);
    let pixel_right1: u8 = image.get(x + 1, y).unwrap_or(255);
    let pixel_down2: u8  = image.get(x, y + 2).unwrap_or(255);
    let pixel_down1: u8  = image.get(x, y + 1).unwrap_or(255);

    let mut bitset_trigram_x = BitSet::with_capacity(256);
    let mut bitset_trigram_y = BitSet::with_capacity(256);
    for candidate in 0..255u8 {
        for record in trigram_x.iter() {
            if record.word0 == pixel_left2 && record.word1 == pixel_left1 && record.word2 == candidate {
                bitset_trigram_x.insert(candidate as usize);
            }
            if record.word0 == candidate && record.word1 == pixel_right1 && record.word2 == pixel_right2 {
                bitset_trigram_x.insert(candidate as usize);
            }
        }
        for record in trigram_y.iter() {
            if record.word0 == pixel_up2 && record.word1 == pixel_up1 && record.word2 == candidate {
                bitset_trigram_y.insert(candidate as usize);
            }
            if record.word0 == candidate && record.word1 == pixel_down1 && record.word2 == pixel_down2 {
                bitset_trigram_y.insert(candidate as usize);
            }
        }
    }
    let mut bitset_trigram = BitSet::with_capacity(256);
    bitset_trigram.clone_from(&bitset_trigram_x);
    bitset_trigram.intersect_with(&bitset_trigram_y);
    if bitset_trigram.len() >= 2 {
        if IMAGE_REPAIR_VERBOSE {
            println!("ambiguous repair color. more than 1 candidate. trigram: {:?}", bitset_trigram);
        }
    }

    let mut found_color: Option<u8> = None;
    for index in bitset_trigram.iter() {
        if index > 255 {
            return Err(anyhow::anyhow!("Integrity error. Encountered bitset index outside of u8 range [0..255]"));
        }
        found_color = Some(index as u8);
        break;
    }
    let set_color: u8 = match found_color {
        Some(value) => value,
        None => {
            if IMAGE_REPAIR_VERBOSE {
                println!("repair ({}, {}) = cannot repair due to insufficient data", x, y);
            }
            // Unable to repair the pixel.
            return Ok(false);
        }
    };

    if IMAGE_REPAIR_VERBOSE {
        println!("repair ({}, {}) = {:?}", x, y, set_color);
    }
    match image.set(x, y, set_color) {
        Some(()) => {
            // We did repair the pixel.
            return Ok(true);
        },
        None => {
            return Err(anyhow::anyhow!("Unable to set pixel inside the result image"));
        }
    }
}

fn repair_image(image: &mut Image, repair_mask: &Image, trigram_x: &Vec<RecordTrigram>, trigram_y: &Vec<RecordTrigram>) -> anyhow::Result<()> {
    if image.width() != repair_mask.width() {
        return Err(anyhow::anyhow!("The width must be the same"));
    }
    if image.height() != repair_mask.height() {
        return Err(anyhow::anyhow!("The height must be the same"));
    }
    if image.is_empty() {
        return Ok(());
    }
    let mut mask_with_1px_border = repair_mask.padding_with_color(1, 255)?;

    for iteration in 0..10 {
        let repairable_pixels: Image = identify_repairable_pixels(&mask_with_1px_border)?;

        if IMAGE_REPAIR_VERBOSE {
            HtmlLog::html(image.to_html());
            HtmlLog::html(mask_with_1px_border.to_html());
            HtmlLog::html(repairable_pixels.to_html());
            println!("iteration#{} repair areas: {:?}", iteration, repairable_pixels);
        }
    
        let mut repair_count: usize = 0;
        for y in 0..repairable_pixels.height() {
            for x in 0..repairable_pixels.width() {
                let pixel_value: u8 = repairable_pixels.get(x as i32, y as i32).unwrap_or(255);
                if pixel_value < 1 {
                    continue;
                }
                let did_repair: bool = repair_pixel(image, x as i32, y as i32, &trigram_x, &trigram_y)?;
                if did_repair {
                    repair_count += 1;

                    // Clear the pixel that just got repaired.
                    // so that we know that we should not attempt to repair it again.
                    _ = mask_with_1px_border.set((x as i32) + 1, (y as i32) + 1, 0);
                }
            }
        }
        if IMAGE_REPAIR_VERBOSE {
            println!("iteration#{} repair_count: {}", iteration, repair_count);
        }
        if repair_count == 0 {
            if IMAGE_REPAIR_VERBOSE {
                println!("repair done. no more pixels to be repaired");
            }
            break;
        }
    }
    if IMAGE_REPAIR_VERBOSE {
        HtmlLog::html(image.to_html());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_repair_trigram_algorithm() {
        // Arrange
        let pixels: Vec<u8> = vec![
            9, 9, 1, 2, 4, 2, 1,
            9, 9, 2, 1, 2, 4, 2,
            1, 2, 4, 2, 9, 2, 4,
            2, 1, 2, 4, 2, 1, 2,
            4, 9, 9, 2, 4, 2, 1,
        ];
        let input: Image = Image::try_create(7, 5, pixels).expect("image");

        // Act
        let mut actual: Image = input.clone();
        actual.repair_trigram_algorithm(9).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            4, 2, 1, 2, 4, 2, 1,
            2, 4, 2, 1, 2, 4, 2,
            1, 2, 4, 2, 1, 2, 4,
            2, 1, 2, 4, 2, 1, 2,
            4, 2, 1, 2, 4, 2, 1,
        ];
        let expected: Image = Image::try_create(7, 5, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_repair_trigram_algorithm() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 2, 2, 5, 2, 2, 5,
            5, 9, 2, 5, 2, 2, 5,
            3, 5, 5, 3, 5, 5, 3,
            5, 2, 2, 5, 2, 9, 9,
            5, 2, 2, 5, 2, 9, 9,
            3, 5, 5, 3, 5, 9, 9,
        ];
        let input: Image = Image::try_create(7, 6, pixels).expect("image");

        // Act
        let mut actual: Image = input.clone();
        actual.repair_trigram_algorithm(9).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            5, 2, 2, 5, 2, 2, 5,
            5, 2, 2, 5, 2, 2, 5,
            3, 5, 5, 3, 5, 5, 3,
            5, 2, 2, 5, 2, 2, 5,
            5, 2, 2, 5, 2, 2, 5,
            3, 5, 5, 3, 5, 5, 3,
        ];
        let expected: Image = Image::try_create(7, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
