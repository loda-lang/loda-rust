#[derive(Debug)]
pub enum ProgramId {
    ProgramWithoutId,
    ProgramOEIS(u64),
}
