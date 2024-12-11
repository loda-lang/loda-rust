use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::LineWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};


trait SimpleLogState {
    fn print(&mut self, content: &String) -> Result<(), Box<dyn Error>>;
}

struct StateLineWriter {
    line_writer: LineWriter<File>
}

impl SimpleLogState for StateLineWriter {
    fn print(&mut self, content: &String) -> Result<(), Box<dyn Error>> {
        // Append to a log file
        self.line_writer.write_all(content.as_bytes())?;
        self.line_writer.flush()?;
        Ok(())
    }
}

struct StateSink {}

impl SimpleLogState for StateSink {
    fn print(&mut self, _content: &String) -> Result<(), Box<dyn Error>> {
        // Silently ignores printing
        Ok(())
    }
}

#[derive(Clone)]
pub struct SimpleLog {
    state: Arc<Mutex<dyn SimpleLogState>>,
}

impl SimpleLog {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::create(path)?;
        let line_writer: LineWriter<File> = LineWriter::new(file);
        let state = StateLineWriter {
            line_writer: line_writer
        };
        let instance = Self {
            state: Arc::new(Mutex::new(state))
        };
        Ok(instance)
    }

    #[allow(dead_code)]
    pub fn sink() -> Self {
        let state = StateSink {};
        Self {
            state: Arc::new(Mutex::new(state))
        }
    }

    pub fn println<I: AsRef<str>>(&self, message: I) {
        let content: String = message.as_ref().into();
        let content2 = content + "\n";
        self.print(&content2).expect("Unable to print");
    }

    pub fn print(&self, content: &String) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.lock().unwrap();
        state.print(content)?;
        Ok(())
    }
}
