use num_bigint::BigInt;

pub type BigIntVec = Vec<BigInt>;

pub fn bigintvec_to_string(items: &BigIntVec) -> String {
    let strings: Vec<String> = items.iter().map(|item| {
        item.to_string()
    }).collect();
    strings.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigInt;

    fn bigintvec_to_string_process(values: Vec<i64>) -> String {
        let mut bigints: BigIntVec = vec!();
        for value in values {
            let value_bigint: BigInt = value.to_bigint().unwrap();
            bigints.push(value_bigint);
        }
        bigintvec_to_string(&bigints)
    }

    #[test]
    fn test_10000_bigintvec_to_string() {
        assert_eq!(bigintvec_to_string_process(vec![]), "");
        assert_eq!(bigintvec_to_string_process(vec![42]), "42");
        assert_eq!(bigintvec_to_string_process(vec![1, 2, 3]), "1,2,3");
        assert_eq!(bigintvec_to_string_process(vec![0, -11, 11]), "0,-11,11");
    }
}
