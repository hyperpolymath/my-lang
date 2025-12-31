//! Common Array/Collection Operations
//!
//! Generic array and collection manipulation functions.

/// Get array length
pub fn len<T>(arr: &[T]) -> usize {
    arr.len()
}

/// Check if array is empty
pub fn is_empty<T>(arr: &[T]) -> bool {
    arr.is_empty()
}

/// Get first element
pub fn first<T: Clone>(arr: &[T]) -> Option<T> {
    arr.first().cloned()
}

/// Get last element
pub fn last<T: Clone>(arr: &[T]) -> Option<T> {
    arr.last().cloned()
}

/// Get element at index
pub fn get<T: Clone>(arr: &[T], idx: usize) -> Option<T> {
    arr.get(idx).cloned()
}

/// Create new array with element appended
pub fn push<T: Clone>(arr: &[T], elem: T) -> Vec<T> {
    let mut result = arr.to_vec();
    result.push(elem);
    result
}

/// Create new array without last element
pub fn pop<T: Clone>(arr: &[T]) -> Vec<T> {
    let mut result = arr.to_vec();
    result.pop();
    result
}

/// Create new array with element prepended
pub fn unshift<T: Clone>(arr: &[T], elem: T) -> Vec<T> {
    let mut result = vec![elem];
    result.extend(arr.iter().cloned());
    result
}

/// Create new array without first element
pub fn shift<T: Clone>(arr: &[T]) -> Vec<T> {
    if arr.is_empty() {
        vec![]
    } else {
        arr[1..].to_vec()
    }
}

/// Set element at index (returns new array)
pub fn set<T: Clone>(arr: &[T], idx: usize, value: T) -> Option<Vec<T>> {
    if idx >= arr.len() {
        return None;
    }
    let mut result = arr.to_vec();
    result[idx] = value;
    Some(result)
}

/// Concatenate two arrays
pub fn concat<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut result = a.to_vec();
    result.extend(b.iter().cloned());
    result
}

/// Get slice of array
pub fn slice<T: Clone>(arr: &[T], start: usize, end: usize) -> Vec<T> {
    let start = start.min(arr.len());
    let end = end.min(arr.len());
    if start <= end {
        arr[start..end].to_vec()
    } else {
        vec![]
    }
}

/// Reverse array
pub fn reverse<T: Clone>(arr: &[T]) -> Vec<T> {
    let mut result = arr.to_vec();
    result.reverse();
    result
}

/// Check if array contains element
pub fn contains<T: PartialEq>(arr: &[T], elem: &T) -> bool {
    arr.contains(elem)
}

/// Find index of element
pub fn index_of<T: PartialEq>(arr: &[T], elem: &T) -> Option<usize> {
    arr.iter().position(|x| x == elem)
}

/// Find last index of element
pub fn last_index_of<T: PartialEq>(arr: &[T], elem: &T) -> Option<usize> {
    arr.iter().rposition(|x| x == elem)
}

/// Create range of integers
pub fn range(start: i64, end: i64) -> Vec<i64> {
    (start..end).collect()
}

/// Create range with step
pub fn range_step(start: i64, end: i64, step: i64) -> Vec<i64> {
    if step == 0 {
        return vec![];
    }
    if step > 0 && start >= end {
        return vec![];
    }
    if step < 0 && start <= end {
        return vec![];
    }

    let mut result = vec![];
    let mut current = start;
    if step > 0 {
        while current < end {
            result.push(current);
            current += step;
        }
    } else {
        while current > end {
            result.push(current);
            current += step;
        }
    }
    result
}

/// Repeat element n times
pub fn repeat<T: Clone>(elem: T, n: usize) -> Vec<T> {
    vec![elem; n]
}

/// Take first n elements
pub fn take<T: Clone>(arr: &[T], n: usize) -> Vec<T> {
    arr.iter().take(n).cloned().collect()
}

/// Drop first n elements
pub fn drop<T: Clone>(arr: &[T], n: usize) -> Vec<T> {
    arr.iter().skip(n).cloned().collect()
}

/// Take elements while predicate is true
pub fn take_while<T: Clone, F: Fn(&T) -> bool>(arr: &[T], pred: F) -> Vec<T> {
    arr.iter().take_while(|x| pred(x)).cloned().collect()
}

/// Drop elements while predicate is true
pub fn drop_while<T: Clone, F: Fn(&T) -> bool>(arr: &[T], pred: F) -> Vec<T> {
    arr.iter().skip_while(|x| pred(x)).cloned().collect()
}

/// Zip two arrays into array of tuples
pub fn zip<T: Clone, U: Clone>(a: &[T], b: &[U]) -> Vec<(T, U)> {
    a.iter().cloned().zip(b.iter().cloned()).collect()
}

/// Unzip array of tuples into two arrays
pub fn unzip<T: Clone, U: Clone>(arr: &[(T, U)]) -> (Vec<T>, Vec<U>) {
    arr.iter().cloned().unzip()
}

/// Flatten nested arrays (one level)
pub fn flatten<T: Clone>(arr: &[Vec<T>]) -> Vec<T> {
    arr.iter().flat_map(|a| a.iter().cloned()).collect()
}

/// Chunk array into smaller arrays
pub fn chunk<T: Clone>(arr: &[T], size: usize) -> Vec<Vec<T>> {
    if size == 0 {
        return vec![];
    }
    arr.chunks(size).map(|c| c.to_vec()).collect()
}

/// Split array at index
pub fn split_at<T: Clone>(arr: &[T], idx: usize) -> (Vec<T>, Vec<T>) {
    let idx = idx.min(arr.len());
    (arr[..idx].to_vec(), arr[idx..].to_vec())
}

/// Get unique elements (preserves order)
pub fn unique<T: Clone + PartialEq>(arr: &[T]) -> Vec<T> {
    let mut result = vec![];
    for elem in arr {
        if !result.contains(elem) {
            result.push(elem.clone());
        }
    }
    result
}

/// Count occurrences of element
pub fn count<T: PartialEq>(arr: &[T], elem: &T) -> usize {
    arr.iter().filter(|x| *x == elem).count()
}

/// Sum integers
pub fn sum_int(arr: &[i64]) -> i64 {
    arr.iter().sum()
}

/// Sum floats
pub fn sum_float(arr: &[f64]) -> f64 {
    arr.iter().sum()
}

/// Product of integers
pub fn product_int(arr: &[i64]) -> i64 {
    arr.iter().product()
}

/// Product of floats
pub fn product_float(arr: &[f64]) -> f64 {
    arr.iter().product()
}

/// Minimum integer
pub fn min_int(arr: &[i64]) -> Option<i64> {
    arr.iter().copied().min()
}

/// Maximum integer
pub fn max_int(arr: &[i64]) -> Option<i64> {
    arr.iter().copied().max()
}

/// Minimum float
pub fn min_float(arr: &[f64]) -> Option<f64> {
    arr.iter().copied().reduce(f64::min)
}

/// Maximum float
pub fn max_float(arr: &[f64]) -> Option<f64> {
    arr.iter().copied().reduce(f64::max)
}

/// Average of floats
pub fn average(arr: &[f64]) -> Option<f64> {
    if arr.is_empty() {
        None
    } else {
        Some(arr.iter().sum::<f64>() / arr.len() as f64)
    }
}

/// Sort integers
pub fn sort_int(arr: &[i64]) -> Vec<i64> {
    let mut result = arr.to_vec();
    result.sort();
    result
}

/// Sort integers descending
pub fn sort_int_desc(arr: &[i64]) -> Vec<i64> {
    let mut result = arr.to_vec();
    result.sort_by(|a, b| b.cmp(a));
    result
}

/// Sort strings
pub fn sort_string(arr: &[String]) -> Vec<String> {
    let mut result = arr.to_vec();
    result.sort();
    result
}

/// Enumerate array (add indices)
pub fn enumerate<T: Clone>(arr: &[T]) -> Vec<(usize, T)> {
    arr.iter().cloned().enumerate().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ops() {
        let arr = vec![1, 2, 3];
        assert_eq!(len(&arr), 3);
        assert_eq!(first(&arr), Some(1));
        assert_eq!(last(&arr), Some(3));
        assert_eq!(get(&arr, 1), Some(2));
    }

    #[test]
    fn test_push_pop() {
        let arr = vec![1, 2, 3];
        assert_eq!(push(&arr, 4), vec![1, 2, 3, 4]);
        assert_eq!(pop(&arr), vec![1, 2]);
    }

    #[test]
    fn test_concat_slice() {
        let a = vec![1, 2];
        let b = vec![3, 4];
        assert_eq!(concat(&a, &b), vec![1, 2, 3, 4]);
        assert_eq!(slice(&vec![1, 2, 3, 4, 5], 1, 4), vec![2, 3, 4]);
    }

    #[test]
    fn test_range() {
        assert_eq!(range(0, 5), vec![0, 1, 2, 3, 4]);
        assert_eq!(range_step(0, 10, 2), vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_sum_product() {
        assert_eq!(sum_int(&[1, 2, 3, 4]), 10);
        assert_eq!(product_int(&[1, 2, 3, 4]), 24);
    }

    #[test]
    fn test_unique() {
        assert_eq!(unique(&[1, 2, 2, 3, 1, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn test_chunk() {
        assert_eq!(chunk(&[1, 2, 3, 4, 5], 2), vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn test_sort() {
        assert_eq!(sort_int(&[3, 1, 4, 1, 5]), vec![1, 1, 3, 4, 5]);
        assert_eq!(sort_int_desc(&[3, 1, 4, 1, 5]), vec![5, 4, 3, 1, 1]);
    }
}
