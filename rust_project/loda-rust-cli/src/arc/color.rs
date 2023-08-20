#[allow(dead_code)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Grey = 5,
    Fuchsia = 6,
    Orange = 7,
    Teal = 8,
    Brown = 9,

    CannotCompute = 254,
}

impl Color {
    /// Returns a (red, green, blue) tuple encoded as a 24bit value.
    #[allow(dead_code)]
    pub fn rgb(symbol: u8) -> u32 {
        if symbol == Self::Black as u8 { return 0; }
        if symbol == Self::Blue as u8 { return 0x0074D9; }
        if symbol == Self::Red as u8 { return 0xFF4136; }
        if symbol == Self::Green as u8 { return 0x2ECC40; }
        if symbol == Self::Yellow as u8 { return 0xFFDC00; }
        if symbol == Self::Grey as u8 { return 0xAAAAAA; }
        if symbol == Self::Fuchsia as u8 { return 0xF012BE; }
        if symbol == Self::Orange as u8 { return 0xFF851B; }
        if symbol == Self::Teal as u8 { return 0x7FDBFF; }
        if symbol == Self::Brown as u8 { return 0x870C25; }
        0xffffff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_rgb() {
        assert_eq!(Color::rgb(0), 0);
        assert_eq!(Color::rgb(1), 0x0074D9);
        assert_eq!(Color::rgb(2), 0xFF4136);
        assert_eq!(Color::rgb(3), 0x2ECC40);
        assert_eq!(Color::rgb(4), 0xFFDC00);
        assert_eq!(Color::rgb(5), 0xAAAAAA);
        assert_eq!(Color::rgb(6), 0xF012BE);
        assert_eq!(Color::rgb(7), 0xFF851B);
        assert_eq!(Color::rgb(8), 0x7FDBFF);
        assert_eq!(Color::rgb(9), 0x870C25);
        assert_eq!(Color::rgb(255), 0xffffff);
    }
}
