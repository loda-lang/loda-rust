use num_bigint::{BigInt, ToBigInt};

pub type BigIntVec = Vec<BigInt>;

pub fn bigintvec_to_string(items: &BigIntVec) -> String {
    let strings: Vec<String> = items.iter().map(|item| {
        item.to_string()
    }).collect();
    strings.join(",")
}

#[allow(dead_code)]
pub fn i64vec_to_bigintvec(values: Vec<i64>) -> BigIntVec {
    let mut bigintvec: BigIntVec = vec!();
    for value in values {
        let value_bigint: BigInt = value.to_bigint().unwrap();
        bigintvec.push(value_bigint);
    }
    bigintvec
}

#[allow(dead_code)]
pub fn is_bigintvec_equal(left: &BigIntVec, right: &BigIntVec) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for i in 0..left.len() {
        if left[i] != right[i] {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bigintvec_to_string_process(values: Vec<i64>) -> String {
        let bigintvec: BigIntVec = i64vec_to_bigintvec(values);
        bigintvec_to_string(&bigintvec)
    }

    #[test]
    fn test_10000_bigintvec_to_string() {
        assert_eq!(bigintvec_to_string_process(vec![]), "");
        assert_eq!(bigintvec_to_string_process(vec![42]), "42");
        assert_eq!(bigintvec_to_string_process(vec![1, 2, 3]), "1,2,3");
        assert_eq!(bigintvec_to_string_process(vec![0, -11, 11]), "0,-11,11");
    }

    fn is_equal(left: Vec<i64>, right: Vec<i64>) -> bool {
        let bigintvec_left: BigIntVec = i64vec_to_bigintvec(left);
        let bigintvec_right: BigIntVec = i64vec_to_bigintvec(right);
        is_bigintvec_equal(&bigintvec_left, &bigintvec_right)
    }

    #[test]
    fn test_20000_is_equal() {
        assert_eq!(is_equal(vec![], vec![]), true);
        assert_eq!(is_equal(vec![1, -1, 2, -2], vec![1, -1, 2, -2]), true);
        assert_eq!(is_equal(vec![1], vec![]), false);
        assert_eq!(is_equal(vec![1, 2, 3], vec![3, 2, 1]), false);
    }
}
