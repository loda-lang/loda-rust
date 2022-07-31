use super::LodaCppError;
use loda_rust_core::util::BigIntVec;
use num_bigint::BigInt;
use std::error::Error;

pub struct LodaCppEvalTerms {
    terms: BigIntVec,
}

impl LodaCppEvalTerms {
    fn new(terms: BigIntVec) -> Self {
        Self {
            terms: terms,
        }
    }

    pub fn parse(raw_output_from_lodacpp: &String, term_count: usize) -> Result<LodaCppEvalTerms, Box<dyn Error>> {
        let trimmed_output: String = raw_output_from_lodacpp.trim().to_string();
        if trimmed_output.is_empty() {
            error!("No output to parse.");
            return Err(Box::new(LodaCppError::NoOutput));
        }
        let term_strings = trimmed_output.split(",");
        let mut terms_bigintvec = BigIntVec::with_capacity(term_count);
        for term_string in term_strings {
            let bytes: &[u8] = term_string.as_bytes();
            let bigint: BigInt = match BigInt::parse_bytes(bytes, 10) {
                Some(value) => value,
                None => {
                    error!("Unable to parse a number as BigInt. '{}'", term_string);
                    return Err(Box::new(LodaCppError::ParseTerms));
                }
            };
            terms_bigintvec.push(bigint);
        };
        Ok(LodaCppEvalTerms::new(terms_bigintvec))
    }

    #[allow(dead_code)]
    pub fn terms(&self) -> &BigIntVec {
        &self.terms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loda_rust_core::util::BigIntVecToString;

    fn parse(input: &str) -> String {
        let s = input.to_string();
        let eval_ok: LodaCppEvalTerms = match LodaCppEvalTerms::parse(&s, 0) {
            Ok(value) => value,
            Err(error) => {
                if let Some(lodacpp_error) = error.downcast_ref::<LodaCppError>() {
                    return format!("ERROR LODACPP: {:?}", lodacpp_error);
                }
                return format!("ERROR OTHER: {:?}", error);
            }
        };
        eval_ok.terms().to_compact_comma_string()
    }

    #[test]
    fn test_10000_parse_ok() {
        assert_eq!(parse("-1,2,-3,4"), "-1,2,-3,4");
        assert_eq!(parse("\n  42\t  "), "42");
        assert_eq!(parse("0"), "0");
        assert_eq!(parse("0,0"), "0,0");
    }
    
    #[test]
    fn test_20000_parse_error() {
        assert!(parse("").starts_with("ERROR LODACPP"));
        assert!(parse(" ").starts_with("ERROR LODACPP"));
        assert!(parse(" \n ").starts_with("ERROR LODACPP"));
        assert!(parse("1,2,overflow").starts_with("ERROR LODACPP"));
        assert!(parse("c++ exception").starts_with("ERROR LODACPP"));
    }
}
