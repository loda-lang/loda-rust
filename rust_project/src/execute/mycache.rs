extern crate lru;
use lru::LruCache;

pub struct MyCache {

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
