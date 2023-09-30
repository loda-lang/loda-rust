use super::Image;

struct GenerateDataset;

impl GenerateDataset {
    /// Convert an Image to a text representation.
    /// 
    /// The text representation is a string with the `0-9a-z` format.
    /// 
    /// The values `0..=35` are converted to `0..=9` and `a..=z`.
    /// 
    /// An error is returned when encountering a value greater than `35`.
    fn image_to_text(image: &Image) -> anyhow::Result<String> {
        let max_value: u16 = 'z' as u16;
        let mut rows = Vec::<String>::new();
        for y in 0..image.height() {
            let mut row = String::new();
            for x in 0..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                if color < 10 {
                    // convert from 0-9 to 0-9
                    row.push_str(color.to_string().as_str());
                } else {
                    // convert from 10-35 to a-z
                    let value: u16 = ('a' as u16) + (color - 10) as u16;
                    if value > max_value {
                        return Err(anyhow::anyhow!("Cannot represent value as 0-9a-z representation. The value {} is greater than {}", value, max_value));
                    }
                    let c: char = match std::char::from_u32(value as u32) {
                        Some(value) => value,
                        None => {
                            return Err(anyhow::anyhow!("Cannot represent value as 0-9a-z representation. The value {} cannot be converted to a Char", value));
                        }
                    };
                    row.push(c);
                }
            }
            rows.push(row);
        }
        let mut result = String::new();
        result += "image='";
        result += &rows.join(",");
        result += "'";
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_image_to_text() {
        let input: Image = Image::color(2, 2, 0);
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='00,00'");
    }

    #[test]
    fn test_10001_image_to_text() {
        let input: Image = Image::color(1, 1, 9);
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='9'");
    }

    #[test]
    fn test_10002_image_to_text() {
        let input: Image = Image::color(1, 1, 10);
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='a'");
    }

    #[test]
    fn test_10003_image_to_text() {
        let input: Image = Image::color(1, 1, 35);
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='z'");
    }

    #[test]
    fn test_10004_image_to_text() {
        let input = Image::try_create(4, 1, vec![0, 9, 10, 35]).expect("ok");
        let actual: String = GenerateDataset::image_to_text(&input).expect("ok");
        assert_eq!(actual, "image='09az'");
    }
}
