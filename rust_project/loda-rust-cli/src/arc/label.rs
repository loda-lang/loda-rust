
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Label {
    OutputSizeEqualToInputSize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_10000_label() {
        let mut set0: HashSet<Label> = HashSet::new();
        set0.insert(Label::OutputSizeEqualToInputSize);

        let mut set1: HashSet<Label> = HashSet::new();
        set1.insert(Label::OutputSizeEqualToInputSize);

        let set2: HashSet<Label> = set0.intersection(&set1).map(|l| l.clone()).collect();
        assert_eq!(set2.contains(&Label::OutputSizeEqualToInputSize), true);
    }
}
