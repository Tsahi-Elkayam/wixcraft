//! LRU Cache for fast lookups

use crate::models::Element;
use std::sync::Mutex;

/// LRU Cache for frequently accessed data
pub struct LruCache {
    elements: Mutex<lru::LruCache<String, Element>>,
}

impl LruCache {
    /// Create a new cache with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            elements: Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap(),
            )),
        }
    }

    /// Get an element from cache
    pub fn get_element(&self, name: &str) -> Option<Element> {
        let mut cache = self.elements.lock().unwrap();
        cache.get(&name.to_lowercase()).cloned()
    }

    /// Put an element in cache
    pub fn put_element(&self, element: Element) {
        let mut cache = self.elements.lock().unwrap();
        cache.put(element.name.to_lowercase(), element);
    }

    /// Check if element is in cache
    pub fn contains_element(&self, name: &str) -> bool {
        let cache = self.elements.lock().unwrap();
        cache.contains(&name.to_lowercase())
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.elements.lock().unwrap();
        cache.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        let cache = self.elements.lock().unwrap();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_element(name: &str) -> Element {
        Element {
            id: 1,
            name: name.to_string(),
            namespace: "wix".to_string(),
            since_version: None,
            deprecated_version: None,
            description: None,
            documentation_url: None,
            remarks: None,
            example: None,
        }
    }

    #[test]
    fn test_cache_basic() {
        let cache = LruCache::new(10);
        assert!(cache.is_empty());

        cache.put_element(make_element("Package"));
        assert_eq!(cache.len(), 1);
        assert!(cache.contains_element("Package"));
        assert!(cache.contains_element("package")); // Case insensitive

        let elem = cache.get_element("Package").unwrap();
        assert_eq!(elem.name, "Package");
    }

    #[test]
    fn test_cache_eviction() {
        let cache = LruCache::new(2);

        cache.put_element(make_element("A"));
        cache.put_element(make_element("B"));
        cache.put_element(make_element("C"));

        // A should be evicted
        assert!(!cache.contains_element("A"));
        assert!(cache.contains_element("B"));
        assert!(cache.contains_element("C"));
    }

    #[test]
    fn test_cache_clear() {
        let cache = LruCache::new(10);
        cache.put_element(make_element("Package"));
        cache.put_element(make_element("Component"));
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_lru_behavior() {
        let cache = LruCache::new(2);

        cache.put_element(make_element("A"));
        cache.put_element(make_element("B"));

        // Access A to make it recently used
        cache.get_element("A");

        // Add C, should evict B (least recently used)
        cache.put_element(make_element("C"));

        assert!(cache.contains_element("A"));
        assert!(!cache.contains_element("B"));
        assert!(cache.contains_element("C"));
    }
}
