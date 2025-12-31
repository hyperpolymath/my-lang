//! Common Library Module
//!
//! Generic utility functions that can be shared across language implementations.
//! These are language-agnostic operations for I/O, math, strings, arrays, and types.

pub mod io;
pub mod math;
pub mod string;
pub mod array;
pub mod types;
pub mod utils;

// Don't use glob re-exports to avoid name collisions between modules
// Users should access functions via qualified paths like `io::print_str` or `math::sqrt`
