use super::{Image, ImageRotate};
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive};
use num_integer::Integer;

const FIND_PERIODICITY_VERBOSE: bool = false;

#[allow(dead_code)]
pub trait ImagePeriodicity {
    fn horizontal_periodicity(&self, ignore_mask: &Image) -> anyhow::Result<Option<u8>>;
    fn vertical_periodicity(&self, ignore_mask: &Image) -> anyhow::Result<Option<u8>>;
}

impl ImagePeriodicity for Image {
    fn horizontal_periodicity(&self, ignore_mask: &Image) -> anyhow::Result<Option<u8>> {
        let instance = FindPeriodicity::measure_with_ignore_mask(&self, ignore_mask)?;
        Ok(instance.period)
    }

    fn vertical_periodicity(&self, ignore_mask: &Image) -> anyhow::Result<Option<u8>> {
        let image: Image = self.rotate_cw()?;
        let ignore_mask: Image = ignore_mask.rotate_cw()?;
        let instance = FindPeriodicity::measure_with_ignore_mask(&image, &ignore_mask)?;
        Ok(instance.period)
    }
}


struct FindPeriodicity {
    period: Option<u8>,
}

impl FindPeriodicity {
    fn measure_with_ignore_mask(image: &Image, ignore_mask: &Image) -> anyhow::Result<FindPeriodicity> {
        if image.width() != ignore_mask.width() || image.height() != ignore_mask.height() {
            return Err(anyhow::anyhow!("Expected same size for 'image' and 'ignore_mask'"));
        }
        if image.is_empty() || image.width() < 2 {
            return Err(anyhow::anyhow!("Image is too small to find pattern"));
        }

        let image_width: u8 = image.width();
        let mut global_found_i = BigUint::one();

        // Loop over the rows
        for y in 0..image.height() as i32 {
            let mut found_i: u8 = 1;

            // Loop over the candidate offsets
            for i in 1..image_width {
                let mut detected_mismatches: bool = false;

                // Loop over the columns
                for x in 0..image_width as i32 {
                    let x_i = x - (i as i32);
                    if x_i < 0 {
                        continue;
                    }
                    let mask: u8 = ignore_mask.get(x, y).unwrap_or(255);
                    if mask > 0 {
                        continue;
                    }
                    let mask_i: u8 = ignore_mask.get(x_i, y).unwrap_or(255);
                    if mask_i > 0 {
                        continue;
                    }
                    let color: u8 = image.get(x, y).unwrap_or(255);
                    let color_i: u8 = image.get(x_i, y).unwrap_or(255);
                    if color != color_i {
                        detected_mismatches = true;
                        break;
                    }
                }
                // Stop when reaching the first match
                if !detected_mismatches {
                    found_i = i;
                    if FIND_PERIODICITY_VERBOSE {
                        println!("row: {} new optima. i: {}", y, found_i);
                    }
                    break;
                }
            }
            if FIND_PERIODICITY_VERBOSE {
                println!("row: {}  i: {}", y, found_i);
            }
            let other = BigUint::from(found_i);
            global_found_i = global_found_i.lcm(&other);
        }
        if FIND_PERIODICITY_VERBOSE {
            println!("found i: {}", global_found_i);
        }
        let period: Option<u8> = global_found_i.to_u8();
        let instance = FindPeriodicity {
            period,
        };
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    /// This function considers all pixels. It doesn't use the ignore mask.
    fn find_periodicity1(image_height: u8, pixels: Vec<u8>) -> anyhow::Result<FindPeriodicity> {
        let image_width_remain_usize: usize = pixels.len() % (image_height as usize);
        if image_width_remain_usize > 0 {
            return Err(anyhow::anyhow!("pixels.len() {} is no divisible by {}", pixels.len(), image_height));
        }
        let image_width_usize: usize = pixels.len() / (image_height as usize);
        if image_width_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("image_width is bigger than max capacity"));
        }
        let image_width: u8 = image_width_usize as u8;

        let image: Image = Image::try_create(image_width, image_height, pixels)?;
        let ignore_mask = Image::zero(image.width(), image.height());
        let instance: FindPeriodicity = FindPeriodicity::measure_with_ignore_mask(&image, &ignore_mask)?;
        Ok(instance)
    }

    /// This function takes an `ignore_pixels` parameter, and uses this to do fuzzy matching of these pixels.
    fn find_periodicity2(image_height: u8, pixels: Vec<u8>, ignore_pixels: Vec<u8>) -> anyhow::Result<FindPeriodicity> {
        if pixels.len() != ignore_pixels.len() {
            return Err(anyhow::anyhow!("Expected same length of 'pixels' and 'ignore_pixels'"));
        }
        let image_width_remain_usize: usize = pixels.len() % (image_height as usize);
        if image_width_remain_usize > 0 {
            return Err(anyhow::anyhow!("pixels.len() {} is no divisible by {}", pixels.len(), image_height));
        }
        let image_width_usize: usize = pixels.len() / (image_height as usize);
        if image_width_usize > (u8::MAX as usize) {
            return Err(anyhow::anyhow!("image_width is bigger than max capacity"));
        }
        let image_width: u8 = image_width_usize as u8;

        let input: Image = Image::try_create(image_width, image_height, pixels)?;
        let ignore_mask: Image = Image::try_create(image_width, image_height, ignore_pixels)?;

        let instance: FindPeriodicity = FindPeriodicity::measure_with_ignore_mask(&input, &ignore_mask)?;
        Ok(instance)
    }

    #[test]
    fn test_10000_find_periodicity_without_mask_1row_period1() {
        let pixels = vec![
            1, 1, 1, 1, 1, 1, 1, // period 1
        ];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(1));
    }

    #[test]
    fn test_10001_find_periodicity_without_mask_1row_period2() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
        ];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_10002_find_periodicity_without_mask_1row_period3_variant1() {
        let pixels = vec![
            1, 2, 3, 1, 2, 3, 1, // period 3
        ];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_10003_find_periodicity_without_mask_1row_period3_variant2() {
        let pixels = vec![
            1, 2, 2, 1, 2, 2, 1, // period 3
        ];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_10004_find_periodicity_without_mask_1row_period4_variant1() {
        let pixels = vec![
            1, 2, 3, 4, 1, 2, 3, // period 4
        ];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_10004_find_periodicity_without_mask_1row_period4_variant2() {
        let pixels = vec![
            1, 2, 1, 4, 1, 2, 1, // period 4
        ];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_20000_find_periodicity_without_mask_2rows_period1() {
        let pixels = vec![
            1, 1, 1, 1, // period 1
            1, 1, 1, 1, // period 1
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(1));
    }

    #[test]
    fn test_20001_find_periodicity_without_mask_2rows_period2_variant1() {
        let pixels = vec![
            1, 2, 1, 2, // period 2
            1, 2, 1, 2, // period 2
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_20001_find_periodicity_without_mask_2rows_period2_variant2() {
        let pixels = vec![
            1, 2, 1, 2, // period 2
            2, 1, 2, 1, // period 2
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_20001_find_periodicity_without_mask_2rows_period2_variant3() {
        let pixels = vec![
            1, 2, 1, 2, // period 2
            3, 3, 3, 3, // period 1
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_20001_find_periodicity_without_mask_2rows_period2_variant4() {
        let pixels = vec![
            3, 3, 3, 3, // period 1
            1, 2, 1, 2, // period 2
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_20002_find_periodicity_without_mask_2rows_period3_variant1() {
        let pixels = vec![
            1, 2, 3, 1, 2, // period 3
            3, 2, 1, 3, 2, // period 3
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_20002_find_periodicity_without_mask_2rows_period3_variant2() {
        let pixels = vec![
            1, 2, 3, 1, 2, // period 3
            1, 1, 1, 1, 1, // period 1
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_20002_find_periodicity_without_mask_2rows_period3_variant3() {
        let pixels = vec![
            1, 1, 1, 1, 1, // period 1
            1, 2, 3, 1, 2, // period 3
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_20003_find_periodicity_without_mask_2rows_period4_variant1() {
        let pixels = vec![
            1, 2, 3, 4, 1, 2, 3, 4, 1, // period 4
            1, 2, 1, 2, 1, 2, 1, 2, 1, // period 2
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_20003_find_periodicity_without_mask_2rows_period4_variant2() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, 2, 1, // period 2
            1, 2, 3, 4, 1, 2, 3, 4, 1, // period 4
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_20003_find_periodicity_without_mask_2rows_period4_variant3() {
        let pixels = vec![
            1, 2, 3, 4, 1, 2, 3, 4, 1, // period 4
            1, 1, 1, 1, 1, 1, 1, 1, 1, // period 1
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_20003_find_periodicity_without_mask_2rows_period4_variant4() {
        let pixels = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, // period 1
            1, 2, 3, 4, 1, 2, 3, 4, 1, // period 4
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_20004_find_periodicity_without_mask_2rows_period5_variant1() {
        let pixels = vec![
            2, 1, 1, 1, 2, 2, 1, 1, 1, 2, // period 5
            1, 1, 2, 2, 5, 1, 1, 2, 2, 5, // period 5
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(5));
    }

    #[test]
    fn test_20005_find_periodicity_without_mask_2rows_period6_variant1() {
        let pixels = vec![
            1, 1, 2, 1, 1, 2, 1, 1, 2, // period 3
            1, 2, 1, 2, 1, 2, 1, 2, 1, // period 2
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(6));
    }

    #[test]
    fn test_20005_find_periodicity_without_mask_2rows_period6_variant2() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, 2, 1, // period 2
            1, 1, 2, 1, 1, 2, 1, 1, 2, // period 3
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(6));
    }

    #[test]
    fn test_20006_find_periodicity_without_mask_2rows_period10_variant1() {
        let pixels = vec![
            1, 1, 1, 1, 2, 1, 1, 1, 1, 2, // period 5
            1, 2, 1, 2, 1, 2, 1, 2, 1, 2, // period 2
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(10));
    }

    #[test]
    fn test_20006_find_periodicity_without_mask_2rows_period10_variant2() {
        let pixels = vec![
            1, 1, 1, 1, 2, 1, 1, 1, 1, 2, 1, // period 5
            1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, // period 2
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(10));
    }

    #[test]
    fn test_30000_find_periodicity_without_mask_3rows_period6_variant1() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, 2, 1, // period 3
            1, 1, 2, 1, 1, 2, 1, 1, 2, // period 2
            1, 1, 1, 1, 1, 1, 1, 1, 1, // period 1
        ];
        let instance = find_periodicity1(3, pixels).expect("ok");
        assert_eq!(instance.period, Some(6));
    }

    #[test]
    fn test_30000_find_periodicity_without_mask_3rows_period6_variant2() {
        let pixels = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, // period 1
            1, 2, 1, 2, 1, 2, 1, 2, 1, // period 2
            1, 1, 2, 1, 1, 2, 1, 1, 2, // period 3
        ];
        let instance = find_periodicity1(3, pixels).expect("ok");
        assert_eq!(instance.period, Some(6));
    }

    #[test]
    fn test_30001_find_periodicity_without_mask_3rows_period30_variant1() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, // period 2
            1, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1, // period 3
            1, 1, 1, 1, 2, 1, 1, 1, 1, 2, 1, // period 5
        ];
        let instance = find_periodicity1(3, pixels).expect("ok");
        assert_eq!(instance.period, Some(30));
    }

    #[test]
    fn test_40000_find_periodicity_with_mask_2rows_period2_masked1_variant1() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 3, 3, 3, 3, 3, 3, // period 1
        ];
        let ignore_pixels = vec![
            0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40000_find_periodicity_with_mask_2rows_period2_masked1_variant2() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 3, 3, 3, 3, 3, 3, // period 1
        ];
        let ignore_pixels = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40000_find_periodicity_with_mask_2rows_period2_masked1_variant3() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 3, 3, 3, 3, 3, 3, // period 1
        ];
        let ignore_pixels = vec![
            0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40000_find_periodicity_with_mask_2rows_period2_masked1_variant4() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 3, 3, 3, 3, 3, 3, // period 1
        ];
        let ignore_pixels = vec![
            1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40001_find_periodicity_with_mask_2rows_period2_masked2_variant1() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 3, 3, 3, 3, 3, 3, // period 1
        ];
        let ignore_pixels = vec![
            0, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40001_find_periodicity_with_mask_2rows_period2_masked2_variant2() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 3, 3, 3, 3, 3, 3, // period 1
        ];
        let ignore_pixels = vec![
            0, 1, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40001_find_periodicity_with_mask_2rows_period2_masked2_variant3() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 3, 3, 3, 3, 3, 3, // period 1
        ];
        let ignore_pixels = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 1,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40002_find_periodicity_with_mask_2rows_period2_masked2_variant1() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 1, 3, 1, 3, 1, 3, // period 2
        ];
        let ignore_pixels = vec![
            0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40002_find_periodicity_with_mask_2rows_period2_masked2_variant2() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 1, 3, 1, 3, 1, 3, // period 2
        ];
        let ignore_pixels = vec![
            0, 0, 0, 1, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_40003_find_periodicity_with_mask_2rows_period3_masked2_variant1() {
        let pixels = vec![
            1, 2, 3, 1, 2, 3, 1, // period 3
            3, 2, 1, 3, 2, 1, 3, // period 3
        ];
        let ignore_pixels = vec![
            0, 0, 0, 1, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_40003_find_periodicity_with_mask_2rows_period3_masked2_variant2() {
        let pixels = vec![
            1, 2, 3, 1, 2, 3, 1, // period 3
            3, 2, 1, 3, 2, 1, 3, // period 3
        ];
        let ignore_pixels = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_40004_find_periodicity_with_mask_2rows_period6_masked2_variant1() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 2, 1, 3, 2, 1, 3, // period 3
        ];
        let ignore_pixels = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(6));
    }

    #[test]
    fn test_40004_find_periodicity_with_mask_2rows_period6_masked2_variant2() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 2, 1, 3, 2, 1, 3, // period 3
        ];
        let ignore_pixels = vec![
            1, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(6));
    }

    #[test]
    fn test_40004_find_periodicity_with_mask_2rows_period6_masked2_variant3() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, // period 2
            3, 2, 1, 3, 2, 1, 3, // period 3
        ];
        let ignore_pixels = vec![
            0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 1,
        ];
        let instance = find_periodicity2(2, pixels, ignore_pixels).expect("ok");
        assert_eq!(instance.period, Some(6));
    }

    #[test]
    fn test_50000_find_horizontal_periodicity() {
        let pixels = vec![
            1, 5, 1, 2, 1, 5, 1, // period 4
            3, 2, 1, 3, 2, 1, 3, // period 3
        ];
        let image: Image = Image::try_create(7, 2, pixels).expect("ok");
        let ignore_pixels = vec![
            0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 1,
        ];
        let ignore_mask: Image = Image::try_create(7, 2, ignore_pixels).expect("ok");
        let period: Option<u8> = image.horizontal_periodicity(&ignore_mask).expect("ok");
        assert_eq!(period, Some(12));
    }

    #[test]
    fn test_60000_find_vertical_periodicity() {
        let pixels = vec![
            // column0 has a period of 4
            // column1 has a period of 3
            1, 3,
            5, 2,
            1, 1,
            2, 3,
            1, 2,
            5, 1, 
            1, 3,
        ];
        let image: Image = Image::try_create(2, 7, pixels).expect("ok");
        let ignore_pixels = vec![
            1, 0, 
            0, 0, 
            0, 0, 
            0, 0, 
            0, 0, 
            0, 0,
            1, 0,
        ];
        let ignore_mask: Image = Image::try_create(2, 7, ignore_pixels).expect("ok");
        let period: Option<u8> = image.vertical_periodicity(&ignore_mask).expect("ok");
        assert_eq!(period, Some(12));
    }
}
