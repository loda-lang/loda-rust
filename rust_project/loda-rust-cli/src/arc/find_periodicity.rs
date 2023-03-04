use super::{Image, ImageMask};

pub struct FindPeriodicity {
    period: Option<u8>,
}

impl FindPeriodicity {
    fn measure_without_mask(image: &Image) -> FindPeriodicity {
        let mask = Image::zero(image.width(), image.height());
        Self::measure_with_ignore_mask(image, &mask)
    }

    fn measure_with_ignore_mask(image: &Image, ignore_mask: &Image) -> FindPeriodicity {
        let image_width: u8 = image.width();
        let mut found_count: u8 = 0;
        let mut found_i: u8 = 0;
        // Loop over the rows
        for y in 0..image.height() as i32 {
            // Loop over the candidate offsets
            for i in 1..image_width {
                if i < found_i {
                    // Ignore i's smaller than what has already been found.
                    continue;
                }
                // if found_count >= (image_width - i) {
                //     // From this point on we cannot find more matches than what has already been found.
                //     break;
                // }
                let mut count_same: u8 = 0;
                // Loop over the columns
                for x in 0..image_width as i32 {
                    let x_i = x - (i as i32);
                    if x_i < 0 {
                        continue;
                    }
                    let mask_i: u8 = ignore_mask.get(x_i, y).unwrap_or(255);
                    if mask_i > 0 {
                        continue;
                    }
                    let color: u8 = image.get(x, y).unwrap_or(255);
                    let color_i: u8 = image.get(x_i, y).unwrap_or(255);
                    if color == color_i {
                        count_same += 1;
                    }
                }
                // Determine if the candidate is better and if so, then save it
                if i > found_i && count_same >= found_count {
                    found_count = count_same;
                    found_i = i;
                    println!("new optima. i: {} count: {} y: {}", found_i, found_count, y);
                }
            }
        }
        println!("found i: {} count: {}", found_i, found_count);
        let period: Option<u8>;
        if found_count > 0 {
            period = Some(found_i);
        } else {
            period = None;
        }
        FindPeriodicity {
            period,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

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

        let input: Image = Image::try_create(image_width, image_height, pixels)?;

        let instance: FindPeriodicity = FindPeriodicity::measure_without_mask(&input);
        Ok(instance)
    }

    #[test]
    fn test_10000_find_periodicity_without_mask_1row_period1() {
        let pixels = vec![1, 1, 1, 1, 1, 1, 1];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(1));
    }

    #[test]
    fn test_10001_find_periodicity_without_mask_1row_period2() {
        let pixels = vec![1, 2, 1, 2, 1, 2, 1];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_10002_find_periodicity_without_mask_1row_period3_variant1() {
        let pixels = vec![1, 2, 3, 1, 2, 3, 1];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_10003_find_periodicity_without_mask_1row_period3_variant2() {
        let pixels = vec![1, 2, 2, 1, 2, 2, 1];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_10004_find_periodicity_without_mask_1row_period4_variant1() {
        let pixels = vec![1, 2, 3, 4, 1, 2, 3];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_10004_find_periodicity_without_mask_1row_period4_variant2() {
        let pixels = vec![1, 2, 1, 4, 1, 2, 1];
        let instance = find_periodicity1(1, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_20000_find_periodicity_without_mask_2rows_period1() {
        let pixels = vec![
            1, 1, 1, 1,
            1, 1, 1, 1
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(1));
    }

    #[test]
    fn test_20001_find_periodicity_without_mask_2rows_period2_variant1() {
        let pixels = vec![
            1, 2, 1, 2,
            1, 2, 1, 2,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_20001_find_periodicity_without_mask_2rows_period2_variant2() {
        let pixels = vec![
            1, 2, 1, 2,
            2, 1, 2, 1,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_20001_find_periodicity_without_mask_2rows_period2_variant3() {
        let pixels = vec![
            1, 2, 1, 2,
            3, 3, 3, 3,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    // #[test]
    fn xtest_20001_find_periodicity_without_mask_2rows_period2_variant4() {
        let pixels = vec![
            3, 3, 3, 3,
            1, 2, 1, 2,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(2));
    }

    #[test]
    fn test_20002_find_periodicity_without_mask_2rows_period3_variant1() {
        let pixels = vec![
            1, 2, 3, 1, 2,
            3, 2, 1, 3, 2,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_20002_find_periodicity_without_mask_2rows_period3_variant2() {
        let pixels = vec![
            1, 2, 3, 1, 2,
            1, 1, 1, 1, 1,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    // #[test]
    fn xtest_20002_find_periodicity_without_mask_2rows_period3_variant3() {
        let pixels = vec![
            1, 1, 1, 1, 1,
            1, 2, 3, 1, 2,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(3));
    }

    #[test]
    fn test_20003_find_periodicity_without_mask_2rows_period4_variant1() {
        let pixels = vec![
            1, 2, 3, 4, 1, 2, 3, 4, 1,
            1, 2, 1, 2, 1, 2, 1, 2, 1,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    // #[test]
    fn xtest_20003_find_periodicity_without_mask_2rows_period4_variant2() {
        let pixels = vec![
            1, 2, 1, 2, 1, 2, 1, 2, 1,
            1, 2, 3, 4, 1, 2, 3, 4, 1,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    #[test]
    fn test_20003_find_periodicity_without_mask_2rows_period4_variant3() {
        let pixels = vec![
            1, 2, 3, 4, 1, 2, 3, 4, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }

    // #[test]
    fn xtest_20003_find_periodicity_without_mask_2rows_period4_variant4() {
        let pixels = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 2, 3, 4, 1, 2, 3, 4, 1,
        ];
        let instance = find_periodicity1(2, pixels).expect("ok");
        assert_eq!(instance.period, Some(4));
    }
}
