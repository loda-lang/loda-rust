use super::Image;

pub trait ImageUnicodeFormatting {
    fn to_unicode_string(&self) -> String;
}

impl ImageUnicodeFormatting for Image {
    fn to_unicode_string(&self) -> String {
        if self.is_empty() {
            return "0x0".to_string();
        }

        let mut has_values_out_of_range09 = false;
        for y in 0..self.height() {
            for x in 0..self.width() {
                let pixel_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                if pixel_value > 9 {
                    has_values_out_of_range09 = true;
                    break;
                }
            }
        }

        let mut s = format!("{}x{}", self.width(), self.height());
        for y in 0..self.height() {
            s += "\n";
            for x in 0..self.width() {
                if has_values_out_of_range09 && x > 0 {
                    s += " ";
                }
                let pixel_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                match pixel_value {
                    0 => {
                        s += "⬛"; // black
                    },
                    1 => {
                        s += "🟦"; // blue
                    },
                    2 => {
                        s += "🟥"; // red
                    },
                    3 => {
                        s += "🟩"; // green
                    },
                    4 => {
                        s += "🟨"; // yellow
                    },
                    5 => {
                        s += "⬜"; // gray has no emoji, white is the closest. Alternatively: ⬜ ⚪
                    },
                    6 => {
                        s += "🟪"; // purple
                    },
                    7 => {
                        s += "🟧"; // orange
                    },
                    8 => {
                        s += "🌐"; // light blue has no emoji, alternatives: 🔷 🌐
                    },
                    9 => {
                        s += "🟫"; // dark red has no emoji, brown is the closest
                    },
                    _ => {
                        s += &format!("{:X?}", pixel_value);
                    }
                }
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_to_unicode() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 2, 3, 4,
            5, 6, 7, 8, 9,
        ];
        let input: Image = Image::try_create(5, 2, pixels).expect("image");

        // Act
        let actual: String = input.to_unicode_string();

        // Assert
        let expected = "5x2\n⬛🟦🟥🟩🟨\n⬜🟪🟧🌐🟫";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_out_of_range() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 10, 15, 16,
            255, 240, 0, 19,
        ];
        let input: Image = Image::try_create(4, 2, pixels).expect("image");

        // Act
        let actual: String = input.to_unicode_string();

        // Assert
        let expected = "4x2\n⬛ A F 10\nFF F0 ⬛ 13";
        assert_eq!(actual, expected);
    }
}
