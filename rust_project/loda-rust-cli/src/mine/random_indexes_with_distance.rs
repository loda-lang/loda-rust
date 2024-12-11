use rand::Rng;
use rand::seq::SliceRandom;

/// Generate indexes that are soft-shuffled.
/// 
/// Usecase: soft-shuffling of a histogram and roughly preserve the ordering.
/// 
/// Roughly preserves the ordering of indexes:
/// - The indexes near the start, stays near the start.
/// - The indexes near the middle, stays near the middle.
/// - The indexes near the end, stays near the end.
pub fn random_indexes_with_distance<R: Rng + ?Sized>(rng: &mut R, number_of_indexes: usize, chunk_size: usize) -> Vec<usize> {
    if number_of_indexes == 0 {
        return vec!();
    }
    if number_of_indexes == 1 {
        return vec![0];
    }
    let mut indexes: Vec<usize> = (0..number_of_indexes).collect();
    if chunk_size <= 1 {
        // No shuffling
        return indexes;
    }
    // Loop over chunks, and shuffle indexes inside each chunk
    let mut start: usize = 0;
    while start+chunk_size < number_of_indexes {
        let slice = &mut indexes[start..start+chunk_size];
        slice.shuffle(rng);
        start += chunk_size;
    }
    // Shuffle the remaining indexes
    if start < number_of_indexes {
        let slice = &mut indexes[start..];
        slice.shuffle(rng);
    }
    indexes
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    
    #[test]
    fn test_10000_empty() {
        let mut rng = StdRng::seed_from_u64(1);
        let indexes = random_indexes_with_distance(&mut rng, 0, 0);
        assert_eq!(indexes.len(), 0);
    }

    #[test]
    fn test_10001_one() {
        let mut rng = StdRng::seed_from_u64(1);
        let indexes = random_indexes_with_distance(&mut rng, 1, 0);
        assert_eq!(indexes, vec![0]);
    }

    #[test]
    fn test_20000_chunksize1() {
        let mut rng = StdRng::seed_from_u64(1);
        let indexes = random_indexes_with_distance(&mut rng, 4, 1);
        assert_eq!(indexes, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_30000_chunksize2_without_remaining() {
        let mut rng = StdRng::seed_from_u64(1);
        let indexes = random_indexes_with_distance(&mut rng, 8, 2);
        assert_eq!(indexes, vec![0, 1, 3, 2, 5, 4, 7, 6]);
    }

    #[test]
    fn test_30001_chunksize2_with_remaining() {
        let mut rng = StdRng::seed_from_u64(1);
        let indexes = random_indexes_with_distance(&mut rng, 7, 2);
        assert_eq!(indexes, vec![0, 1, 3, 2, 5, 4, 6]);
    }

    #[test]
    fn test_30002_chunksize3_without_remaining() {
        let mut rng = StdRng::seed_from_u64(1);
        let indexes = random_indexes_with_distance(&mut rng, 8, 3);
        assert_eq!(indexes, vec![0, 1, 2, 4, 5, 3, 7, 6]);
    }

    #[test]
    fn test_30003_chunksize3_with_remaining1() {
        let mut rng = StdRng::seed_from_u64(1);
        let indexes = random_indexes_with_distance(&mut rng, 7, 3);
        assert_eq!(indexes, vec![0, 1, 2, 4, 5, 3, 6]);
    }

    #[test]
    fn test_30004_chunksize3_with_remaining2() {
        let mut rng = StdRng::seed_from_u64(1);
        let indexes = random_indexes_with_distance(&mut rng, 6, 3);
        assert_eq!(indexes, vec![0, 1, 2, 4, 5, 3]);
    }

    #[test]
    fn test_30005_chunksize10_with_remaining3_a() {
        let mut rng = StdRng::seed_from_u64(2);
        let indexes = random_indexes_with_distance(&mut rng, 3, 10);
        assert_eq!(indexes, vec![1, 2, 0]);
    }

    #[test]
    fn test_30006_chunksize10_with_remaining3_b() {
        let mut rng = StdRng::seed_from_u64(2);
        let indexes = random_indexes_with_distance(&mut rng, 13, 10);
        assert_eq!(indexes, vec![4, 8, 5, 1, 7, 6, 3, 2, 0, 9, 11, 10, 12]);
    }
}
