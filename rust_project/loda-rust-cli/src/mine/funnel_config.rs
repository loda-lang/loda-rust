pub struct FunnelConfig;

impl FunnelConfig {
    pub const MINIMUM_NUMBER_OF_REQUIRED_TERMS: usize = 10;
    pub const TERM_COUNT: usize = 40;

    // As of june 2022, the OEIS contains around 360k sequences.
    // So an approx count of 400k and there should be room for them all.
    pub const APPROX_BLOOM_ITEMS_COUNT: usize = 400000;
}
