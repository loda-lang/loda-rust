use std::collections::HashSet;
use loda_rust_core::util::{BigIntVec, bigintvec_to_string};

// The `mine-event` dir can quickly get filled up with candidate programs
// that output the same terms. I have experienced that 80k files was
// created in a few seconds with nearly identical programs.
//
// This is a mechanism to prevent the `mine-event` dir from getting filled up.
// The way it works. If there already is a program with the same terms in the `mine-event` dir,
// then prevent the new program from being written to the dir.
//
// On load, the `mine-event` dir is scanned for what programs are there.
pub struct PreventFlooding {
    hashset: HashSet<String>,
}

pub enum PreventFloodingError {
    AlreadyRegistered
}

impl PreventFlooding {
    pub fn new() -> Self {
        Self {
            hashset: HashSet::<String>::new(),
        }
    }

    #[allow(dead_code)]
    pub fn contains(&self, bigintvec: &BigIntVec) -> bool {
        let s: String = bigintvec_to_string(&bigintvec);
        if self.hashset.contains(&s) {
            return true;
        }
        false
    }

    pub fn try_register(&mut self, bigintvec: &BigIntVec) -> Result<(), PreventFloodingError> {
        let s: String = bigintvec_to_string(&bigintvec);
        if self.hashset.contains(&s) {
            // The `mine-event` dir already contains a program with these terms.
            return Err(PreventFloodingError::AlreadyRegistered);
        }
        self.hashset.insert(s);

        // Successfully registered the new program.
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.hashset.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loda_rust_core::util::i64vec_to_bigintvec;
    
    #[test]
    fn test_10000_try_register() {
        let mut pf = PreventFlooding::new();
        let bigintvec: BigIntVec = i64vec_to_bigintvec(vec![1, 2, 3, 4, 5]);
        assert_eq!(pf.try_register(&bigintvec).is_ok(), true);
        assert_eq!(pf.try_register(&bigintvec).is_ok(), false);
    }

    #[test]
    fn test_10001_contains() {
        let mut pf = PreventFlooding::new();
        assert_eq!(pf.contains(&i64vec_to_bigintvec(vec![1, 1, 1, 1, 1])), false);
        assert_eq!(pf.contains(&i64vec_to_bigintvec(vec![1, 2, 3, 4, 5])), false);
        assert_eq!(pf.try_register(&i64vec_to_bigintvec(vec![1, 2, 3, 4, 5])).is_ok(), true);
        assert_eq!(pf.try_register(&i64vec_to_bigintvec(vec![1, 1, 1, 1, 1])).is_ok(), true);
        assert_eq!(pf.contains(&i64vec_to_bigintvec(vec![1, 1, 1, 1, 1])), true);
        assert_eq!(pf.contains(&i64vec_to_bigintvec(vec![1, 2, 3, 4, 5])), true);
        assert_eq!(pf.contains(&i64vec_to_bigintvec(vec![1984, 1984, 1984])), false);
    }
}
