extern crate lru;
use lru::LruCache;

pub struct MyCache {
    count_hit: u64,
    count_miss: u64,
}

impl MyCache {
    pub fn new() -> Self {
        Self {
            count_hit: 0,
            count_miss: 0,
        }
    }

    pub fn increment_hit(&mut self) {
        self.count_hit += 1;
    }

    pub fn increment_miss(&mut self) {
        self.count_miss += 1;
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
