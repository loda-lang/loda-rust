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
}
