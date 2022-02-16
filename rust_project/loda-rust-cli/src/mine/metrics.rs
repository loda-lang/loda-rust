#[derive(Debug, Eq, Hash, PartialEq)]
pub enum KeyMetricU32 {
    NumberOfMinerLoopIterations,
    Funnel10TermsPassingBasicCheck,
    Funnel10TermsInBloomfilter,
    Funnel20TermsInBloomfilter,
    Funnel30TermsInBloomfilter,
    Funnel40TermsInBloomfilter,
}
