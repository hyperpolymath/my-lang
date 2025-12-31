//! Property-based testing framework for My Language
//!
//! Inspired by Echidna's approach to invariant testing:
//! - Define properties that should ALWAYS hold
//! - Generate random inputs to try to break those properties
//! - Minimize failing cases for easy debugging


/// Property test result
#[derive(Debug, Clone)]
pub enum PropertyResult {
    Passed { iterations: usize },
    Failed { counterexample: String, iteration: usize },
    Skipped { reason: String },
}

/// Configuration for property tests
#[derive(Debug, Clone)]
pub struct PropertyConfig {
    pub iterations: usize,
    pub max_size: usize,
    pub seed: Option<u64>,
    pub shrink_iterations: usize,
}

impl Default for PropertyConfig {
    fn default() -> Self {
        PropertyConfig {
            iterations: 100,
            max_size: 100,
            seed: None,
            shrink_iterations: 100,
        }
    }
}

/// Simple random generator for property tests
pub struct TestRng {
    state: u64,
}

impl TestRng {
    pub fn new(seed: u64) -> Self {
        TestRng { state: seed }
    }

    pub fn from_entropy() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self::new(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        // xorshift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn next_usize(&mut self, max: usize) -> usize {
        (self.next_u64() as usize) % max.max(1)
    }

    pub fn next_bool(&mut self) -> bool {
        self.next_u64() % 2 == 0
    }

    pub fn choose<T: Clone>(&mut self, items: &[T]) -> Option<T> {
        if items.is_empty() {
            None
        } else {
            Some(items[self.next_usize(items.len())].clone())
        }
    }
}

/// Generator trait for creating random test inputs
pub trait Arbitrary {
    fn arbitrary(rng: &mut TestRng, size: usize) -> Self;
    fn shrink(&self) -> Vec<Self> where Self: Sized {
        vec![]
    }
}

impl Arbitrary for String {
    fn arbitrary(rng: &mut TestRng, size: usize) -> Self {
        let len = rng.next_usize(size + 1);
        (0..len)
            .map(|_| {
                let c = (rng.next_usize(95) + 32) as u8 as char;
                c
            })
            .collect()
    }

    fn shrink(&self) -> Vec<Self> {
        let mut results = vec![];
        if self.len() > 1 {
            results.push(self[..self.len() / 2].to_string());
            results.push(self[self.len() / 2..].to_string());
        }
        if !self.is_empty() {
            results.push(String::new());
        }
        results
    }
}

impl Arbitrary for i64 {
    fn arbitrary(rng: &mut TestRng, size: usize) -> Self {
        let val = rng.next_u64() as i64;
        val % (size as i64 + 1)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut results = vec![];
        if *self != 0 {
            results.push(0);
            results.push(self / 2);
            if *self > 0 {
                results.push(self - 1);
            } else {
                results.push(self + 1);
            }
        }
        results
    }
}

/// Run a property test
pub fn check_property<F>(_name: &str, config: &PropertyConfig, mut prop: F) -> PropertyResult
where
    F: FnMut(&mut TestRng) -> bool,
{
    let seed = config.seed.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    });

    let mut rng = TestRng::new(seed);

    for i in 0..config.iterations {
        if !prop(&mut rng) {
            return PropertyResult::Failed {
                counterexample: format!("Iteration {} with seed {}", i, seed),
                iteration: i,
            };
        }
    }

    PropertyResult::Passed {
        iterations: config.iterations,
    }
}

/// Macro for defining property tests
#[macro_export]
macro_rules! property_test {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let config = $crate::proptest::PropertyConfig::default();
            let result = $crate::proptest::check_property(stringify!($name), &config, $body);
            match result {
                $crate::proptest::PropertyResult::Passed { iterations } => {
                    println!("Property {} passed ({} iterations)", stringify!($name), iterations);
                }
                $crate::proptest::PropertyResult::Failed { counterexample, iteration } => {
                    panic!(
                        "Property {} failed at iteration {}: {}",
                        stringify!($name),
                        iteration,
                        counterexample
                    );
                }
                $crate::proptest::PropertyResult::Skipped { reason } => {
                    println!("Property {} skipped: {}", stringify!($name), reason);
                }
            }
        }
    };
}
