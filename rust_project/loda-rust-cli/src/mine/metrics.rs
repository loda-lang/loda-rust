#[derive(Debug)]
pub enum KeyMetricU32 {
    Funnel10TermsPassingBasicCheck,
    Funnel10TermsInBloomfilter,
    Funnel20TermsInBloomfilter,
    Funnel30TermsInBloomfilter,
    Funnel40TermsInBloomfilter,
}
