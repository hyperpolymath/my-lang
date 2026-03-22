//! Common Utility Functions
//!
//! Miscellaneous utility functions for runtime operations.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// Time Functions
// ============================================================================

/// Get current Unix timestamp in seconds (as float with subsecond precision)
pub fn timestamp() -> f64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1_000_000_000.0
}

/// Get current Unix timestamp in milliseconds
pub fn timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Get current Unix timestamp in seconds
pub fn timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Sleep for given duration in seconds
pub fn sleep_secs(secs: f64) {
    std::thread::sleep(Duration::from_secs_f64(secs));
}

/// Sleep for given duration in milliseconds
pub fn sleep_millis(millis: u64) {
    std::thread::sleep(Duration::from_millis(millis));
}

// ============================================================================
// Random Number Generation (Simple LCG - not cryptographically secure)
// ============================================================================

/// Simple random number generator state
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    /// Create new RNG with seed
    pub fn new(seed: u64) -> Self {
        SimpleRng {
            state: seed.max(1),
        }
    }

    /// Create RNG seeded from current time
    pub fn from_time() -> Self {
        Self::new(timestamp_millis())
    }

    /// Generate next u64
    pub fn next_u64(&mut self) -> u64 {
        // LCG parameters from Knuth
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    /// Generate random float in [0, 1)
    pub fn next_float(&mut self) -> f64 {
        self.next_u64() as f64 / u64::MAX as f64
    }

    /// Generate random integer in [0, max)
    pub fn next_int(&mut self, max: u64) -> u64 {
        if max == 0 {
            return 0;
        }
        self.next_u64() % max
    }

    /// Generate random integer in [min, max]
    pub fn next_range(&mut self, min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        let range = (max - min + 1) as u64;
        min + (self.next_int(range) as i64)
    }

    /// Generate random bool
    pub fn next_bool(&mut self) -> bool {
        self.next_u64() % 2 == 0
    }

    /// Generate random bool with probability p of being true
    pub fn next_bool_p(&mut self, p: f64) -> bool {
        self.next_float() < p
    }

    /// Shuffle array in place
    pub fn shuffle<T>(&mut self, arr: &mut [T]) {
        let len = arr.len();
        for i in (1..len).rev() {
            let j = self.next_int((i + 1) as u64) as usize;
            arr.swap(i, j);
        }
    }

    /// Pick random element from slice
    pub fn choice<'a, T>(&mut self, arr: &'a [T]) -> Option<&'a T> {
        if arr.is_empty() {
            None
        } else {
            let idx = self.next_int(arr.len() as u64) as usize;
            Some(&arr[idx])
        }
    }
}

// Global RNG for simple random functions
thread_local! {
    static GLOBAL_RNG: std::cell::RefCell<SimpleRng> = std::cell::RefCell::new(SimpleRng::from_time());
}

/// Get random float in [0, 1)
pub fn random() -> f64 {
    GLOBAL_RNG.with(|rng| rng.borrow_mut().next_float())
}

/// Get random integer in [min, max]
pub fn random_int(min: i64, max: i64) -> i64 {
    GLOBAL_RNG.with(|rng| rng.borrow_mut().next_range(min, max))
}

/// Get random bool
pub fn random_bool() -> bool {
    GLOBAL_RNG.with(|rng| rng.borrow_mut().next_bool())
}

// ============================================================================
// Environment
// ============================================================================

/// Get environment variable
pub fn env_get(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Get environment variable with default
pub fn env_get_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Set environment variable
pub fn env_set(name: &str, value: &str) {
    std::env::set_var(name, value);
}

/// Remove environment variable
pub fn env_remove(name: &str) {
    std::env::remove_var(name);
}

/// Get all environment variables
pub fn env_all() -> Vec<(String, String)> {
    std::env::vars().collect()
}

/// Get current working directory
pub fn cwd() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
}

/// Get home directory
pub fn home_dir() -> Option<String> {
    dirs::home_dir().and_then(|p| p.to_str().map(|s| s.to_string()))
}

// ============================================================================
// Assertions
// ============================================================================

/// Assert condition is true
pub fn assert_true(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(format!("Assertion failed: {}", message))
    }
}

/// Assert two values are equal
pub fn assert_eq<T: PartialEq + std::fmt::Debug>(a: &T, b: &T) -> Result<(), String> {
    if a == b {
        Ok(())
    } else {
        Err(format!("Assertion failed: {:?} != {:?}", a, b))
    }
}

/// Assert two values are not equal
pub fn assert_ne<T: PartialEq + std::fmt::Debug>(a: &T, b: &T) -> Result<(), String> {
    if a != b {
        Ok(())
    } else {
        Err(format!("Assertion failed: {:?} == {:?}", a, b))
    }
}

// ============================================================================
// Miscellaneous
// ============================================================================

/// Identity function
pub fn identity<T>(x: T) -> T {
    x
}

/// Constant function generator
pub fn constant<T: Clone>(x: T) -> impl Fn() -> T {
    move || x.clone()
}

/// Compose two functions
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

/// Flip argument order of a binary function
pub fn flip<A, B, C, F>(f: F) -> impl Fn(B, A) -> C
where
    F: Fn(A, B) -> C,
{
    move |b, a| f(a, b)
}

/// Apply function n times
pub fn iterate<T: Clone, F>(mut x: T, n: usize, f: F) -> T
where
    F: Fn(T) -> T,
{
    for _ in 0..n {
        x = f(x);
    }
    x
}

/// Memoize a function (simple version - not thread-safe)
pub fn memoize<A, B, F>(f: F) -> impl FnMut(A) -> B
where
    A: Clone + std::hash::Hash + Eq,
    B: Clone,
    F: Fn(A) -> B,
{
    use std::collections::HashMap;
    let mut cache: HashMap<A, B> = HashMap::new();
    move |arg: A| {
        if let Some(result) = cache.get(&arg) {
            result.clone()
        } else {
            let result = f(arg.clone());
            cache.insert(arg, result.clone());
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let t1 = timestamp();
        std::thread::sleep(Duration::from_millis(10));
        let t2 = timestamp();
        assert!(t2 > t1);
    }

    #[test]
    fn test_random() {
        let r1 = random();
        let r2 = random();
        // Random numbers should be in [0, 1)
        assert!(r1 >= 0.0 && r1 < 1.0);
        assert!(r2 >= 0.0 && r2 < 1.0);
    }

    #[test]
    fn test_random_int() {
        for _ in 0..100 {
            let r = random_int(5, 10);
            assert!(r >= 5 && r <= 10);
        }
    }

    #[test]
    fn test_rng() {
        let mut rng = SimpleRng::new(12345);
        let a = rng.next_u64();
        let b = rng.next_u64();
        assert_ne!(a, b);

        let mut rng2 = SimpleRng::new(12345);
        assert_eq!(rng2.next_u64(), a);
    }

    #[test]
    fn test_shuffle() {
        let mut arr = vec![1, 2, 3, 4, 5];
        let original = arr.clone();
        let mut rng = SimpleRng::new(42);
        rng.shuffle(&mut arr);
        // Array should be permuted (very unlikely to be same)
        assert_eq!(arr.len(), original.len());
        arr.sort();
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_assertions() {
        assert!(assert_true(true, "test").is_ok());
        assert!(assert_true(false, "test").is_err());
        assert!(assert_eq(&1, &1).is_ok());
        assert!(assert_eq(&1, &2).is_err());
    }

    #[test]
    fn test_identity() {
        assert_eq!(identity(42), 42);
        assert_eq!(identity("hello"), "hello");
    }

    #[test]
    fn test_iterate() {
        let result = iterate(1, 5, |x| x * 2);
        assert_eq!(result, 32); // 1 * 2^5 = 32
    }
}

// Bring in dirs crate functionality stub if not available
mod dirs {
    pub fn home_dir() -> Option<std::path::PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(std::path::PathBuf::from)
    }
}
