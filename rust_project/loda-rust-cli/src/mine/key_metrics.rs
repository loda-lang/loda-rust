#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeyMetricU32 {
    NumberOfMinerLoopIterations,
    PreventedFlooding,
    NumberOfFailedMutations,
    NumberOfProgramsThatCannotParse,
    NumberOfProgramsWithoutOutput,
    NumberOfProgramsThatCannotRun,
    NumberOfFailedGenomeLoads,
}
