use num_bigint::BigInt;

extern crate lru;
use std::num::NonZeroUsize;
use lru::LruCache;

const DEFAULT_CACHE_CAPACITY: usize = 3000;

#[derive(PartialEq, Eq, Hash)]
struct CacheKey {
    program_id: u64,
    index: BigInt,
}

pub struct CacheValue {
    pub value: BigInt,
    pub step_count: u64,
}

pub struct ProgramCache {
    cache: LruCache<CacheKey, CacheValue>,
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
        let cache: LruCache<CacheKey, CacheValue> = LruCache::new(capacity);
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
        self.cache.get(&key)
    }

    pub fn set(&mut self, program_id: u64, index: &BigInt, value: &BigInt, step_count: u64) {
        let key = CacheKey {
            program_id: program_id,
            index: index.clone(),
        };
        let value = CacheValue {
            value: value.clone(),
            step_count: step_count,
        };
        self.cache.put(key, value);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_lru_cache() {
        let capacity = NonZeroUsize::new(2).unwrap();
        let mut cache = LruCache::new(capacity);
        cache.put("apple", 3);
        cache.put("banana", 2);
        assert_eq!(*cache.get(&"apple").unwrap(), 3);
        assert_eq!(*cache.get(&"banana").unwrap(), 2);
    }
}
