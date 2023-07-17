use super::Image;

pub trait ImageToHTML {
    fn to_html(&self) -> String;
    fn to_html_with_prefix(&self, prefix: &str) -> String;
    fn to_interactive_html(&self, prefix: &str, context: Option<String>) -> String;
}

impl ImageToHTML for Image {
    fn to_html(&self) -> String {
        self.to_html_with_prefix("")
    }

    fn to_html_with_prefix(&self, prefix: &str) -> String {
        if self.is_empty() {
            return format!("<div class=\"themearc image empty\"><span class=\"themearc image size\">{}0x0</span></div>", prefix);
        }

        let mut s = "<div class=\"themearc image nonempty\">".to_string();
        s += &format!("<span class=\"size\">{}{}x{}</span>", prefix, self.width(), self.height());
        s += "<div class=\"themearc image rows-container\">";
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
        s += "</div>";
        s
    }

    fn to_interactive_html(&self, prefix: &str, context: Option<String>) -> String {
        if self.is_empty() {
            return format!("<div class=\"themearc image empty\"><span class=\"themearc image size\">{}0x0</span></div>", prefix);
        }

        let resolved_context: String = context.unwrap_or("".to_string());

        let mut s = "<div class=\"themearc image nonempty interactive-image\">".to_string();
        s += &format!("<span class=\"size\">{}{}x{}</span>", prefix, self.width(), self.height());
        s += "<div class=\"themearc image\" ";
        s += &format!("data-image=\"{}\">", resolved_context);
        s += "<span class=\"json-data\">[";
        for y in 0..self.height() {
            if y > 0 {
                s += ", ";
            }
            s += "[";
            for x in 0..self.width() {
                if x > 0 {
                    s += ", ";
                }
                let pixel_value: u8 = self.get(x as i32, y as i32).unwrap_or(255);
                s += &format!("{}", pixel_value);
            }
            s += "]";
        }
        s += "]</span></div>";

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
        let expected = "<div class=\"themearc image nonempty\"><span class=\"size\">2x2</span><div class=\"themearc image rows-container\"><span class=\"themearc image rows\"><span class=\"themearc image row\"><span class=\"themearc symbol_0\">0</span><span class=\"themearc symbol_1\">1</span></span><span class=\"themearc image row\"><span class=\"themearc symbol_2\">2</span><span class=\"themearc symbol_3\">3</span></span></span></div></div>";
        assert_eq!(actual, expected);
    }
}
