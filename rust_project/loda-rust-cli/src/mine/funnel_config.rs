pub struct FunnelConfig;

impl FunnelConfig {
    /// The OEIS 'stripped' file has sequences with variable length.
    /// The average sequence length is 38 terms.
    /// 
    /// I rounded that up to 40, and made a funnel with 4 bloomfilters.
    /// - The 1st bloomfilter checks 10 terms (0..9).
    /// - The 2nd bloomfilter checks 20 terms (0..19).
    /// - The 3rd bloomfilter checks 30 terms (0..29).
    /// - The 4th bloomfilter checks 40 terms (0..39).
    /// 
    /// A candidate program must satisfy all the bloomfilters.
    /// 
    /// The sooner a candidate program can be rejected, 
    /// the sooner it's possible to mutate and try again.
    pub const TERM_COUNT: usize = 40;

    /// The OEIS 'stripped' file has half of its sequences with fewer than 38 terms.
    /// Adjust this parameter, in order to add these short sequences to the bloomfilter.
    pub const MINIMUM_NUMBER_OF_REQUIRED_TERMS: usize = 10;

    /// As of july 2022, the OEIS database contains around 360k sequences.
    /// So an approx count of 400k and there should be room for the near future.
    pub const BLOOMFILTER_CAPACITY: usize = 400000;

    /// The 'false-positive' rate is a percentage between `0.0001` and `1.0`.
    /// 
    /// This constant impacts how many hash operations that the bloomfilter is performing.
    /// 
    /// Picking a too low value, and the bloomfilter will have to do many hash operations.
    /// This harms performance.
    /// 
    /// Picking a too high value, and the bloomfilter has only a few hash operations to do.
    /// However there are lots of false positives.
    /// 
    /// There seems to be sweet spot around `1%`, with few false positives, but low performance.
    /// 
    /// Adjust this parameter, in order to tweak performace.
    pub const BLOOMFILTER_FALSE_POSITIVE_RATE: f64 = 0.01;

    /// Magic value to indicate that it's a wildcard term in the bloomfilter.
    /// 
    /// The rate of false-positives is impacted by this magic value.
    /// Initially I had picked 0 as the magic value, 
    /// however I learned the hard way that 0 is the most popular term
    /// in the OEIS 'stripped' file. This caused LOTS of programs to be generated,
    /// most of them being false-positives.
    /// 
    /// To decide on a better magic value, I made a histogram of the terms 
    /// being used in the OEIS 'stripped' file. Only considering the values 
    /// in the range -400 and +400.
    /// 
    /// The value "-67" only occurs 85 times, and "-86" occurs 61 times,
    /// so these may be good choices for a magic value.
    /// 
    /// It frequently happens that there is a collision with the magic value and 
    /// the actual term value in OEIS. This is not a problem, since this yields more 
    /// false positives. The magic value doesn't harm the ability to check if a value 
    /// is contained in the OEIS 'stipped' file.
    pub const WILDCARD_MAGIC_VALUE: i32 = -86;
}
