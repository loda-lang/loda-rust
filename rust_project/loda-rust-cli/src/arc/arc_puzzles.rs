#[cfg(test)]
mod tests {
    use crate::arc::{Bitmap, convolution3x3, Padding, Model, GridToBitmap};

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
}
