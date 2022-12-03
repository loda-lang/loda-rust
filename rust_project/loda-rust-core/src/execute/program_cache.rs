use num_bigint::BigInt;
use std::num::NonZeroUsize;
use cached::{SizedCache, Cached};

const DEFAULT_CACHE_CAPACITY: usize = 3000;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct CacheKey {
    program_id: u64,
    index: BigInt,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CacheValue {
    pub value: BigInt,
    pub step_count: u64,
}

pub struct ProgramCache {
    cache: SizedCache<CacheKey, CacheValue>,
    metric_hit: u64,
    metric_miss_for_program_oeis: u64,
    metric_miss_for_program_without_id: u64,
}

impl ProgramCache {
    pub fn new() -> Self {
        let capacity = NonZeroUsize::new(DEFAULT_CACHE_CAPACITY).unwrap();
        Self::with_capacity(capacity)
    }

    pub fn with_capacity(capacity: NonZeroUsize) -> Self {
        let cache: SizedCache<CacheKey, CacheValue> = SizedCache::with_size(capacity.get());
        Self {
            cache: cache,
            metric_hit: 0,
            metric_miss_for_program_oeis: 0,
            metric_miss_for_program_without_id: 0,
        }
    }

    pub fn register_cache_hit(&mut self) {
        self.metric_hit += 1;
    }

    pub fn register_cache_miss_for_program_oeis(&mut self) {
        self.metric_miss_for_program_oeis += 1;
    }

    pub fn register_cache_miss_for_program_without_id(&mut self) {
        self.metric_miss_for_program_without_id += 1;
    }

    pub fn metric_hit(&self) -> u64 {
        self.metric_hit
    }

    pub fn metric_miss_for_program_oeis(&self) -> u64 {
        self.metric_miss_for_program_oeis
    }

    pub fn metric_miss_for_program_without_id(&self) -> u64 {
        self.metric_miss_for_program_without_id
    }

    pub fn reset_metrics(&mut self) {
        self.metric_hit = 0;
        self.metric_miss_for_program_oeis = 0;
        self.metric_miss_for_program_without_id = 0;
    }

    pub fn hit_miss_info(&self) -> String {
        format!("hit:{} miss:{},{}", self.metric_hit, self.metric_miss_for_program_oeis, self.metric_miss_for_program_without_id)
    }

    pub fn get(&mut self, program_id: u64, index: &BigInt) -> Option<&CacheValue> {
        let key = CacheKey {
            program_id: program_id,
            index: index.clone(),
        };
        self.cache.cache_get(&key)
    }

    pub fn set(&mut self, program_id: u64, index: BigInt, value: BigInt, step_count: u64) {
        let key = CacheKey {
            program_id: program_id,
            index: index,
        };
        let value = CacheValue {
            value: value,
            step_count: step_count,
        };
        self.cache.cache_set(key, value);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigInt;

    #[test]
    fn test_10000_remove_oldest_data() {
        // Arrange
        let capacity = NonZeroUsize::new(2).unwrap();
        let mut cache = ProgramCache::with_capacity(capacity);
        assert_eq!(cache.get(40, &0u8.to_bigint().unwrap()), None, "initially the cache is empty");
        assert_eq!(cache.get(40, &1u8.to_bigint().unwrap()), None, "initially the cache is empty");
        assert_eq!(cache.get(40, &2u8.to_bigint().unwrap()), None, "initially the cache is empty");
        cache.set(40, 0u8.to_bigint().unwrap(), 2u8.to_bigint().unwrap(), 1);
        cache.set(40, 1u8.to_bigint().unwrap(), 3u8.to_bigint().unwrap(), 1);
        assert_ne!(cache.get(40, &0u8.to_bigint().unwrap()), None, "has data");
        assert_ne!(cache.get(40, &1u8.to_bigint().unwrap()), None, "has data");
        assert_eq!(cache.get(40, &2u8.to_bigint().unwrap()), None, "empty");

        // Act
        cache.set(40, 2u8.to_bigint().unwrap(), 5u8.to_bigint().unwrap(), 1);

        // Assert
        assert_eq!(cache.get(40, &0u8.to_bigint().unwrap()), None, "empty, removed oldest data");
        assert_ne!(cache.get(40, &1u8.to_bigint().unwrap()), None, "has data");
        assert_ne!(cache.get(40, &2u8.to_bigint().unwrap()), None, "has data");
    }
}
