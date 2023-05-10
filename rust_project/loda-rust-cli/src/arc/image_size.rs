#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ImageSize {
    pub width: u8, 
    pub height: u8,
}

impl ImageSize {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self { width: 0, height: 0 }
    }
}
