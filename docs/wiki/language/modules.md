# Modules and Imports

My Language uses a module system for organizing code into reusable, encapsulated units.

## Module Basics

### Defining Modules

```ml
// Inline module definition
mod math {
    pub fn add(a: Int, b: Int) -> Int {
        a + b
    }

    pub fn multiply(a: Int, b: Int) -> Int {
        a * b
    }

    // Private function (not exported)
    fn helper() {
        // ...
    }
}

// Using the module
fn main() {
    let sum = math::add(2, 3);
    let product = math::multiply(4, 5);
}
```

### File-Based Modules

Project structure:
```
src/
├── main.ml
├── math.ml
└── utils/
    ├── mod.ml
    └── string_utils.ml
```

```ml
// src/main.ml
mod math;           // Loads from src/math.ml
mod utils;          // Loads from src/utils/mod.ml

fn main() {
    let x = math::add(1, 2);
    let s = utils::string_utils::capitalize("hello");
}
```

```ml
// src/math.ml
pub fn add(a: Int, b: Int) -> Int {
    a + b
}

pub fn subtract(a: Int, b: Int) -> Int {
    a - b
}
```

```ml
// src/utils/mod.ml
pub mod string_utils;  // Loads string_utils.ml

pub fn log(msg: String) {
    print("[LOG] {msg}");
}
```

```ml
// src/utils/string_utils.ml
pub fn capitalize(s: String) -> String {
    // Implementation
}

pub fn trim(s: String) -> String {
    // Implementation
}
```

## Visibility

### Public vs Private

```ml
mod example {
    // Public: accessible from outside the module
    pub fn public_function() { }
    pub struct PublicStruct { }
    pub const PUBLIC_CONST: Int = 42;

    // Private: only accessible within this module
    fn private_function() { }
    struct PrivateStruct { }
    const PRIVATE_CONST: Int = 0;

    // Struct with mixed visibility
    pub struct MixedStruct {
        pub public_field: Int,
        private_field: String,  // Private by default
    }
}
```

### Pub Variants

```ml
mod example {
    // Public to parent module only
    pub(super) fn parent_only() { }

    // Public within crate only
    pub(crate) fn crate_only() { }

    // Public to specific module
    pub(in crate::specific::path) fn path_only() { }
}
```

### Re-exporting

```ml
mod internal {
    pub fn helper() { }
}

// Re-export for public API
pub use internal::helper;

// Rename on re-export
pub use internal::helper as util_helper;
```

## Import Syntax

### Basic Imports

```ml
// Import single item
use std::collections::HashMap;

// Import multiple items
use std::collections::{HashMap, HashSet, BTreeMap};

// Import all public items (glob import)
use std::collections::*;

// Import with alias
use std::collections::HashMap as Map;
```

### Nested Imports

```ml
use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write, BufReader},
    sync::{Arc, Mutex},
};
```

### Self and Super

```ml
mod parent {
    pub fn parent_fn() { }

    mod child {
        use super::parent_fn;  // Import from parent

        pub fn child_fn() {
            parent_fn();
        }

        mod grandchild {
            use super::super::parent_fn;  // Two levels up
        }
    }
}

mod another {
    use self::submodule::helper;  // Explicit self

    mod submodule {
        pub fn helper() { }
    }
}
```

### Crate Root

```ml
// Import from crate root
use crate::utils::helper;
use crate::models::User;

// In nested module
mod deeply {
    mod nested {
        mod module {
            use crate::top_level_function;
        }
    }
}
```

## Module Patterns

### Prelude Pattern

```ml
// src/prelude.ml
pub use crate::types::{User, Post, Comment};
pub use crate::traits::{Readable, Writable};
pub use crate::macros::*;
pub use crate::constants::*;

// Usage in other modules
use crate::prelude::*;
```

### Facade Pattern

```ml
// src/lib.ml - Public API
pub mod api {
    pub use crate::internal::service::PublicService;
    pub use crate::internal::types::{Request, Response};
    // Internal implementation details hidden
}

mod internal {
    pub mod service { }
    pub mod types { }
    mod implementation { }  // Not exposed
}
```

### Feature Modules

```ml
// Conditionally compiled modules
#[cfg(feature = "async")]
pub mod async_support;

#[cfg(feature = "serde")]
pub mod serialization;

#[cfg(test)]
mod tests;
```

## External Dependencies

### Cargo.toml Dependencies

```toml
[dependencies]
http = "1.0"
json = "2.0"
async-runtime = { version = "0.5", features = ["full"] }

[dev-dependencies]
testing = "1.0"
```

### Using External Crates

```ml
// External crate imports
use http::{Client, Request, Response};
use json::{parse, stringify};

fn fetch_data(url: String) -> Result<json::Value, Error> {
    let client = Client::new();
    let response = client.get(url).send()?;
    let data = json::parse(response.body())?;
    Ok(data)
}
```

### Aliasing Crates

```toml
[dependencies]
serde_json = { package = "json", version = "2.0" }
```

```ml
use serde_json as json;
```

## Workspace Organization

### Multi-Crate Project

```
my-project/
├── Cargo.toml (workspace)
├── core/
│   ├── Cargo.toml
│   └── src/lib.ml
├── cli/
│   ├── Cargo.toml
│   └── src/main.ml
└── web/
    ├── Cargo.toml
    └── src/lib.ml
```

```toml
# Root Cargo.toml
[workspace]
members = ["core", "cli", "web"]
```

```toml
# cli/Cargo.toml
[dependencies]
core = { path = "../core" }
```

```ml
// cli/src/main.ml
use core::types::Config;
use core::process;

fn main() {
    let config = Config::load();
    process::run(config);
}
```

## Module Best Practices

### 1. One Responsibility Per Module

```ml
// Good: Focused modules
mod auth {
    pub fn login() { }
    pub fn logout() { }
    pub fn verify_token() { }
}

mod users {
    pub fn create_user() { }
    pub fn get_user() { }
    pub fn update_user() { }
}

// Avoid: Kitchen sink modules
mod utils {
    pub fn login() { }      // Should be in auth
    pub fn format_date() { } // Should be in date
    pub fn parse_json() { }  // Should be in json
}
```

### 2. Minimize Public API

```ml
// Good: Expose only what's needed
pub mod api {
    pub fn public_interface() { }

    // Internal helpers stay private
    fn internal_helper() { }
}

// Avoid: Everything public
pub mod api {
    pub fn public_interface() { }
    pub fn internal_helper() { }  // Why is this public?
}
```

### 3. Use Preludes for Common Items

```ml
// Good: Convenient prelude
pub mod prelude {
    pub use crate::Result;
    pub use crate::Error;
    pub use crate::Context;
}

// Users can: use mylibrary::prelude::*;
```

### 4. Group Related Types

```ml
// Good: Related items together
mod user {
    pub struct User { }
    pub struct UserBuilder { }
    pub enum UserRole { }
    pub trait UserRepository { }

    impl User { }
    impl UserBuilder { }
}
```

### 5. Document Module Purpose

```ml
//! # Authentication Module
//!
//! This module handles user authentication including:
//! - Password-based login
//! - Token generation and validation
//! - Session management
//!
//! ## Example
//! ```ml
//! use myapp::auth;
//!
//! let token = auth::login("user", "password")?;
//! auth::verify_token(token)?;
//! ```

pub fn login(username: &str, password: &str) -> Result<Token> { }
pub fn verify_token(token: Token) -> Result<User> { }
```

## Import Resolution

### Resolution Order

1. Built-in types (Int, String, etc.)
2. Items in current scope
3. Imported items
4. Items from prelude

```ml
use std::collections::HashMap;

fn example() {
    let x: Int = 5;              // 1. Built-in
    let m: HashMap = ...;         // 3. Imported
    let s: String = ...;         // 4. Prelude
}
```

### Shadowing

```ml
use std::collections::HashMap;

fn example() {
    // Local definition shadows import
    struct HashMap { }

    let m: HashMap = HashMap { };  // Uses local HashMap
}
```

### Avoiding Conflicts

```ml
use std::io::Result as IoResult;
use std::fmt::Result as FmtResult;

fn process() -> IoResult<()> { }
fn format() -> FmtResult { }

// Or use qualified paths
fn process() -> std::io::Result<()> { }
fn format() -> std::fmt::Result { }
```
