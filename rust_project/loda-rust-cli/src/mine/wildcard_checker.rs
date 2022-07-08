use loda_rust_core::util::BigIntVec;
use num_bigint::BigInt;

pub trait WildcardChecker {
    /// Beware that a bloomfilter is probabilistic, so false positives happens several times per second.
    /// 
    /// - If the bloomfilter contains the values, it always returns true.
    /// - If the bloomfilter doesn't contains the values, it sporadic returns true.
    /// - If the bloomfilter doesn't contains the values, it most of the time returns false.
    fn bloomfilter_check(&self, bigint_vec_ref: &BigIntVec) -> bool;

    /// Returns the magic value that indicates that it's a wildcard term.
    fn bloomfilter_wildcard_magic_value(&self) -> &BigInt;

    fn check_with_wildcards(&self, bigint_vec_ref: &BigIntVec, minimum_number_of_required_terms: usize) -> Option<usize> {
        let mut bigint_vec: BigIntVec = bigint_vec_ref.clone();
        self.mut_check_with_wildcards(&mut bigint_vec, minimum_number_of_required_terms)
    }

    /// Perform a fuzzy comparison.
    /// Checks if the prefix is contained in the bloomfilter.
    ///
    /// First it checks if all the values are contained in the bloomfilter.
    /// If there isn't a match in the bloomfilter, then it repeats
    /// replacing the last terms with the `WILDCARD_MAGIC_VALUE` until there is a match.
    ///
    /// If there is a match, it returns the number of wildcard terms.
    /// 
    /// If there is no match, it returns `None`.
    ///
    /// The bloomfilter is populated with data from the OEIS 'stripped' file.
    /// The initial terms are always known.
    /// Some sequences may only be 5 terms long where it's yet unknown what the 6th term may be.
    /// Some sequences grows exponentially with so many digits that it makes sense to truncate.
    /// Half of sequences are longer than 38 terms.
    /// The wildcard handling is used for comparing sequences shorter than 40 terms.
    fn mut_check_with_wildcards(&self, bigint_vec: &mut BigIntVec, minimum_number_of_required_terms: usize) -> Option<usize> {
        let len = bigint_vec.len();
        if len < minimum_number_of_required_terms {
            return None;
        }
        if self.bloomfilter_check(&bigint_vec) {
            return Some(0);
        }
        let number_of_wildcards: usize = len - minimum_number_of_required_terms + 1;
        for i in 1..number_of_wildcards {
            bigint_vec[len - i] = self.bloomfilter_wildcard_magic_value().clone();
            if self.bloomfilter_check(&bigint_vec) {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loda_rust_core::util::{BigIntVecToString, BigIntVecFromI64, IsBigIntVecEqual};
    use num_traits::Zero;

    struct MockCheckerImpl {
        bigint_vec: BigIntVec,
        bloomfilter_wildcard_magic_value: BigInt,
    }

    impl MockCheckerImpl {
        fn new_with_wildcard_value(bigint_vec: BigIntVec, bloomfilter_wildcard_magic_value: BigInt) -> Self {
            Self {
                bigint_vec: bigint_vec,
                bloomfilter_wildcard_magic_value: bloomfilter_wildcard_magic_value
            }
        }

        fn new(bigint_vec: BigIntVec) -> Self {
            Self::new_with_wildcard_value(bigint_vec, BigInt::zero())
        }
    }

    impl WildcardChecker for MockCheckerImpl {
        fn bloomfilter_check(&self, bigint_vec_ref: &BigIntVec) -> bool {
            self.bigint_vec.is_bigintvec_equal(bigint_vec_ref)
        }

        fn bloomfilter_wildcard_magic_value(&self) -> &BigInt {
            &self.bloomfilter_wildcard_magic_value
        }
    }

    fn bigints(values: &[i64]) -> BigIntVec {
        BigIntVec::from_i64array(values)
    }
    
    #[test]
    fn test_10000_bloomfilter_check() {
        let checker = MockCheckerImpl::new(bigints(&[1,2,3,4]));
        assert_eq!(checker.bloomfilter_check(&bigints(&[1,2,3,4])), true);
        assert_eq!(checker.bloomfilter_check(&bigints(&[4,3,2,1])), false);
    }
    
    #[test]
    fn test_20000_check_with_wildcards_none() {
        let checker = MockCheckerImpl::new(bigints(&[1,2,3,4]));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4]), 0), Some(0));
        assert_eq!(checker.check_with_wildcards(&bigints(&[4,3,2,1]), 0), None);
        assert_eq!(checker.check_with_wildcards(&bigints(&[0,0,0,0]), 0), None);
    }

    #[test]
    fn test_20001_check_with_wildcards_multiple() {
        let checker = MockCheckerImpl::new(bigints(&[1,2,3,4,0,0,0]));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4,0,0,0]), 0), Some(0));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4,5,6,7]), 0), Some(3));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4,5,6,9]), 0), Some(3));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4,5,9,9]), 0), Some(3));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4,9,9,9]), 0), Some(3));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,9,9,9,9]), 0), None);
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,9,9,9,9,9]), 0), None);
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,9,9,9,9,9,9]), 0), None);
        assert_eq!(checker.check_with_wildcards(&bigints(&[9,9,9,9,9,9,9]), 0), None);
    }

    #[test]
    fn test_20002_check_with_wildcards_fewer_than_minium_required_terms() {
        let checker = MockCheckerImpl::new(bigints(&[1,2,3,4]));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4]), 4), Some(0));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3]), 4), None);
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2]), 4), None);
        assert_eq!(checker.check_with_wildcards(&bigints(&[1]), 4), None);
        assert_eq!(checker.check_with_wildcards(&bigints(&[]), 4), None);
    }

    #[test]
    fn test_20003_check_with_wildcards_minium_required_terms1() {
        let checker = MockCheckerImpl::new(bigints(&[1,1,1,1,0,0,0]));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,1,1,1,0,0,0]), 5), Some(0));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,1,1,1,0,0,9]), 5), Some(1));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,1,1,1,0,9,9]), 5), Some(2));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,1,1,1,9,9,9]), 5), None);
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,1,1,9,9,9,9]), 5), None);
    }

    #[test]
    fn test_20004_check_with_wildcards_minium_required_terms2() {
        let checker = MockCheckerImpl::new(bigints(&[1,2,3,4,5,6,7,9,10,0]));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4,5,6,7,9,10,0]), 10), Some(0));
        assert_eq!(checker.check_with_wildcards(&bigints(&[1,2,3,4,5,6,7,9,10,12]), 10), None);
    }

    #[test]
    fn test_30000_mut_check_with_wildcards() {
        // Arrange
        let checker = MockCheckerImpl::new(bigints(&[2,3,5,7,0,0,0]));
        let mut values: BigIntVec = bigints(&[2,3,5,7,11,13,17]);

        // Act
        assert_eq!(checker.mut_check_with_wildcards(&mut values, 0), Some(3));

        // Assert
        assert_eq!(values.to_compact_comma_string(), "2,3,5,7,0,0,0");
    }
}
