use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Label {
    OutputSizeEqualToInputSize,
    OutputSizeWidth { width: u8 },
    OutputSizeHeight { height: u8 },
}

pub type LabelSet = HashSet<Label>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_intersection() {
        let mut set0: LabelSet = HashSet::new();
        set0.insert(Label::OutputSizeEqualToInputSize);

        let mut set1: LabelSet = HashSet::new();
        set1.insert(Label::OutputSizeEqualToInputSize);

        let set2: LabelSet = set0.intersection(&set1).map(|l| *l).collect();
        assert_eq!(set2.contains(&Label::OutputSizeEqualToInputSize), true);
    }
}
