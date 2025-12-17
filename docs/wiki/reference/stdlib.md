# Standard Library Reference

Complete reference for the My Language standard library.

## Overview

The standard library is organized into modules:

```
std
├── prelude          # Auto-imported essentials
├── string           # String manipulation
├── collections      # Data structures
├── io               # Input/output
├── fs               # Filesystem
├── net              # Networking
├── sync             # Synchronization
├── async            # Async runtime
├── time             # Date and time
├── math             # Mathematics
├── random           # Random numbers
├── fmt              # Formatting
├── iter             # Iterators
├── ai               # AI integration
├── env              # Environment
├── process          # Process management
└── mem              # Memory utilities
```

## std::prelude

Automatically imported into every module:

```ml
// Types
pub use Option::{self, Some, None};
pub use Result::{self, Ok, Err};
pub use String;
pub use Vec;
pub use Box;

// Traits
pub use Clone;
pub use Copy;
pub use Debug;
pub use Default;
pub use Display;
pub use Eq;
pub use Ord;
pub use PartialEq;
pub use PartialOrd;
pub use Iterator;
pub use Into;
pub use From;

// Functions
pub use print;
pub use println;
pub use eprint;
pub use eprintln;
pub use format;
pub use panic;
pub use assert;
pub use assert_eq;
pub use assert_ne;
pub use dbg;
```

## std::option

The `Option` type for optional values.

```ml
enum Option<T> {
    Some(T),
    None,
}

impl<T> Option<T> {
    /// Returns true if the option is Some
    fn is_some(&self) -> Bool;

    /// Returns true if the option is None
    fn is_none(&self) -> Bool;

    /// Unwraps the value, panics if None
    fn unwrap(self) -> T;

    /// Unwraps or returns default
    fn unwrap_or(self, default: T) -> T;

    /// Unwraps or computes default
    fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T;

    /// Maps the value if Some
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U>;

    /// Flat maps the value
    fn and_then<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> Option<U>;

    /// Returns None if None, otherwise returns optb
    fn and<U>(self, optb: Option<U>) -> Option<U>;

    /// Returns the option if Some, otherwise returns optb
    fn or(self, optb: Option<T>) -> Option<T>;

    /// Filters the value
    fn filter<P: FnOnce(&T) -> Bool>(self, predicate: P) -> Option<T>;

    /// Converts to Result
    fn ok_or<E>(self, err: E) -> Result<T, E>;

    /// Expects value, panics with message if None
    fn expect(self, msg: &str) -> T;

    /// Takes the value, leaving None
    fn take(&mut self) -> Option<T>;

    /// Replaces the value
    fn replace(&mut self, value: T) -> Option<T>;

    /// Zips with another Option
    fn zip<U>(self, other: Option<U>) -> Option<(T, U)>;
}
```

## std::result

The `Result` type for error handling.

```ml
enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    /// Returns true if Ok
    fn is_ok(&self) -> Bool;

    /// Returns true if Err
    fn is_err(&self) -> Bool;

    /// Unwraps the Ok value, panics if Err
    fn unwrap(self) -> T;

    /// Unwraps the Err value, panics if Ok
    fn unwrap_err(self) -> E;

    /// Unwraps or returns default
    fn unwrap_or(self, default: T) -> T;

    /// Unwraps or computes default from error
    fn unwrap_or_else<F: FnOnce(E) -> T>(self, f: F) -> T;

    /// Maps the Ok value
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result<U, E>;

    /// Maps the Err value
    fn map_err<F, O: FnOnce(E) -> F>(self, f: O) -> Result<T, F>;

    /// Flat maps the Ok value
    fn and_then<U, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<U, E>;

    /// Returns res if Ok, otherwise returns self
    fn and<U>(self, res: Result<U, E>) -> Result<U, E>;

    /// Returns self if Ok, otherwise returns res
    fn or<F>(self, res: Result<T, F>) -> Result<T, F>;

    /// Converts to Option<T>
    fn ok(self) -> Option<T>;

    /// Converts to Option<E>
    fn err(self) -> Option<E>;

    /// Expects Ok, panics with message if Err
    fn expect(self, msg: &str) -> T;
}
```

## std::string

String manipulation.

```ml
impl String {
    /// Creates empty string
    fn new() -> String;

    /// Creates string with capacity
    fn with_capacity(capacity: Int) -> String;

    /// Returns length in bytes
    fn len(&self) -> Int;

    /// Returns true if empty
    fn is_empty(&self) -> Bool;

    /// Pushes a character
    fn push(&mut self, c: Char);

    /// Pushes a string slice
    fn push_str(&mut self, s: &str);

    /// Clears the string
    fn clear(&mut self);

    /// Returns as str slice
    fn as_str(&self) -> &str;

    /// Converts to uppercase
    fn to_uppercase(&self) -> String;

    /// Converts to lowercase
    fn to_lowercase(&self) -> String;

    /// Trims whitespace
    fn trim(&self) -> &str;
    fn trim_start(&self) -> &str;
    fn trim_end(&self) -> &str;

    /// Checks prefix/suffix
    fn starts_with(&self, pat: &str) -> Bool;
    fn ends_with(&self, pat: &str) -> Bool;

    /// Finds substring
    fn find(&self, pat: &str) -> Option<Int>;
    fn rfind(&self, pat: &str) -> Option<Int>;

    /// Contains check
    fn contains(&self, pat: &str) -> Bool;

    /// Replaces occurrences
    fn replace(&self, from: &str, to: &str) -> String;
    fn replacen(&self, from: &str, to: &str, count: Int) -> String;

    /// Splits string
    fn split(&self, pat: &str) -> Split;
    fn split_whitespace(&self) -> SplitWhitespace;
    fn lines(&self) -> Lines;

    /// Repeats string
    fn repeat(&self, n: Int) -> String;

    /// Parses to type
    fn parse<T: FromStr>(&self) -> Result<T, T::Err>;
}
```

## std::collections

### Vec<T>

Dynamic array.

```ml
impl<T> Vec<T> {
    fn new() -> Vec<T>;
    fn with_capacity(cap: Int) -> Vec<T>;

    fn len(&self) -> Int;
    fn is_empty(&self) -> Bool;
    fn capacity(&self) -> Int;

    fn push(&mut self, value: T);
    fn pop(&mut self) -> Option<T>;

    fn insert(&mut self, index: Int, value: T);
    fn remove(&mut self, index: Int) -> T;

    fn get(&self, index: Int) -> Option<&T>;
    fn get_mut(&mut self, index: Int) -> Option<&mut T>;

    fn first(&self) -> Option<&T>;
    fn last(&self) -> Option<&T>;

    fn clear(&mut self);
    fn truncate(&mut self, len: Int);
    fn resize(&mut self, new_len: Int, value: T);

    fn contains(&self, value: &T) -> Bool where T: PartialEq;
    fn sort(&mut self) where T: Ord;
    fn sort_by<F: FnMut(&T, &T) -> Ordering>(&mut self, f: F);

    fn iter(&self) -> Iter<T>;
    fn iter_mut(&mut self) -> IterMut<T>;
    fn into_iter(self) -> IntoIter<T>;

    fn reverse(&mut self);
    fn append(&mut self, other: &mut Vec<T>);
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I);
}
```

### HashMap<K, V>

Hash map.

```ml
impl<K: Hash + Eq, V> HashMap<K, V> {
    fn new() -> HashMap<K, V>;
    fn with_capacity(cap: Int) -> HashMap<K, V>;

    fn len(&self) -> Int;
    fn is_empty(&self) -> Bool;

    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn remove(&mut self, key: &K) -> Option<V>;

    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    fn contains_key(&self, key: &K) -> Bool;

    fn keys(&self) -> Keys<K, V>;
    fn values(&self) -> Values<K, V>;
    fn iter(&self) -> Iter<K, V>;

    fn entry(&mut self, key: K) -> Entry<K, V>;
    fn clear(&mut self);
}
```

### HashSet<T>

Hash set.

```ml
impl<T: Hash + Eq> HashSet<T> {
    fn new() -> HashSet<T>;

    fn len(&self) -> Int;
    fn is_empty(&self) -> Bool;

    fn insert(&mut self, value: T) -> Bool;
    fn remove(&mut self, value: &T) -> Bool;
    fn contains(&self, value: &T) -> Bool;

    fn iter(&self) -> Iter<T>;

    fn union(&self, other: &HashSet<T>) -> HashSet<T>;
    fn intersection(&self, other: &HashSet<T>) -> HashSet<T>;
    fn difference(&self, other: &HashSet<T>) -> HashSet<T>;

    fn is_subset(&self, other: &HashSet<T>) -> Bool;
    fn is_superset(&self, other: &HashSet<T>) -> Bool;
}
```

## std::io

Input/output operations.

```ml
/// Standard streams
fn stdin() -> Stdin;
fn stdout() -> Stdout;
fn stderr() -> Stderr;

/// Read trait
trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<Int, Error>;
    fn read_to_string(&mut self, buf: &mut String) -> Result<Int, Error>;
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<Int, Error>;
}

/// Write trait
trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<Int, Error>;
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error>;
    fn flush(&mut self) -> Result<(), Error>;
}

/// BufRead trait
trait BufRead: Read {
    fn read_line(&mut self, buf: &mut String) -> Result<Int, Error>;
    fn lines(&self) -> Lines<Self>;
}

/// BufReader
struct BufReader<R: Read>;

impl<R: Read> BufReader<R> {
    fn new(inner: R) -> BufReader<R>;
    fn with_capacity(cap: Int, inner: R) -> BufReader<R>;
}

/// BufWriter
struct BufWriter<W: Write>;

impl<W: Write> BufWriter<W> {
    fn new(inner: W) -> BufWriter<W>;
    fn with_capacity(cap: Int, inner: W) -> BufWriter<W>;
}
```

## std::fs

Filesystem operations.

```ml
/// Read entire file to string
fn read_to_string(path: &str) -> Result<String, Error>;

/// Read entire file to bytes
fn read(path: &str) -> Result<Vec<u8>, Error>;

/// Write string to file
fn write(path: &str, contents: &str) -> Result<(), Error>;

/// Create directory
fn create_dir(path: &str) -> Result<(), Error>;
fn create_dir_all(path: &str) -> Result<(), Error>;

/// Remove file/directory
fn remove_file(path: &str) -> Result<(), Error>;
fn remove_dir(path: &str) -> Result<(), Error>;
fn remove_dir_all(path: &str) -> Result<(), Error>;

/// Copy/rename
fn copy(from: &str, to: &str) -> Result<Int, Error>;
fn rename(from: &str, to: &str) -> Result<(), Error>;

/// Metadata
fn metadata(path: &str) -> Result<Metadata, Error>;
fn exists(path: &str) -> Bool;

/// Directory listing
fn read_dir(path: &str) -> Result<ReadDir, Error>;

/// File struct
struct File;

impl File {
    fn open(path: &str) -> Result<File, Error>;
    fn create(path: &str) -> Result<File, Error>;
    fn options() -> OpenOptions;
}
```

## std::time

Date and time.

```ml
/// Duration
struct Duration;

impl Duration {
    fn from_secs(secs: Int) -> Duration;
    fn from_millis(millis: Int) -> Duration;
    fn from_micros(micros: Int) -> Duration;
    fn from_nanos(nanos: Int) -> Duration;

    fn as_secs(&self) -> Int;
    fn as_millis(&self) -> Int;
    fn subsec_nanos(&self) -> Int;

    fn checked_add(&self, other: Duration) -> Option<Duration>;
    fn checked_sub(&self, other: Duration) -> Option<Duration>;
}

/// Instant (monotonic time)
struct Instant;

impl Instant {
    fn now() -> Instant;
    fn elapsed(&self) -> Duration;
    fn duration_since(&self, earlier: Instant) -> Duration;
}

/// SystemTime (wall clock)
struct SystemTime;

impl SystemTime {
    fn now() -> SystemTime;
    fn duration_since(&self, earlier: SystemTime) -> Result<Duration, Error>;

    const UNIX_EPOCH: SystemTime;
}

/// Sleep
async fn sleep(duration: Duration);
fn sleep_blocking(duration: Duration);
```

## std::math

Mathematical functions.

```ml
/// Constants
const PI: Float = 3.14159265358979323846;
const E: Float = 2.71828182845904523536;
const TAU: Float = 6.28318530717958647692;

/// Basic operations
fn abs(x: Float) -> Float;
fn min<T: Ord>(a: T, b: T) -> T;
fn max<T: Ord>(a: T, b: T) -> T;
fn clamp<T: Ord>(value: T, min: T, max: T) -> T;

/// Rounding
fn floor(x: Float) -> Float;
fn ceil(x: Float) -> Float;
fn round(x: Float) -> Float;
fn trunc(x: Float) -> Float;

/// Powers and roots
fn sqrt(x: Float) -> Float;
fn cbrt(x: Float) -> Float;
fn pow(base: Float, exp: Float) -> Float;
fn exp(x: Float) -> Float;
fn exp2(x: Float) -> Float;
fn ln(x: Float) -> Float;
fn log(x: Float, base: Float) -> Float;
fn log2(x: Float) -> Float;
fn log10(x: Float) -> Float;

/// Trigonometry
fn sin(x: Float) -> Float;
fn cos(x: Float) -> Float;
fn tan(x: Float) -> Float;
fn asin(x: Float) -> Float;
fn acos(x: Float) -> Float;
fn atan(x: Float) -> Float;
fn atan2(y: Float, x: Float) -> Float;
fn sinh(x: Float) -> Float;
fn cosh(x: Float) -> Float;
fn tanh(x: Float) -> Float;

/// Degrees/radians
fn to_degrees(radians: Float) -> Float;
fn to_radians(degrees: Float) -> Float;
```

## std::ai

AI integration utilities.

```ml
/// AI configuration
fn set_default_provider(provider: Provider);
fn get_default_provider() -> Provider;

/// Providers
enum Provider {
    OpenAI { api_key: String, model: String },
    Anthropic { api_key: String, model: String },
    Local { model_path: String },
}

/// Embedding utilities
fn embed(text: &str) -> AI<Vec<Float>>;
fn cosine_similarity(a: &[Float], b: &[Float]) -> Float;

/// Token counting
fn count_tokens(text: &str, model: &str) -> Int;

/// Rate limiting
struct RateLimiter;

impl RateLimiter {
    fn new(requests_per_minute: Int) -> RateLimiter;
    async fn acquire(&self);
}

/// Caching
struct AICache;

impl AICache {
    fn new(capacity: Int) -> AICache;
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: String, value: String);
}
```

## std::iter

Iterator utilities.

```ml
trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;

    // Provided methods
    fn count(self) -> Int;
    fn last(self) -> Option<Self::Item>;
    fn nth(&mut self, n: Int) -> Option<Self::Item>;

    fn map<B, F: FnMut(Self::Item) -> B>(self, f: F) -> Map<Self, F>;
    fn filter<P: FnMut(&Self::Item) -> Bool>(self, p: P) -> Filter<Self, P>;
    fn filter_map<B, F: FnMut(Self::Item) -> Option<B>>(self, f: F) -> FilterMap<Self, F>;

    fn enumerate(self) -> Enumerate<Self>;
    fn skip(self, n: Int) -> Skip<Self>;
    fn take(self, n: Int) -> Take<Self>;

    fn fold<B, F: FnMut(B, Self::Item) -> B>(self, init: B, f: F) -> B;
    fn reduce<F: FnMut(Self::Item, Self::Item) -> Self::Item>(self, f: F) -> Option<Self::Item>;

    fn collect<B: FromIterator<Self::Item>>(self) -> B;
    fn for_each<F: FnMut(Self::Item)>(self, f: F);

    fn find<P: FnMut(&Self::Item) -> Bool>(&mut self, p: P) -> Option<Self::Item>;
    fn position<P: FnMut(Self::Item) -> Bool>(&mut self, p: P) -> Option<Int>;

    fn any<F: FnMut(Self::Item) -> Bool>(&mut self, f: F) -> Bool;
    fn all<F: FnMut(Self::Item) -> Bool>(&mut self, f: F) -> Bool;

    fn sum(self) -> Self::Item where Self::Item: Sum;
    fn product(self) -> Self::Item where Self::Item: Product;

    fn min(self) -> Option<Self::Item> where Self::Item: Ord;
    fn max(self) -> Option<Self::Item> where Self::Item: Ord;

    fn zip<U: IntoIterator>(self, other: U) -> Zip<Self, U::IntoIter>;
    fn chain<U: IntoIterator<Item = Self::Item>>(self, other: U) -> Chain<Self, U::IntoIter>;

    fn flatten(self) -> Flatten<Self> where Self::Item: IntoIterator;
    fn flat_map<U: IntoIterator, F: FnMut(Self::Item) -> U>(self, f: F) -> FlatMap<Self, U, F>;

    fn rev(self) -> Rev<Self> where Self: DoubleEndedIterator;
}
```

## std::sync

Synchronization primitives.

```ml
/// Mutex
struct Mutex<T>;

impl<T> Mutex<T> {
    fn new(value: T) -> Mutex<T>;
    fn lock(&self) -> MutexGuard<T>;
    fn try_lock(&self) -> Option<MutexGuard<T>>;
}

/// RwLock
struct RwLock<T>;

impl<T> RwLock<T> {
    fn new(value: T) -> RwLock<T>;
    fn read(&self) -> RwLockReadGuard<T>;
    fn write(&self) -> RwLockWriteGuard<T>;
}

/// Arc (atomic reference counting)
struct Arc<T>;

impl<T> Arc<T> {
    fn new(value: T) -> Arc<T>;
    fn strong_count(&self) -> Int;
    fn clone(&self) -> Arc<T>;
}

/// Channels
fn channel<T>() -> (Sender<T>, Receiver<T>);
fn bounded_channel<T>(cap: Int) -> (Sender<T>, Receiver<T>);

struct Sender<T>;
struct Receiver<T>;

impl<T> Sender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>>;
    fn try_send(&self, value: T) -> Result<(), TrySendError<T>>;
}

impl<T> Receiver<T> {
    fn recv(&self) -> Result<T, RecvError>;
    fn try_recv(&self) -> Result<T, TryRecvError>;
}

/// Atomic types
struct AtomicBool;
struct AtomicInt;
struct AtomicUInt;

/// Once
struct Once;

impl Once {
    const fn new() -> Once;
    fn call_once<F: FnOnce()>(&self, f: F);
}
```

## std::async

Async runtime utilities.

```ml
/// Spawning
fn spawn<F: Future>(future: F) -> JoinHandle<F::Output>;

/// Joining
async fn join<A, B>(a: A, b: B) -> (A::Output, B::Output)
where A: Future, B: Future;

async fn join_all<I: IntoIterator>(futures: I) -> Vec<I::Item::Output>
where I::Item: Future;

/// Select
async fn select<A, B>(a: A, b: B) -> Either<A::Output, B::Output>
where A: Future, B: Future;

/// Timeout
async fn timeout<F: Future>(duration: Duration, future: F) -> Result<F::Output, TimeoutError>;

/// Yielding
async fn yield_now();

/// Blocking
fn block_on<F: Future>(future: F) -> F::Output;
```
