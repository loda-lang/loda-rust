pub struct FunnelConfig;

impl FunnelConfig {
    // The OEIS 'stripped' file has sequences with variable length.
    // The average sequence length is 38 terms.
    // I rounded that up to 40, and made a funnel with 4 bloomfilters.
    // The 1st bloomfilter checks 10 terms (0..9).
    // The 2nd bloomfilter checks 20 terms (0..19).
    // The 3rd bloomfilter checks 30 terms (0..29).
    // The 4th bloomfilter checks 40 terms (0..39).
    // Thus this constant exist.
    pub const TERM_COUNT: usize = 40;

    // The OEIS 'stripped' file has half of its sequences with fewer than 38 terms.
    // Adjust this parameter, in order to add these short sequences to the bloomfilter.
    pub const MINIMUM_NUMBER_OF_REQUIRED_TERMS: usize = 10;

    // As of june 2022, the OEIS database contains around 360k sequences.
    // So an approx count of 400k and there should be room for the near future.
    pub const APPROX_BLOOM_ITEMS_COUNT: usize = 400000;
}
