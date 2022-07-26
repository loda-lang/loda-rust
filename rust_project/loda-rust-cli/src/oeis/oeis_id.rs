use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash)]
pub struct OeisId(u32);

impl OeisId {
    /// Every sequence in the OEIS has an `A-number`, such as `"A001850"`.
    pub fn a_number(&self) -> String {
        format!("A{:0>6}", self.0)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl Ord for OeisId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for OeisId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }    
}    

impl PartialEq for OeisId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl From<u32> for OeisId {
    fn from(val: u32) -> OeisId {
        OeisId(val)
    }
}

impl fmt::Display for OeisId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.a_number())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_10000_from() {
        assert_eq!(OeisId::from(45), OeisId(45));
        assert_eq!(OeisId::from(10051), OeisId(10051));
    }

    #[test]
    fn test_20000_raw() {
        assert_eq!(OeisId::from(45).raw(), 45);
        assert_eq!(OeisId::from(10051).raw(), 10051);
        assert_eq!(OeisId::from(354995).raw(), 354995);
        assert_eq!(OeisId::from(999123456).raw(), 999123456);
    }

    #[test]
    fn test_30000_a_number() {
        assert_eq!(OeisId::from(45).a_number(), "A000045");
        assert_eq!(OeisId::from(10051).a_number(), "A010051");
        assert_eq!(OeisId::from(354995).a_number(), "A354995");
        assert_eq!(OeisId::from(999123456).a_number(), "A999123456");
    }

    #[test]
    fn test_40000_format() {
        assert_eq!(format!("{}", OeisId::from(45)), "A000045");
        assert_eq!(format!("{:?}", OeisId::from(45)), "OeisId(45)");
    }

    #[test]
    fn test_50000_hashset() {
        let mut oeis_ids = HashSet::<OeisId>::new();
        oeis_ids.insert(OeisId::from(45));
        oeis_ids.insert(OeisId::from(10051));
        assert_eq!(oeis_ids.len(), 2);
        oeis_ids.insert(OeisId::from(45));
        assert_eq!(oeis_ids.len(), 2);
    }

    #[test]
    fn test_60000_sorting() {
        // Arrange
        let mut oeis_ids: Vec::<OeisId> = vec![
            OeisId::from(45),
            OeisId::from(10051),
            OeisId::from(40),
            OeisId::from(96)
        ];
        // Act
        oeis_ids.sort();
        // Assert
        let a_number_vec: Vec<String> = oeis_ids.iter().map(|oeis_id| oeis_id.a_number()).collect();
        let a_numbers: String = a_number_vec.join(",");
        assert_eq!(a_numbers, "A000040,A000045,A000096,A010051");
    }
}
