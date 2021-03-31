pub struct ProgramSerializer {
    indentation: usize,
    rows: Vec<String>,
}

impl ProgramSerializer {
    pub fn new() -> Self {
        Self {
            indentation: 0,
            rows: vec!(),
        }
    }

    pub fn indent_increment(&mut self) {
        self.indentation += 1;
    }

    pub fn indent_decrement(&mut self) {
        assert!(self.indentation > 0);
        self.indentation -= 1;
    }

    pub fn append<S>(&mut self, content: S) where S: Into<String> {
        const indent_size: usize = 2;
        let prefix: String = " ".repeat(self.indentation * indent_size);
        let row = format!("{}{}", prefix, content.into());
        self.rows.push(row);
    }

    pub fn to_string(&self) -> String {
        self.rows.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000() {
        let mut ps = ProgramSerializer::new();
        ps.append("a");
        ps.indent_increment();
        ps.append("b");
        ps.indent_increment();
        ps.append("c");
        ps.indent_decrement();
        ps.append("d");
        ps.indent_decrement();
        ps.append("e");
        let expected = "a\n  b\n    c\n  d\ne";
        assert_eq!(ps.to_string(), expected);
    }
}
