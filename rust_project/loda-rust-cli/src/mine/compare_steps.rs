use std::cmp;

pub fn compare_steps(steps0: &Vec<u64>, steps1: &Vec<u64>) -> (usize, usize, usize) {
    let mut count_same: usize = 0;
    let mut count_less_than: usize = 0;
    let mut count_greater_than: usize = 0;
    let len = cmp::min(steps0.len(), steps1.len());
    for i in 0..len {
        let steps0: u64 = steps0[i];
        let steps1: u64 = steps1[i];
        if steps0 < steps1 {
            count_less_than += 1;
            break;
        }
        if steps0 > steps1 {
            count_greater_than += 1;
            break;
        }
        count_same += 1;
    }
    (count_less_than, count_same, count_greater_than)
}
