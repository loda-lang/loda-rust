#[cfg(test)]
mod tests {
    use crate::arc::{Bitmap, BitmapToNumber, NumberToBitmap};
    use num_bigint::BigUint;

    #[test]
    fn test_10000_empty() {
        // Arrange
        let bm = Bitmap::empty();

        // Act
        let number: BigUint = bm.to_number().expect("biguint");
        let bm_copy: Bitmap = number.to_bitmap().expect("bitmap");

        // Assert
        assert_eq!(bm_copy.width(), 0);
        assert_eq!(bm_copy.height(), 0);
        assert_eq!(bm_copy.pixels().is_empty(), true);
    }

    #[test]
    fn test_10001_zeroes() {
        // Arrange
        let bm = Bitmap::zeroes(2, 3);

        // Act
        let number: BigUint = bm.to_number().expect("biguint");
        let bm_copy: Bitmap = number.to_bitmap().expect("bitmap");

        // Assert
        assert_eq!(bm_copy.width(), 2);
        assert_eq!(bm_copy.height(), 3);
        let mut sum: usize = 0;
        for pixel in bm_copy.pixels() {
            sum += *pixel as usize;
        }
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_10002_rectangle() {
        // Arrange
        let mut bm = Bitmap::zeroes(10, 20);
        bm.set(0, 0, 1).expect("ok");
        bm.set(9, 0, 2).expect("ok");
        bm.set(0, 19, 3).expect("ok");
        bm.set(9, 19, 4).expect("ok");

        // Act
        let number: BigUint = bm.to_number().expect("biguint");
        let bm_copy: Bitmap = number.to_bitmap().expect("bitmap");

        // Assert
        assert_eq!(bm_copy.width(), 10);
        assert_eq!(bm_copy.height(), 20);
        assert_eq!(bm_copy.get(0, 0), Some(1));
        assert_eq!(bm_copy.get(9, 0), Some(2));
        assert_eq!(bm_copy.get(0, 19), Some(3));
        assert_eq!(bm_copy.get(9, 19), Some(4));
    }

    #[test]
    fn test_10003_square() {
        // Arrange
        let mut bm = Bitmap::zeroes(11, 11);
        bm.set(5, 5, 255).expect("ok");

        // Act
        let number: BigUint = bm.to_number().expect("biguint");
        let bm_copy: Bitmap = number.to_bitmap().expect("bitmap");

        // Assert
        assert_eq!(bm_copy.width(), 11);
        assert_eq!(bm_copy.height(), 11);
        assert_eq!(bm_copy.get(5, 5), Some(255));
    }
}
