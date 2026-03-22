//! My Language Library
//!
//! This module provides both common utilities and language-specific features
//! for the My Language implementation.
//!
//! ## Structure
//!
//! - `common`: Generic utilities for I/O, math, strings, arrays, types, and more.
//!   These are language-agnostic and can be reused across implementations.
//!
//! - `mylang`: Language-specific features including AI integration, prompt building,
//!   streaming responses, and tool/function calling support.
//!
//! ## Usage
//!
//! ```rust
//! use my_lang::library::common::math;
//! use my_lang::library::mylang::ai;
//!
//! let x = math::sqrt(16.0);
//! let model = ai::AiModelConfig::new("claude-3-opus");
//! ```

pub mod common;
pub mod mylang;

/// Prelude module for commonly used items
pub mod prelude {
    // Re-export submodules for qualified access
    pub use super::common::io;
    pub use super::common::math;
    pub use super::common::string;
    pub use super::common::array;
    pub use super::common::types;
    pub use super::common::utils;

    pub use super::mylang::ai;
    pub use super::mylang::prompt;
    pub use super::mylang::stream;
    pub use super::mylang::tools;

    // Math constants (these don't conflict)
    pub use super::common::math::constants::{PI, E, TAU};

    // Common utilities (non-conflicting)
    pub use super::common::utils::{
        timestamp, timestamp_millis, timestamp_secs,
        random, random_int, random_bool,
        SimpleRng,
    };

    // AI types
    pub use super::mylang::ai::{
        AiModelType, AiModelConfig, AiResponse, FinishReason, TokenUsage,
        MessageRole, Message, Conversation, MockAiClient,
    };

    // Prompt utilities
    pub use super::mylang::prompt::{PromptTemplate, PromptBuilder, PromptLibrary};

    // Streaming types
    pub use super::mylang::stream::{StreamChunk, StreamBuffer, MockStream};

    // Tool types
    pub use super::mylang::tools::{ToolDef, ToolCall, ToolValue, ToolResult, ToolRegistry};
}
