struct TextColumn {
    rows: Vec<String>,
    max_len: usize,
}

impl TextColumn {
    fn find_max_len(rows: &Vec<String>) -> usize {
        let mut found: usize = 0;
        for row in rows {
            found = usize::max(row.len(), found);
        }
        found
    }

    fn pad_to_row_count(&mut self, row_count: usize) {
        while self.rows.len() < row_count {
            self.rows.push("".to_string());
        }
    }

    fn pad_to_width(rows: &Vec<String>, column_count: usize) -> Vec<String> {
        let mut padded_rows = Vec::<String>::new();
        for row in rows {
            let mut s = row.clone();
            while s.len() < column_count {
                s += " ";
            }
            padded_rows.push(s);
        }
        padded_rows
    }
}

pub struct StackStrings;

impl StackStrings {
    pub fn hstack(strings: Vec<String>, separator: &str) -> String {
        let mut columns = Vec::<TextColumn>::new();
        for s in &strings {
            let rows: Vec<String> = s.split("\n").map(|row| row.to_string()).collect();
            let max_len: usize = TextColumn::find_max_len(&rows);
            let column = TextColumn { rows, max_len };
            columns.push(column);
        }
        let mut max_row_count: usize = 0;
        for col in &columns {
            max_row_count = usize::max(col.rows.len(), max_row_count);
        }
        for col in columns.iter_mut() {
            col.pad_to_row_count(max_row_count);
        }
        for col in columns.iter_mut() {
            col.rows = TextColumn::pad_to_width(&col.rows, col.max_len);
        }

        let mut all_rows = Vec::<String>::new();
        for row_index in 0..max_row_count {
            let mut cells = Vec::<String>::new();
            for col in &columns {
                let s = col.rows[row_index].clone();
                cells.push(s);
            }
            let row_string = cells.join(&separator);
            all_rows.push(row_string);
        }

        all_rows.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_10000_without_separator() {
        // Arrange
        let a = "a\naa\naaa".to_string();
        let b = "bbb\nb\nbb\nb".to_string();
        let c = "c\nc\nc\nc".to_string();

        // Act
        let actual: String = StackStrings::hstack(vec![a, b, c], "");

        // Assert
        let expected = "a  bbbc\naa b  c\naaabb c\n   b  c";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_10001_with_separator() {
        // Arrange
        let a = "a\naa\naaa".to_string();
        let b = "bbb\nb\nbb\nb".to_string();
        let c = "c\nc\nc\nc".to_string();

        // Act
        let actual: String = StackStrings::hstack(vec![a, b, c], "|");

        // Assert
        let expected = "a  |bbb|c\naa |b  |c\naaa|bb |c\n   |b  |c";
        assert_eq!(actual, expected);
    }

}
