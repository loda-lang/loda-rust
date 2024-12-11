use num_bigint::{BigInt, ToBigInt};

pub type BigIntVec = Vec<BigInt>;

pub trait BigIntVecToString {
    fn to_compact_comma_string(&self) -> String;
}

impl BigIntVecToString for BigIntVec {
    fn to_compact_comma_string(&self) -> String {
        let strings: Vec<String> = self.iter().map(|item| {
            item.to_string()
        }).collect();
        strings.join(",")
    }
}

pub trait BigIntVecFromI64 {
    fn from_i64array(values: &[i64]) -> BigIntVec;
}

impl BigIntVecFromI64 for BigIntVec {
    fn from_i64array(values: &[i64]) -> BigIntVec {
        let mut bigintvec = BigIntVec::with_capacity(values.len());
        for value in values {
            let value_bigint: BigInt = value.to_bigint().unwrap();
            bigintvec.push(value_bigint);
        }
        bigintvec
    }
}

pub trait IsBigIntVecEqual {
    fn is_bigintvec_equal(&self, other: &BigIntVec) -> bool;
}

impl IsBigIntVecEqual for BigIntVec {
    fn is_bigintvec_equal(&self, other: &BigIntVec) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for i in 0..self.len() {
            if self[i] != other[i] {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convert_to_string(values: &[i64]) -> String {
        let bigintvec = BigIntVec::from_i64array(values);
        bigintvec.to_compact_comma_string()
    }

    #[test]
    fn test_10000_convert_to_string() {
        assert_eq!(convert_to_string(&[]), "");
        assert_eq!(convert_to_string(&[42]), "42");
        assert_eq!(convert_to_string(&[1, 2, 3]), "1,2,3");
        assert_eq!(convert_to_string(&[0, -11, 11]), "0,-11,11");
    }

    fn is_equal(left: &[i64], right: &[i64]) -> bool {
        let bigintvec_left = BigIntVec::from_i64array(left);
        let bigintvec_right = BigIntVec::from_i64array(right);
        bigintvec_left.is_bigintvec_equal(&bigintvec_right)
    }

    #[test]
    fn test_20000_is_equal() {
        assert_eq!(is_equal(&[], &[]), true);
        assert_eq!(is_equal(&[1, -1, 2, -2], &[1, -1, 2, -2]), true);
        assert_eq!(is_equal(&[1], &[]), false);
        assert_eq!(is_equal(&[1, 2, 3], &[3, 2, 1]), false);
    }
}
