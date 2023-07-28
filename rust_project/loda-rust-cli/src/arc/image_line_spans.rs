use super::{Histogram, Image, ImageHistogram, ImageMask};

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

    /// Run-length encode an image.
    /// https://en.wikipedia.org/wiki/Run-length_encoding
    /// 
    /// I have experimented with GPT4 and it's able to decode this RLE format.
    /// I had to add a space character between the lines to make it work.
    /// I had to use B (Black) and W (White), so it's the same as on wikipedia.
    fn serialize_rle(image: &Image) -> anyhow::Result<String> {
        let histogram: Histogram = image.histogram_all();
        let mut s = String::new();
        for (_count, color) in histogram.pairs_ordered_by_color() {
            s += &format!("{};", color);
            s += &Self::serialize_rle_color(image, color)?;
            s += "\n";
        }
        Ok(s.trim().to_string())
    }

    fn serialize_rle_color(image: &Image, color: u8) -> anyhow::Result<String> {
        let mask: Image = image.to_mask_where_color_is(color);
        let line_span = LineSpan::scan(&mask)?;
        let mut s = String::new();
        for item in &line_span.items {
            if item.x == 0 && item.y > 0 {
                s += " ";
            }
            s += &format!("{}", item.length);
            if item.color == 0 {
                s += "W";
            } else {
                s += "B";
            }
        }
        Ok(s)
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

    #[test]
    fn test_20000_run_length_encoding() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 0, 1, 1, 1,
            0, 7, 0, 1, 1,
            0, 0, 7, 0, 0,
            0, 0, 0, 7, 0,
        ];
        let input: Image = Image::try_create(5, 4, pixels).expect("image");

        // Act
        let actual: String = LineSpan::serialize_rle(&input).expect("ok");

        // Assert
        let expected = "0;1W1B3W 1B1W1B2W 2B1W2B 3B1W1B\n1;2W3B 3W2B 5W 5W\n7;1B4W 1W1B3W 2W1B2W 3W1B1W";
        assert_eq!(actual, expected);
    }
}
