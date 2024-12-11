pub trait ProgramSerializerContext {
    fn sequence_name_for_oeis_id(&self, oeis_id: u64) -> Option<String>;
}
