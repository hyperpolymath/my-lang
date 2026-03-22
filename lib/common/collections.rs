// SPDX-License-Identifier: PMPL-1.0-or-later
//! Collections utilities for My Language
//!
//! Provides HashMap, HashSet, and Vec helper functions.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Create a new HashMap
pub fn hashmap_new<K, V>() -> HashMap<K, V>
where
    K: Eq + Hash,
{
    HashMap::new()
}

/// Create a HashMap with capacity
pub fn hashmap_with_capacity<K, V>(capacity: usize) -> HashMap<K, V>
where
    K: Eq + Hash,
{
    HashMap::with_capacity(capacity)
}

/// Get HashMap size
pub fn hashmap_len<K, V>(map: &HashMap<K, V>) -> usize {
    map.len()
}

/// Check if HashMap is empty
pub fn hashmap_is_empty<K, V>(map: &HashMap<K, V>) -> bool {
    map.is_empty()
}

/// Insert into HashMap
pub fn hashmap_insert<K, V>(map: &mut HashMap<K, V>, key: K, value: V) -> Option<V>
where
    K: Eq + Hash,
{
    map.insert(key, value)
}

/// Get from HashMap
pub fn hashmap_get<K, V>(map: &HashMap<K, V>, key: &K) -> Option<&V>
where
    K: Eq + Hash,
{
    map.get(key)
}

/// Check if HashMap contains key
pub fn hashmap_contains_key<K, V>(map: &HashMap<K, V>, key: &K) -> bool
where
    K: Eq + Hash,
{
    map.contains_key(key)
}

/// Remove from HashMap
pub fn hashmap_remove<K, V>(map: &mut HashMap<K, V>, key: &K) -> Option<V>
where
    K: Eq + Hash,
{
    map.remove(key)
}

/// Clear HashMap
pub fn hashmap_clear<K, V>(map: &mut HashMap<K, V>) {
    map.clear()
}

/// Get all keys from HashMap
pub fn hashmap_keys<K, V>(map: &HashMap<K, V>) -> Vec<K>
where
    K: Eq + Hash + Clone,
{
    map.keys().cloned().collect()
}

/// Get all values from HashMap
pub fn hashmap_values<K, V>(map: &HashMap<K, V>) -> Vec<V>
where
    V: Clone,
{
    map.values().cloned().collect()
}

/// Create a new HashSet
pub fn hashset_new<T>() -> HashSet<T>
where
    T: Eq + Hash,
{
    HashSet::new()
}

/// Create a HashSet with capacity
pub fn hashset_with_capacity<T>(capacity: usize) -> HashSet<T>
where
    T: Eq + Hash,
{
    HashSet::with_capacity(capacity)
}

/// Get HashSet size
pub fn hashset_len<T>(set: &HashSet<T>) -> usize {
    set.len()
}

/// Check if HashSet is empty
pub fn hashset_is_empty<T>(set: &HashSet<T>) -> bool {
    set.is_empty()
}

/// Insert into HashSet
pub fn hashset_insert<T>(set: &mut HashSet<T>, value: T) -> bool
where
    T: Eq + Hash,
{
    set.insert(value)
}

/// Check if HashSet contains value
pub fn hashset_contains<T>(set: &HashSet<T>, value: &T) -> bool
where
    T: Eq + Hash,
{
    set.contains(value)
}

/// Remove from HashSet
pub fn hashset_remove<T>(set: &mut HashSet<T>, value: &T) -> bool
where
    T: Eq + Hash,
{
    set.remove(value)
}

/// Clear HashSet
pub fn hashset_clear<T>(set: &mut HashSet<T>) {
    set.clear()
}

/// Union of two HashSets
pub fn hashset_union<T>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
{
    a.union(b).cloned().collect()
}

/// Intersection of two HashSets
pub fn hashset_intersection<T>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
{
    a.intersection(b).cloned().collect()
}

/// Difference of two HashSets (a - b)
pub fn hashset_difference<T>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
{
    a.difference(b).cloned().collect()
}

/// Vec helpers

/// Create a new Vec
pub fn vec_new<T>() -> Vec<T> {
    Vec::new()
}

/// Create a Vec with capacity
pub fn vec_with_capacity<T>(capacity: usize) -> Vec<T> {
    Vec::with_capacity(capacity)
}

/// Get Vec length
pub fn vec_len<T>(vec: &Vec<T>) -> usize {
    vec.len()
}

/// Check if Vec is empty
pub fn vec_is_empty<T>(vec: &Vec<T>) -> bool {
    vec.is_empty()
}

/// Push to Vec
pub fn vec_push<T>(vec: &mut Vec<T>, value: T) {
    vec.push(value)
}

/// Pop from Vec
pub fn vec_pop<T>(vec: &mut Vec<T>) -> Option<T> {
    vec.pop()
}

/// Get from Vec by index
pub fn vec_get<T>(vec: &Vec<T>, index: usize) -> Option<&T> {
    vec.get(index)
}

/// Set Vec element at index
pub fn vec_set<T>(vec: &mut Vec<T>, index: usize, value: T) -> Result<(), String> {
    if index < vec.len() {
        vec[index] = value;
        Ok(())
    } else {
        Err(format!("Index {} out of bounds for vec of length {}", index, vec.len()))
    }
}

/// Insert into Vec at index
pub fn vec_insert<T>(vec: &mut Vec<T>, index: usize, value: T) {
    vec.insert(index, value)
}

/// Remove from Vec at index
pub fn vec_remove<T>(vec: &mut Vec<T>, index: usize) -> T {
    vec.remove(index)
}

/// Clear Vec
pub fn vec_clear<T>(vec: &mut Vec<T>) {
    vec.clear()
}

/// Reverse Vec in place
pub fn vec_reverse<T>(vec: &mut Vec<T>) {
    vec.reverse()
}

/// Sort Vec (requires Ord)
pub fn vec_sort<T>(vec: &mut Vec<T>)
where
    T: Ord,
{
    vec.sort()
}

/// Filter Vec
pub fn vec_filter<T, F>(vec: &Vec<T>, predicate: F) -> Vec<T>
where
    T: Clone,
    F: Fn(&T) -> bool,
{
    vec.iter().filter(|x| predicate(x)).cloned().collect()
}

/// Map Vec
pub fn vec_map<T, U, F>(vec: &Vec<T>, f: F) -> Vec<U>
where
    F: Fn(&T) -> U,
{
    vec.iter().map(f).collect()
}

/// Reduce Vec (fold)
pub fn vec_reduce<T, U, F>(vec: &Vec<T>, init: U, f: F) -> U
where
    F: Fn(U, &T) -> U,
{
    vec.iter().fold(init, f)
}

/// Check if Vec contains element
pub fn vec_contains<T>(vec: &Vec<T>, value: &T) -> bool
where
    T: PartialEq,
{
    vec.contains(value)
}

/// Find index of first matching element
pub fn vec_index_of<T>(vec: &Vec<T>, value: &T) -> Option<usize>
where
    T: PartialEq,
{
    vec.iter().position(|x| x == value)
}

/// Create Vec from slice
pub fn vec_from_slice<T>(slice: &[T]) -> Vec<T>
where
    T: Clone,
{
    slice.to_vec()
}

/// Concatenate two Vecs
pub fn vec_concat<T>(a: &Vec<T>, b: &Vec<T>) -> Vec<T>
where
    T: Clone,
{
    let mut result = a.clone();
    result.extend_from_slice(b);
    result
}

/// Get Vec slice
pub fn vec_slice<T>(vec: &Vec<T>, start: usize, end: usize) -> Vec<T>
where
    T: Clone,
{
    vec[start..end].to_vec()
}

/// Deduplicate Vec (requires Eq)
pub fn vec_dedup<T>(vec: &mut Vec<T>)
where
    T: PartialEq,
{
    vec.dedup()
}

/// Check if Vec is sorted
pub fn vec_is_sorted<T>(vec: &Vec<T>) -> bool
where
    T: Ord,
{
    vec.windows(2).all(|w| w[0] <= w[1])
}

/// Binary search in sorted Vec
pub fn vec_binary_search<T>(vec: &Vec<T>, value: &T) -> Result<usize, usize>
where
    T: Ord,
{
    vec.binary_search(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap_operations() {
        let mut map = hashmap_new();
        hashmap_insert(&mut map, "key", "value");
        assert_eq!(hashmap_get(&map, &"key"), Some(&"value"));
        assert_eq!(hashmap_len(&map), 1);
    }

    #[test]
    fn test_hashset_operations() {
        let mut set = hashset_new();
        hashset_insert(&mut set, 1);
        hashset_insert(&mut set, 2);
        assert!(hashset_contains(&set, &1));
        assert_eq!(hashset_len(&set), 2);
    }

    #[test]
    fn test_vec_operations() {
        let mut vec = vec_new();
        vec_push(&mut vec, 1);
        vec_push(&mut vec, 2);
        vec_push(&mut vec, 3);
        assert_eq!(vec_len(&vec), 3);
        assert_eq!(vec_pop(&mut vec), Some(3));
        assert!(vec_contains(&vec, &1));
    }
}
