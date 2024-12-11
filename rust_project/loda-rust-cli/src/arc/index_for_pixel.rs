pub fn index_for_pixel(x: i32, y: i32, width: u8, height: u8) -> Option<usize> {
    if (x < 0) || (x > (u8::MAX as i32)) || ((x as u8) >= width) {
        return None;
    }
    if (y < 0) || (y > (u8::MAX as i32)) || ((y as u8) >= height) {
        return None;
    }
    Some((y as usize) * (width as usize) + (x as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_index_for_pixel() {
        // top left
        assert_eq!(index_for_pixel(0, 0, 100, 200), Some(0));

        // top right
        assert_eq!(index_for_pixel(99, 0, 100, 200), Some(99));

        // bottom left
        assert_eq!(index_for_pixel(0, 199, 100, 200), Some(199 * 100));

        // bottom right
        assert_eq!(index_for_pixel(99, 199, 100, 200), Some(199 * 100 + 99));

        // center
        assert_eq!(index_for_pixel(5, 5, 11, 11), Some(5 * 11 + 5));

        // out of bounds
        assert_eq!(index_for_pixel(-1, 0, 11, 11), None);
        assert_eq!(index_for_pixel(0, -1, 11, 11), None);
        assert_eq!(index_for_pixel(12, 0, 11, 11), None);
        assert_eq!(index_for_pixel(0, 12, 11, 11), None);
    }
}
