use num_bigint::BigInt;

extern crate lru;
use lru::LruCache;

const CACHE_CAPACITY: usize = 1000;

#[derive(PartialEq, Eq, Hash)]
struct CacheKey {
    program_id: u64,
    index: BigInt,
}

pub struct CacheValue {
    pub value: BigInt,
    pub step_count: u64,
}

pub struct MyCache {
    cache: LruCache<CacheKey, CacheValue>,
    count_hit: u64,
    count_miss_for_program_oeis: u64,
    count_miss_for_program_without_id: u64,
}

impl MyCache {
    pub fn new() -> Self {
        let cache: LruCache<CacheKey, CacheValue> = LruCache::new(CACHE_CAPACITY);
        Self {
            cache: cache,
            count_hit: 0,
            count_miss_for_program_oeis: 0,
            count_miss_for_program_without_id: 0,
        }
    }

    pub fn register_cache_hit(&mut self) {
        self.count_hit += 1;
    }

    pub fn register_cache_miss_for_program_oeis(&mut self) {
        self.count_miss_for_program_oeis += 1;
    }

    pub fn register_cache_miss_for_program_without_id(&mut self) {
        self.count_miss_for_program_without_id += 1;
    }

    pub fn print_statistics(&self) {
        println!("hits: {}, miss0: {}, miss1: {}", self.count_hit, self.count_miss_for_program_oeis, self.count_miss_for_program_without_id)
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
        let mut cache = LruCache::new(2);
        cache.put("apple", 3);
        cache.put("banana", 2);
        assert_eq!(*cache.get(&"apple").unwrap(), 3);
        assert_eq!(*cache.get(&"banana").unwrap(), 2);
    }
}
