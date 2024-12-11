#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnofficialFunctionId {
    InputOutput { id: u32, inputs: u8, outputs: u8 }
}
