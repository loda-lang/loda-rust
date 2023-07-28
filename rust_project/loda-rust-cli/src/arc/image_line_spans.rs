use super::Image;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct SpanItem {
    color: u8,
    x: u8,
    y: u8,
    length: u8,
}

#[derive(Debug)]
struct LineSpan {
    items: Vec<SpanItem>,
}

impl LineSpan {
    fn scan(image: &Image) -> anyhow::Result<Self> {
        let mut items: Vec<SpanItem> = Vec::new();
        for y in 0..image.height() {
            // Run length encoding
            let mut found_color: u8 = image.get(0, y as i32).unwrap_or(255);
            let mut found_x: u8 = 0;
            let mut found_length: u8 = 1;
            for x in 1..image.width() {
                let color: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                if color == found_color {
                    found_length += 1;
                    continue;
                }
                items.push(SpanItem { color: found_color, x: found_x, y, length: found_length });
                // Save data for next span
                found_x = x;
                found_length = 1;
                found_color = color;
            }
            if found_length > 0 {
                items.push(SpanItem { color: found_color, x: found_x, y, length: found_length });
            }
        }
        let instance = Self {
            items
        };
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_line_spans() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 0, 1, 1, 1,
            0, 7, 0, 1, 1,
            0, 0, 7, 0, 0,
            0, 0, 0, 7, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual = LineSpan::scan(&input).expect("ok");

        // Assert
        let mut expected_items = Vec::<SpanItem>::new();
        // y=0
        expected_items.push(SpanItem { color: 7, x: 0, y: 0, length: 1 });
        expected_items.push(SpanItem { color: 0, x: 1, y: 0, length: 1 });
        expected_items.push(SpanItem { color: 1, x: 2, y: 0, length: 3 });
        // y=1
        expected_items.push(SpanItem { color: 0, x: 0, y: 1, length: 1 });
        expected_items.push(SpanItem { color: 7, x: 1, y: 1, length: 1 });
        expected_items.push(SpanItem { color: 0, x: 2, y: 1, length: 1 });
        expected_items.push(SpanItem { color: 1, x: 3, y: 1, length: 2 });
        // y=2
        expected_items.push(SpanItem { color: 0, x: 0, y: 2, length: 2 });
        expected_items.push(SpanItem { color: 7, x: 2, y: 2, length: 1 });
        expected_items.push(SpanItem { color: 0, x: 3, y: 2, length: 2 });
        // y=3
        expected_items.push(SpanItem { color: 0, x: 0, y: 3, length: 3 });
        expected_items.push(SpanItem { color: 7, x: 3, y: 3, length: 1 });
        expected_items.push(SpanItem { color: 0, x: 4, y: 3, length: 1 });
        assert_eq!(actual.items, expected_items);
    }
}
