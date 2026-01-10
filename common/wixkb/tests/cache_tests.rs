//! Cache tests

use wixkb::cache::LruCache;
use wixkb::models::Element;

fn make_element(name: &str) -> Element {
    Element {
        id: 1,
        name: name.to_string(),
        namespace: "wix".to_string(),
        since_version: None,
        deprecated_version: None,
        description: Some(format!("{} description", name)),
        documentation_url: None,
        remarks: None,
        example: None,
    }
}

#[test]
fn test_cache_put_and_get() {
    let cache = LruCache::new(10);

    cache.put_element(make_element("Package"));
    cache.put_element(make_element("Component"));

    assert_eq!(cache.len(), 2);

    let pkg = cache.get_element("Package").unwrap();
    assert_eq!(pkg.name, "Package");

    let comp = cache.get_element("Component").unwrap();
    assert_eq!(comp.name, "Component");
}

#[test]
fn test_cache_get_nonexistent() {
    let cache = LruCache::new(10);
    let result = cache.get_element("NonExistent");
    assert!(result.is_none());
}

#[test]
fn test_cache_case_insensitive() {
    let cache = LruCache::new(10);
    cache.put_element(make_element("Package"));

    assert!(cache.get_element("Package").is_some());
    assert!(cache.get_element("package").is_some());
    assert!(cache.get_element("PACKAGE").is_some());
}

#[test]
fn test_cache_contains() {
    let cache = LruCache::new(10);
    cache.put_element(make_element("Package"));

    assert!(cache.contains_element("Package"));
    assert!(cache.contains_element("package"));
    assert!(!cache.contains_element("Component"));
}

#[test]
fn test_cache_eviction() {
    let cache = LruCache::new(3);

    cache.put_element(make_element("A"));
    cache.put_element(make_element("B"));
    cache.put_element(make_element("C"));
    assert_eq!(cache.len(), 3);

    // Adding D should evict A (least recently used)
    cache.put_element(make_element("D"));
    assert_eq!(cache.len(), 3);
    assert!(!cache.contains_element("A"));
    assert!(cache.contains_element("B"));
    assert!(cache.contains_element("C"));
    assert!(cache.contains_element("D"));
}

#[test]
fn test_cache_lru_order() {
    let cache = LruCache::new(3);

    cache.put_element(make_element("A"));
    cache.put_element(make_element("B"));
    cache.put_element(make_element("C"));

    // Access A to make it recently used
    cache.get_element("A");

    // Adding D should evict B (now least recently used)
    cache.put_element(make_element("D"));

    assert!(cache.contains_element("A")); // Recently accessed
    assert!(!cache.contains_element("B")); // Evicted
    assert!(cache.contains_element("C"));
    assert!(cache.contains_element("D"));
}

#[test]
fn test_cache_clear() {
    let cache = LruCache::new(10);

    cache.put_element(make_element("A"));
    cache.put_element(make_element("B"));
    cache.put_element(make_element("C"));
    assert_eq!(cache.len(), 3);

    cache.clear();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
fn test_cache_is_empty() {
    let cache = LruCache::new(10);
    assert!(cache.is_empty());

    cache.put_element(make_element("A"));
    assert!(!cache.is_empty());

    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn test_cache_update_existing() {
    let cache = LruCache::new(10);

    let elem1 = Element {
        id: 1,
        name: "Package".to_string(),
        namespace: "wix".to_string(),
        since_version: None,
        deprecated_version: None,
        description: Some("Original".to_string()),
        documentation_url: None,
        remarks: None,
        example: None,
    };
    cache.put_element(elem1);

    let elem2 = Element {
        id: 1,
        name: "Package".to_string(),
        namespace: "wix".to_string(),
        since_version: None,
        deprecated_version: None,
        description: Some("Updated".to_string()),
        documentation_url: None,
        remarks: None,
        example: None,
    };
    cache.put_element(elem2);

    assert_eq!(cache.len(), 1);
    let retrieved = cache.get_element("Package").unwrap();
    assert_eq!(retrieved.description, Some("Updated".to_string()));
}

#[test]
fn test_cache_capacity_one() {
    let cache = LruCache::new(1);

    cache.put_element(make_element("A"));
    assert_eq!(cache.len(), 1);

    cache.put_element(make_element("B"));
    assert_eq!(cache.len(), 1);
    assert!(!cache.contains_element("A"));
    assert!(cache.contains_element("B"));
}

#[test]
fn test_cache_large_capacity() {
    let cache = LruCache::new(1000);

    for i in 0..500 {
        cache.put_element(make_element(&format!("Element{}", i)));
    }

    assert_eq!(cache.len(), 500);
    assert!(cache.contains_element("Element0"));
    assert!(cache.contains_element("Element499"));
}
