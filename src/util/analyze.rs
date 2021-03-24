use crate::oeis::stripped_sequence::BigIntVec;
use num_bigint::{BigInt, ToBigInt};
use num_traits::Zero;

pub struct Analyze {}

impl Analyze {
    pub fn is_all_the_same_value(terms: &BigIntVec) -> bool {
        if terms.is_empty() {
            return true;
        }
        let first: &BigInt = terms.first().unwrap();
        for term in terms {
            if term != first {
                return false;
            }
        }
        true
    }

    pub fn is_constant_step(terms: &BigIntVec) -> bool {
        let length: usize = terms.len();
        if length < 3 {
            return false;
        }
        let mut first_diff = BigInt::zero();
        for i in 1..length {
            let value0: &BigInt = &terms[i];
            let value1: &BigInt = &terms[i-1];
            let diff: BigInt = value0 - value1;
            if i == 1 {
                first_diff = diff;
            } else {
                if diff != first_diff {
                    return false;
                }
            }
        }
        true
    }

    pub fn count_zero(terms: &BigIntVec) -> usize {
        let mut count = 0;
        for term in terms {
            if term.is_zero() {
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_is_all_the_same_value() {
        fn process(values: Vec<i64>) -> bool {
            let mut terms: BigIntVec = vec!();
            for value in values {
                terms.push(value.to_bigint().unwrap());
            }
            Analyze::is_all_the_same_value(&terms)
        }

        assert_eq!(process(vec!()), true);
        assert_eq!(process(vec![42]), true);
        assert_eq!(process(vec![123, 123, 123]), true);
        assert_eq!(process(vec![123, 666, 123]), false);
        assert_eq!(process(vec![0, 1, 2]), false);
    }

    #[test]
    fn test_10001_is_constant_step() {
        fn process(values: Vec<i64>) -> bool {
            let mut terms: BigIntVec = vec!();
            for value in values {
                terms.push(value.to_bigint().unwrap());
            }
            Analyze::is_constant_step(&terms)
        }

        assert_eq!(process(vec!()), false);
        assert_eq!(process(vec![42]), false);
        assert_eq!(process(vec![42, 42]), false);
        assert_eq!(process(vec![42, 42, 42]), true);
        assert_eq!(process(vec![1, 2, 3]), true);
        assert_eq!(process(vec![3, 2, 1]), true);
        assert_eq!(process(vec![123, 666, 123]), false);
        assert_eq!(process(vec![-20, -10, 0, 10, 20]), true);
    }

    #[test]
    fn test_10002_count_zero() {
        fn process(values: Vec<i64>) -> usize {
            let mut terms: BigIntVec = vec!();
            for value in values {
                terms.push(value.to_bigint().unwrap());
            }
            Analyze::count_zero(&terms)
        }

        assert_eq!(process(vec!()), 0);
        assert_eq!(process(vec![42]), 0);
        assert_eq!(process(vec![0]), 1);
        assert_eq!(process(vec![1, 0, 1, 0]), 2);
        assert_eq!(process(vec![0, 0, 0, 0, 0]), 5);
    }
}
