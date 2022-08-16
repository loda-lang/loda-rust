use super::ProgramSerializerContext;

struct DummyProgramSerializerContext {
}

impl DummyProgramSerializerContext {
    fn new() -> Self {
        Self {
        }
    }
}

impl ProgramSerializerContext for DummyProgramSerializerContext {
    fn sequence_name_for_oeis_id(&self, oeis_id: u64) -> Option<String> {
        if oeis_id == 40 {
            return Some(String::from("The prime numbers"));
        }
        None
    }
}

pub struct ProgramSerializer {
    context: Box<dyn ProgramSerializerContext>,
    indentation: usize,
    rows: Vec<String>,
}

impl ProgramSerializer {
    pub fn new() -> Self {
        Self {
            context: Box::new(DummyProgramSerializerContext::new()),
            indentation: 0,
            rows: vec!(),
        }
    }

    pub fn context(&self) -> &dyn ProgramSerializerContext {
        self.context.as_ref()
    }

    pub fn set_context(&mut self, new_context: Box<dyn ProgramSerializerContext>) {
        self.context = new_context;
    }

    pub fn indent_increment(&mut self) {
        self.indentation += 1;
    }

    pub fn indent_decrement(&mut self) {
        assert!(self.indentation > 0);
        self.indentation -= 1;
    }

    pub fn append_raw<S>(&mut self, content: S) where S: Into<String> {
        const INDENT_SIZE: usize = 2;
        let prefix: String = " ".repeat(self.indentation * INDENT_SIZE);
        let row = format!("{}{}", prefix, content.into());
        self.rows.push(row);
    }

    pub fn append_comment<S>(&mut self, content: S) where S: Into<String> {
        self.append_raw(format!("; {}", content.into()));
    }

    pub fn append_empty_line(&mut self) {
        self.rows.push(String::new());
    }

    pub fn to_string(&self) -> String {
        self.rows.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_append_raw() {
        let mut ps = ProgramSerializer::new();
        ps.append_raw("a");
        ps.indent_increment();
        ps.append_raw("b");
        ps.indent_increment();
        ps.append_raw("c");
        ps.indent_decrement();
        ps.append_raw("d");
        ps.indent_decrement();
        ps.append_raw("e");
        let expected = "a\n  b\n    c\n  d\ne";
        assert_eq!(ps.to_string(), expected);
    }

    #[test]
    fn test_10001_append_comment() {
        let mut ps = ProgramSerializer::new();
        ps.append_comment("root");
        ps.indent_increment();
        ps.append_comment("level 1");
        ps.indent_increment();
        ps.append_comment("level 2");
        ps.indent_decrement();
        ps.append_comment("level 1");
        ps.indent_decrement();
        ps.append_comment("root");
        let expected = "; root\n  ; level 1\n    ; level 2\n  ; level 1\n; root";
        assert_eq!(ps.to_string(), expected);
    }

    #[test]
    fn test_10002_append_empty_line() {
        let mut ps = ProgramSerializer::new();
        ps.append_raw("a");
        ps.indent_increment();
        ps.append_raw("b");
        ps.append_empty_line();
        ps.append_raw("c");
        ps.indent_decrement();
        ps.append_raw("d");
        ps.append_empty_line();
        ps.append_empty_line();
        ps.append_raw("e");
        let expected = "a\n  b\n\n  c\nd\n\n\ne";
        assert_eq!(ps.to_string(), expected);
    }
}
