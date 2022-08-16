use crate::oeis::{OeisId, OeisIdHashSet};
use std::iter::FromIterator;

pub trait ToOeisIdVec {
    fn sorted_vec(&self) -> Vec<OeisId>;
}

impl ToOeisIdVec for OeisIdHashSet {
    fn sorted_vec(&self) -> Vec<OeisId> {
        let v0 = Vec::<&OeisId>::from_iter(self);
        let mut v1: Vec<OeisId> = v0.iter().map(|&oeis_id_ref| *oeis_id_ref).collect();
        v1.sort();
        v1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_oeis_id_sorted() {
        // Arrange
        let mut hs = OeisIdHashSet::new();
        hs.insert(OeisId::from(2));
        hs.insert(OeisId::from(5));
        hs.insert(OeisId::from(3));
        hs.insert(OeisId::from(1));
        hs.insert(OeisId::from(4));

        // Act
        let actual: Vec<OeisId> = hs.sorted_vec();

        // Assert
        let expected: Vec<OeisId> = vec![
            OeisId::from(1), OeisId::from(2), OeisId::from(3), OeisId::from(4), OeisId::from(5),
        ];
        assert_eq!(actual, expected);
    }
}
