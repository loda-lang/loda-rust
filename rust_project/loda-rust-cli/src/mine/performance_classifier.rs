use std::cmp;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum PerformanceClassifierResult {
    ErrorTooShortInputVector,
    ErrorDifferentInputVectorLengths,
    Identical,
    NewProgramIsAlwaysFaster,
    NewProgramIsEqualOrFaster,
    NewProgramIsAlwaysFasterWhenSkippingTheFirstSlice,
    RejectNewProgram,
}

pub struct PerformanceClassifier {
    number_of_items_in_first_slice: usize,
}

impl PerformanceClassifier {
    pub fn new(number_of_items_in_first_slice: usize) -> Self {
        assert!(number_of_items_in_first_slice > 0);
        Self {
            number_of_items_in_first_slice: number_of_items_in_first_slice
        }
    }

    pub fn analyze(&self, steps0: &Vec<u64>, steps1: &Vec<u64>) -> PerformanceClassifierResult {
        if steps0.len() != steps1.len() {
            return PerformanceClassifierResult::ErrorDifferentInputVectorLengths;
        }
        let len = cmp::min(steps0.len(), steps1.len());
        if len <= self.number_of_items_in_first_slice {
            return PerformanceClassifierResult::ErrorTooShortInputVector;
        }
        let first_slice0: &[u64] = &steps0[..self.number_of_items_in_first_slice];
        let first_slice1: &[u64] = &steps1[..self.number_of_items_in_first_slice];
        assert!(first_slice0.len() == self.number_of_items_in_first_slice);
        assert!(first_slice1.len() == self.number_of_items_in_first_slice);

        let last_slice0: &[u64] = &steps0[self.number_of_items_in_first_slice..];
        let last_slice1: &[u64] = &steps1[self.number_of_items_in_first_slice..];
        assert!(first_slice0.len() + last_slice0.len() == steps0.len());
        assert!(first_slice1.len() + last_slice1.len() == steps1.len());

        let (first_count_program0, first_count_same, first_count_program1) = Self::histogram(first_slice0, first_slice1);
        let (last_count_program0, last_count_same, last_count_program1) = Self::histogram(last_slice0, last_slice1);

        let count_program0 = first_count_program0 + last_count_program0;
        let count_same = first_count_same + last_count_same;
        let count_program1 = first_count_program1 + last_count_program1;

        if count_program0 == 0 && count_same > 0 && count_program1 == 0 {
            return PerformanceClassifierResult::Identical;
        }
        if count_program0 > 0 && count_same == 0 && count_program1 == 0  {
            return PerformanceClassifierResult::NewProgramIsAlwaysFaster;
        }
        if count_program0 > 0 && count_same > 0 && count_program1 == 0  {
            return PerformanceClassifierResult::NewProgramIsEqualOrFaster;
        }
        if first_count_program1 > 0 &&
            last_count_program0 > 0 && last_count_same == 0 && last_count_program1 == 0 {
            return PerformanceClassifierResult::NewProgramIsAlwaysFasterWhenSkippingTheFirstSlice;
        }
        PerformanceClassifierResult::RejectNewProgram
    }

    fn histogram(steps0: &[u64], steps1: &[u64]) -> (usize, usize, usize) {
        let mut count_same: usize = 0;
        let mut count_less_than: usize = 0;
        let mut count_greater_than: usize = 0;
        let len = cmp::min(steps0.len(), steps1.len());
        for i in 0..len {
            let steps0: u64 = steps0[i];
            let steps1: u64 = steps1[i];
            if steps0 < steps1 {
                count_less_than += 1;
                continue;
            }
            if steps0 > steps1 {
                count_greater_than += 1;
                continue;
            }
            count_same += 1;
        }
        (count_less_than, count_same, count_greater_than)
    }    
}

#[cfg(test)]
mod tests {
    use super::*;

    fn histogram(steps0: Vec<u64>, steps1: Vec<u64>) -> String {
        let (a, b, c) = PerformanceClassifier::histogram(&steps0, &steps1);
        format!("{} {} {}", a, b, c)
    }
    
    #[test]
    fn test_10000_histogram() {
        assert_eq!(histogram(vec![1, 1], vec![2, 2]), "2 0 0");
        assert_eq!(histogram(vec![1, 1], vec![1, 1]), "0 2 0");
        assert_eq!(histogram(vec![2, 2], vec![1, 1]), "0 0 2");
        assert_eq!(histogram(vec![1, 1, 2, 1], vec![2, 2, 2, 2]), "3 1 0");
        assert_eq!(histogram(vec![2, 2, 1, 2], vec![1, 1, 1, 1]), "0 1 3");
        assert_eq!(histogram(vec![5, 1, 5, 1], vec![2, 4, 2, 4]), "2 0 2");
    }
    
    #[test]
    fn test_20000_analyze_error_different_lengths() {
        let steps0: Vec<u64> = vec![5, 5, 5, 5, 5];
        let steps1: Vec<u64> = vec![5, 5, 5, 5, 5, 666];
        let pc = PerformanceClassifier::new(2);
        let result = pc.analyze(&steps0, &steps1);
        assert_eq!(result, PerformanceClassifierResult::ErrorDifferentInputVectorLengths);
    }
    
    #[test]
    fn test_20001_analyze_error_too_short() {
        let steps: Vec<u64> = vec![5, 5, 5, 5, 5];
        let pc = PerformanceClassifier::new(10);
        let result = pc.analyze(&steps, &steps);
        assert_eq!(result, PerformanceClassifierResult::ErrorTooShortInputVector);
    }
    
    #[test]
    fn test_30000_analyze_identical() {
        let steps: Vec<u64> = vec![5, 5, 5, 5, 5];
        let pc = PerformanceClassifier::new(2);
        let result = pc.analyze(&steps, &steps);
        assert_eq!(result, PerformanceClassifierResult::Identical);
    }

    #[test]
    fn test_30001_analyze_is_always_faster() {
        let steps0: Vec<u64> = vec![4, 4, 3, 4, 2];
        let steps1: Vec<u64> = vec![5, 5, 5, 5, 5];
        let pc = PerformanceClassifier::new(3);
        let result = pc.analyze(&steps0, &steps1);
        assert_eq!(result, PerformanceClassifierResult::NewProgramIsAlwaysFaster);
    }

    #[test]
    fn test_30002_analyze_new_program_is_equal_or_faster() {
        let steps0: Vec<u64> = vec![4, 5, 3, 5, 2];
        let steps1: Vec<u64> = vec![5, 5, 5, 5, 5];
        let pc = PerformanceClassifier::new(3);
        let result = pc.analyze(&steps0, &steps1);
        assert_eq!(result, PerformanceClassifierResult::NewProgramIsEqualOrFaster);
    }

    #[test]
    fn test_30003_analyze_new_program_is_equal_or_faster() {
        let steps0: Vec<u64> = vec![40, 40, 40, 5, 5, 5];
        let steps1: Vec<u64> = vec![5, 5, 5, 40, 40, 40];
        let pc = PerformanceClassifier::new(3);
        let result = pc.analyze(&steps0, &steps1);
        assert_eq!(result, PerformanceClassifierResult::NewProgramIsAlwaysFasterWhenSkippingTheFirstSlice);
    }

    #[test]
    fn test_40000_analyze_reject_new_program() {
        let steps0: Vec<u64> = vec![40, 41, 43, 46, 50, 60];
        let steps1: Vec<u64> = vec![5, 5, 5, 5, 5, 5];
        let pc = PerformanceClassifier::new(3);
        let result = pc.analyze(&steps0, &steps1);
        assert_eq!(result, PerformanceClassifierResult::RejectNewProgram);
    }
}
