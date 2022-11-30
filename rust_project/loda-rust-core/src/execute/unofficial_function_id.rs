#[derive(Clone, Copy, Debug)]
pub enum UnofficialFunctionId {
    InputOutput { id: u32, inputs: u8, outputs: u8 }
}
