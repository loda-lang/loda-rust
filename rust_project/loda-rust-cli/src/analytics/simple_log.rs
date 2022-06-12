use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::LineWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};

struct State {
    line_writer: LineWriter<File>
}

#[derive(Clone)]
pub struct SimpleLog {
    state: Arc<Mutex<State>>,
}

impl SimpleLog {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::create(path)?;
        let line_writer: LineWriter<File> = LineWriter::new(file);
        let state = State {
            line_writer: line_writer
        };
        let instance = Self {
            state: Arc::new(Mutex::new(state))
        };
        Ok(instance)
    }

    pub fn print(&self, content: &String) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.lock().unwrap();
        state.line_writer.write_all(content.as_bytes())?;
        state.line_writer.flush()?;
        Ok(())
    }
}
