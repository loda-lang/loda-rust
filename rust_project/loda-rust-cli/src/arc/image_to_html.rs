use super::Image;

pub trait ImageToHTML {
    fn to_html(&self) -> String;
}

impl ImageToHTML for Image {
    #[allow(dead_code)]
    fn to_html(&self) -> String {
        if self.is_empty() {
            return "<div class=\"themearc image empty\"><span class=\"themearc image size\">0x0</span></div>".to_string();
        }

        let mut s = "<div class=\"themearc image nonempty\">".to_string();
        s += &format!("<span class=\"size\">{}x{}</span>", self.width(), self.height());
        s += "<span class=\"themearc image rows\">";
        for y in 0..self.height() {
            s += "<span class=\"themearc image row\">";
            for x in 0..self.width() {
                let pixel_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                s += &format!("<span class=\"themearc symbol_{}\">{}</span>", pixel_value, pixel_value);
            }
            s += "</span>";
        }
        s += "</span>";
        s += "</div>";
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_to_html() {
        // Arrange
        let input: Image = Image::empty();

        // Act
        let actual: String = input.to_html();

        // Assert
        let expected = "<div class=\"themearc image empty\"><span class=\"themearc image size\">0x0</span></div>";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_to_html() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 1, 
            2, 3,
        ];
        let input: Image = Image::try_create(2, 2, pixels).expect("image");

        // Act
        let actual: String = input.to_html();

        // Assert
        let expected = "<div class=\"themearc image nonempty\"><span class=\"size\">2x2</span><span class=\"themearc image rows\"><span class=\"themearc image row\"><span class=\"themearc symbol_0\">0</span><span class=\"themearc symbol_1\">1</span></span><span class=\"themearc image row\"><span class=\"themearc symbol_2\">2</span><span class=\"themearc symbol_3\">3</span></span></span></div>";
        assert_eq!(actual, expected);
    }
}
